//! Completed nominal contracts reach actual body publication and final region proof.
use crate::{
    body::regions,
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
        model::DefId,
        unit::NodeKey,
    },
    sema::{self, model::Declarations},
    support::source::SourceFile,
    types::{
        GenericArgs, ItemId, Kind, Primitive, RegionId, TypeId,
        arguments::Argument,
        parameters::{
            clauses::{Clause, Quantified},
            predicates::{Bound, Predicate},
        },
        traits::Application,
    },
};
use std::{path::Path, sync::Arc};
type ResultTest<T = ()> = Result<T, Box<dyn std::error::Error>>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
struct Fixture {
    report: Report,
    facts: Declarations,
}
impl Fixture {
    fn new(source: &str) -> ResultTest<Self> {
        let report = resolve::analyze_expanded(resolve::expansion::analyze(
            InputFile {
                identity: "body-quantified.rs".into(),
                path: "body-quantified.rs".into(),
                source: SourceFile::new(source.as_bytes().to_vec())?,
            },
            &mut NoFiles,
            &Config::default(),
        )?)?;
        let facts = sema::declarations::analyze(&report)?;
        Ok(Self { report, facts })
    }
    fn id(&self, name: &str) -> ResultTest<DefId> {
        Ok(DefId(u32::try_from(
            self.report
                .resolution
                .definitions
                .iter()
                .position(|d| self.report.package.names.get(d.name) == name)
                .ok_or("missing declaration")?,
        )?))
    }
    fn formal(&self, owner: DefId) -> ResultTest<TypeId> {
        self.facts
            .generics
            .parameters
            .get(&owner)
            .into_iter()
            .flatten()
            .find_map(|p| {
                if let Argument::Type(ty) = p.argument {
                    Some(ty)
                } else {
                    None
                }
            })
            .ok_or_else(|| "missing type parameter".into())
    }
    fn clause(&mut self, count: u32, subject: TypeId, regions: &[RegionId]) -> ResultTest<Clause> {
        let item = ItemId(self.id("A")?.0);
        let arguments = self.facts.types.intern_arguments(
            &regions
                .iter()
                .copied()
                .map(Argument::Lifetime)
                .collect::<Vec<_>>(),
        )?;
        let application = self
            .facts
            .types
            .intern_trait_application(Application { item, arguments })?;
        Ok(Clause::ForAll(self.facts.types.intern_quantified(
            Quantified {
                bound_regions: count,
                predicate: Predicate {
                    ty: subject,
                    bound: Bound::Trait(application),
                },
            },
        )?))
    }
    fn require(
        &mut self,
        nominal: &str,
        argument: TypeId,
        clauses: &[Clause],
    ) -> ResultTest<TypeId> {
        // The public source gate remains closed. Install the independently
        // qualified canonical immediate contract at its production phase boundary.
        let item = ItemId(self.id(nominal)?.0);
        let arguments = self
            .facts
            .types
            .intern_arguments(&[Argument::Type(argument)])?;
        let ty = self.facts.types.intern(Kind::Adt(item, arguments))?;
        self.facts
            .generics
            .nominal_requirements
            .insert(ty, clauses.into());
        self.facts.generics.has_nominal_requirements = true;
        Ok(ty)
    }
    fn assume(&mut self, owner: DefId, clause: Clause) -> ResultTest {
        let t = self.formal(owner)?;
        let Some(Kind::Parameter(parameter)) = self.facts.types.kind(t) else {
            return Err("missing caller parameter identity".into());
        };
        let abilities = self
            .facts
            .generics
            .environments
            .get(&owner)
            .and_then(|env| env.abilities(parameter))
            .ok_or("missing caller abilities")?;
        let predicates = self.facts.generics.predicates.entry(owner).or_default();
        // Replace the ordinary seed clause, matching the qualified source with
        // one universal premise; retaining the seed would introduce a second bound.
        predicates.retain(|p| {
            !matches!(
                p.value,
                Clause::Predicate(Predicate {
                    bound: Bound::Trait(_),
                    ..
                })
            )
        });
        let mut clauses: Vec<_> = predicates.iter().map(|p| p.value).collect();
        clauses.push(clause);
        let env = crate::types::parameters::environment::Environment::new(
            &self.facts.types,
            &[(parameter, abilities)],
        )?
        .with_clauses(&self.facts.types, &clauses)?;
        self.facts
            .generics
            .environments
            .insert(owner, Arc::new(env));
        let source = self
            .facts
            .functions
            .iter()
            .find(|f| f.definition == owner)
            .and_then(|f| f.body)
            .ok_or("missing caller body")?;
        self.facts
            .generics
            .predicates
            .get_mut(&owner)
            .ok_or("missing caller predicates")?
            .push(sema::generics::Predicate {
                source,
                value: clause,
            });
        self.facts
            .generics
            .traits
            .as_mut()
            .ok_or("missing completed trait facts")?
            .formation(&mut self.facts.types, clause)?;
        Ok(())
    }
    fn binding(&self, name: &str) -> ResultTest<NodeKey> {
        self.report.resolution.definitions[self.id(name)?.0 as usize]
            .node
            .ok_or_else(|| "missing binding source".into())
    }
    fn check(self, expected: bool) -> ResultTest<Declarations> {
        let binding = self.binding("n")?;
        let expected_span = self
            .report
            .package
            .unit(binding.file)
            .configured
            .ast
            .node(binding.node)
            .span;
        let facts = sema::bodies::analyze(&self.report, self.facts)?;
        let contracts = regions::collect(&facts)?;
        let checked =
            crate::borrow::regions::check(&facts.types, &facts.lifetimes.variances, &contracts);
        assert_eq!(checked.is_ok(), expected, "{:?}", checked.as_ref().err());
        if let Err(error) = checked {
            assert!(error.message.contains("does not outlive"), "{error}");
            let source = error.source.ok_or("missing failing obligation source")?;
            assert_eq!(
                self.report
                    .package
                    .unit(source.file)
                    .configured
                    .ast
                    .node(source.node)
                    .span,
                expected_span
            );
        }
        Ok(facts)
    }
}

#[test]
fn completed_body_checks_universal_nominal_requirements_before_execution() -> ResultTest {
    for (implementation, expected) in [
        ("impl<'a>A<'a>for u8{}", true),
        ("impl A<'static>for u8{}", false),
        ("impl<'a:'static>A<'a>for u8{}", false),
        ("impl<'a:'a>A<'a>for u8{}", true),
    ] {
        let mut f = Fixture::new(&format!(
            "trait A<'a>{{}}{implementation}struct Need<T>{{value:T}}fn f(){{let n=Need{{value:0u8}};}}"
        ))?;
        let clause = f.clause(1, BYTE, &[RegionId::bound(0, 0)?])?;
        f.require("Need", BYTE, &[clause])?;
        let facts = f.check(expected)?;
        assert_eq!(facts.quantified_proofs.len(), 1);
        assert_eq!(facts.quantified_proofs[0].evidence.proof.scope_count(), 1);
    }
    Ok(())
}

#[test]
fn body_nominal_queries_keep_independent_slots_and_outer_captures() -> ResultTest {
    for captured in [false, true] {
        for (implementation, expected) in [
            ("impl<'a,'b>A<'a,'b>for u8{}", true),
            ("impl<'a>A<'a,'a>for u8{}", false),
        ] {
            let mut f = Fixture::new(&format!(
                "trait A<'a,'b>{{}}{implementation}struct Need<T>{{value:T}}fn f<'x>(){{let n=Need{{value:0u8}};}}"
            ))?;
            let second = if captured {
                RegionId::bound(1, 0)?
            } else {
                RegionId::bound(0, 1)?
            };
            let clause = f.clause(
                if captured { 1 } else { 2 },
                BYTE,
                &[RegionId::bound(0, 0)?, second],
            )?;
            f.require("Need", BYTE, &[clause])?;
            f.check(expected)?;
        }
    }
    Ok(())
}

#[test]
fn universal_and_static_caller_premises_have_distinct_body_proofs() -> ResultTest {
    for universal in [false, true] {
        let mut f = Fixture::new(
            "trait A<'a>{}struct Need<T>{value:T}fn f<T:A<'static>>(value:T){let n=Need{value};}",
        )?;
        let owner = f.id("f")?;
        let t = f.formal(owner)?;
        let clause = f.clause(1, t, &[RegionId::bound(0, 0)?])?;
        if universal {
            f.assume(owner, clause)?;
        }
        f.require("Need", t, &[clause])?;
        f.check(universal)?;
    }
    Ok(())
}

#[test]
fn distinct_nominal_uses_keep_the_unpaid_roots_location() -> ResultTest {
    let mut f = Fixture::new(
        "trait A<'a>{}impl A<'static>for u8{}impl<'a>A<'a>for u16{}struct Need<T>{value:T}fn f(){let good=Need{value:0u16};let bad=Need{value:0u8};}",
    )?;
    let good = TypeId::primitive(Primitive::U16);
    for ty in [good, BYTE] {
        let clause = f.clause(1, ty, &[RegionId::bound(0, 0)?])?;
        f.require("Need", ty, &[clause])?;
    }
    let bad = f.binding("bad")?;
    let expected = f
        .report
        .package
        .unit(bad.file)
        .configured
        .ast
        .node(bad.node)
        .span;
    let facts = sema::bodies::analyze(&f.report, f.facts)?;
    let contracts = regions::collect(&facts)?;
    let error = crate::borrow::regions::check(&facts.types, &facts.lifetimes.variances, &contracts)
        .err()
        .ok_or("unpaid body requirement was accepted")?;
    let source = error.source.ok_or("missing body requirement source")?;
    assert_eq!(
        f.report
            .package
            .unit(source.file)
            .configured
            .ast
            .node(source.node)
            .span,
        expected
    );
    assert!(error.message.contains("does not outlive"));
    assert_eq!(facts.quantified_proofs.len(), 1);
    assert_eq!(facts.quantified_proofs[0].evidence.proof.scope_count(), 1);
    Ok(())
}

#[test]
fn captured_caller_lifetimes_require_both_directions_of_equality() -> ResultTest {
    for (bounds, equal) in [
        ("", false),
        (",'x:'y", false),
        (",'y:'x", false),
        (",'x:'y,'y:'x", true),
    ] {
        for same in [false, true] {
            let mut f = Fixture::new(&format!(
                "trait A<'a,'b>{{}}struct Need<T>{{value:T}}fn f<'x,'y,T>(value:T)where T:A<'static,'x>{bounds}{{let n=Need{{value}};}}"
            ))?;
            let owner = f.id("f")?;
            let t = f.formal(owner)?;
            let assumption = f.clause(1, t, &[RegionId::bound(0, 0)?, RegionId::bound(1, 0)?])?;
            f.assume(owner, assumption)?;
            let required = f.clause(
                1,
                t,
                &[
                    RegionId::bound(0, 0)?,
                    RegionId::bound(1, u16::from(!same))?,
                ],
            )?;
            f.require("Need", t, &[required])?;
            f.check(same || equal)?;
        }
    }
    Ok(())
}

#[test]
fn body_associated_results_keep_their_quantified_and_captured_lifetimes() -> ResultTest {
    for (result, captured, expected) in [
        ("&'a u8", false, true),
        ("&'static u8", false, false),
        ("&'a u8", true, false),
    ] {
        let mut f = Fixture::new(&format!(
            "trait A<'a>{{type Out;}}impl<'a>A<'a>for u8{{type Out={result};}}struct Need<T>{{value:T}}fn f<'x>(){{let n=Need{{value:0u8}};}}"
        ))?;
        let membership = f.clause(1, BYTE, &[RegionId::bound(0, 0)?])?;
        let Clause::ForAll(id) = membership else {
            return Err("missing universal membership".into());
        };
        let quantified = f.facts.types.quantified(id).ok_or("missing clause")?;
        let Bound::Trait(trait_application) = quantified.predicate.bound else {
            return Err("missing parent trait".into());
        };
        let item = ItemId(f.id("Out")?.0);
        let ty = f
            .facts
            .types
            .intern(Kind::Projection(crate::types::projection::Application {
                item,
                subject: BYTE,
                trait_application,
                arguments: GenericArgs::EMPTY,
            }))?;
        let result = f.facts.types.intern(Kind::Reference {
            mutable: false,
            region: RegionId::bound(u16::from(captured), 0)?,
            referent: BYTE,
        })?;
        let equality = Clause::ForAll(f.facts.types.intern_quantified(Quantified {
            bound_regions: 1,
            predicate: Predicate {
                ty,
                bound: Bound::ProjectionEqual(result),
            },
        })?);
        f.require("Need", BYTE, &[membership, equality])?;
        f.check(expected)?;
    }
    Ok(())
}

#[test]
fn shared_body_evidence_excludes_intervening_and_final_ordinary_allocations() -> ResultTest {
    let mut f = Fixture::new(
        "trait A<'a>{}impl<'a>A<'a>for u8{}impl<'a>A<'a>for u16{}impl<'a>A<'a>for u32{}impl<'a>A<'a>for u64{}struct First<T>{value:T}struct Middle<T>{value:T}struct Last<T>{value:T}struct Tail<T>{value:T}fn f(){let first=First{value:0u8};let middle=Middle{value:0u32};let n=Last{value:0u16};let tail=Tail{value:0u64};}",
    )?;
    for (name, primitive, universal) in [
        ("First", Primitive::U8, true),
        ("Middle", Primitive::U32, false),
        ("Last", Primitive::U16, true),
        ("Tail", Primitive::U64, false),
    ] {
        let ty = TypeId::primitive(primitive);
        let region = if universal {
            RegionId::bound(0, 0)?
        } else {
            RegionId::STATIC
        };
        let clause = f.clause(1, ty, &[region])?;
        let clause = if let Clause::ForAll(id) = clause
            && !universal
        {
            Clause::Predicate(
                f.facts
                    .types
                    .quantified(id)
                    .ok_or("missing clause")?
                    .predicate,
            )
        } else {
            clause
        };
        f.require(name, ty, &[clause])?;
    }
    let facts = f.check(true)?;
    assert_eq!(facts.quantified_proofs.len(), 1);
    let proof = &facts.quantified_proofs[0].evidence.proof;
    assert_eq!(proof.scope_count(), 1);
    assert!(proof.has_gaps());
    let owned: std::collections::HashSet<_> = proof.owned_regions().flatten().collect();
    let mut foreign = 0;
    for (index, region) in facts.body_regions.iter().enumerate() {
        let id = u32::try_from(index + 1)?;
        let ours = matches!(
            region.origin,
            sema::model::BodyRegionOrigin::QuantifiedProof
        );
        assert_eq!(owned.contains(&id), ours);
        foreign += usize::from(!ours);
    }
    assert_eq!(foreign, 2);
    assert_eq!(
        proof.regions.end,
        u32::try_from(facts.body_regions.len() + 1)?
    );
    Ok(())
}

#[test]
fn growing_body_roots_share_opening_and_nominal_prefixes() -> ResultTest {
    let mut previous = None;
    for width in [16, 128, 1024] {
        let mut f = Fixture::new(
            "trait A<'a,T>{}impl<'a,T>A<'a,T>for u8{}struct Need<T>{value:T}fn f<'x>(){}",
        )?;
        let item = ItemId(f.id("A")?.0);
        let mut outer = BYTE;
        let mut captured = BYTE;
        let mut roots = Vec::new();
        for _ in 0..width {
            for (ty, depth) in [(&mut outer, 0), (&mut captured, 1)] {
                *ty = f.facts.types.intern(Kind::Reference {
                    mutable: false,
                    region: RegionId::bound(depth, 0)?,
                    referent: *ty,
                })?;
            }
            let arguments = f.facts.types.intern_arguments(&[
                Argument::Lifetime(RegionId::bound(0, 0)?),
                Argument::Type(captured),
            ])?;
            let application = f
                .facts
                .types
                .intern_trait_application(Application { item, arguments })?;
            let clause = Clause::ForAll(f.facts.types.intern_quantified(Quantified {
                bound_regions: 1,
                predicate: Predicate {
                    ty: BYTE,
                    bound: Bound::Trait(application),
                },
            })?);
            roots.push(f.require("Need", outer, &[clause])?);
        }
        let function = *f.facts.functions.last().ok_or("missing function")?;
        let body = function.body.ok_or("missing function body")?;
        let before = f.facts.types.len();
        let mut checker = super::Checker::new(&f.report, &mut f.facts);
        checker.function(function)?;
        let mut requirements = super::Requirements::default();
        for root in &roots {
            checker.nominal_requirements(body.node, *root, &mut requirements)?;
        }
        let nominal_work = requirements.ordinary.steps();
        let shared = requirements
            .quantified
            .as_ref()
            .ok_or("missing shared proof")?;
        let query_work = shared.context.steps;
        assert_eq!(shared.evidence.proof.scope_count(), 1);
        assert_eq!(shared.evidence.sources.len(), width);
        // Repeated roots must add no query work, sources or allocations.
        let first = checker.facts.body_regions.len();
        for root in &roots {
            checker.nominal_requirements(body.node, *root, &mut requirements)?;
        }
        let shared = requirements
            .quantified
            .as_ref()
            .ok_or("missing shared proof")?;
        assert_eq!(shared.context.steps, query_work);
        assert_eq!(shared.evidence.sources.len(), width);
        assert_eq!(checker.facts.body_regions.len(), first);
        requirements.finish(&mut checker)?;
        let growth = checker.facts.types.len() - before;
        let contracts = regions::collect(checker.facts)?;
        crate::borrow::regions::check(
            &checker.facts.types,
            &checker.facts.lifetimes.variances,
            &contracts,
        )?;
        println!(
            "body universal width={width} nominal={nominal_work} query={query_work} growth={growth}"
        );
        assert!(nominal_work <= 8 * width + 32);
        assert!(query_work <= 256 * width + 1024);
        assert!(growth <= 16 * width + 64);
        if let Some((previous_work, previous_growth)) = previous {
            assert!(query_work <= 8 * previous_work + 256);
            assert!(growth <= 8 * previous_growth + 64);
        }
        previous = Some((query_work, growth));
    }
    Ok(())
}

#[test]
fn body_query_caches_never_supply_another_callers_assumptions() -> ResultTest {
    let good = "fn f<T:A<'static>>(value:T){let good=Need{value};}";
    let bad = "fn g<T:A<'static>>(value:T){let n=Need{value};}";
    for functions in [format!("{good}{bad}"), format!("{bad}{good}")] {
        let mut f = Fixture::new(&format!(
            "trait A<'a>{{}}struct Need<T>{{value:T}}{functions}"
        ))?;
        for name in ["f", "g"] {
            let owner = f.id(name)?;
            let parameter = f.formal(owner)?;
            let clause = f.clause(1, parameter, &[RegionId::bound(0, 0)?])?;
            if name == "f" {
                f.assume(owner, clause)?;
            }
            f.require("Need", parameter, &[clause])?;
        }
        let facts = f.check(false)?;
        assert_eq!(facts.quantified_proofs.len(), 2);
        assert_ne!(
            facts.quantified_proofs[0].body,
            facts.quantified_proofs[1].body
        );
    }
    Ok(())
}

#[test]
fn body_proofs_restore_caller_captures_beneath_nested_callable_locals() -> ResultTest {
    for (implementation, expected) in [
        ("impl<'a,'c>A<'a,for<'b>fn(&'b u8,&'c u8)>for u8{}", true),
        ("impl<'a>A<'a,for<'b>fn(&'b u8,&'static u8)>for u8{}", false),
    ] {
        let mut f = Fixture::new(&format!(
            "trait A<'a,T>{{}}{implementation}struct Need<T>{{value:T}}fn f<'x>(){{let n=Need{{value:0u8}};}}"
        ))?;
        let mut references = Vec::new();
        // The function's local slot, then the caller beyond both local binders.
        for depth in [0, 2] {
            references.push(f.facts.types.intern(Kind::Reference {
                mutable: false,
                region: RegionId::bound(depth, 0)?,
                referent: BYTE,
            })?);
        }
        let parameters = f.facts.types.intern_list(&references)?;
        let function = f.facts.types.intern(Kind::Function {
            parameters,
            result: TypeId::UNIT,
            bound_regions: 1,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
        })?;
        let arguments = f.facts.types.intern_arguments(&[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Type(function),
        ])?;
        let item = ItemId(f.id("A")?.0);
        let application = f
            .facts
            .types
            .intern_trait_application(Application { item, arguments })?;
        let clause = Clause::ForAll(f.facts.types.intern_quantified(Quantified {
            bound_regions: 1,
            predicate: Predicate {
                ty: BYTE,
                bound: Bound::Trait(application),
            },
        })?);
        f.require("Need", BYTE, &[clause])?;
        f.check(expected)?;
    }
    Ok(())
}
