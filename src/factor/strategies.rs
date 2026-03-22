mod common_factor;
mod polynomial;

use super::Factorizer;

pub(super) fn install_default_strategies(factorizer: &mut Factorizer) {
    polynomial::install(factorizer);
    common_factor::install(factorizer);
}
