//! External compile-fail fixture for the private implementation path.

use gauss::i18n::catalog::localizer::Localizer;

fn main() {
    let _type_name = std::any::type_name::<Localizer>();
}
