//! Parses bounded literal and folded block-scalar bodies.

use super::Value;

impl super::Reader {
    /// Reads literal or folded block text with strip, clip, or keep chomping.
    pub(super) fn block_scalar(
        &mut self,
        header: &str,
        indent: usize,
        number: usize,
    ) -> Result<Value, String> {
        let folded = header.starts_with('>');
        let marker = if folded { '>' } else { '|' };
        let suffix = header
            .strip_prefix(marker)
            .ok_or_else(|| format!("line {number}: invalid block scalar header"))?;
        let chomping = match suffix {
            "" => ' ',
            "+" => '+',
            "-" => '-',
            _ => {
                return Err(format!(
                    "line {number}: block scalar indicators are unsupported"
                ));
            }
        };
        let (indentation, body) = self.collect_block_scalar_body(indent);
        let Some(body_indent) = indentation else {
            if folded && !body.is_empty() {
                return Err(format!("line {number}: folded blank lines are unsupported"));
            }
            return Ok(Value::Scalar(String::new()));
        };
        let content = dedent_block_scalar_body(body, body_indent, number, chomping)?;
        if folded && content.iter().any(String::is_empty) {
            return Err(format!("line {number}: folded blank lines are unsupported"));
        }
        let has_content = !content.is_empty();
        let mut text = if folded {
            fold_lines(&content)
        } else {
            content.join("\n")
        };
        if has_content && chomping != '-' {
            text.push('\n');
        }
        Ok(Value::Scalar(text))
    }

    /// Collects literal source lines and their minimum indentation from one block scalar.
    fn collect_block_scalar_body(&mut self, indent: usize) -> (Option<usize>, Vec<String>) {
        let mut body = Vec::new();
        let mut body_indent = None;
        while let Some(line) = self.lines.get(self.cursor) {
            if line.literal.trim().is_empty() {
                body.push(String::new());
                self.cursor += 1;
                continue;
            }
            if line.indent <= indent {
                break;
            }
            body_indent =
                Some(body_indent.map_or(line.indent, |found: usize| found.min(line.indent)));
            body.push(line.literal.clone());
            self.cursor += 1;
        }
        (body_indent, body)
    }
}

/// Removes block indentation and applies strip/clip/keep chomping.
fn dedent_block_scalar_body(
    body: Vec<String>,
    body_indent: usize,
    number: usize,
    chomping: char,
) -> Result<Vec<String>, String> {
    let mut content = body
        .into_iter()
        .map(|line| {
            if line.is_empty() {
                Ok(String::new())
            } else {
                line.get(body_indent..)
                    .map(str::to_owned)
                    .ok_or_else(|| format!("line {number}: invalid block scalar indentation"))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    if chomping != '+' {
        while content.last().is_some_and(String::is_empty) {
            content.pop();
        }
    }
    Ok(content)
}

/// Folds nonblank bodies; the parser rejects blank lines before calling this helper.
fn fold_lines(lines: &[String]) -> String {
    let mut output = String::new();
    let mut previous_more_indented = false;
    for line in lines {
        let more_indented = line.starts_with(' ');
        if !output.is_empty() {
            let folds_to_space = !previous_more_indented && !more_indented;
            output.push(if folds_to_space { ' ' } else { '\n' });
        }
        output.push_str(line);
        previous_more_indented = more_indented;
    }
    output
}
