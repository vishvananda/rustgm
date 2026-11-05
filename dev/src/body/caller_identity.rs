//! Source-computation identities must agree with the retained semantic body owner.
use super::ir::Owner;
use crate::sema::model::InitializerKind;
impl Owner {
    pub(crate) fn source_caller(
        self,
        facts: &crate::sema::model::Declarations,
        source: crate::body::ir::BodyOrigin,
        instance: Option<crate::types::instance::Instance>,
    ) -> Result<crate::sema::values::execution::CallerInstance, &'static str> {
        use crate::{sema::values::execution::CallerInstance, types::Kind};
        let source = source
            .source()
            .ok_or("imported body cannot reuse a source caller catalog")?;
        match self {
            Self::AsyncBody { .. } => self.async_original_caller(facts, source, instance)?,
            Self::AsyncApplication { future, .. } => {
                let definition = facts.checked_future_definition(future)?;
                if instance.is_some()
                    || facts
                        .function_indices
                        .get(&definition)
                        .and_then(|&index| facts.functions.get(index))
                        .and_then(|function| function.body)
                        != Some(source)
                {
                    return Err("async application caller belongs to another actual source body");
                }
                return Ok(CallerInstance::AsyncBody(future));
            }
            Self::FutureEntry { .. } => {
                return Err(
                    "future application requires its actual checked source capture namespace",
                );
            }
            Self::Function {
                definition,
                signature,
                ..
            } => {
                Self::function_caller(facts, source, instance, definition, signature)?;
            }
            Self::Closure {
                identity,
                signature,
                ..
            } => {
                let declared = facts
                    .closures
                    .identity(identity)
                    .is_some_and(|closure| closure.semantic_source() == source);
                let selected = instance.is_none_or(|instance| matches!(
                    facts.types.kind(instance.ty()),
                    Some(Kind::Closure { identity: selected, signature: selected_signature, .. })
                        if selected == identity && selected_signature == signature
                ));
                if !declared || !selected {
                    return Err("source caller does not match its closure owner");
                }
            }
            Self::Initializer { definition, kind }
            | Self::InitializerInstance {
                instance: crate::sema::values::Instance { definition, .. },
                kind,
            } => {
                let declared = facts
                    .initializer_index(definition)
                    .and_then(|n| facts.initializers.get(n))
                    .ok_or("source caller initializer is missing")?;
                let same = matches!(
                    (declared.kind, kind),
                    (InitializerKind::Const, InitializerKind::Const)
                ) || matches!((declared.kind, kind),
                        (InitializerKind::Static { mutable: a }, InitializerKind::Static { mutable: b }) if a == b);
                if instance.is_some() || declared.semantic_source() != source || !same {
                    return Err("source caller does not match its initializer owner");
                }
            }
            Self::AnonymousConstant => {
                let registered = facts.constant_body_type(source).is_some()
                    || facts
                        .inline_constants
                        .as_deref()
                        .and_then(|r| r.get(source))
                        .is_some();
                if instance.is_some() || !registered {
                    return Err("source caller does not match its anonymous owner");
                }
            }
            Self::AnonymousInstance(selected) => {
                if instance.is_some()
                    || selected.source() != source
                    || crate::sema::inline_constants::InlineInstance::select(
                        facts,
                        source,
                        selected.enclosing(),
                    )? != selected
                {
                    return Err("source caller does not match its inline instance");
                }
            }
            Self::ClosureAdapter { .. } => {
                return Err("closure adapters cannot own source associated computation");
            }
        }
        Ok(match self {
            Self::InitializerInstance { instance, .. } => CallerInstance::Initializer(instance),
            Self::AnonymousInstance(instance) => CallerInstance::Inline(instance),
            _ => instance.map_or(CallerInstance::Original, CallerInstance::Callable),
        })
    }
    fn async_original_caller(
        self,
        facts: &crate::sema::model::Declarations,
        source: crate::body::ir::NodeKey,
        instance: Option<crate::types::instance::Instance>,
    ) -> Result<(), &'static str> {
        let Self::AsyncBody {
            identity,
            future,
            signature,
        } = self
        else {
            return Err("deferred source caller requires its actual original owner");
        };
        let definition = facts
            .opaques
            .declarations
            .get(&identity)
            .ok_or("deferred caller lost its actual declaration")?
            .owner;
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&index| facts.functions.get(index))
            .ok_or("deferred caller lost its actual function")?;
        let actual = facts
            .authored_async_body(function)?
            .ok_or("deferred caller lost its actual authored input contract")?;
        if instance.is_some()
            || function.body != Some(source)
            || actual.identity != identity
            || actual.future != future
            || actual.signature != signature
        {
            return Err(
                "deferred caller changed its actual body, capture frame or Output signature",
            );
        }
        Ok(())
    }
    fn function_caller(
        facts: &crate::sema::model::Declarations,
        source: crate::body::ir::NodeKey,
        instance: Option<crate::types::instance::Instance>,
        definition: crate::resolve::model::DefId,
        signature: crate::types::TypeId,
    ) -> Result<(), &'static str> {
        let declared = facts
            .function_indices
            .get(&definition)
            .and_then(|&n| facts.functions.get(n))
            .and_then(|f| f.body);
        let selected = instance.is_none_or(|instance| {
            matches!(
                facts.types.kind(instance.ty()),
                Some(crate::types::Kind::FunctionItem { item, signature: selected, .. })
                    if item.0 == definition.0 && selected == signature
            )
        });
        if declared != Some(source) || !selected {
            return Err("source caller does not match its function owner");
        }
        Ok(())
    }
}
