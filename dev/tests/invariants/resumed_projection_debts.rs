//! Instance normalization extends the original lifetime proof and shares only its structural work.
use super::normalize;
use crate::{
    body::regions::{Contract, Contracts},
    expand::config::Config,
    resolve::{
        self,
        load::{InputFile, Sources},
        unit::{FileId, NodeKey},
    },
    sema::{self, model::Declarations},
    support::source::SourceFile,
    syntax::ast::NodeId,
    types::{
        GenericArgs, ItemId, Kind, RegionId, TypeId,
        arguments::Argument,
        lifetime_proof::{Obligations, ScopeId},
        parameters::predicates::Bound,
        projection, traits,
    },
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
struct Fixture {
    facts: Declarations,
    subject: TypeId,
    member: ItemId,
    trait_item: ItemId,
}
impl Fixture {
    fn new(restricted: bool) -> Result<Self> {
        let bound = if restricted { ":'static" } else { "" };
        let source = format!(
            "trait Q<'a>{{type Out;}}struct S;impl<'a{bound}>Q<'a>for S{{type Out=&'a u8;}}"
        );
        let input = InputFile {
            identity: "resumed.rs".into(),
            path: "resumed.rs".into(),
            source: SourceFile::new(source.into_bytes())?,
        };
        let report = resolve::analyze_expanded(resolve::expansion::analyze(
            input,
            &mut NoFiles,
            &Config::default(),
        )?)?;
        let facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
        let implementation = facts
            .generics
            .implementations()
            .next()
            .ok_or("missing projection implementation")?;
        let Bound::Trait(application) = implementation.predicate.bound else {
            return Err("missing implemented trait".into());
        };
        let trait_item = facts
            .types
            .trait_application(application)
            .ok_or("missing trait inputs")?
            .item;
        let subject = implementation.predicate.ty;
        let member = ItemId(implementation.types.first().ok_or("missing member")?.0.0);
        Ok(Self {
            facts,
            subject,
            member,
            trait_item,
        })
    }
    fn projected(&mut self, region: RegionId) -> Result<TypeId> {
        let arguments = self
            .facts
            .types
            .intern_arguments(&[Argument::Lifetime(region)])?;
        let trait_application = self
            .facts
            .types
            .intern_trait_application(traits::Application {
                item: self.trait_item,
                arguments,
            })?;
        Ok(self
            .facts
            .types
            .intern(Kind::Projection(projection::Application {
                item: self.member,
                subject: self.subject,
                trait_application,
                arguments: GenericArgs::EMPTY,
            }))?)
    }
    fn check(&self, contract: &Contract, valid: bool) {
        let contracts = Contracts {
            callables: &self.facts.lifetimes.callables,
            nominal_requirements: &self.facts.lifetimes.nominal_requirements,
            bodies: vec![contract.clone()],
        };
        let result = crate::borrow::regions::check(
            &self.facts.types,
            &self.facts.lifetimes.variances,
            &contracts,
        );
        assert_eq!(result.is_ok(), valid, "{:?}", result.as_ref().err());
        if let Err(error) = result {
            assert_eq!(error.source, Some(source(2)));
            assert!(
                matches!(
                    error.message.as_str(),
                    "required lifetime does not outlive its target"
                        | "required type does not outlive its target"
                ),
                "unexpected rejection: {error}"
            );
        }
    }
}
fn source(node: u32) -> NodeKey {
    NodeKey {
        file: FileId(0),
        node: NodeId(node),
    }
}
fn contract(proof: Obligations) -> Contract {
    Contract {
        source: source(1),
        signature: None,
        universals: 0,
        assumptions: Vec::new(),
        parameters: Vec::new(),
        type_assumptions: Vec::new(),
        obligations: Vec::new(),
        proofs: vec![(source(2), proof)],
    }
}
fn alias(first: u32, region: RegionId) -> Obligations {
    let mut proof = Obligations::new(first);
    proof.regions.end += 1;
    proof.equalities.push((RegionId(first), region));
    proof
}

#[test]
fn original_local_witnesses_survive_empty_ancestors_and_foreign_intervals() -> Result {
    let local = RegionId::bound(0, 0)?;
    for empty in [false, true] {
        for captured_witness in [false, true] {
            for static_target in [false, true] {
                let mut f = Fixture::new(false)?;
                let mut proof = Obligations::new(1);
                let parent = if empty {
                    proof.scope(ScopeId::ROOT, 0)?
                } else {
                    ScopeId::ROOT
                };
                let scope = proof.scope(parent, 1)?;
                let mut debt = alias(1, local);
                let ty = f.projected(if captured_witness { RegionId(1) } else { local })?;
                debt.types.push((
                    ty,
                    if static_target {
                        RegionId::STATIC
                    } else {
                        local
                    },
                ));
                proof.append_at(debt, scope)?;
                let scopes = proof.scope_count();
                let mut c = contract(proof);
                c.proofs.push((source(3), alias(2, RegionId::STATIC)));
                normalize(&mut f.facts, &mut c, 3)?;
                assert_eq!(c.proofs.len(), 2);
                assert_eq!(c.proofs[0].1.scope_count(), scopes);
                assert!(!c.proofs[0].1.owns_region(RegionId(2)));
                assert!(c.proofs[1].1.owns_region(RegionId(2)));
                assert!(c.proofs[0].1.type_roots().any(|(_, result)| {
                    matches!(f.facts.types.kind(result), Some(Kind::Reference { .. }))
                }));
                f.check(&c, !static_target);
            }
        }
    }
    Ok(())
}

#[test]
fn identical_type_ids_do_not_merge_sibling_proof_scopes() -> Result {
    let local = RegionId::bound(0, 0)?;
    for crossed in [false, true] {
        let mut f = Fixture::new(false)?;
        let ty = f.projected(local)?;
        let mut proof = Obligations::new(1);
        let first = proof.scope(ScopeId::ROOT, 1)?;
        let second = proof.scope(ScopeId::ROOT, 1)?;
        let mut a = alias(1, local);
        a.types.push((ty, local));
        proof.append_at(a, first)?;
        let mut b = alias(2, local);
        b.types
            .push((ty, if crossed { RegionId(1) } else { local }));
        proof.append_at(b, second)?;
        let mut c = contract(proof);
        normalize(&mut f.facts, &mut c, 3)?;
        assert_eq!(c.proofs[0].1.scope_count(), 2);
        f.check(&c, !crossed);
    }
    Ok(())
}

#[test]
fn normalization_retains_selected_impl_lifetime_debts_inside_nested_callables() -> Result {
    let local = RegionId::bound(0, 0)?;
    for restricted in [false, true] {
        for callable in [false, true] {
            let mut f = Fixture::new(restricted)?;
            let mut ty = f.projected(local)?;
            if callable {
                let parameters = f.facts.types.intern_list(&[ty])?;
                ty = f.facts.types.intern(Kind::Function {
                    bound_regions: 1,
                    parameters,
                    result: TypeId::UNIT,
                    unsafe_: false,
                })?;
            }
            let mut proof = Obligations::new(1);
            let scope = proof.scope(ScopeId::ROOT, 1)?;
            let mut debt = Obligations::new(1);
            debt.types
                .push((ty, if callable { RegionId::STATIC } else { local }));
            proof.append_at(debt, scope)?;
            let mut c = contract(proof);
            normalize(&mut f.facts, &mut c, 1)?;
            assert_eq!(c.proofs[0].1.scope_count(), if callable { 2 } else { 1 });
            f.check(&c, !restricted);
        }
    }
    Ok(())
}

#[test]
fn repeated_shared_roots_and_independent_scopes_have_additive_work() -> Result {
    let local = RegionId::bound(0, 0)?;
    for count in [8, 64, 512] {
        for mode in 0..3 {
            let independent = mode != 0;
            let mut f = Fixture::new(false)?;
            let projected = f.projected(local)?;
            let mut ty = projected;
            let mut proof = Obligations::new(1);
            let shared = proof.scope(ScopeId::ROOT, 1)?;
            let mut previous = shared;
            for _ in 0..count {
                let scope = if independent {
                    let parent = if mode == 2 { previous } else { ScopeId::ROOT };
                    proof.scope(parent, 1)?
                } else {
                    shared
                };
                previous = scope;
                if !independent {
                    let elements = f.facts.types.intern_list(&[ty, projected])?;
                    ty = f.facts.types.intern(Kind::Tuple(elements))?;
                }
                let mut debt = Obligations::new(1);
                debt.types.extend([(ty, local), (ty, local)]);
                proof.append_at(debt, scope)?;
            }
            let scopes = proof.scope_count();
            let mut c = contract(proof);
            let steps = normalize(&mut f.facts, &mut c, 1)?;
            assert!(steps < 128 * count, "{count}: {steps}");
            assert_eq!(c.proofs[0].1.scope_count(), scopes);
            f.check(&c, true);
        }
    }
    Ok(())
}

#[test]
fn outer_captures_keep_the_body_identity_through_empty_and_nonempty_scopes() -> Result {
    let captured = RegionId::bound(1, 0)?;
    for count in [0, 1] {
        for (static_target, assumed) in [(false, false), (true, false), (true, true)] {
            let mut f = Fixture::new(false)?;
            let ty = f.projected(captured)?;
            let mut proof = Obligations::new(1);
            let scope = proof.scope(ScopeId::ROOT, count)?;
            let mut debt = Obligations::new(1);
            debt.types.push((
                ty,
                if static_target {
                    RegionId::STATIC
                } else {
                    captured
                },
            ));
            proof.append_at(debt, scope)?;
            let mut c = contract(proof);
            c.universals = 1;
            if assumed {
                c.assumptions
                    .push((RegionId::bound(0, 0)?, RegionId::STATIC));
            }
            normalize(&mut f.facts, &mut c, 1)?;
            assert_eq!(c.proofs[0].1.scope_count(), 1);
            assert!(c.proofs[0].1.type_roots().any(|(_, result)| {
                matches!(f.facts.types.kind(result), Some(Kind::Reference { region, .. }) if region == captured)
            }));
            f.check(&c, !static_target || assumed);
        }
    }
    Ok(())
}

fn caller_clause(
    f: &mut Fixture,
    count: u32,
    ty: TypeId,
    shorter: RegionId,
) -> Result<crate::types::parameters::clauses::Clause> {
    use crate::types::parameters::{
        clauses::{Clause, Quantified},
        predicates::Predicate,
    };
    Ok(Clause::ForAll(f.facts.types.intern_quantified(
        Quantified {
            bound_regions: count,
            predicate: Predicate {
                ty,
                bound: Bound::Outlives(shorter),
            },
        },
    )?))
}

#[test]
fn caller_projection_premises_keep_normalization_requirements_as_debts() -> Result {
    for restricted in [false, true] {
        let mut f = Fixture::new(restricted)?;
        let local = RegionId::bound(0, 0)?;
        let ty = f.projected(local)?;
        let clause = caller_clause(&mut f, 1, ty, local)?;
        let mut c = contract(Obligations::new(1));
        c.type_assumptions.push(clause);
        let mut evidence = Obligations::new(1);
        super::super::assumptions::normalize(&mut f.facts, &mut c, &mut evidence)?;
        assert_eq!(c.type_assumptions, [clause]);
        assert!(c.assumptions.is_empty());
        assert_eq!(evidence.scope_count(), 1);
        c.proofs.push((source(2), evidence));
        f.check(&c, !restricted);
    }
    Ok(())
}

#[test]
fn concrete_projection_premises_supply_only_their_own_independent_components() -> Result {
    use crate::body::regions::{Kind as Debt, Obligation};
    for count in [0, 1] {
        for (given, goal) in [(false, 0), (true, 0), (true, 1)] {
            let mut f = Fixture::new(false)?;
            let projected = f.projected(RegionId::bound(1, 0)?)?;
            let target = if count == 0 {
                RegionId::STATIC
            } else {
                RegionId::bound(0, 0)?
            };
            let clause = caller_clause(&mut f, count, projected, target)?;
            let mut c = contract(Obligations::new(1));
            c.universals = 2;
            if given {
                c.type_assumptions.push(clause);
            }
            let ty = f.facts.types.intern(Kind::Reference {
                mutable: false,
                region: RegionId::bound(0, goal)?,
                referent: TypeId::UNIT,
            })?;
            c.obligations.push(Obligation {
                source: source(2),
                kind: Debt::TypeOutlives {
                    ty,
                    shorter: RegionId::STATIC,
                },
            });
            let mut evidence = Obligations::new(1);
            super::super::assumptions::normalize(&mut f.facts, &mut c, &mut evidence)?;
            c.proofs.push((source(2), evidence));
            f.check(&c, given && goal == 0);
        }
    }
    Ok(())
}

#[test]
fn growing_caller_projection_premises_share_normalization_and_component_walks() -> Result {
    for count in [16, 128, 1024] {
        let mut f = Fixture::new(false)?;
        let mut subject = f.subject;
        let mut c = contract(Obligations::new(1));
        c.universals = 1;
        for _ in 0..count {
            let ty = f.projected(RegionId::bound(1, 0)?)?;
            let list = f.facts.types.intern_list(&[subject, ty])?;
            subject = f.facts.types.intern(Kind::Tuple(list))?;
            c.type_assumptions
                .push(caller_clause(&mut f, 1, subject, RegionId::STATIC)?);
        }
        let mut evidence = Obligations::new(1);
        let steps = super::super::assumptions::normalize(&mut f.facts, &mut c, &mut evidence)?;
        assert!(steps < 64 * count, "{count}: {steps}");
        assert_eq!(c.assumptions, [(RegionId::bound(0, 0)?, RegionId::STATIC)]);
        assert_eq!(evidence.scope_count(), 1);
        c.proofs.push((source(2), evidence));
        f.check(&c, true);
    }
    Ok(())
}
