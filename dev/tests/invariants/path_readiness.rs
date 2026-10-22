//! Lookup retains one trial while independent declarations allocate body lifetimes.
use super::super::{Mode, OwnedCaller, Progress, Query};
use crate::support::hash::HashMap;
use crate::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
        model::DefId,
    },
    sema::{
        model::{BodyRegion, BodyRegionOrigin, Declarations},
        traits::{
            heads::Head,
            implementations::{Availability, Implementation, Implementations, PendingHeader},
        },
    },
    support::source::SourceFile,
    types::{
        ItemId, Kind, Primitive, RegionId, TypeId, infer::Context, parameters::predicates::Bound,
    },
};
use std::{fmt::Write as _, path::Path};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}

fn fixture(depth: usize) -> TestResult<(Report, Declarations, DefId)> {
    let mut source = String::from("trait B0<'a>{}impl<'a>B0<'a>for &'a u8{}\n");
    for i in 1..=depth {
        writeln!(
            source,
            "trait B{i}<'a>{{}}impl<'a,T:B{}<'a>>B{i}<'a>for T{{}}",
            i - 1
        )?;
    }
    writeln!(
        source,
        "trait A<'a>{{const C:usize;}}impl<'a,T:B{depth}<'a>>A<'a>for T{{const C:usize=3;}}"
    )?;
    let input = InputFile {
        identity: "lookup.rs".into(),
        path: "lookup.rs".into(),
        source: SourceFile::new(source.into_bytes())?,
    };
    let expanded = resolve::expansion::analyze(input, &mut NoFiles, &Config::default())?;
    let report = resolve::analyze_expanded(expanded)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let member = facts
        .associated
        .members()
        .iter()
        .find(|member| facts.associated.trait_owner(member.definition).is_some())
        .ok_or("missing associated constant")?
        .definition;
    Ok((report, facts, member))
}

fn delay(facts: &mut Declarations) -> TestResult<HashMap<DefId, (ItemId, Implementation)>> {
    let mut delayed = HashMap::default();
    let mut seeds = Vec::new();
    for header in facts.generics.implementations() {
        let Bound::Trait(application) = header.predicate.bound else {
            return Err("missing trait application".into());
        };
        let item = facts
            .types
            .trait_application(application)
            .ok_or("invalid trait")?
            .item;
        let head = facts
            .types
            .kind(header.predicate.ty)
            .ok_or("missing header type")?;
        seeds.push(PendingHeader {
            owner: header.owner,
            item: item.into(),
            head: (!matches!(head, Kind::Parameter(_) | Kind::Projection(_)))
                .then_some(Head::canonical(head)),
        });
        delayed.insert(
            header.owner,
            (
                item,
                Implementation {
                    owner: header.owner,
                    source: header.source,
                    predicate: header.predicate,
                    methods: header.methods.clone(),
                    types: header.types.clone(),
                    constants: header.constants.clone(),
                },
            ),
        );
    }
    let mut registry = Implementations::default();
    registry.reserve(seeds)?;
    facts.generics.implementations = Some(Box::new(registry));
    Ok(delayed)
}

#[test]
fn a_suspended_path_retains_its_prefix_and_trial_without_publishing_probe_regions() -> TestResult {
    for depth in [8, 32, 128] {
        let (report, mut facts, member) = fixture(depth)?;
        let key = report.resolution.definitions[member.0 as usize]
            .node
            .ok_or("missing member source")?;
        let region = BodyRegion {
            body: key,
            source: key,
            origin: BodyRegionOrigin::Annotation,
        };
        facts.body_regions.push(region);
        let ty = facts.types.intern(Kind::Reference {
            region: RegionId(1),
            mutable: false,
            referent: TypeId::primitive(Primitive::U8),
        })?;
        let mut prefix = Context::default();
        let term = prefix.known(&facts.types, ty)?;
        let mut delayed = delay(&mut facts)?;
        let proofs = (
            facts.quantified_proofs.len(),
            facts.region_equalities.len(),
            facts.outlives.len(),
            facts.type_outlives.len(),
        );
        let initial_probes = facts.methods.selection_probes;
        let mut query = Query {
            prefix,
            term,
            mode: Mode::Trait { bound: false },
            members: vec![member].into_iter(),
            lexical: None,
            selected: None,
            trial: None,
            caller: OwnedCaller {
                availability: Availability::Declarations,
                environment: None,
                traits: None,
            },
        };
        let mut waits = 0;
        loop {
            let owner = match query.poll(&report, &mut facts, key)? {
                Progress::Ready(selected) => {
                    assert_eq!(selected, Some(member));
                    break;
                }
                Progress::Pending(owner) => owner,
            };
            let before = observations(&query, &facts)?;
            for _ in 0..8 {
                assert!(
                    matches!(query.poll(&report, &mut facts, key)?, Progress::Pending(actual) if actual == owner)
                );
                assert_eq!(observations(&query, &facts)?, before);
            }
            facts.body_regions.extend([region; 3]);
            let crate::sema::traits::selection::Dependency::Header(owner) = owner else {
                return Err("expected an implementation header dependency".into());
            };
            let (item, header) = delayed.remove(&owner).ok_or("unknown pending owner")?;
            facts
                .generics
                .implementations
                .as_mut()
                .ok_or("missing registry")?
                .prepare(&mut facts.types, item, header)?;
            waits += 1;
        }
        assert!(delayed.is_empty());
        assert_eq!(waits, depth + 2);
        assert_eq!(facts.body_regions.len(), 1 + 3 * waits);
        assert_eq!(
            (
                facts.quantified_proofs.len(),
                facts.region_equalities.len(),
                facts.outlives.len(),
                facts.type_outlives.len()
            ),
            proofs
        );
        assert_eq!(facts.methods.selection_probes - initial_probes, 1);
        assert_eq!(query.prefix.resolved(&mut facts.types, term)?, ty);
    }
    Ok(())
}

fn observations(
    query: &Query,
    facts: &Declarations,
) -> TestResult<(usize, usize, usize, usize, usize, u32)> {
    let trial = query.trial.as_ref().ok_or("missing retained trial")?;
    Ok((
        query.prefix.steps,
        trial.context.steps,
        facts.methods.selection_probes,
        facts.body_regions.len(),
        trial.pending.cursor,
        trial.pending.first,
    ))
}
