//! External pass fixture for the public `gauss::i18n` facade.
//!
//! Construction and lookup use only names re-exported by that facade.

use std::collections::HashMap;

use gauss::i18n::{Catalog, I18nError, Locale, Localizer, MessageId};

fn main() {
    let mut catalogs = HashMap::new();
    catalogs.insert(Locale::en_gb(), Catalog::default_en_gb());

    let localizer = Localizer::with_catalogs(catalogs, Locale::en_gb());
    let draw_result: Result<String, I18nError> =
        localizer.lookup(&Locale::en_gb(), &MessageId::tool_mode_draw());

    assert!(
        matches!(draw_result.as_ref(), Ok(draw) if draw.as_str() == "Draw"),
        "public en-GB lookup returns the Draw label",
    );
}
