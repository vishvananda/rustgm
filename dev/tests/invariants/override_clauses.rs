//! Override proofs preserve quantified trait and lifetime contracts before source activation.
use super::{Clause, Declarations, DefId, Definitions, Function, Predicate, check};
use crate::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    sema::{associated::OwnerKind, generics},
    support::source::SourceFile,
    types::{
        GenericArgs, ItemId, Kind, RegionId, TypeId,
        arguments::Argument,
        parameters::{clauses::Quantified, predicates::Bound},
        projection, traits,
    },
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const METHODS: &str = "trait Api{fn f<T>();}impl Api for u8{fn f<T>(){}}";
const _: [(); 12] = [(); size_of::<super::Method>()];

#[test]
fn method_view_preserves_source_contract_and_rejects_imported_authority() -> Result {
    let mut fixture = Fixture::new(METHODS)?;
    let method = super::Method::source(&fixture.facts, fixture.expected);
    assert_eq!(method.definition, fixture.expected.definition);
    assert_eq!(method.signature, fixture.expected.signature);
    assert_eq!(method.written_signature, fixture.expected.signature);
    assert!(fixture.expected.body.is_none());
    let before = fixture.facts.body_work.canonical_steps;
    let error = super::Method::imported(&mut fixture.facts, fixture.expected.definition)
        .err()
        .ok_or("source method acquired imported authority")?;
    assert_eq!(
        error,
        "imported conformance cannot replace an actual source method"
    );
    assert_eq!(fixture.facts.body_work.canonical_steps, before + 1);
    fixture.valid(true)
}

#[test]
fn method_abi_must_match_for_foreign_and_rust_declarations() -> Result {
    let mut foreign = Fixture::new(
        "trait Api{extern \"C\" fn f<T>();}impl Api for u8{extern \"C\" fn f<T>(){}}",
    )?;
    foreign.valid(true)?;
    for source in [
        METHODS,
        "trait Api{extern \"C\" fn f<T>();}impl Api for u8{extern \"C\" fn f<T>(){}}",
    ] {
        let mut fixture = Fixture::new(source)?;
        let Some(Kind::Function {
            parameters,
            result,
            bound_regions,
            unsafe_,
            convention,
        }) = fixture.facts.types.kind(fixture.actual.signature)
        else {
            return Err("actual implementation method signature".into());
        };
        let changed = if convention == crate::types::callable::Convention::RUST {
            crate::types::callable::Convention::C
        } else {
            crate::types::callable::Convention::RUST
        };
        fixture.actual.signature = fixture.facts.types.intern(Kind::Function {
            parameters,
            result,
            bound_regions,
            unsafe_,
            convention: changed,
        })?;
        fixture.valid(false)?;
    }
    Ok(())
}
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
struct Fixture {
    facts: Declarations,
    report: Report,
    expected: Function,
    actual: Function,
    owner: DefId,
    predicate: Predicate,
}
impl Fixture {
    fn new(source: &str) -> Result<Self> {
        let input = InputFile {
            identity: "override.rs".into(),
            path: "override.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        };
        let report = resolve::analyze_expanded(resolve::expansion::analyze(
            input,
            &mut NoFiles,
            &Config::default(),
        )?)?;
        let facts = crate::sema::declarations::analyze(&report)?;
        let expected = facts
            .functions
            .iter()
            .copied()
            .find(|f| facts.associated.trait_owner(f.definition).is_some())
            .ok_or("missing declared method")?;
        let actual = facts
            .functions
            .iter()
            .copied()
            .find(|f| {
                facts
                    .associated
                    .owner(f.definition)
                    .and_then(|id| facts.associated.get(id))
                    .is_some_and(|owner| matches!(owner.kind, OwnerKind::Impl { .. }))
            })
            .ok_or("missing implementing method")?;
        let owner = facts
            .associated
            .owner(actual.definition)
            .and_then(|id| facts.associated.get(id))
            .ok_or("missing implementation owner")?
            .self_definition;
        let predicate = facts
            .generics
            .implementations
            .as_ref()
            .and_then(|registry| registry.header(owner))
            .ok_or("missing implementation header")?
            .predicate;
        Ok(Self {
            facts,
            report,
            expected,
            actual,
            owner,
            predicate,
        })
    }
    fn definition(&self, name: &str) -> Result<ItemId> {
        let index = self
            .report
            .resolution
            .definitions
            .iter()
            .position(|d| self.report.package.names.get(d.name) == name)
            .ok_or("missing fixture declaration")?;
        Ok(ItemId(u32::try_from(index)?))
    }
    fn parameter(&self, method: Function) -> Result<TypeId> {
        self.facts
            .generics
            .parameters
            .get(&method.definition)
            .into_iter()
            .flatten()
            .find_map(|p| {
                if let Argument::Type(ty) = p.argument {
                    Some(ty)
                } else {
                    None
                }
            })
            .ok_or_else(|| "missing method parameter".into())
    }
    fn clause(&mut self, predicate: Predicate, count: Option<u32>) -> Result<Clause> {
        Ok(if let Some(bound_regions) = count {
            Clause::ForAll(self.facts.types.intern_quantified(Quantified {
                bound_regions,
                predicate,
            })?)
        } else {
            Clause::Predicate(predicate)
        })
    }
    fn trait_bound(
        &mut self,
        method: Function,
        regions: &[RegionId],
        count: Option<u32>,
    ) -> Result<Clause> {
        let ty = self.parameter(method)?;
        let item = self.definition("A")?;
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
            .intern_trait_application(traits::Application { item, arguments })?;
        self.clause(
            Predicate {
                ty,
                bound: Bound::Trait(application),
            },
            count,
        )
    }
    fn lifetime(
        &mut self,
        method: Function,
        shorter: RegionId,
        count: Option<u32>,
        tuple: bool,
    ) -> Result<Clause> {
        let mut ty = self.parameter(method)?;
        if tuple {
            let list = self.facts.types.intern_list(&[ty])?;
            ty = self.facts.types.intern(Kind::Tuple(list))?;
        }
        self.clause(
            Predicate {
                ty,
                bound: Bound::Outlives(shorter),
            },
            count,
        )
    }
    fn install(&mut self, method: Function, clauses: &[Clause]) {
        self.facts.generics.predicates.insert(
            method.definition,
            clauses
                .iter()
                .copied()
                .map(|value| generics::Predicate {
                    source: method.node,
                    value,
                })
                .collect(),
        );
    }
    fn valid(&mut self, expected: bool) -> Result {
        let definitions = Definitions::new(&self.facts, self.predicate, &[])?;
        let result = check(
            &mut self.facts,
            self.owner,
            self.predicate,
            self.expected,
            self.actual,
            &definitions,
        );
        assert_eq!(result.is_ok(), expected, "{result:?}");
        if let Err(error) = result {
            assert!(
                matches!(
                    error.as_str(),
                    "implementation method imposes a stronger generic requirement"
                        | "implementation method lifetime contract does not satisfy trait: required lifetime does not outlive its target"
                        | "implementation method lifetime contract does not satisfy trait: required type does not outlive its target"
                ),
                "unexpected rejection gate: {error}"
            );
        }
        Ok(())
    }
}

#[test]
fn universal_trait_method_bounds_can_weaken_but_cannot_strengthen() -> Result {
    let a = RegionId::bound(0, 0)?;
    for (expected_region, expected_count, actual_region, actual_count, valid) in [
        (a, Some(1), a, Some(1), true),
        (a, Some(1), RegionId::STATIC, None, true),
        (RegionId::STATIC, None, a, Some(1), false),
    ] {
        let mut f = Fixture::new(&format!("trait A<'a>{{}}{METHODS}"))?;
        let expected = f.trait_bound(f.expected, &[expected_region], expected_count)?;
        let actual = f.trait_bound(f.actual, &[actual_region], actual_count)?;
        f.install(f.expected, &[expected]);
        f.install(f.actual, &[actual]);
        f.valid(valid)?;
    }
    Ok(())
}

#[test]
fn independent_and_correlated_requirements_preserve_their_direction() -> Result {
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    for independent in [false, true] {
        let mut f = Fixture::new(&format!("trait A<'a,'b>{{}}{METHODS}"))?;
        let expected = f.trait_bound(f.expected, &[a, if independent { b } else { a }], Some(2))?;
        let actual = f.trait_bound(f.actual, &[a, if independent { a } else { b }], Some(2))?;
        f.install(f.expected, &[expected]);
        f.install(f.actual, &[actual]);
        f.valid(independent)?;
    }
    Ok(())
}

#[test]
fn captured_owner_lifetimes_do_not_become_universal_method_permissions() -> Result {
    let a = RegionId::bound(0, 0)?;
    for trait_bound in [false, true] {
        let mut f = Fixture::new(
            "trait A<'a>{}trait Api<'x>{fn f<T>();}impl<'x>Api<'x>for u8{fn f<T>(){}}",
        )?;
        let (expected, actual) = if trait_bound {
            (
                f.trait_bound(f.expected, &[a], None)?,
                f.trait_bound(f.actual, &[a], Some(1))?,
            )
        } else {
            (
                f.lifetime(f.expected, a, None, false)?,
                f.lifetime(f.actual, a, Some(1), false)?,
            )
        };
        f.install(f.expected, &[expected]);
        f.install(f.actual, &[actual]);
        f.valid(false)?;
    }
    Ok(())
}

#[test]
fn all_regions_parameter_bounds_and_static_are_equivalent_but_tuple_premises_are_not() -> Result {
    let a = RegionId::bound(0, 0)?;
    for (expected_region, expected_count, tuple, actual_region, actual_count, valid) in [
        (a, Some(1), false, RegionId::STATIC, None, true),
        (RegionId::STATIC, None, false, a, Some(1), true),
        (a, Some(1), true, RegionId::STATIC, None, false),
    ] {
        let mut f = Fixture::new(METHODS)?;
        let expected = f.lifetime(f.expected, expected_region, expected_count, tuple)?;
        let actual = f.lifetime(f.actual, actual_region, actual_count, false)?;
        f.install(f.expected, &[expected]);
        f.install(f.actual, &[actual]);
        f.valid(valid)?;
    }
    Ok(())
}

impl Fixture {
    fn projected_lifetime(&mut self, method: Function, shorter: RegionId) -> Result<Clause> {
        let item = self.definition("Out")?;
        let subject = self.parameter(method)?;
        let trait_item = self.definition("A")?;
        let arguments = self
            .facts
            .types
            .intern_arguments(&[Argument::Lifetime(RegionId::bound(0, 0)?)])?;
        let trait_application = self
            .facts
            .types
            .intern_trait_application(traits::Application {
                item: trait_item,
                arguments,
            })?;
        let ty = self
            .facts
            .types
            .intern(Kind::Projection(projection::Application {
                item,
                subject,
                trait_application,
                arguments: GenericArgs::EMPTY,
            }))?;
        self.clause(
            Predicate {
                ty,
                bound: Bound::Outlives(shorter),
            },
            Some(1),
        )
    }
}

#[test]
fn projected_results_preserve_the_quantified_input_target_correlation() -> Result {
    let a = RegionId::bound(0, 0)?;
    for expected_static in [false, true] {
        let mut f = Fixture::new(&format!("trait A<'a>{{type Out;}}{METHODS}"))?;
        let caller_trait = f.trait_bound(f.expected, &[a], Some(1))?;
        let implementation_trait = f.trait_bound(f.actual, &[a], Some(1))?;
        let expected = f.projected_lifetime(
            f.expected,
            if expected_static { RegionId::STATIC } else { a },
        )?;
        let actual =
            f.projected_lifetime(f.actual, if expected_static { a } else { RegionId::STATIC })?;
        f.install(f.expected, &[caller_trait, expected]);
        f.install(f.actual, &[implementation_trait, actual]);
        f.valid(expected_static)?;
    }
    Ok(())
}

#[test]
fn conformance_elaborates_self_after_mapping_to_the_implementation_subject() -> Result {
    for (tuple, quantified, valid) in [
        (false, true, false),
        (true, true, false),
        (false, false, true),
    ] {
        let mut f = Fixture::new("trait Api{fn f();}impl<'x>Api for &'x u8{fn f(){}}")?;
        let owner = f
            .facts
            .associated
            .trait_owner(f.expected.definition)
            .ok_or("missing method trait owner")?;
        let mut ty = f
            .facts
            .associated
            .schema(owner)
            .ok_or("missing trait")?
            .self_type;
        if tuple {
            let elements = f.facts.types.intern_list(&[ty])?;
            ty = f.facts.types.intern(Kind::Tuple(elements))?;
        }
        let declared = f.clause(
            Predicate {
                ty,
                bound: Bound::Outlives(if quantified {
                    RegionId::bound(0, 0)?
                } else {
                    RegionId::STATIC
                }),
            },
            quantified.then_some(1),
        )?;
        let required = Predicate {
            ty: f.predicate.ty,
            bound: Bound::Outlives(RegionId::STATIC),
        }
        .into();
        f.install(f.expected, &[declared]);
        f.install(f.actual, &[required]);
        f.valid(valid)?;
    }
    Ok(())
}
