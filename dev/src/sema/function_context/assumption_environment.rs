//! One defining environment construction; callers retain actual owner and alias provenance.
use crate::{
    sema::model::Declarations,
    types::{
        Kind, RegionId, Store, TypeId,
        arguments::Argument,
        outlives::Queries,
        parameters::{clauses::Clause, environment::Environment},
        relations::{Constraints, RegionEqualities},
    },
};
use std::sync::Arc;
#[cfg(test)]
#[path = "../../../tests/invariants/provided_environments.rs"]
mod tests;

/// Canonical lifetime inputs may come from completed source facts or checked
/// staged import scopes. Supplying them is not proof of their provenance.
pub(in crate::sema) struct Scope<'a, P, C> {
    pub signature: Option<TypeId>,
    pub universals: u32,
    pub parameters: P,
    pub bounds: &'a [(RegionId, RegionId)],
    pub clauses: C,
}
pub(in crate::sema) struct Context<'a> {
    pub types: &'a mut Store,
    pub requirements: &'a mut crate::types::outlives::Bounds,
    pub variances: &'a crate::types::variance::Variances,
    pub callables: &'a crate::types::callable::Templates,
}

/// Declaration assumptions are shared unchanged. Defining clauses, regional
/// identities and body aliases belong only to the resulting body environment.
/// This construction supplies no selected implementation or execution proof.
pub(in crate::sema) struct Prepared {
    environment: Option<Arc<Environment>>,
    equalities: Option<RegionEqualities>,
    changed: bool,
    aliases_required: bool,
}
pub(in crate::sema) struct Completed {
    pub environment: Option<Arc<Environment>>,
    pub changed: bool,
}

impl Prepared {
    /// Borrow complete declaration inputs without allocating temporary vectors.
    pub(in crate::sema) fn for_declaration(
        facts: &mut Declarations,
        owner: crate::resolve::model::DefId,
        signature: Option<TypeId>,
        universals: u32,
        environment: Option<Arc<Environment>>,
        clauses: &[Clause],
    ) -> Result<Self, String> {
        if clauses.is_empty()
            && environment
                .as_ref()
                .is_none_or(|actual| !actual.has_region_builtins())
        {
            return Ok(Self {
                environment,
                equalities: None,
                changed: false,
                aliases_required: false,
            });
        }
        let scope = Scope {
            signature,
            universals,
            parameters: facts
                .generics
                .parameters_in_scope(owner)
                .map(|p| p.argument),
            bounds: facts
                .lifetimes
                .functions
                .get(&owner)
                .map_or(&[], |scheme| &scheme.bounds),
            clauses: crate::sema::declaration_inputs::all(
                &facts.generics,
                facts.crates.as_deref(),
                owner,
            )?
            .map(|input| input.value),
        };
        Self::begin(
            Context {
                types: &mut facts.types,
                requirements: &mut facts.lifetimes.nominal_requirements,
                variances: &facts.lifetimes.variances,
                callables: &facts.lifetimes.callables,
            },
            scope,
            environment,
            clauses,
        )
    }
    /// Defining clauses have already been translated through the actual body's
    /// universal substitution. Import callers authenticate their complete staged
    /// scope and reserve the envelope before entering this same producer.
    pub(in crate::sema) fn begin<P, C>(
        mut context: Context<'_>,
        scope: Scope<'_, P, C>,
        mut environment: Option<Arc<Environment>>,
        clauses: &[Clause],
    ) -> Result<Self, String>
    where
        P: Iterator<Item = Argument>,
        C: Iterator<Item = Clause>,
    {
        if !clauses.is_empty() {
            let original = environment
                .as_ref()
                .ok_or("missing bounded body environment")?;
            environment = Some(Arc::new(
                original
                    .as_ref()
                    .clone()
                    .with_clauses(context.types, clauses)?,
            ));
        }
        let equalities = if environment
            .as_ref()
            .is_some_and(|environment| environment.has_region_builtins())
        {
            Some(region_equalities(&mut context, scope)?)
        } else {
            None
        };
        let changed = !clauses.is_empty() || equalities.is_some();
        let aliases_required = !clauses.is_empty()
            || equalities
                .as_ref()
                .is_some_and(|equalities| !equalities.is_empty());
        Ok(Self {
            environment,
            equalities,
            changed,
            aliases_required,
        })
    }
    pub(in crate::sema) fn has_environment(&self) -> bool {
        self.environment.is_some()
    }
    /// The source producer allocates universals lazily only when this exact
    /// assumed equality graph needs its aliases, or clauses already needed them.
    pub(in crate::sema) fn needs_aliases(&self) -> bool {
        self.aliases_required
    }
    pub(in crate::sema) fn finish(
        self,
        universals: u32,
        aliases: Vec<crate::types::RegionId>,
    ) -> Result<Completed, String> {
        if (!aliases.is_empty() || self.needs_aliases()) && aliases.len() != universals as usize {
            return Err("body aliases do not match the actual defining scope".into());
        }
        let Some(mut environment) = self.environment else {
            return Ok(Completed {
                environment: None,
                changed: self.changed,
            });
        };
        if let Some(equalities) = self.equalities {
            let mut updated = environment.as_ref().clone();
            if !equalities.is_empty() {
                updated = updated.with_region_equalities(
                    equalities
                        .with_aliases(&aliases)
                        .map_err(|e| e.to_string())?,
                );
            }
            environment = Arc::new(updated);
        }
        let has_ledger = !aliases.is_empty();
        if has_ledger {
            environment = Arc::new(environment.as_ref().clone().with_body_aliases(aliases)?);
        }
        Ok(Completed {
            environment: Some(environment),
            changed: self.changed || has_ledger,
        })
    }
}

fn region_equalities<P, C>(
    context: &mut Context<'_>,
    scope: Scope<'_, P, C>,
) -> Result<RegionEqualities, String>
where
    P: Iterator<Item = Argument>,
    C: Iterator<Item = Clause>,
{
    if let Some(signature) = scope.signature {
        context.requirements.prepare(context.types, signature)?;
    }
    let mut proof = Constraints::new(
        context.types,
        context.variances,
        scope.universals,
        1_000_000,
    )
    .map_err(|e| e.to_string())?;
    for parameter in scope.parameters {
        if let Argument::Type(ty) = parameter
            && let Some(Kind::Parameter(id)) = context.types.kind(ty)
        {
            proof.parameter(id).map_err(|e| e.to_string())?;
        }
    }
    for &(longer, shorter) in scope.bounds {
        proof.assume(longer, shorter).map_err(|e| e.to_string())?;
    }
    let mut queries = Queries::new(context.types, context.requirements, context.callables);
    if let Some(signature) = scope.signature {
        proof.assume_signature(&mut queries, signature)?;
    }
    proof.assume_outlives_clauses(&mut queries, scope.clauses)?;
    proof.assumed_equalities().map_err(|e| e.to_string())
}
