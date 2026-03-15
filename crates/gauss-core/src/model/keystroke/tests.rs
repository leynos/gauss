//! Tests for keystroke module.

use super::*;
use rstest::rstest;

// === Modifiers tests ===

#[test]
fn modifiers_default_has_no_flags() {
    let mods = Modifiers::default();
    assert!(!mods.control);
    assert!(!mods.alt);
    assert!(!mods.shift);
    assert!(!mods.secondary);
    assert!(mods.none());
    assert!(!mods.any());
}

#[test]
fn modifiers_with_secondary_sets_flag() {
    let mods = Modifiers::default().with_secondary();
    assert!(mods.secondary);
    assert!(mods.any());
    assert!(!mods.none());
}

#[test]
fn modifiers_builder_is_chainable() {
    let mods = Modifiers::default()
        .with_secondary()
        .with_shift()
        .with_alt()
        .with_control();
    assert!(mods.control);
    assert!(mods.alt);
    assert!(mods.shift);
    assert!(mods.secondary);
}

// === active_in_order tests ===

#[test]
fn active_in_order_empty_for_no_modifiers() {
    let mods = Modifiers::default();
    let active: Vec<_> = mods.active_in_order().collect();
    assert!(active.is_empty());
}

#[test]
fn active_in_order_returns_modifiers_in_canonical_order() {
    // Set modifiers in non-canonical order to verify ordering
    let mods = Modifiers::default()
        .with_secondary()
        .with_control()
        .with_shift()
        .with_alt();

    let active: Vec<_> = mods.active_in_order().collect();
    assert_eq!(
        active,
        vec![
            Modifier::Control,
            Modifier::Alt,
            Modifier::Shift,
            Modifier::Secondary,
        ]
    );
}

#[test]
fn active_in_order_filters_inactive_modifiers() {
    let mods = Modifiers::default().with_shift().with_secondary();
    let active: Vec<_> = mods.active_in_order().collect();
    assert_eq!(active, vec![Modifier::Shift, Modifier::Secondary]);
}

// === Keystroke construction tests ===

#[test]
fn new_creates_keystroke_without_modifiers() {
    let ks = Keystroke::new("tab");
    assert_eq!(ks.key, "tab");
    assert!(ks.modifiers.none());
}

#[test]
fn secondary_creates_keystroke_with_secondary_modifier() {
    let ks = Keystroke::secondary("z");
    assert_eq!(ks.key, "z");
    assert!(ks.modifiers.secondary);
    assert!(!ks.modifiers.shift);
}

#[test]
fn secondary_shift_creates_keystroke_with_secondary_and_shift() {
    let ks = Keystroke::secondary_shift("z");
    assert_eq!(ks.key, "z");
    assert!(ks.modifiers.secondary);
    assert!(ks.modifiers.shift);
}

#[test]
fn alt_creates_keystroke_with_alt() {
    let ks = Keystroke::alt("f4");
    assert_eq!(ks.key, "f4");
    assert!(ks.modifiers.alt);
}

// === Display tests ===

#[cfg(not(target_os = "macos"))]
#[rstest]
#[case(Keystroke::new("tab"), "TAB")]
#[case(Keystroke::secondary("z"), "Ctrl+Z")]
#[case(Keystroke::secondary_shift("a"), "Shift+Ctrl+A")]
#[case(Keystroke::alt("f4"), "Alt+F4")]
fn display_name_non_macos(#[case] keystroke: Keystroke, #[case] expected: &str) {
    assert_eq!(keystroke.display_name(), expected);
}

#[cfg(not(target_os = "macos"))]
#[test]
fn display_name_other_shows_both_control_and_secondary() {
    // When both control and secondary are set, both should be output
    let mods = Modifiers::default().with_control().with_secondary();
    let ks = Keystroke::with_modifiers("a", mods);
    assert_eq!(ks.display_name(), "Ctrl+Ctrl+A");
}

#[test]
fn keystroke_is_hashable() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Keystroke::new("a"));
    set.insert(Keystroke::secondary("a"));
    assert_eq!(set.len(), 2);
}

#[test]
fn keystroke_equality() {
    let a = Keystroke::secondary("z");
    let b = Keystroke::secondary("z");
    let c = Keystroke::secondary("y");
    assert_eq!(a, b);
    assert_ne!(a, c);
}
