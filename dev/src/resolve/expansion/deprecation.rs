//! Actual selected macro calls produce immutable early lint events once.
use super::{Engine, Error, NodeKey, Report, Sources};
use crate::resolve::{
    lint_policy::{Level, Lint},
    package::Warning,
};

impl<S: Sources> Engine<'_, S> {
    pub(super) fn deprecated_macro(
        &mut self,
        index: usize,
        definition: usize,
    ) -> Result<(), Error> {
        if matches!(self.calls[index].grammar, super::CallGrammar::Derive) {
            return Ok(());
        }
        let Some(target) = self.definitions[definition].node else {
            return Ok(());
        };
        let Some(producer) = self
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.policy.as_ref())
        else {
            return Ok(());
        };
        let Some((origin, note)) = producer.deprecation_in(
            &self.loader.package,
            target,
            self.definitions[definition].module,
        )?
        else {
            return Ok(());
        };
        let key = NodeKey {
            file: self.calls[index].file,
            node: self.calls[index].visit.node,
        };
        producer.diagnostic_read(key, note.len())?;
        let note = note.to_owned();
        let policy = self.lint_call_policy(index)?;
        if policy.deprecated_origin == Some(origin) {
            return Ok(());
        }
        let level = policy.deprecated;
        if level == Level::Expect {
            return self.lint_fulfill_call(index, Lint::Deprecated);
        }
        if level == Level::Allow {
            return Ok(());
        }
        if level.denies() {
            return Err(self.diagnostic(
                self.calls[index].diagnostic(&format!("use of deprecated macro: {note}")),
            ));
        }
        self.attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.policy.as_mut())
            .ok_or_else(|| Error::at(key.file, 0, "macro deprecation producer disappeared"))?
            .early_warning(
                key,
                (target, origin),
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
                .early_warnings(&self.package, |error| self.diagnostic(error))
                .map_err(Error::from)
        })
    }
}
