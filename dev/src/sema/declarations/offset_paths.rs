//! Qualified pointer methods bind the exact typed prefix before argument checking.
use super::{Checker, Error, NodeId, NodeKey, Target, Work, regions};
impl Checker<'_> {
    pub(super) fn complete_pointer_prefix(
        &mut self,
        path: (NodeKey, NodeKey, NodeKey),
        definition: crate::resolve::model::DefId,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<(), Error> {
        let (key, prefix, suffix) = path;
        let method =
            super::super::methods::pointers::prepare(self.report, &mut self.facts, definition)
                .map_err(|e| self.error(key, e))?;
        let declaration = self
            .report
            .resolution
            .intrinsics
            .get(
                self.report
                    .resolution
                    .intrinsics
                    .identify(definition)
                    .ok_or_else(|| self.error(key, "missing pointer intrinsic"))?,
            )
            .ok_or_else(|| self.error(key, "missing pointer declaration"))?;
        for id in [Some(definition), declaration.owner, declaration.parameter]
            .into_iter()
            .flatten()
        {
            self.states[id.0 as usize] = super::State::Complete;
        }
        self.method_path_segments(key, prefix, suffix)?;
        self.facts
            .methods
            .retain_pointer_prefix(key, prefix, method.definition)
            .map_err(|e| self.error(key, e))?;
        self.facts
            .node_mut(key)
            .ok_or_else(|| Error::at(key.file, 0, "missing pointer path fact"))?
            .reference = Some(Target::Definition(definition));
        self.work.push(Work::MethodArguments(key, suffix));
        self.schedule_item_arguments(suffix, definition, context, steps, limit, scan)?;
        Ok(())
    }
    pub(super) fn lazy_library_method(&self, id: crate::resolve::model::DefId) -> bool {
        self.report
            .resolution
            .library_types
            .slice_length()
            .is_some_and(|length| length.definition == id)
            || self
                .report
                .resolution
                .intrinsics
                .pointer_method(id)
                .is_some()
    }
}
