//! An authenticated source builtin selects real parsed syntax in its caller unit.
use super::{
    Body, Builtin, Engine, Error, ExpansionId, Input, Invocation, MacroContext, Sources, Visit,
};
use crate::{
    expand::builtin::choice,
    resolve::{
        bindings::{Source, panic_contracts},
        unit::NodeKey,
    },
    syntax::token_tree::error,
};
impl<S: Sources> Engine<'_, S> {
    pub(super) fn invoke_choice(
        &mut self,
        visit: Visit,
        grammar: MacroContext,
        definition: usize,
        context: ExpansionId,
        input: &Input,
    ) -> Result<ExpansionId, Error> {
        self.invoke_typed_builtin(visit, grammar, definition, context, input, false)
    }
    pub(super) fn invoke_pattern(
        &mut self,
        visit: Visit,
        grammar: MacroContext,
        definition: usize,
        context: ExpansionId,
        input: &Input,
    ) -> Result<ExpansionId, Error> {
        self.invoke_typed_builtin(visit, grammar, definition, context, input, true)
    }
    fn invoke_typed_builtin(
        &mut self,
        visit: Visit,
        grammar: MacroContext,
        definition: usize,
        context: ExpansionId,
        input: &Input,
        pattern: bool,
    ) -> Result<ExpansionId, Error> {
        let file = self
            .loader
            .package
            .modules
            .get(visit.module.0 as usize)
            .ok_or_else(|| Error::unlocated("missing cfg_select caller module"))?
            .file;
        let unit = self.loader.package.unit(file);
        let actual = unit
            .configured
            .ast
            .nodes
            .get(visit.node.get() as usize)
            .ok_or_else(|| Error::at(file, 0, "missing cfg_select invocation"))?;
        let original = actual.span;
        let failure = |message| Error::at(file, original.start, message);
        checked_contract(
            &self.loader.package,
            &self.paths,
            &self.definitions,
            definition,
            if pattern {
                Builtin::PatternType
            } else {
                Builtin::CfgSelect
            },
        )?;
        check_invocation(
            &self.paths,
            self.invocations.len(),
            context,
            visit.introduction,
        )
        .map_err(failure)?;
        let Some(Source::Core(source)) = self.loader.package.source.as_deref() else {
            return Err(failure("missing actual cfg_select source provider"));
        };
        source
            .macros
            .charge(1, size_of::<Invocation>().saturating_mul(4))
            .map_err(failure)?;
        self.invocations
            .try_reserve(1)
            .map_err(|_| failure("cfg_select invocation allocation failed"))?;
        // No RefCell borrow is held across the parser or configuration callbacks.
        let charge = |steps, bytes| {
            source
                .macros
                .charge(steps, bytes)
                .map_err(|message| error(original.start, message))
        };
        let (before, tail) = self.loader.package.units.split_at_mut(file.0 as usize);
        let (unit, after) = tail
            .split_first_mut()
            .ok_or_else(|| failure("missing cfg_select unit"))?;
        let prepared = unit
            .prepare_builtin_syntax(
                choice::Site {
                    file,
                    target: visit.node,
                    original,
                    grammar,
                },
                input,
                &self.captures,
                |id| {
                    if id.0 < file.0 {
                        before.get(id.0 as usize)
                    } else {
                        after.get((id.0 - file.0 - 1) as usize)
                    }
                },
                self.config,
                &charge,
                pattern,
            )
            .map_err(|error| Error::at(file, error.offset, error.message))?;
        if pattern {
            self.check_pattern_gates(file, original)?;
        }
        self.check_cfg_uses(file, prepared.cfg_uses())?;
        let installed = self.loader.package.units[file.0 as usize]
            .install_prepared_choice(prepared)
            .map_err(|error| Error::at(file, error.offset, error.message))?;
        self.paths
            .invocation(context, visit.introduction, &self.definitions[definition])
            .map_err(failure)?;
        self.invocations.push(Invocation {
            context,
            node: NodeKey {
                file,
                node: visit.node,
            },
            original,
            definition,
            rule: None,
            generated: installed.generated,
            parser_calls: installed.parser_calls,
            parent: visit.introduction,
        });
        // The selected tokens keep their original raw contexts. No definition
        // mark, new opaque payload or ancestor allowance is manufactured here.
        Ok(context)
    }
}

impl<S: Sources> Engine<'_, S> {
    fn check_pattern_gates(
        &mut self,
        file: super::FileId,
        span: crate::syntax::token_tree::Span,
    ) -> Result<(), Error> {
        let package = &self.loader.package;
        let view = super::permission_frames::FrameView {
            package,
            definitions: &self.definitions,
            invocations: &self.invocations,
            facts: &self.paths.state.facts,
        };
        let mut query = super::permission_session::Query {
            view,
            memo: &mut self.cfg_memo,
        };
        for (syntax, name) in [(false, "pattern_type_macro"), (true, "pattern_types")] {
            if !package
                .requested_pattern_feature(syntax)
                .map_err(Error::unlocated)?
                && !query.allows(
                    super::Location { file, span },
                    &crate::resolve::bindings::FeatureName::new(name).map_err(Error::unlocated)?,
                )?
            {
                return Err(Error::at(
                    file,
                    span.start,
                    if syntax {
                        "pattern types require pattern_types or actual literal allowance"
                    } else {
                        "pattern_type requires pattern_type_macro or actual literal allowance"
                    },
                ));
            }
        }
        Ok(())
    }
}

fn checked_contract(
    package: &super::Package,
    paths: &super::paths::Paths,
    definitions: &[super::Definition],
    index: usize,
    role: Builtin,
) -> Result<NodeKey, Error> {
    let failure = |message: &'static str| Error::unlocated(message);
    let entry = definitions
        .get(index)
        .ok_or_else(|| failure("unknown cfg_select definition"))?;
    if !matches!(entry.body, Body::Builtin(actual) if actual == role) {
        return Err(failure("cfg_select changed its actual selected role"));
    }
    let owner = paths.checked_source_macro(index, entry).map_err(failure)?;
    let contract = panic_contracts::published(package, owner, entry.header.as_ref(), 1)?;
    if contract.node != owner
        || contract.builtin != Some(role)
        || contract.edition != entry.edition
        || contract.allows(crate::resolve::bindings::KnownFeature::EditionPanic)
            != entry.edition_panic
    {
        return Err(failure(
            "cfg_select differs from its actual defining contract",
        ));
    }
    if package
        .modules
        .get(entry.module.0 as usize)
        .is_none_or(|module| module.file != owner.file)
    {
        return Err(failure("cfg_select defining module changed"));
    }
    if package
        .source_core()
        .is_none_or(|source| source.current != source.provider)
    {
        return Err(failure(
            "cfg_select requires its actual local source provider",
        ));
    }
    Ok(owner)
}

fn check_invocation(
    paths: &super::paths::Paths,
    count: usize,
    context: ExpansionId,
    parent: ExpansionId,
) -> Result<(), &'static str> {
    let expected = u32::try_from(count)
        .ok()
        .and_then(|count| count.checked_add(1));
    let previous = ExpansionId(context.0.saturating_sub(1));
    let introduced = &paths.state.facts.introductions;
    if expected != Some(context.0)
        || !introduced.ancestry.ancestor(previous, previous)
        || introduced.ancestry.ancestor(context, context)
        || !introduced.ancestry.ancestor(parent, parent)
        || (previous.0 != 0 && introduced.definition_scope(previous).is_err())
        || introduced.definition_scope(context).is_ok()
    {
        return Err("cfg_select changed its actual invocation identity");
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/chosen_authority.rs"]
mod tests;
