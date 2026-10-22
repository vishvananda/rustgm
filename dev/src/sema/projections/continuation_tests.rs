//! Definition waits preserve graph sharing, exact member authority and callable lifetimes.
use super::{Availability, Cursor, Declarations, DefId, Obligations, Progress, ScopeId, TypeId};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::{
        self,
        load::{InputFile, Sources},
    },
    sema,
    support::source::SourceFile,
    types::{
        GenericArgs, ItemId, Kind, Primitive, Store, parameters::predicates::Bound,
        projection::Application, substitute::Request,
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
struct Fixture {
    facts: Declarations,
    roots: Vec<(DefId, TypeId)>,
}
fn fixture(count: usize, lifetimes: bool, restricted: bool) -> TestResult<Fixture> {
    let mut source = if lifetimes {
        "trait Q<'a>{type Out;}".to_owned()
    } else {
        "trait Q{type Out;}".to_owned()
    };
    for id in 0..count {
        write!(source, "struct S{id};")?;
        if lifetimes {
            let bound = if restricted { ":'static" } else { "" };
            write!(source, "impl<'a{bound}>Q<'a> for S{id}{{type Out=&'a u8;}}")?;
        } else {
            write!(source, "impl Q for S{id}{{type Out=u8;}}")?;
        }
    }
    let mut facts = analyze(source)?;
    let implementations: Vec<_> = facts
        .generics
        .implementations()
        .map(|value| (value.predicate, value.types[0]))
        .collect();
    let mut roots = Vec::new();
    for (predicate, (item, definition)) in implementations {
        let Bound::Trait(trait_application) = predicate.bound else {
            return Err("missing trait application".into());
        };
        let ty = facts.types.intern(Kind::Projection(Application {
            item: ItemId(item.0),
            subject: predicate.ty,
            trait_application,
            arguments: GenericArgs::EMPTY,
        }))?;
        roots.push((definition, ty));
    }
    Ok(Fixture { facts, roots })
}

#[expect(
    clippy::default_trait_access,
    reason = "semantic fixtures use the resolution entrypoint defaults without an expansion dependency"
)]
fn analyze(source: String) -> TestResult<Declarations> {
    let input = InputFile {
        identity: "cursor.rs".into(),
        path: "cursor.rs".into(),
        source: SourceFile::new(source.into_bytes())?,
    };
    let expanded = resolve::expansion::analyze(input, &mut NoFiles, &Default::default())?;
    let report = resolve::analyze_expanded(expanded)?;
    let facts = sema::declarations::analyze(&report)?;
    sema::bodies::analyze(&report, facts).map_err(Into::into)
}

fn pair(store: &mut Store, child: TypeId) -> Result<TypeId, &'static str> {
    let elements = store.intern_list(&[child, child])?;
    store.intern(Kind::Tuple(elements))
}

#[test]
fn a_wait_retains_complete_arguments_and_never_reselects_a_member() -> TestResult {
    use crate::types::arguments::{Argument, Constant};
    let source = "trait Q<T,const N:usize>{type Out;}struct S;impl<T,const N:usize>Q<T,N> for S{type Out=[T;N];}";
    for (primitive, length) in [(Primitive::U8, 3_u64), (Primitive::U16, 5)] {
        let mut facts = analyze(source.to_owned())?;
        let header = facts
            .generics
            .implementations()
            .next()
            .ok_or("missing header")?;
        let (member, definition) = header.types[0];
        let subject = header.predicate.ty;
        let Bound::Trait(application) = header.predicate.bound else {
            return Err("missing trait application".into());
        };
        let item = facts
            .types
            .trait_application(application)
            .ok_or("missing trait")?
            .item;
        let element = TypeId::primitive(primitive);
        let constant = facts.types.intern_constant(Constant {
            ty: TypeId::primitive(Primitive::Usize),
            bits: u128::from(length),
        })?;
        let arguments = facts
            .types
            .intern_arguments(&[Argument::Type(element), Argument::Const(constant)])?;
        let trait_application = facts
            .types
            .intern_trait_application(crate::types::traits::Application { item, arguments })?;
        let root = facts.types.intern(Kind::Projection(Application {
            item: ItemId(member.0),
            subject,
            trait_application,
            arguments: GenericArgs::EMPTY,
        }))?;
        let expected = facts.types.intern(Kind::Array { element, length })?;
        let template = facts.definitions[definition.0 as usize].take();
        let mut proof = Obligations::new(1);
        let mut cursor = Cursor::new(None, &proof, ScopeId::ROOT, Availability::Validated)?;
        let mut calls = 0;
        let mut provider = |_: &mut Declarations, _: Request| {
            calls += 1;
            Ok(None)
        };
        cursor.start(&mut facts, root)?;
        assert_eq!(
            cursor.advance(&mut facts, &mut proof, &mut provider)?,
            Progress::Definition(definition)
        );
        // This deliberately removes lookup authority, so a successful resume
        // must consume the already selected owner and complete arguments.
        facts.generics.implementations = None;
        facts.definitions[definition.0 as usize] = template;
        let Progress::Complete { ty, .. } =
            cursor.advance(&mut facts, &mut proof, &mut provider)?
        else {
            return Err("selected definition did not complete".into());
        };
        assert_eq!(ty, expected);
        assert_eq!(calls, 1);
    }
    Ok(())
}

#[test]
fn shared_graphs_resume_each_selection_once_and_reuse_completed_roots() -> TestResult {
    for count in [8, 32, 128] {
        let Fixture { mut facts, roots } = fixture(count, false, false)?;
        let mut templates = HashMap::default();
        let mut child = TypeId::primitive(Primitive::U8);
        let mut expected = child;
        for &(definition, root) in &roots {
            templates.insert(definition, pair(&mut facts.types, child)?);
            expected = pair(&mut facts.types, expected)?;
            facts.definitions[definition.0 as usize] = None;
            child = root;
        }
        let mut proof = Obligations::new(1);
        let mut cursor = Cursor::new(None, &proof, ScopeId::ROOT, Availability::Validated)?;
        let mut requests = HashSet::default();
        let mut provider = |_: &mut Declarations, request: Request| {
            assert!(requests.insert(request.alias), "selection was replayed");
            Ok(None)
        };
        cursor.start(&mut facts, child)?;
        let mut waited = HashSet::default();
        let steps = loop {
            match cursor.advance(&mut facts, &mut proof, &mut provider)? {
                Progress::Dependency(_) => {
                    return Err("validated registry has an unfinished header".into());
                }
                Progress::Definition(definition) => {
                    assert!(waited.insert(definition));
                    // Polling an unfinished dependency must not walk or select again.
                    for _ in 0..8 {
                        assert_eq!(
                            cursor.advance(&mut facts, &mut proof, &mut provider)?,
                            Progress::Definition(definition)
                        );
                    }
                    assert!(cursor.start(&mut facts, child).is_err());
                    facts.definitions[definition.0 as usize] = templates.remove(&definition);
                }
                Progress::Complete { ty, steps } => {
                    assert_eq!(ty, expected);
                    break steps;
                }
            }
        };
        assert_eq!(waited.len(), count);
        assert!(
            steps <= 24 * count + 32,
            "{count} selections used {steps} fold steps"
        );
        for &(_, root) in &roots {
            cursor.start(&mut facts, root)?;
            let Progress::Complete { steps, .. } =
                cursor.advance(&mut facts, &mut proof, &mut provider)?
            else {
                return Err("completed shared root waited again".into());
            };
            assert!(steps <= 2);
        }
        assert_eq!(requests.len(), count);
        assert!(proof.is_empty());
    }
    Ok(())
}

fn require(facts: &Declarations, proof: &Obligations) -> Result<usize, String> {
    use crate::types::{
        outlives::{Bounds, Queries},
        relations::Constraints,
        variance::Variances,
    };
    let variances = Variances::default();
    let bounds = Bounds::default();
    let callables = crate::types::callable::Templates::default();
    let mut queries = Queries::new(&facts.types, &bounds, &callables);
    let mut constraints =
        Constraints::new(&facts.types, &variances, 0, 100_000).map_err(|e| e.to_string())?;
    proof.require_source(&mut constraints, &mut queries, 0)?;
    constraints
        .prove()
        .map(|p| p.identities.variables().len())
        .map_err(|e| e.to_string())
}

#[test]
fn callable_proofs_survive_interleaved_waits_without_gaining_permissions() -> TestResult {
    for restricted in [false, true] {
        let Fixture { mut facts, roots } = fixture(16, true, restricted)?;
        let mut templates = HashMap::default();
        let mut functions = Vec::new();
        let mut expected = Vec::new();
        for &(definition, projection) in &roots {
            let template = facts.definitions[definition.0 as usize]
                .take()
                .ok_or("missing template")?;
            templates.insert(definition, template);
            let parameters = facts.types.intern_list(&[])?;
            functions.push(facts.types.intern(Kind::Function {
                parameters,
                result: projection,
                bound_regions: 1,
                unsafe_: false,
                convention: crate::types::callable::Convention::RUST,
            })?);
            expected.push(facts.types.intern(Kind::Function {
                parameters,
                result: template,
                bound_regions: 1,
                unsafe_: false,
                convention: crate::types::callable::Convention::RUST,
            })?);
        }
        let functions = facts.types.intern_list(&functions)?;
        let root = facts.types.intern(Kind::Tuple(functions))?;
        let expected = facts.types.intern_list(&expected)?;
        let expected = facts.types.intern(Kind::Tuple(expected))?;
        let mut proof = Obligations::new(1);
        let mut cursor = Cursor::new(None, &proof, ScopeId::ROOT, Availability::Validated)?;
        let mut requests = HashSet::default();
        let mut provider = |_: &mut Declarations, request: Request| {
            assert!(requests.insert((request.alias, request.scope)));
            Ok(None)
        };
        cursor.start(&mut facts, root)?;
        let mut waits = 0;
        loop {
            match cursor.advance(&mut facts, &mut proof, &mut provider)? {
                Progress::Dependency(_) => {
                    return Err("validated registry has an unfinished header".into());
                }
                Progress::Definition(definition) => {
                    waits += 1;
                    assert!(proof.regions.end > proof.regions.start);
                    proof.resume_at(proof.regions.end + 7)?;
                    facts.definitions[definition.0 as usize] = templates.remove(&definition);
                }
                Progress::Complete { ty, .. } => {
                    assert_eq!(ty, expected);
                    break;
                }
            }
        }
        assert_eq!(waits, 16);
        assert_eq!(requests.len(), waits);
        assert!(proof.scope_count() <= 2 * waits);
        let owned: usize = proof.owned_regions().map(|r| r.len()).sum();
        assert_eq!(proof.regions.len() - owned, 7 * waits);
        if restricted {
            assert!(require(&facts, &proof).is_err());
        } else {
            assert_eq!(require(&facts, &proof)?, owned);
        }
    }
    Ok(())
}
