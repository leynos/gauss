//! Decodes the bounded YAML syntax needed to inspect every local workflow without coercion.

use std::collections::BTreeMap;

use super::Value;
use super::lines;

/// Keeps block-scalar decoding beside the shared physical-line reader.
#[path = "ci_reachability_parser_blocks.rs"]
mod blocks;

/// One physical source line with both structural and literal-block views.
#[derive(Clone, Debug)]
pub(super) struct SourceLine {
    /// One-based position used in parser errors.
    pub(super) number: usize,
    /// Leading space count before the first non-whitespace source character.
    pub(super) indent: usize,
    /// Comment-free content with indentation removed.
    pub(super) content: String,
    /// Original physical line, including indentation, retained for block scalars.
    pub(super) literal: String,
}

/// Splits source into physical lines and rejects tabs in indentation.
pub(super) fn read(source: &str) -> Result<Vec<SourceLine>, String> {
    let mut lines = Vec::new();
    for (index, original) in source.lines().enumerate() {
        let raw = original.strip_suffix('\r').unwrap_or(original);
        let indent = raw
            .chars()
            .take_while(|character| *character == ' ')
            .count();
        let stripped = raw
            .get(indent..)
            .ok_or_else(|| format!("line {}: invalid indentation boundary", index + 1))?
            .to_owned();
        if stripped.starts_with('\t') {
            return Err(format!(
                "line {}: tab indentation is unsupported",
                index + 1
            ));
        }
        let content = lines::without_comment(&stripped)
            .map_err(|error| format!("line {}: {error}", index + 1))?
            .trim_end()
            .to_owned();
        lines.push(SourceLine {
            number: index + 1,
            indent,
            content,
            literal: raw.to_owned(),
        });
    }
    Ok(lines)
}

/// Parses one complete YAML document, preserving scalar spelling and rejecting ambiguity.
pub(super) fn parse(source: &str) -> Result<Value, String> {
    let lines = read(source)?;
    Reader { lines, cursor: 0 }.document()
}

/// Recognizes document delimiters only at the source root.
/// For example, a column-zero `...` is a boundary, while an indented `...` is not.
fn is_root_document_boundary(line: &SourceLine) -> bool {
    line.indent == 0 && matches!(line.content.as_str(), "---" | "...")
}

/// Tracks physical workflow lines while recursively decoding block and flow collections.
struct Reader {
    /// Source lines retain literal bodies separately from comment-free structure.
    lines: Vec<SourceLine>,
    /// Next physical line not yet consumed by the parser.
    cursor: usize,
}

impl Reader {
    /// Parses one document and rejects empty input or additional documents.
    fn document(mut self) -> Result<Value, String> {
        let first = self.first_document_content()?;
        let indent = self.line(first)?.indent;
        let value = self.block(indent)?;
        self.finish_document()?;
        if matches!(value, Value::Empty) {
            Err("workflow document is empty".to_owned())
        } else {
            Ok(value)
        }
    }

    /// Reads required content after an optional root-level start marker.
    /// For example, a lone `---` still reports an empty workflow document.
    fn first_document_content(&mut self) -> Result<usize, String> {
        let Some(mut first) = self.next_significant() else {
            return Err("workflow document is empty".to_owned());
        };
        let has_start_marker = {
            let line = self.line(first)?;
            is_root_document_boundary(line) && line.content == "---"
        };
        if has_start_marker {
            self.cursor += 1;
            first = self
                .next_significant()
                .ok_or_else(|| "workflow document is empty".to_owned())?;
        }
        Ok(first)
    }

    /// Consumes an optional end marker before refusing additional document content.
    /// For example, `...` may finish one document, but a following document fails.
    fn finish_document(&mut self) -> Result<(), String> {
        if let Some(index) = self.next_significant() {
            let has_end_marker = {
                let line = self.line(index)?;
                is_root_document_boundary(line) && line.content == "..."
            };
            if has_end_marker {
                self.cursor += 1;
            }
        }
        if self.next_significant().is_some() {
            return Err("multiple YAML documents or trailing content are unsupported".to_owned());
        }
        Ok(())
    }

    /// Returns the next significant line and moves the cursor past ignored lines.
    fn next_significant(&mut self) -> Option<usize> {
        while self
            .lines
            .get(self.cursor)
            .is_some_and(|line| line.content.trim().is_empty())
        {
            self.cursor += 1;
        }
        self.lines.get(self.cursor).map(|_| self.cursor)
    }

    /// Reads a physical line or reports a bounded-parser position error.
    fn line(&self, index: usize) -> Result<&SourceLine, String> {
        self.lines
            .get(index)
            .ok_or_else(|| format!("line {}: missing source line", index + 1))
    }

    /// Selects one block parser from the first line at the required indentation.
    fn block(&mut self, indent: usize) -> Result<Value, String> {
        let Some(index) = self.next_significant() else {
            return Err("expected an indented YAML value".to_owned());
        };
        let line = self.line(index)?;
        if line.indent != indent {
            return Err(format!("line {}: unexpected indentation", line.number));
        }
        if sequence_item(&line.content).is_some() {
            self.sequence(indent)
        } else {
            self.mapping(indent)
        }
    }

    /// Decodes all mapping entries at one indentation and rejects duplicate decoded keys.
    fn mapping(&mut self, indent: usize) -> Result<Value, String> {
        let mut entries = BTreeMap::new();
        while let Some(index) = self.next_significant() {
            let line = self.line(index)?.clone();
            if is_root_document_boundary(&line) || line.indent < indent {
                break;
            }
            if line.indent > indent {
                return Err(format!("line {}: unexpected indentation", line.number));
            }
            if sequence_item(&line.content).is_some() {
                return Err(format!("line {}: mixed mapping and sequence", line.number));
            }
            self.cursor += 1;
            self.insert_entry(&mut entries, &line)?;
        }
        if entries.is_empty() {
            Err("empty block mapping is unsupported; use `{}` explicitly".to_owned())
        } else {
            Ok(Value::Mapping(entries))
        }
    }

    /// Decodes sequence items, including the conventional inline mapping form.
    fn sequence(&mut self, indent: usize) -> Result<Value, String> {
        let mut items = Vec::new();
        while let Some(index) = self.next_significant() {
            let line = self.line(index)?.clone();
            if is_root_document_boundary(&line) || line.indent < indent {
                break;
            }
            if line.indent > indent {
                return Err(format!(
                    "line {}: unexpected sequence indentation",
                    line.number
                ));
            }
            let item = sequence_item(&line.content)
                .ok_or_else(|| format!("line {}: mixed sequence and mapping", line.number))?;
            self.cursor += 1;
            let value = if item.is_empty() {
                self.empty_value(indent)?
            } else if lines::mapping_separator(item)?.is_some() {
                self.sequence_mapping(&line)?
            } else {
                lines::parse_inline(item, line.number)?
            };
            items.push(value);
        }
        if items.is_empty() {
            Err("empty block sequence is unsupported; use `[]` explicitly".to_owned())
        } else {
            Ok(Value::Sequence(items))
        }
    }

    /// Uses the number of spaces after a dash as the inline mapping indentation.
    fn sequence_mapping(&mut self, first: &SourceLine) -> Result<Value, String> {
        let parent_indent = first.indent;
        let entry_indent = sequence_mapping_indent(&first.content, parent_indent, first.number)?;
        let mut entries = BTreeMap::new();
        let item = sequence_item(&first.content)
            .ok_or_else(|| format!("line {}: missing sequence item", first.number))?;
        let first_entry = SourceLine {
            number: first.number,
            indent: entry_indent,
            content: item.to_owned(),
            literal: first.literal.clone(),
        };
        self.insert_entry(&mut entries, &first_entry)?;
        while let Some(index) = self.next_significant() {
            let line = self.line(index)?.clone();
            if line.indent <= parent_indent {
                break;
            }
            if line.indent != entry_indent {
                return Err(format!(
                    "line {}: unsupported sequence mapping indentation",
                    line.number
                ));
            }
            if sequence_item(&line.content).is_some() {
                return Err(format!(
                    "line {}: nested sequence mapping is unsupported",
                    line.number
                ));
            }
            self.cursor += 1;
            self.insert_entry(&mut entries, &line)?;
        }
        Ok(Value::Mapping(entries))
    }

    /// Adds one decoded key/value pair after rejecting merges and duplicate keys.
    fn insert_entry(
        &mut self,
        entries: &mut BTreeMap<String, Value>,
        line: &SourceLine,
    ) -> Result<(), String> {
        let separator = lines::mapping_separator(&line.content)?
            .ok_or_else(|| format!("line {}: expected a mapping entry", line.number))?;
        let key_source = line
            .content
            .get(..separator)
            .ok_or_else(|| format!("line {}: invalid mapping key boundary", line.number))?
            .trim();
        let key = match lines::parse_inline(key_source, line.number)? {
            Value::Scalar(value) if !value.is_empty() => value,
            _ => {
                return Err(format!(
                    "line {}: mapping keys must be non-empty scalars",
                    line.number
                ));
            }
        };
        if key == "<<" {
            return Err(format!(
                "line {}: YAML merge keys are unsupported",
                line.number
            ));
        }
        if entries.contains_key(&key) {
            return Err(format!(
                "line {}: duplicate decoded mapping key `{key}`",
                line.number
            ));
        }
        let value_source = line
            .content
            .get(separator + 1..)
            .ok_or_else(|| format!("line {}: invalid mapping value boundary", line.number))?
            .trim();
        let value = self.value(value_source, line.indent, line.number)?;
        entries.insert(key, value);
        Ok(())
    }

    /// Parses an inline value, a nested block, or one supported block scalar.
    fn value(&mut self, source: &str, indent: usize, number: usize) -> Result<Value, String> {
        let value = source.trim();
        if value.is_empty() {
            return self.empty_value(indent);
        }
        if value.starts_with('|') || value.starts_with('>') {
            return self.block_scalar(value, indent, number);
        }
        lines::parse_inline(value, number)
    }

    /// Parses a nested block when the next significant line is deeper than its parent.
    fn empty_value(&mut self, parent_indent: usize) -> Result<Value, String> {
        let Some(index) = self.next_significant() else {
            return Ok(Value::Empty);
        };
        let child_indent = self.line(index)?.indent;
        if child_indent > parent_indent {
            self.block(child_indent)
        } else {
            Ok(Value::Empty)
        }
    }
}

/// Computes a sequence mapping's indentation and rejects uncommon dash spacing.
fn sequence_mapping_indent(content: &str, indent: usize, number: usize) -> Result<usize, String> {
    let rest = content
        .strip_prefix('-')
        .ok_or_else(|| format!("line {number}: missing sequence marker"))?;
    let spaces = rest.len() - rest.trim_start().len();
    if spaces > 1 {
        return Err(format!(
            "line {number}: multiple spaces after a sequence marker are unsupported"
        ));
    }
    Ok(indent + 1 + spaces)
}

/// Returns the content of a sequence item marker, if this line starts with one.
fn sequence_item(content: &str) -> Option<&str> {
    let rest = content.strip_prefix('-')?;
    if rest.is_empty() || rest.chars().next().is_some_and(char::is_whitespace) {
        Some(rest.trim_start())
    } else {
        None
    }
}
