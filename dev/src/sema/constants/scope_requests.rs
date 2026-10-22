//! Source pattern requests keep declared binders while dependent values select full instances.
use super::{
    Error, InitializerKind, PatternValue, Queries, Query, Value, Work, instances::Context,
    pending_patterns::FrameId,
};
use crate::{resolve::unit::NodeKey, sema::patterns::PatternId};
impl Queries<'_> {
    pub(super) fn prepare_pattern_requirement(
        &mut self,
        requirement: super::values::execution::Requirement,
    ) -> Result<bool, Error> {
        use super::values::execution::Requirement;
        let source = match requirement {
            Requirement::SourceAssociatedBody(source) => Some(
                source
                    .identity(self.typing.facts)
                    .map_err(|e| Error::at(super::FileId(0), 0, e))?
                    .1,
            ),
            Requirement::DiscriminantValue(variant) => {
                super::values::Reader::new(self.typing.facts)
                    .discriminant(variant)
                    .map_err(|e| Error::at(super::FileId(0), 0, e))?;
                self.work.push(Work::Enter(Query::Variant(variant)));
                return Ok(true);
            }
            Requirement::FunctionBody(instance) => {
                let Some(crate::types::Kind::FunctionItem { item, .. }) =
                    self.typing.facts.types.kind(instance.ty())
                else {
                    return Ok(false);
                };
                self.typing
                    .facts
                    .function_indices
                    .get(&crate::resolve::model::DefId(item.0))
                    .and_then(|&index| self.typing.facts.functions.get(index))
                    .map(|function| function.body.unwrap_or(function.node))
            }
            Requirement::InitializerBody(definition)
            | Requirement::AssociatedBody {
                instance: crate::sema::values::Instance { definition, .. },
                ..
            } => self
                .typing
                .facts
                .initializer_index(definition)
                .and_then(|index| self.typing.facts.initializers.get(index))
                .map(crate::sema::model::Initializer::semantic_source),
            Requirement::InlineBody { source, .. } => Some(source),
            Requirement::AnonymousBody { instance, .. } => Some(instance.source()),
        };
        if let Some(source) = source {
            self.schedule_pattern_owner(source, None)
        } else {
            Ok(false)
        }
    }
    pub(super) fn pattern_query(
        &mut self,
        key: NodeKey,
        definition: PatternValue,
    ) -> Result<Query, Error> {
        match definition {
            PatternValue::Definition(_) if self.typing.facts.constant_uses.contains_key(&key) => {
                let instance = self.typing.facts.constant_uses[&key];
                Ok(Query::Instance(self.selected_constant(key, instance)?))
            }
            PatternValue::Definition(definition)
                if self
                    .typing
                    .facts
                    .crates
                    .as_deref()
                    .is_some_and(|c| c.imported_constant(definition)) =>
            {
                Ok(Query::Node(key))
            }
            PatternValue::Definition(definition) => {
                let index = self
                    .typing
                    .facts
                    .initializer_index(definition)
                    .ok_or_else(|| {
                        crate::sema::model::Declarations::error(
                            self.report,
                            key,
                            "pattern path requires a named constant",
                        )
                    })?;
                if !matches!(
                    self.typing.facts.initializers[index].kind,
                    InitializerKind::Const
                ) {
                    return Err(crate::sema::model::Declarations::error(
                        self.report,
                        key,
                        "a static is not a constant pattern",
                    ));
                }
                Ok(Query::Constant(index))
            }
            PatternValue::Literal(source) => Ok(Query::Node(source)),
        }
    }
    pub(super) fn publish_pattern_value(
        &mut self,
        key: NodeKey,
        pattern: Option<PatternId>,
        value: Value,
    ) -> Result<(), Error> {
        if let Some(pattern) = pattern {
            self.complete_pattern(key, value)?;
            return super::patterns::expand(self.report, self.typing.facts, pattern, value);
        }
        let value = value
            .scalar()
            .map_err(|e| crate::sema::model::Declarations::error(self.report, key, e))?;
        self.typing
            .facts
            .node_mut(key)
            .ok_or_else(|| {
                crate::sema::model::Declarations::error(
                    self.report,
                    key,
                    "missing constant pattern node",
                )
            })?
            .scalar = Some(value);
        Ok(())
    }
    pub(super) fn next_pattern_request(&mut self, id: FrameId) -> Result<(), Error> {
        self.resume_pattern_dispatch()?;
        let frame = self
            .pattern_frames
            .as_mut()
            .ok_or_else(|| Error::at(super::FileId(0), 0, "missing pattern frames"))?
            .frame(id)
            .map_err(|e| Error::at(super::FileId(0), 0, e))?;
        let request = frame
            .prepared
            .as_ref()
            .ok_or_else(|| Error::at(super::FileId(0), 0, "missing prepared pattern scope"))?
            .request(frame.cursor);
        let Some((key, definition, _)) = request else {
            self.work.push(Work::PatternFinish(id));
            return Ok(());
        };
        // Pattern validity is a property of the original generic declaration.
        // A concrete enclosing value query cannot supply a forbidden T::N value.
        self.work.context = Context::default();
        let query = self.pattern_query(key, definition)?;
        self.reserve_pattern_dispatch()?;
        self.check_pattern_storage(0)?;
        self.work.push(Work::PatternPublish(id, query));
        self.work.push(Work::Enter(query));
        if let Query::Node(source) = query {
            self.work.push(Work::PrepareNode(source));
        }
        Ok(())
    }
    pub(super) fn publish_pattern_request(
        &mut self,
        id: FrameId,
        query: Query,
    ) -> Result<(), Error> {
        let frame = self
            .pattern_frames
            .as_mut()
            .ok_or_else(|| Error::at(super::FileId(0), 0, "missing pattern frames"))?
            .frame(id)
            .map_err(|e| Error::at(super::FileId(0), 0, e))?;
        let (key, _, pattern) = frame
            .prepared
            .as_ref()
            .and_then(|p| p.request(frame.cursor))
            .ok_or_else(|| Error::at(super::FileId(0), 0, "missing current pattern request"))?;
        frame.cursor += 1;
        let value = self.value(query)?;
        self.publish_pattern_value(key, pattern, value)?;
        self.work.push(Work::PatternNext(id));
        Ok(())
    }
}
