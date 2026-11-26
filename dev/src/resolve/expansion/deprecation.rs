//! Actual selected macro calls produce immutable early lint events once.
use super::{Engine, Error, NodeKey, Report, Sources, transported_macros};
use crate::resolve::{
    lint_policy::{DeprecationNotice, EarlyDeprecation, Level, Lint, transported_active},
    model::Resolution,
    package::Warning,
};

pub(super) fn imported_notice(facts: &Resolution, index: usize) -> Option<DeprecationNotice<'_>> {
    let imported = transported_macros::imported(facts, index)?;
    let notice = facts
        .crates
        .as_deref()?
        .dependencies()
        .artifacts()
        .get(imported.artifact)?
        .macros()?
        .deprecation(imported.row)?;
    let future = !transported_active(&notice.since, notice.staged);
    let note = notice.note.as_deref().unwrap_or(if future {
        "no deprecation reason supplied"
    } else {
        "use of deprecated declaration"
    });
    Some(DeprecationNotice { note, future })
}

impl<S: Sources> Engine<'_, S> {
    fn selected_deprecation(
        &self,
        definition: usize,
    ) -> Result<Option<(EarlyDeprecation, bool)>, Error> {
        let definition_data = &self.definitions[definition];
        if let Some(target) = definition_data.node {
            let Some(producer) = self
                .attribute_effects
                .as_deref()
                .and_then(|effects| effects.policy.as_ref())
            else {
                return Ok(None);
            };
            return Ok(producer
                .deprecation_in(&self.loader.package, target, definition_data.module)?
                .map(|(origin, notice)| (EarlyDeprecation::Local(target, origin), notice.future)));
        }
        Ok(imported_notice(&self.paths.state.facts, definition)
            .map(|notice| (EarlyDeprecation::Imported(definition), notice.future)))
    }

    pub(super) fn deprecated_macro(
        &mut self,
        index: usize,
        definition: usize,
    ) -> Result<(), Error> {
        if matches!(self.calls[index].grammar, super::CallGrammar::Derive) {
            return Ok(());
        }
        let key = Self::visit_key(self.calls[index].visit);
        if self.loader.package.lint_cap().suppresses_lints() {
            if let Some(target) = self.definitions[definition].node
                && let Some(producer) = self
                    .attribute_effects
                    .as_deref()
                    .and_then(|effects| effects.policy.as_ref())
            {
                producer.validate_cap(&self.loader.package, target)?;
            }
            return Ok(());
        }
        let Some((target, future)) = self.selected_deprecation(definition)? else {
            return Ok(());
        };
        self.ensure_attribute_policy(key)?;
        let policy = self.lint_call_policy(index)?;
        if let EarlyDeprecation::Local(_, origin) = target
            && policy.deprecated_origin == Some(origin)
        {
            return Ok(());
        }
        let (level, lint) = if future {
            (policy.deprecated_in_future, Lint::DeprecatedInFuture)
        } else {
            (policy.deprecated, Lint::Deprecated)
        };
        if level == Level::Expect {
            return self.lint_fulfill_call(index, lint);
        }
        if level == Level::Allow {
            return Ok(());
        }
        let producer = self
            .attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.policy.as_mut())
            .ok_or_else(|| Error::at(key.file, 0, "macro deprecation producer disappeared"))?;
        if level.denies() {
            let notice = target.notice(key, producer, &self.loader.package, |index| {
                imported_notice(&self.paths.state.facts, index)
            })?;
            producer.diagnostic_read(key, notice.note.len())?;
            let message = notice.subject("macro");
            return Err(self.diagnostic(self.calls[index].diagnostic(&message)));
        }
        producer.early_warning(
            key,
            target,
            self.calls[index].offset,
            self.calls[index].visit.introduction,
        )?;
        Ok(())
    }
}
impl Report {
    pub(crate) fn macro_deprecation_warnings(&self) -> Result<Vec<(NodeKey, Warning)>, Error> {
        self.lint_producer().map_or(Ok(Vec::new()), |producer| {
            producer
                .early_warnings(
                    &self.package,
                    |index| imported_notice(&self.resolution, index),
                    |error| self.diagnostic(error),
                )
                .map_err(Error::from)
        })
    }
}
