//! Represents reviewed source pairs and validates complete Git blob identifiers.

use camino::Utf8Path;

/// A lowercase, complete Git blob identifier that has passed format validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct GitBlobId {
    /// The exact 40 lowercase ASCII hexadecimal characters returned by Git.
    text: String,
}

impl GitBlobId {
    /// Parses the exact lowercase hexadecimal representation emitted by Git.
    ///
    /// For example, 40 lowercase hexadecimal characters are accepted, while uppercase
    /// characters return `git hash-object returned an invalid blob identifier`.
    pub(super) fn parse(text: &str) -> Result<Self, String> {
        let has_valid_format = text.len() == 40
            && text
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'));
        if has_valid_format {
            Ok(Self {
                text: text.to_owned(),
            })
        } else {
            Err("git hash-object returned an invalid blob identifier".to_owned())
        }
    }
}

/// Pairs a repository-relative path with raw expected blob text for deferred validation.
///
/// For example, a registry entry keeps `scripts/check.py` beside the identifier that is
/// checked only after the source has been read and its observed blob has been validated.
#[derive(Clone, Copy, Debug)]
pub(super) struct ReviewedSource<'a> {
    /// Repository-relative UTF-8 path opened through the workspace capability.
    pub(super) path: &'a str,
    /// Expected Git blob text, kept raw so validation cannot preempt source or process errors.
    pub(super) expected_blob: &'a str,
}

impl ReviewedSource<'_> {
    /// Interprets the stored repository path at the UTF-8 capability boundary.
    ///
    /// For example, `scripts/check.py` becomes the path passed to `Dir::read_to_string`.
    pub(super) fn path(&self) -> &Utf8Path {
        Utf8Path::new(self.path)
    }
}
