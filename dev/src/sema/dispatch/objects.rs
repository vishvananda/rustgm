//! Automatic object implementations preserve virtual callable identity and proof.
use super::{Declarations, Dispatcher, Obligations, arguments, implementation, selection};
use crate::{
    resolve::model::DefId,
    types::{GenericArgs, Kind, TypeId, parameters::predicates::Predicate},
};
pub(super) enum Method {
    Implementation {
        owner: DefId,
        arguments: GenericArgs,
        proof: Obligations,
    },
    Object {
        proof: Obligations,
    },
    Language {
        descriptor: crate::sema::primitive_methods::Descriptor,
        proof: Obligations,
    },
}

/// Read a complete trait method's Self/application from its retained arguments.
/// This identifies a potential virtual method; it does not grant dispatch permission.
pub(crate) fn object_method(
    facts: &mut Declarations,
    ty: TypeId,
) -> Result<Option<(DefId, Predicate)>, &'static str> {
    if !facts.types.has_dynamic_shapes() {
        return Ok(None);
    }
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(ty) else {
        return Ok(None);
    };
    let method = DefId(item.0);
    let Some(owner) = facts.associated.trait_owner(method) else {
        return Ok(None);
    };
    let predicate = arguments::Arguments::new(facts, ty)?.goal(facts, owner)?;
    Ok(
        matches!(facts.types.kind(predicate.ty), Some(Kind::Dynamic { .. }))
            .then_some((method, predicate)),
    )
}

impl Dispatcher {
    pub(super) fn method_implementation(
        &mut self,
        facts: &mut Declarations,
        predicate: Predicate,
    ) -> Result<Method, String> {
        let language = match predicate.bound {
            crate::types::parameters::predicates::Bound::Trait(id) => facts
                .types
                .trait_application(id)
                .and_then(|application| facts.associated.operation(DefId(application.item.0)))
                .is_some_and(|(kind, _)| crate::sema::primitive_methods::supported(kind)),
            _ => false,
        };
        if !language && !matches!(facts.types.kind(predicate.ty), Some(Kind::Dynamic { .. })) {
            return self
                .implementation(facts, predicate)
                .map(|(owner, arguments, proof)| Method::Implementation {
                    owner,
                    arguments,
                    proof,
                });
        }
        let before = self.normalize.steps;
        let predicate = self.normalize.predicate(&mut facts.types, predicate)?;
        self.context
            .account_work(
                self.normalize
                    .steps
                    .saturating_sub(before)
                    .saturating_add(1),
            )
            .map_err(|e| e.to_string())?;
        let empty = super::Assumptions::default();
        let caller = self.caller.as_ref();
        let evidence = implementation::select_source(
            facts,
            &mut self.context,
            predicate,
            selection::Caller {
                assumptions: caller.map_or(&empty, |caller| &caller.assumptions),
                environment: caller.and_then(|caller| caller.environment.as_ref()),
                first_region: self.obligations.regions.end,
                retain_sized: false,
            },
            self.availability,
        )?;
        self.provisional_size |= evidence.provisional_size;
        match evidence.source {
            selection::Source::Implementation { owner, arguments } => Ok(Method::Implementation {
                owner,
                arguments,
                proof: evidence.proof,
            }),
            selection::Source::ResultBound(_) | selection::Source::QuantifiedResultBound { .. } => {
                Ok(Method::Object {
                    proof: evidence.proof,
                })
            }
            selection::Source::LanguageOperation { descriptor } => Ok(Method::Language {
                descriptor,
                proof: evidence.proof,
            }),
            _ => Err("object dispatch requires intrinsic object or implementation evidence".into()),
        }
    }
}
