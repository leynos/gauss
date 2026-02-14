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

use crate::model::{Document, PaintStyle, Shape};
use crate::svg::metadata;
use crate::svg::metadata::NamespacePolicyError;

mod gauss_attrs;
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
    /// Raw content of the `<metadata>` block from the imported SVG, if present.
    pub gauss_metadata_block: Option<String>,
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
    /// The shape index exceeds the maximum representable z-index.
    ShapeIndexOverflow,
    /// A paint reference points to a missing resource.
    MissingReferencedResource(String),
    /// `gauss` prefix points to the wrong namespace URI.
    InvalidGaussNamespaceBinding(String),
    /// Gauss metadata was used without canonical `xmlns:gauss` declaration.
    MissingGaussNamespaceDeclaration,
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
            Self::ShapeIndexOverflow => write!(f, "shape index exceeds maximum z-index"),
            Self::MissingReferencedResource(id) => {
                write!(f, "paint references missing resource id '{id}'")
            }
            Self::InvalidGaussNamespaceBinding(found) => {
                write!(f, "invalid Gauss namespace binding '{found}'")
            }
            Self::MissingGaussNamespaceDeclaration => write!(
                f,
                "Gauss metadata requires canonical xmlns:gauss namespace declaration"
            ),
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
    // Namespace policy currently runs as an XML preflight check. If import
    // profiling shows the duplicate parse as hot, fold this into a single
    // shared XML parse path.
    metadata::validate_namespace_policy(svg).map_err(map_namespace_policy_error)?;

    let mut resources = crate::model::ResourceStore::new();
    resource_tags::parse_resources(resource_tags::SvgContent::from(svg), &mut resources)?;

    let shape_gauss_meta = gauss_attrs::extract_shape_gauss_metadata(svg);
    let gauss_metadata_block = gauss_attrs::extract_metadata_block(svg);

    let mut doc = Document::new();

    let raw_tags = resource_tags::extract_shape_path_tags(resource_tags::SvgContent::from(svg));

    for (index, raw_tag) in raw_tags.iter().enumerate() {
        let gauss_meta = shape_gauss_meta
            .iter()
            .find(|meta| svg.get(meta.byte_range.clone()) == Some(raw_tag.as_str()));
        let shape = parse_shape_from_tag(raw_tag, index, &resources, gauss_meta)?;
        doc.append_shape(shape);
    }

    Ok(ImportedSvg {
        document: doc,
        resources,
        gauss_metadata_block,
    })
}

fn parse_shape_from_tag(
    raw_tag: &str,
    index: usize,
    resources: &crate::model::ResourceStore,
    gauss_meta: Option<&gauss_attrs::ShapeGaussMetadata>,
) -> Result<Shape, SvgImportError> {
    let tag_content = resource_tags::SvgContent::from(raw_tag);
    let d = resource_tags::attribute_value(tag_content, resource_tags::AttributeName::from("d"))
        .ok_or(SvgImportError::MissingPathData)?;

    let stroke = resource_tags::parse_paint_with_opacity(
        tag_content,
        resource_tags::AttributeName::from("stroke"),
        resource_tags::AttributeName::from("stroke-opacity"),
        resources,
    )?;
    let fill = resource_tags::parse_paint_with_opacity(
        tag_content,
        resource_tags::AttributeName::from("fill"),
        resource_tags::AttributeName::from("fill-opacity"),
        resources,
    )?;

    let stroke_width = resource_tags::attribute_value(
        tag_content,
        resource_tags::AttributeName::from("stroke-width"),
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

    let shape_id = gauss_meta
        .and_then(|meta| meta.gauss_id.as_deref())
        .and_then(metadata::shape_id_from_hex)
        .unwrap_or_default();

    Ok(Shape {
        id: shape_id,
        z: index
            .try_into()
            .map_err(|_| SvgImportError::ShapeIndexOverflow)?,
        style,
        path,
        name: gauss_meta.and_then(|meta| meta.name.clone()),
        locked: gauss_meta.is_some_and(|meta| meta.locked),
        hidden: gauss_meta.is_some_and(|meta| meta.hidden),
        gauss_metadata: gauss_meta
            .map(|meta| meta.opaque_attrs.clone())
            .unwrap_or_default(),
    })
}

fn map_namespace_policy_error(error: NamespacePolicyError) -> SvgImportError {
    match error {
        NamespacePolicyError::InvalidXml => SvgImportError::MalformedSvg,
        NamespacePolicyError::InvalidGaussNamespaceBinding(found) => {
            SvgImportError::InvalidGaussNamespaceBinding(found)
        }
        NamespacePolicyError::MissingGaussNamespaceDeclaration => {
            SvgImportError::MissingGaussNamespaceDeclaration
        }
    }
}
