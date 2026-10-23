//! Complete defining/public frames and canonical declared clauses retain ownership.
use super::{
    Body, BodyRef, Contract, DeclarationRef, Export, GenericRequest, HashMap, NodeKey, Owner,
    TypeId, g,
};
use crate::types::{
    Kind,
    callable::Convention,
    parameters::{clauses::Clause, predicates::Bound},
};
use std::sync::Arc;

pub(super) fn validate(
    export: &Export<'_>,
    request: &GenericRequest,
    body: &Body,
    input: &Contract,
    environment: &Arc<crate::types::parameters::environment::Environment>,
) -> Result<(), &'static str> {
    let Owner::Function {
        definition,
        signature,
        execution: None,
    } = body.owner
    else {
        return Err("generic export original is not a function");
    };
    let header = export
        .facts
        .function_header(definition)?
        .ok_or("generic export header is absent")?;
    if definition != request.definition
        || header.body != Some(body.source)
        || header.signature != signature
        || header.declaration.source().is_none()
        || body.instance.is_some()
        || input.key != body.key()
        || input.source != body.source
        || input.signature != Some(signature)
        || input.universals != 0
        || input.output.is_some()
        || !input.assumptions.is_empty()
        || !input.proofs.is_empty()
        || header
            .environment
            .is_none_or(|actual| !Arc::ptr_eq(actual, environment))
    {
        return Err("generic export lacks its exact supported original contract and environment");
    }
    if export
        .facts
        .foreign
        .as_deref()
        .and_then(|r| r.function(definition))
        .is_some_and(|f| !matches!(f.naming, crate::support::linkage::Naming::Rust))
    {
        return Err("generic row version zero cannot export exact foreign symbol contracts");
    }
    for obligation in &input.obligations {
        match obligation.kind {
            crate::body::regions::Kind::WellFormed(_)
            | crate::body::regions::Kind::ScopedWellFormed {
                bound_regions: 0, ..
            } => {}
            crate::body::regions::Kind::Coercion {
                actual,
                expected,
                unsize: false,
                reference: None,
            } if actual == expected => {}
            _ => return Err("generic original requires a contract outside row version zero"),
        }
    }
    Ok(())
}
impl Export<'_> {
    pub(super) fn ty(&mut self, ty: TypeId) -> Result<g::TypeRef, &'static str> {
        self.charge(1)?;
        if let Some(&root) = self.types.get(&ty) {
            return Ok(root);
        }
        let value = crate::sema::crates::export_scalar(self.facts, ty)
            .map_err(|_| "generic operation type is outside scalar/declared-parameter transport")?;
        self.reserve(4 * (size_of::<g::Type>() + size_of::<(TypeId, g::TypeRef)>()))?;
        let root = g::TypeRef(Self::ordinal(self.rows.types.len())?);
        self.rows.types.push(g::Type::Scalar(value));
        self.types.insert(ty, root);
        Ok(root)
    }
    fn frame(
        &mut self,
        owner: DeclarationRef,
        parent: Option<g::BinderRef>,
        kind: g::BinderKind,
    ) -> Result<g::BinderRef, &'static str> {
        self.reserve(4 * size_of::<g::Binder>())?;
        let reference = g::BinderRef(Self::ordinal(self.rows.binders.len())?);
        self.rows.binders.push(g::Binder {
            owner,
            parent,
            kind,
            count: 0,
        });
        Ok(reference)
    }
    fn signature(&mut self, ty: TypeId, binder: g::BinderRef) -> Result<g::TypeRef, &'static str> {
        let Some(Kind::Function {
            bound_regions: 0,
            parameters,
            result,
            unsafe_,
            convention,
        }) = self.facts.types.kind(ty)
        else {
            return Err(
                "generic row version zero requires a complete zero-region function signature",
            );
        };
        if convention != Convention::RUST {
            return Err("generic convention requires its complete executable contract");
        }
        let first = Self::ordinal(self.rows.roots.len())?;
        for &parameter in self
            .facts
            .types
            .list(parameters)
            .ok_or("missing generic callable parameters")?
        {
            let root = self.ty(parameter)?;
            self.reserve(4 * size_of::<g::TypeRef>())?;
            self.rows.roots.push(root);
        }
        let end = Self::ordinal(self.rows.roots.len())?;
        let result = self.ty(result)?;
        self.reserve(4 * size_of::<g::Type>())?;
        let reference = g::TypeRef(Self::ordinal(self.rows.types.len())?);
        self.rows.types.push(g::Type::Function(g::Signature {
            binder,
            parameters: g::Range { start: first, end },
            result,
            unsafe_,
            convention: g::Convention::Rust,
            variadic: false,
        }));
        Ok(reference)
    }
}
pub(super) fn emit(
    export: &mut Export<'_>,
    request: &GenericRequest,
    body: BodyRef,
) -> Result<g::Function, &'static str> {
    let owner = export.declaration(request.definition)?;
    let original_frame = export.frame(owner, None, g::BinderKind::OriginalFunction)?;
    let callable_frame = export.frame(owner, None, g::BinderKind::PublicFunction)?;
    let header = export
        .facts
        .function_header(request.definition)?
        .ok_or("missing actual export header")?;
    let original_ty = header.signature;
    let Some(Kind::FunctionItem { signature, .. }) = export.facts.definitions
        [request.definition.0 as usize]
        .and_then(|t| export.facts.types.kind(t))
    else {
        return Err("missing actual generic callable");
    };
    let original = export.signature(original_ty, original_frame)?;
    let callable = export.signature(signature, callable_frame)?;
    let first = Export::ordinal(export.rows.parameters.len())?;
    for parameter in export
        .facts
        .generics
        .parameters_in_scope(request.definition)
    {
        let reference = export.declaration(parameter.binding)?;
        export.reserve(4 * size_of::<DeclarationRef>())?;
        export.rows.parameters.push(reference);
    }
    let parameters = g::Range {
        start: first,
        end: Export::ordinal(export.rows.parameters.len())?,
    };
    let clauses = clauses(export, request, owner, original_frame)?;
    Ok(g::Function {
        declaration: owner,
        body,
        original,
        callable,
        original_frame,
        callable_frame,
        parameters,
        clauses,
        declared_clauses: clauses.end - clauses.start,
        constant: export.facts.is_const_function(request.definition),
    })
}

fn clauses(
    export: &mut Export<'_>,
    request: &GenericRequest,
    owner: DeclarationRef,
    original_frame: g::BinderRef,
) -> Result<g::Range, &'static str> {
    let header = export
        .facts
        .function_header(request.definition)?
        .ok_or("missing original clause owner")?;
    let first = Export::ordinal(export.rows.clauses.len())?;
    let mut sites = HashMap::default();
    sites.insert(
        header
            .declaration
            .source()
            .ok_or("generic original is not a source declaration")?,
        0_u32,
    );
    let inputs = crate::sema::declaration_inputs::declared(
        &export.facts.generics,
        export.facts.crates.as_deref(),
        request.definition,
    )?;
    for input in inputs {
        let (predicate, has_frame) = match input.value {
            Clause::Predicate(predicate) => (predicate, false),
            Clause::ForAll(id) => {
                let value = export
                    .facts
                    .types
                    .quantified(id)
                    .ok_or("unknown generic quantified clause")?;
                if value.bound_regions != 0 {
                    return Err(
                        "quantified lifetime transport is outside generic row version zero",
                    );
                }
                (value.predicate, true)
            }
        };
        let source = input
            .source
            .source()
            .ok_or("source export clause has a foreign declaration owner")?;
        let next = Export::ordinal(sites.len())?;
        let site = *sites.entry(source).or_insert(next);
        export.reserve(4 * (size_of::<g::Clause>() + size_of::<(NodeKey, u32)>()))?;
        let quantifier = has_frame
            .then(|| export.frame(owner, Some(original_frame), g::BinderKind::Clause))
            .transpose()?;
        let bound = match predicate.bound {
            Bound::Copy => g::Bound::Copy(g::CopyIdentity::LegacyBuiltin),
            Bound::Sized => g::Bound::Sized,
            _ => {
                return Err(
                    "generic clause requires its complete trait, effect or lifetime contract",
                );
            }
        };
        let subject = export.ty(predicate.ty)?;
        export.rows.clauses.push(g::Clause {
            owner,
            site,
            subject,
            quantifier,
            bound,
        });
    }
    export.rows.declaration_sites[owner.0 as usize] = Export::ordinal(sites.len())?;
    let clauses = g::Range {
        start: first,
        end: Export::ordinal(export.rows.clauses.len())?,
    };
    Ok(clauses)
}
