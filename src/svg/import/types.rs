//! Domain-specific string wrappers for SVG import parsing.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TagName<'a>(&'a str);

impl<'a> TagName<'a> {
    pub(super) const fn new(inner: &'a str) -> Self {
        Self(inner)
    }

    pub(super) const fn as_str(&self) -> &'a str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AttributeName<'a>(&'a str);

impl<'a> AttributeName<'a> {
    pub(super) const fn new(inner: &'a str) -> Self {
        Self(inner)
    }

    pub(super) const fn as_str(&self) -> &'a str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SvgContent<'a>(&'a str);

impl<'a> SvgContent<'a> {
    pub(super) const fn new(inner: &'a str) -> Self {
        Self(inner)
    }

    pub(super) const fn as_str(&self) -> &'a str {
        self.0
    }
}
