mod algebra;
mod builtin;
mod polynomial;
mod rational;

use crate::rewrite::RewriteEngine;

pub(super) fn install_default_rules(engine: &mut RewriteEngine) {
    algebra::install(engine);
    rational::install(engine);
    polynomial::install(engine);
    builtin::install(engine);
}
