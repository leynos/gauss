//! Shape-level Gauss metadata attribute serialization helpers.

use super::write_fmt;
use crate::model::{GaussAttribute, Shape};
use crate::svg::metadata::shape_id_to_hex;
use slotmap::Key;

pub(super) fn write_shape_gauss_metadata(out: &mut String, shape: &Shape) {
    if !shape.id.is_null() {
        let hex = shape_id_to_hex(shape.id);
        write_fmt(out, format_args!(r#" gauss:id="{hex}""#));
    }
    if let Some(name) = shape.name.as_deref() {
        let escaped_name = escape_attr_value(name);
        write_fmt(out, format_args!(r#" gauss:name="{escaped_name}""#));
    }
    if shape.locked {
        out.push_str(r#" gauss:locked="true""#);
    }
    if shape.hidden {
        out.push_str(r#" gauss:hidden="true""#);
    }
    write_opaque_gauss_attrs(out, &shape.gauss_metadata);
}

fn write_opaque_gauss_attrs(out: &mut String, metadata: &[GaussAttribute]) {
    for attr in metadata {
        let escaped_value = escape_attr_value(&attr.value);
        write_fmt(
            out,
            format_args!(r#" gauss:{}="{escaped_value}""#, attr.name),
        );
    }
}

fn escape_attr_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
