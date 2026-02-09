//! SVG import for the Gauss document model.
//!
//! This importer round-trips SVG emitted by
//! `crate::svg::export::export_svg_with_resources` and supports a practical
//! subset of user-authored files.

#![expect(
    clippy::float_arithmetic,
    reason = "SVG import requires floating-point parsing and scaling"
)]

use std::{error::Error, fmt};

use crate::model::{Document, PaintStyle, Shape, ShapeId};

mod path_data;
mod resource_tag_attributes;
mod resource_tags;
mod types;

/// Imported SVG state containing document geometry and shared resources.
#[derive(Clone, Debug, PartialEq)]
pub struct ImportedSvg {
    /// Imported document geometry.
    pub document: Document,
    /// Imported shared resources.
    pub resources: crate::model::ResourceStore,
}

/// Errors returned by [`import_svg`] and [`import_svg_with_resources`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SvgImportError {
    /// The input did not contain valid UTF-8 XML-like data we can parse.
    MalformedSvg,
    /// A `<path>` element was missing its required `d` attribute.
    MissingPathData,
    /// A path `d` attribute contained an unsupported command.
    UnsupportedPathCommand(char),
    /// A path `d` attribute was syntactically invalid.
    InvalidPathData,
    /// A colour or paint attribute was present but not understood.
    InvalidColour,
    /// A numeric attribute was present but not understood.
    InvalidNumber,
    /// An opacity attribute was present but not understood.
    InvalidOpacity,
    /// A paint reference points to a missing resource.
    MissingReferencedResource(String),
}

impl fmt::Display for SvgImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedSvg => write!(f, "malformed SVG"),
            Self::MissingPathData => write!(f, "missing path data attribute"),
            Self::UnsupportedPathCommand(cmd) => write!(f, "unsupported path command: {cmd}"),
            Self::InvalidPathData => write!(f, "invalid path data"),
            Self::InvalidColour => write!(f, "invalid colour"),
            Self::InvalidNumber => write!(f, "invalid number"),
            Self::InvalidOpacity => write!(f, "invalid opacity"),
            Self::MissingReferencedResource(id) => {
                write!(f, "paint references missing resource id '{id}'")
            }
        }
    }
}

impl Error for SvgImportError {}

/// Import an SVG string as a [`Document`].
///
/// This compatibility helper imports shared resources but returns only document
/// geometry.
///
/// # Errors
///
/// Returns an [`SvgImportError`] for malformed SVG, unsupported commands, or
/// unsupported/invalid style attributes.
pub fn import_svg(svg: &str) -> Result<Document, SvgImportError> {
    import_svg_with_resources(svg).map(|imported| imported.document)
}

/// Import an SVG string including shared resources.
///
/// # Errors
///
/// Returns an [`SvgImportError`] for malformed SVG, unsupported commands, or
/// invalid references to resources.
pub fn import_svg_with_resources(svg: &str) -> Result<ImportedSvg, SvgImportError> {
    let mut resources = crate::model::ResourceStore::new();
    resource_tags::parse_resources(resource_tags::SvgContent::new(svg), &mut resources)?;

    let mut doc = Document::new();

    for (index, raw_tag) in resource_tags::extract_single_tags(
        resource_tags::SvgContent::new(svg),
        resource_tags::TagName::new("path"),
    )
    .into_iter()
    .enumerate()
    {
        let tag_content = resource_tags::SvgContent::new(raw_tag.as_str());
        let d = resource_tags::attribute_value(tag_content, resource_tags::AttributeName::new("d"))
            .ok_or(SvgImportError::MissingPathData)?;

        let stroke = resource_tags::parse_paint_with_opacity(
            tag_content,
            resource_tags::AttributeName::new("stroke"),
            resource_tags::AttributeName::new("stroke-opacity"),
            &resources,
        )?;
        let fill = resource_tags::parse_paint_with_opacity(
            tag_content,
            resource_tags::AttributeName::new("fill"),
            resource_tags::AttributeName::new("fill-opacity"),
            &resources,
        )?;

        let stroke_width = resource_tags::attribute_value(
            tag_content,
            resource_tags::AttributeName::new("stroke-width"),
        )
        .map(|value| {
            value
                .parse::<f32>()
                .map_err(|_| SvgImportError::InvalidNumber)
        })
        .transpose()?
        .unwrap_or(1.0);

        let path = path_data::parse_path_data(&d)?;
        let style = PaintStyle::new_with_paint(stroke, stroke_width, fill);

        let shape = Shape {
            id: ShapeId::default(),
            z: index.try_into().map_err(|_| SvgImportError::MalformedSvg)?,
            style,
            path,
        };
        doc.append_shape(shape);
    }

    Ok(ImportedSvg {
        document: doc,
        resources,
    })
}
