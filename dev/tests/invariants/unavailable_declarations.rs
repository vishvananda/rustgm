//! Actual producers count matcher preparation; unavailable calls retain no work.
use super::*;
use crate::{
    expand::{config::Config, marks::Transparency},
    resolve::{
        self,
        load::{InputFile, Sources},
        model::{DefKind, Namespace},
        package::{ModuleId, Package},
    },
    support::source::SourceFile,
};
use std::{
    cell::{Cell, RefCell},
    path::Path,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::resolve) struct Snapshot {
    pub target: usize,
    pub owner: Option<NodeKey>,
    pub captures: usize,
    pub invocations: usize,
    pub marks: usize,
}
#[derive(Clone, Copy, Debug)]
pub(in crate::resolve) enum Event {
    Compile {
        module: ModuleId,
        key: NodeKey,
        owner: Option<NodeId>,
        trees: usize,
    },
    Published {
        module: ModuleId,
        key: NodeKey,
    },
    Selected(Snapshot),
    Denied(Snapshot),
}
#[derive(Default)]
struct Trace {
    events: Vec<Event>,
    truncated: bool,
}
thread_local! {
    static TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) };
    static LOST: Cell<bool> = const { Cell::new(false) };
}
pub(in crate::resolve) fn note(event: Event) {
    TRACE.with(|slot| {
        let Ok(mut slot) = slot.try_borrow_mut() else {
            LOST.set(true);
            return;
        };
        if let Some(trace) = slot.as_mut() {
            if trace.events.len() >= 4096 {
                trace.truncated = true;
            } else {
                trace.events.push(event);
            }
        }
    });
}
struct Probe {
    active: bool,
}
impl Probe {
    fn start() -> TestResult<Self> {
        TRACE.with(|slot| {
            let mut slot = slot.try_borrow_mut()?;
            if slot.is_some() {
                return Err("nested scoped declaration probe".into());
            }
            *slot = Some(Trace::default());
            LOST.set(false);
            Ok(Self { active: true })
        })
    }
    fn finish(mut self) -> TestResult<Trace> {
        let mut trace = TRACE.with(|slot| -> TestResult<Trace> {
            Ok(slot
                .try_borrow_mut()?
                .take()
                .ok_or("missing scoped trace")?)
        })?;
        trace.truncated |= LOST.get();
        self.active = false;
        assert!(!trace.truncated, "incomplete actual scoped trace");
        Ok(trace)
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        if self.active {
            TRACE.with(|slot| *slot.borrow_mut() = None);
        }
    }
}
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn loaded(source: &str) -> TestResult<Package> {
    Ok(resolve::load::load(
        InputFile {
            identity: "scoped.rs".into(),
            path: "scoped.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )?)
}

#[test]
fn real_scoped_declarations_compile_once_across_width_and_repeated_imports() -> TestResult {
    use std::fmt::Write;
    for width in [8_usize, 32, 128] {
        let mut source = "#![feature(decl_macro)]mod owner{".to_owned();
        for index in 0..width {
            write!(
                source,
                "pub macro m{index}($value:expr){{unresolved($value)}}"
            )?;
        }
        source.push('}');
        for repetition in 0..8 {
            for index in 0..width {
                write!(source, "use owner::m{index} as a{index}_{repetition};")?;
            }
        }
        source.push_str("pub fn answer()->u32{7u32}");
        let mut package = loaded(&source)?;
        let probe = Probe::start()?;
        let direct = resolve::analyze(&mut package)?;
        let trace = probe.finish()?;
        assert_eq!(
            trace
                .events
                .iter()
                .filter(|event| matches!(event, Event::Compile { .. }))
                .count(),
            width
        );
        assert_eq!(
            trace
                .events
                .iter()
                .filter(|event| matches!(event, Event::Published { .. }))
                .count(),
            width
        );
        let mut compiled = crate::support::hash::HashSet::default();
        let mut published = crate::support::hash::HashSet::default();
        let mut trees = 0;
        for event in &trace.events {
            match event {
                Event::Compile {
                    module,
                    key,
                    owner,
                    trees: count,
                } => {
                    assert!(owner.is_none());
                    assert!(*count > 0);
                    trees += *count;
                    assert!(compiled.insert((*module, *key)));
                }
                Event::Published { module, key } => {
                    assert!(published.insert((*module, *key)));
                }
                Event::Selected(_) | Event::Denied(_) => {
                    return Err("unused declaration executed".into());
                }
            }
        }
        assert_eq!(compiled, published);
        assert_eq!(direct.imports.len(), 8 * width);
        let probe = Probe::start()?;
        let expanded = crate::test_source::resolve(&source, true)?;
        let trace = probe.finish()?;
        assert_eq!(
            trace
                .events
                .iter()
                .filter(|event| matches!(event, Event::Compile { .. }))
                .count(),
            width
        );
        assert_eq!(
            trace
                .events
                .iter()
                .filter(|event| matches!(event, Event::Published { .. }))
                .count(),
            width
        );
        assert_eq!(expanded.invocations.len(), 0);
        assert_eq!(expanded.captures.len(), 0);
        assert_eq!(expanded.package.marks.context_count(), 1);
        assert_eq!(
            expanded
                .definitions
                .iter()
                .filter(|definition| matches!(
                    definition.body,
                    crate::resolve::expansion::Body::ScopedDeclaration(Transparency::Opaque)
                ))
                .count(),
            width
        );
        eprintln!(
            "scoped declarations width={width} aliases={} compiles={width} input-trees={trees} no-invocations; body={} definition={} descriptor={}",
            8 * width,
            size_of::<crate::resolve::expansion::Body>(),
            size_of::<crate::resolve::expansion::Definition>(),
            size_of::<ScopedDeclaration>()
        );
    }
    Ok(())
}

#[test]
fn selected_scoped_calls_fail_before_their_matching_capture_mark_and_invocation_work() -> TestResult
{
    for name in [
        "direct",
        "qualified",
        "alias",
        "forward",
        "shadow-builtin",
        "nested-legacy",
        "eager",
    ] {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/declarative-macros/declaration-invoke-{name}.rs"
        ))?;
        let probe = Probe::start()?;
        let error = crate::test_source::resolve(&source, true)
            .err()
            .ok_or("scoped invocation executed")?;
        let trace = probe.finish()?;
        assert!(
            error.contains("scoped macro invocation is not implemented"),
            "{name}: {error}"
        );
        let selected: Vec<_> = trace
            .events
            .iter()
            .filter_map(|event| {
                if let Event::Selected(snapshot) = event {
                    Some(*snapshot)
                } else {
                    None
                }
            })
            .collect();
        let denied: Vec<_> = trace
            .events
            .iter()
            .filter_map(|event| {
                if let Event::Denied(snapshot) = event {
                    Some(*snapshot)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(selected.len(), 1, "{name}");
        assert_eq!(selected, denied, "scoped child consumed work in {name}");
        assert!(selected[0].owner.is_some());
        eprintln!(
            "scoped denial {name}: target={} owner={:?} captures={} invocations={} marks={}",
            selected[0].target,
            selected[0].owner,
            selected[0].captures,
            selected[0].invocations,
            selected[0].marks
        );
    }
    Ok(())
}

#[test]
fn ordinary_macro_rules_allocate_no_scoped_descriptor_and_keep_their_real_execution() -> TestResult
{
    let probe = Probe::start()?;
    let report =
        crate::test_source::resolve("macro_rules!m{()=>{7u32};}fn answer()->u32{m!()}", true)?;
    let trace = probe.finish()?;
    assert!(trace.events.is_empty());
    assert_eq!(report.invocations.len(), 1);
    assert!(report.definitions.iter().any(|definition| matches!(
        definition.body,
        crate::resolve::expansion::Body::Declarative(_)
    )));
    assert!(!report.definitions.iter().any(|definition| matches!(
        definition.body,
        crate::resolve::expansion::Body::ScopedDeclaration(_)
    )));
    Ok(())
}

fn first_scoped(package: &Package) -> TestResult<(NodeId, Item, NodeId)> {
    let unit = &package.units[0];
    for owner in &unit.configured.owners {
        let mut node = owner.node;
        while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
            node = value;
        }
        if let Kind::Item(index) = unit.configured.ast.node(node).kind {
            let item = unit.configured.ast.items[index as usize];
            if matches!(item.kind, ItemKind::DeclarativeMacro { .. }) {
                return Ok((node, item, owner.node));
            }
        }
    }
    Err("missing real attributed scoped item".into())
}

#[test]
fn failed_actual_owner_feature_and_matcher_checks_publish_no_declaration() -> TestResult {
    for (prefix, attribute, body, wrong_owner) in [
        (
            "#![feature(rustc_attrs)]",
            "#[rustc_macro_transparency=\"opaque\"]",
            "7u32",
            false,
        ),
        (
            "#![feature(decl_macro,rustc_attrs)]",
            "#[rustc_macro_transparency=\"opaque\"]",
            "7u32",
            true,
        ),
        (
            "#![feature(decl_macro,rustc_attrs)]",
            "#[rustc_macro_transparency=\"opaque\"]",
            "$missing",
            false,
        ),
    ] {
        let mut package = loaded(&format!(
            "{prefix}{attribute}pub macro one(){{{body}}}{attribute}pub macro two(){{7u32}}"
        ))?;
        let (node, item, owner) = first_scoped(&package)?;
        let other=package.units[0].configured.owners.iter().find_map(|candidate| {
            if candidate.node==owner {return None;}
            let mut node=candidate.node;
            while let Kind::Attributed{value,..}=package.units[0].configured.ast.node(node).kind{node=value;}
            matches!(package.units[0].configured.ast.node(node).kind,Kind::Item(index)
                if matches!(package.units[0].configured.ast.items[index as usize].kind,ItemKind::DeclarativeMacro{..}))
                .then_some(candidate.node)
        }).ok_or("second actual configured scoped owner")?;
        let mut state = super::super::State::default();
        let mut resolver = state.bind(&mut package);
        let scope = resolver.new_scope(None, ModuleId(0), true)?;
        let context = resolver.context(ModuleId(0), scope);
        let probe = Probe::start()?;
        let error = resolver
            .declare_scoped(
                context,
                node,
                item,
                Access::Public,
                Some(if wrong_owner { other } else { owner }),
            )
            .err()
            .ok_or("invalid scoped input published")?;
        let trace = probe.finish()?;
        assert!(resolver.facts.definitions.is_empty());
        assert!(!resolver.facts.item_definitions.contains_key(&(
            ModuleId(0),
            node,
            Namespace::Macro
        )));
        assert!(
            !trace
                .events
                .iter()
                .any(|event| matches!(event, Event::Published { .. }))
        );
        if wrong_owner {
            assert!(error.message.contains("another item"), "{error}");
        }
    }
    Ok(())
}

#[test]
fn duplicates_never_replace_an_existing_actual_macro_binding() -> TestResult {
    let mut package =
        loaded("#![feature(decl_macro)]pub macro same(){7u32}pub macro same(){8u32}")?;
    let probe = Probe::start()?;
    let error = resolve::analyze(&mut package)
        .err()
        .ok_or("duplicate scoped items accepted")?;
    let trace = probe.finish()?;
    assert!(error.message.contains("duplicate"), "{error}");
    assert_eq!(
        trace
            .events
            .iter()
            .filter(|event| matches!(event, Event::Compile { .. }))
            .count(),
        2
    );
    assert_eq!(
        trace
            .events
            .iter()
            .filter(|event| matches!(event, Event::Published { .. }))
            .count(),
        1
    );
    Ok(())
}

#[test]
fn scoped_markers_have_no_legacy_textual_binding_or_external_metadata_permission() -> TestResult {
    use crate::expand::textual::{Lookup, Scope};
    let mut report = crate::test_source::resolve(
        "#![feature(decl_macro)]pub macro unused(){unresolved()}pub fn answer()->u32{7u32}",
        true,
    )?;
    let marker = report
        .definitions
        .iter()
        .position(|definition| {
            matches!(
                definition.body,
                crate::resolve::expansion::Body::ScopedDeclaration(_)
            )
        })
        .ok_or("actual scoped marker")?;
    let definition = &report.definitions[marker];
    assert_eq!(
        report
            .textual_scopes
            .lookup(Scope::ROOT, definition.name)
            .map_err(|error| error.message)?,
        Lookup::Missing
    );
    let key = definition.node.ok_or("actual source macro")?;
    assert!(
        report
            .resolution
            .definitions
            .iter()
            .any(|entry| entry.node == Some(key) && entry.kind == DefKind::Macro)
    );
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let identity = crate::support::crates::CrateIdentity::new("scope_export", "opponent")?;
    let error = crate::sema::crates::export(&report, &mut facts, identity)
        .err()
        .ok_or("unsupported macro metadata exported")?;
    assert!(error.message.contains("metadata profile"), "{error}");
    Ok(())
}

#[test]
fn forged_report_frames_cannot_borrow_an_unexecuted_scoped_allowance() -> TestResult {
    use crate::resolve::{bindings::KnownFeature, expansion::Body};
    let source =
        "#![feature(no_core,lang_items,decl_macro,rustc_attrs,allow_internal_unstable)]#![no_core]
#[lang=\"pointee_sized\"]trait PointeeSized{}#[lang=\"meta_sized\"]trait MetaSized:PointeeSized{}
#[lang=\"sized\"]trait Sized:MetaSized{}
#[allow_internal_unstable(panic_internals)]macro unused(){probe()}
macro_rules!live{()=>{probe()};}fn probe()->u32{7u32}fn answer()->u32{live!()}";
    let mut report = crate::test_source::source_provider(source, "unavailable_frame")?;
    let scoped = report
        .definitions
        .iter()
        .position(|definition| matches!(definition.body, Body::ScopedDeclaration(_)))
        .ok_or("actual unavailable scoped definition")?;
    let invocation = report
        .invocations
        .first()
        .ok_or("actual legacy invocation")?;
    let live = invocation.definition;
    let file = invocation.node.file;
    let unit = &report.package.units[file.0 as usize];
    let site = unit
        .configured
        .ast
        .nodes
        .iter()
        .enumerate()
        .find_map(|(index, node)| {
            if let Kind::Name(span, _) = node.kind
                && unit.configured.active[index]
                && unit
                    .source
                    .text()
                    .get(span.start as usize..span.end as usize)
                    == Some("probe")
            {
                Some(crate::expand::stream::Location { file, span })
            } else {
                None
            }
        })
        .ok_or("actual expanded probe token")?;
    assert!(
        !report
            .macro_allowances()
            .allows(site, &KnownFeature::PanicInternals.name())?
    );
    report.invocations[0].definition = scoped;
    let error = report
        .macro_allowances()
        .allows(site, &KnownFeature::PanicInternals.name())
        .err()
        .ok_or("unexecuted scoped marker supplied provenance")?;
    assert!(error.message.contains("no executed expansion frame"));
    let compiled = std::mem::replace(
        &mut report.definitions[live].body,
        Body::Builtin(crate::expand::builtin::Builtin::Line),
    );
    report.definitions[scoped].body = compiled;
    let error = report
        .macro_allowances()
        .allows(site, &KnownFeature::PanicInternals.name())
        .err()
        .ok_or("substituted body hid the actual scoped declaration")?;
    assert!(error.message.contains("no executed expansion frame"));
    Ok(())
}
