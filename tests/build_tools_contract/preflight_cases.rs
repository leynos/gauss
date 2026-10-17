//! Named literal inputs for preflight contract cases.
//!
//! Values are kept separate from test operations so each case retains an explicit input record.

use super::Entry;

/// Inputs for one missing or malformed version-pin case.
#[derive(Clone, Copy)]
pub(super) struct PinCase {
    /// The script entry point being checked.
    pub(super) entry: Entry,
    /// The candidate-relative version-pin path.
    pub(super) path: &'static str,
    /// The optional exact file contents; `None` means the file is absent.
    pub(super) contents: Option<&'static str>,
    /// The expected diagnostic fragment.
    pub(super) diagnostic: &'static str,
}

/// Inputs for one rejected toolchain-file case.
#[derive(Clone, Copy)]
pub(super) struct ToolchainRejection {
    /// The exact malformed toolchain file contents.
    pub(super) contents: &'static str,
    /// The expected diagnostic fragment.
    pub(super) diagnostic: &'static str,
}

/// Inputs and expected component inventory for one accepted toolchain case.
#[derive(Clone, Copy)]
pub(super) struct InstalledToolchainCase {
    /// The exact valid toolchain file contents.
    pub(super) contents: &'static str,
    /// The declared toolchain channel.
    pub(super) channel: &'static str,
    /// Additional components requested beyond the standard set.
    pub(super) extras: &'static [&'static str],
    /// The exact installed-component listing supplied to the fake Rustup.
    pub(super) component_output: &'static str,
}

/// `install` with no mold version pin.
pub(super) const INSTALL_MISSING_MOLD: PinCase = PinCase {
    entry: Entry::Install,
    path: "tools/mold/VERSION",
    contents: None,
    diagnostic: "missing version pin",
};

/// `install` with no stable Rust version pin.
pub(super) const INSTALL_MISSING_STABLE: PinCase = PinCase {
    entry: Entry::Install,
    path: "tools/rust-stable/VERSION",
    contents: None,
    diagnostic: "missing version pin",
};

/// `install` with an empty mold version pin.
pub(super) const INSTALL_EMPTY: PinCase = PinCase {
    entry: Entry::Install,
    path: "tools/mold/VERSION",
    contents: Some("\n"),
    diagnostic: "empty version pin",
};

/// `install` with a whitespace-only mold version pin.
pub(super) const INSTALL_WHITESPACE_ONLY: PinCase = PinCase {
    entry: Entry::Install,
    path: "tools/mold/VERSION",
    contents: Some(" \t \n"),
    diagnostic: "empty version pin",
};

/// `install` with multiple mold version-pin lines.
pub(super) const INSTALL_MULTILINE: PinCase = PinCase {
    entry: Entry::Install,
    path: "tools/mold/VERSION",
    contents: Some("2.41.0\n2.41.0\n"),
    diagnostic: "expected one line in version pin",
};

/// `install` with whitespace embedded in the mold version.
pub(super) const INSTALL_EMBEDDED_WHITESPACE: PinCase = PinCase {
    entry: Entry::Install,
    path: "tools/mold/VERSION",
    contents: Some("2.41. 0\n"),
    diagnostic: "version pin contains whitespace",
};

/// `install` with a non-version mold pin.
pub(super) const INSTALL_NON_VERSION_MOLD: PinCase = PinCase {
    entry: Entry::Install,
    path: "tools/mold/VERSION",
    contents: Some("stable\n"),
    diagnostic: "invalid version pin",
};

/// `install` with a non-version stable Rust pin.
pub(super) const INSTALL_NON_VERSION_STABLE: PinCase = PinCase {
    entry: Entry::Install,
    path: "tools/rust-stable/VERSION",
    contents: Some("nightly\n"),
    diagnostic: "invalid version pin",
};

/// `check` with no mold version pin.
pub(super) const CHECK_MISSING_MOLD: PinCase = PinCase {
    entry: Entry::Check,
    path: "tools/mold/VERSION",
    contents: None,
    diagnostic: "missing version pin",
};

/// `check` with no stable Rust version pin.
pub(super) const CHECK_MISSING_STABLE: PinCase = PinCase {
    entry: Entry::Check,
    path: "tools/rust-stable/VERSION",
    contents: None,
    diagnostic: "missing version pin",
};

/// `check` with an empty mold version pin.
pub(super) const CHECK_EMPTY: PinCase = PinCase {
    entry: Entry::Check,
    path: "tools/mold/VERSION",
    contents: Some("\n"),
    diagnostic: "empty version pin",
};

/// `check` with a whitespace-only mold version pin.
pub(super) const CHECK_WHITESPACE_ONLY: PinCase = PinCase {
    entry: Entry::Check,
    path: "tools/mold/VERSION",
    contents: Some(" \t \n"),
    diagnostic: "empty version pin",
};

/// `check` with multiple mold version-pin lines.
pub(super) const CHECK_MULTILINE: PinCase = PinCase {
    entry: Entry::Check,
    path: "tools/mold/VERSION",
    contents: Some("2.41.0\n2.41.0\n"),
    diagnostic: "expected one line in version pin",
};

/// `check` with whitespace embedded in the mold version.
pub(super) const CHECK_EMBEDDED_WHITESPACE: PinCase = PinCase {
    entry: Entry::Check,
    path: "tools/mold/VERSION",
    contents: Some("2.41. 0\n"),
    diagnostic: "version pin contains whitespace",
};

/// `check` with a non-version mold pin.
pub(super) const CHECK_NON_VERSION_MOLD: PinCase = PinCase {
    entry: Entry::Check,
    path: "tools/mold/VERSION",
    contents: Some("stable\n"),
    diagnostic: "invalid version pin",
};

/// `check` with a non-version stable Rust pin.
pub(super) const CHECK_NON_VERSION_STABLE: PinCase = PinCase {
    entry: Entry::Check,
    path: "tools/rust-stable/VERSION",
    contents: Some("nightly\n"),
    diagnostic: "invalid version pin",
};

/// A toolchain file without a channel.
pub(super) const MISSING_CHANNEL: ToolchainRejection = ToolchainRejection {
    contents: "[toolchain]\ncomponents=[]\n",
    diagnostic: "expected one valid channel",
};

/// A toolchain file with a duplicate channel key.
pub(super) const DUPLICATE_CHANNEL: ToolchainRejection = ToolchainRejection {
    contents: "[toolchain]\nchannel = \"1.92.0\"\nchannel=\"1.92.0\"\n",
    diagnostic: "expected one valid channel",
};

/// A toolchain file with an unquoted channel.
pub(super) const UNQUOTED_CHANNEL: ToolchainRejection = ToolchainRejection {
    contents: "[toolchain]\nchannel=1.92.0\n",
    diagnostic: "expected one valid channel",
};

/// A toolchain file with a non-version channel.
pub(super) const NON_VERSION_CHANNEL: ToolchainRejection = ToolchainRejection {
    contents: "[toolchain]\nchannel=\"stable\"\n",
    diagnostic: "expected one valid channel",
};

/// A toolchain file with duplicate component keys.
pub(super) const DUPLICATE_COMPONENTS: ToolchainRejection = ToolchainRejection {
    contents: "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[]\ncomponents=[]\n",
    diagnostic: "could not read components",
};

/// A toolchain component array with an invalid prefix.
pub(super) const STRAY_PREFIX: ToolchainRejection = ToolchainRejection {
    contents: "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[prefix \"rustfmt\"]\n",
    diagnostic: "could not read components",
};

/// A toolchain component array with adjacent strings.
pub(super) const ADJACENT_STRINGS: ToolchainRejection = ToolchainRejection {
    contents: "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\"rustfmt\" \"clippy\"]\n",
    diagnostic: "could not read components",
};

/// A toolchain component array missing a comma.
pub(super) const MISSING_COMMA: ToolchainRejection = ToolchainRejection {
    contents: "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\n\"rustfmt\"\n\"clippy\"\n]\n",
    diagnostic: "could not read components",
};

/// A toolchain component array with a duplicate comma.
pub(super) const DOUBLE_COMMA: ToolchainRejection = ToolchainRejection {
    contents: "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\"rustfmt\",,\"clippy\"]\n",
    diagnostic: "could not read components",
};

/// A toolchain component with forbidden whitespace.
pub(super) const INVALID_COMPONENT: ToolchainRejection = ToolchainRejection {
    contents: "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\"rust src\"]\n",
    diagnostic: "could not read components",
};

/// A toolchain component array with no closing bracket.
pub(super) const UNTERMINATED_ARRAY: ToolchainRejection = ToolchainRejection {
    contents: "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\"rustfmt\",\n",
    diagnostic: "could not read components",
};

/// A toolchain component array followed by invalid trailing text.
pub(super) const BAD_CLOSE_TAIL: ToolchainRejection = ToolchainRejection {
    contents: "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\"rustfmt\"] garbage\n",
    diagnostic: "could not read components",
};

/// A toolchain file with a duplicate toolchain table.
pub(super) const DUPLICATE_TOOLCHAIN_TABLE: ToolchainRejection = ToolchainRejection {
    contents: "[toolchain]\nchannel = \"1.92.0\"\n[toolchain]\n",
    diagnostic: "expected one valid channel",
};

/// A toolchain file followed by an unterminated table header.
pub(super) const LATER_HEADER_MISSING_CLOSE: ToolchainRejection = ToolchainRejection {
    contents: "[toolchain]\nchannel = \"1.92.0\"\n[toolchain\n",
    diagnostic: "expected one valid channel",
};

/// A toolchain file followed by a table header with trailing text.
pub(super) const LATER_HEADER_TRAILING_GARBAGE: ToolchainRejection = ToolchainRejection {
    contents: "[toolchain]\nchannel = \"1.92.0\"\n[toolchain] trailing\n",
    diagnostic: "expected one valid channel",
};

/// A valid empty component list with the standard installed inventory.
pub(super) const EMPTY_ARRAY: InstalledToolchainCase = InstalledToolchainCase {
    contents: "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[]\n",
    channel: "1.92.0",
    extras: &[],
    component_output: "rustfmt (installed)\nclippy (installed)\nrust-analyzer (installed)\n",
};

/// A valid component list with duplicate standard entries.
pub(super) const INLINE_DUPLICATES: InstalledToolchainCase = InstalledToolchainCase {
    contents: "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\"rust-analyzer\", \"rustfmt\", \"clippy\", \"rustfmt\"]\n",
    channel: "1.92.0",
    extras: &[],
    component_output: "rustfmt (installed)\nclippy (installed)\nrust-analyzer (installed)\n",
};

/// A valid multiline component list with a trailing comma.
pub(super) const MULTILINE_TRAILING_COMMA: InstalledToolchainCase = InstalledToolchainCase {
    contents: "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\n\"rustfmt\",\n\"rust-src\",\n]\n",
    channel: "1.92.0",
    extras: &["rust-src"],
    component_output: "rustfmt (installed)\nclippy (installed)\nrust-analyzer (installed)\nrust-src (installed)\n",
};

/// A valid component list containing the LLVM tools preview alias.
pub(super) const EXTRA_AND_LLVM_ALIAS: InstalledToolchainCase = InstalledToolchainCase {
    contents: "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[\"rust-src\", \"llvm-tools-preview\", \"rust-src\"]\n",
    channel: "1.92.0",
    extras: &["rust-src", "llvm-tools-preview"],
    component_output: "rustfmt (installed)\nclippy (installed)\nrust-analyzer (installed)\nrust-src (installed)\nllvm-tools (installed)\n",
};

/// A valid toolchain file with unrelated later tables ignored.
pub(super) const VALID_UNRELATED_TABLE_AFTER_TOOLCHAIN: InstalledToolchainCase =
    InstalledToolchainCase {
        contents: "[toolchain]\nchannel = \"1.92.0\"\ncomponents=[]\n[package.metadata]\nchannel=\"ignored\"\n[[targets.windows]]\nname=\"ignored\"\n",
        channel: "1.92.0",
        extras: &[],
        component_output: "rustfmt (installed)\nclippy (installed)\nrust-analyzer (installed)\n",
    };
