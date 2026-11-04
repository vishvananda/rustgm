//! Configure dependencies only after the entire source owner survives cfg removal.
use super::{
    config::{self, Config},
    configured::{ConfiguredUses, Effect},
    crate_syntax::AttributeEffect,
};
use crate::{
    lex::token::LexError,
    syntax::{ast::Syntax, interpolation::Inputs, link_arguments, token_tree::error},
};

pub(super) fn configure<L>(
    ast: &Syntax<L>,
    effects: &mut [AttributeEffect],
    source: &str,
    config: &Config,
    inputs: &dyn Inputs,
    uses: &mut Option<Box<ConfiguredUses>>,
) -> Result<(), LexError> {
    let mut reordered = false;
    for (index, effect) in effects.iter_mut().enumerate() {
        let forest = &ast.attributes[effect.capture as usize].forest;
        let attribute = effect.attribute.attribute;
        if effect.attribute.effect != Effect::Keep || attribute.name(forest, source) != Some("link")
        {
            continue;
        }
        let mut selected = None;
        for argument in link_arguments::arguments(forest, attribute)? {
            if forest.name(source, argument.start) != Some("cfg") {
                continue;
            }
            if selected.is_some() {
                return Err(error(attribute.span.start, "duplicate link cfg predicate"));
            }
            let range = link_arguments::predicate(forest, argument)?;
            let answer = config::predicate_checked(forest, source, range, config, inputs)?;
            selected = Some(answer.value);
            if !answer.uses.is_empty() {
                let index = u32::try_from(index).map_err(|_| error(0, "cfg effect overflow"))?;
                reordered = true;
                uses.get_or_insert_with(Box::default)
                    .record(index, answer.uses)?;
            }
        }
        if let Some(selected) = selected {
            effect.attribute.effect = Effect::ConditionalLink(selected);
        }
    }
    // Existing owner cfg uses and later-discovered link uses share canonical effects.
    if reordered && let Some(uses) = uses {
        uses.order_by_effect();
    }
    Ok(())
}
