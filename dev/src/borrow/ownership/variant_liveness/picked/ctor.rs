//! Tentative constructor tags become eligibility data only after full signature proof.
use super::super::trans::Candidate;
use super::{Parts, Step, rooted::Resolver};
use crate::{
    body::ir::{Body, Rvalue},
    support::retained::Growth,
    types::{Kind, Store, TypeId, aggregate::Definitions, callable::Convention, substitute::Bound},
};
impl<'a> Resolver<'a> {
    pub(super) fn constructors(
        &mut self,
        types: &mut Store,
        definitions: &'a Definitions,
        parts: &Parts,
        body: &Body,
        candidates: &[Candidate],
    ) -> Result<(), &'static str> {
        for candidate in candidates {
            self.constructor(types, definitions, parts, body, candidate)?;
        }
        Ok(())
    }
    fn constructor(
        &mut self,
        types: &mut Store,
        definitions: &'a Definitions,
        parts: &Parts,
        body: &Body,
        candidate: &Candidate,
    ) -> Result<(), &'static str> {
        parts.charge(1)?;
        let value = body
            .values
            .get(candidate.value.0 as usize)
            .ok_or("selected constructor value is missing")?;
        let Rvalue::Call {
            callee,
            signature,
            arguments,
        } = value.kind
        else {
            return Err("selected constructor changed its actual call");
        };
        let callee = body
            .values
            .get(callee.0 as usize)
            .ok_or("selected constructor callee is missing")?;
        let Some(Kind::FunctionItem {
            item,
            signature: public,
            arguments: actual,
        }) = types.kind(callee.ty)
        else {
            return Err("selected constructor lost its retained function item");
        };
        if callee.ty != candidate.callee
            || signature != candidate.signature
            || value.ty != candidate.result
            || !matches!(callee.kind, Rvalue::Function(definition) if definition.0 == item.0)
        {
            return Err("selected constructor changed its exact callable context");
        }
        let (result, bound_regions) = self.public(
            types,
            definitions,
            parts,
            (public, actual),
            candidate.variant,
        )?;
        let opened = open(types, parts, public, result, value.ty, bound_regions)?;
        if opened != signature {
            return Err(
                "selected constructor call signature differs from its actual public opening",
            );
        }
        let Some(Kind::Function {
            parameters, result, ..
        }) = types.kind(opened)
        else {
            return Err("selected constructor lost its opened signature");
        };
        let inputs = body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("selected constructor call arguments are missing")?;
        let parameters = types
            .list(parameters)
            .ok_or("selected constructor inputs are missing")?;
        parts.charge(inputs.len())?;
        if result != value.ty
            || inputs.len() != parameters.len()
            || inputs
                .iter()
                .zip(parameters)
                .any(|(v, ty)| body.values.get(v.0 as usize).is_none_or(|v| v.ty != *ty))
        {
            return Err("selected constructor call changed its full input or result application");
        }
        Ok(())
    }
    fn public(
        &mut self,
        types: &mut Store,
        definitions: &'a Definitions,
        parts: &Parts,
        (public, actual): (TypeId, crate::types::GenericArgs),
        variant: u32,
    ) -> Result<(TypeId, u32), &'static str> {
        let Some(Kind::Function {
            parameters,
            result,
            bound_regions,
            unsafe_: false,
            convention,
        }) = types.kind(public)
        else {
            return Err("selected constructor requires its safe public signature");
        };
        let Some(Kind::Adt(owner, application)) = types.kind(result) else {
            return Err("selected constructor requires its full nominal result");
        };
        if convention != Convention::RUST
            || (bound_regions == 0 && actual != application)
            || (bound_regions != 0 && actual != crate::types::GenericArgs::EMPTY)
        {
            return Err("selected constructor changed its public arguments or convention");
        }
        let definition = definitions
            .get(owner)
            .ok_or("selected constructor header is missing")?;
        let fields = definitions
            .variants(definition)
            .get(variant as usize)
            .and_then(|v| types.list(v.fields))
            .ok_or("selected constructor variant is missing")?
            .len();
        if types.list(parameters).is_none_or(|p| p.len() != fields) {
            return Err("selected constructor changed its public field arity");
        }
        for field in 0..fields {
            let expected = self.field(
                types,
                definitions,
                parts,
                result,
                Step::Nominal {
                    variant,
                    field: super::parts::compact(field)?,
                },
            )?;
            if types.list(parameters).and_then(|p| p.get(field)).copied() != Some(expected) {
                return Err(
                    "selected constructor public signature differs from its semantic field",
                );
            }
        }
        Ok((result, bound_regions))
    }
}
fn open(
    types: &mut Store,
    parts: &Parts,
    public: TypeId,
    result: TypeId,
    actual: TypeId,
    count: u32,
) -> Result<TypeId, &'static str> {
    if count == 0 {
        return Ok(public);
    }
    parts.pay(Growth {
        work: count as usize,
        bytes: (count as usize).saturating_mul(32),
    })?;
    let candidates = crate::types::callable::late_result::candidates(types, result, actual, count)?;
    parts.charge(candidates.steps)?;
    let regions = candidates
        .regions
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or("selected constructor has an unbound public lifetime")?;
    if regions.len() != count as usize {
        return Err("selected constructor result cannot open its public lifetimes");
    }
    Bound::new(regions).signature_charged(types, public, 0, &|work, bytes| {
        parts.pay(Growth { bytes, work })
    })
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/ctopen.rs"]
mod tests;
