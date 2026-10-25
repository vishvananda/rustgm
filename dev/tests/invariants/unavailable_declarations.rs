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

fn assert_prepared(trace: &Trace, width: usize) -> TestResult<usize> {
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
    Ok(trees)
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
        let trees = assert_prepared(&trace, width)?;
        assert_eq!(direct.imports.len(), 8 * width);
        let probe = Probe::start()?;
        let expanded = crate::test_source::resolve(&source, true)?;
        let trace = probe.finish()?;
        assert_eq!(assert_prepared(&trace, width)?, trees);
        assert_eq!(expanded.invocations.len(), 0);
        assert_eq!(expanded.captures.len(), 0);
        assert_eq!(expanded.package.marks.context_count(), 1);
        assert_eq!(
            expanded
                .definitions
                .iter()
                .filter(|definition| matches!(
                    definition.body,
                    crate::resolve::expansion::Body::Scoped {
                        mode: Transparency::Opaque,
                        ..
                    }
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
fn selected_scoped_calls_use_the_prepared_matcher_and_real_invocation() -> TestResult {
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
        let report = crate::test_source::resolve(&source, true)?;
        let trace = probe.finish()?;
        assert_eq!(
            trace
                .events
                .iter()
                .filter(|event| matches!(event, Event::Compile { .. }))
                .count(),
            1
        );
        assert_eq!(
            trace
                .events
                .iter()
                .filter(|event| matches!(event, Event::Published { .. }))
                .count(),
            1
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
        assert!(!selected.is_empty(), "{name}");
        for snapshot in selected {
            assert!(snapshot.owner.is_some());
            assert!(snapshot.captures <= report.captures.len());
            assert!(snapshot.invocations <= report.invocations.len());
            assert!(snapshot.marks <= report.package.marks.context_count());
            let calls: Vec<_> = report
                .invocations
                .iter()
                .filter(|call| call.definition == snapshot.target)
                .collect();
            assert_eq!(calls.len(), 1, "{name}");
            assert_eq!(calls[0].rule, Some(0));
            assert_eq!(
                report
                    .resolution
                    .introductions
                    .definition_owner(calls[0].context)?,
                snapshot.owner
            );
        }
        crate::pipeline::analyze_source(&report)?;
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
        crate::resolve::expansion::Body::Scoped { .. }
    )));
    assert_eq!(size_of::<crate::resolve::expansion::Body>(), 96);
    assert_eq!(size_of::<crate::resolve::expansion::Definition>(), 136);
    Ok(())
}

#[path = "unavailable_authority.rs"]
mod authority;

#[path = "format_declarations.rs"]
mod formatting;

#[path = "attribute_declarations.rs"]
mod attributes;
