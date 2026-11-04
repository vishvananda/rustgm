//! Expansion supplies configuration values to the shared syntax-owned grammar.
use super::{Config, Evaluation, Forest, LexError, TreeRange};
pub(super) fn evaluate(
    forest: &Forest,
    source: &str,
    range: TreeRange,
    config: &Config,
    inputs: &dyn crate::syntax::interpolation::Inputs,
    depth: u32,
) -> Result<Evaluation, LexError> {
    let lookup = |name: &str, value: Option<&str>| config.contains(name, value);
    crate::syntax::configuration::evaluate(
        forest,
        source,
        range,
        Some(config.edition()),
        inputs,
        Some(&lookup),
        depth,
    )
}

#[cfg(test)]
#[path = "../../../tests/invariants/forwarded_configuration.rs"]
mod tests;
#[cfg(test)]
#[path = "../../../tests/invariants/configuration_values.rs"]
mod values;
