//! An observed source witness cannot select another initializer, owner or result.
use super::*;
use crate::sema::values::execution::{Executor, Progress, Request, Target};
use std::{cell::Cell, rc::Rc};
struct Capture(Rc<Cell<usize>>);
impl Executor for Capture {
    fn check(
        &mut self,
        _report: &crate::resolve::expansion::Report,
        facts: &mut Declarations,
        target: Target,
    ) -> Result<(), Error> {
        let Target::SourceAssociated { source, result } = target else {
            return Ok(());
        };
        assert!(source.validate(facts, result).is_ok());
        let (entry, caller) = source
            .entry(facts)
            .map_err(|e| Error::at(super::super::FileId(0), 0, e))?;
        assert_eq!(
            entry.selected.result_type(facts),
            Err("selected constant arguments are not concrete")
        );
        let other = facts
            .functions
            .iter()
            .find_map(|f| f.body.filter(|body| *body != caller.source))
            .ok_or_else(|| Error::at(super::super::FileId(0), 0, "missing foreign body"))?;
        let forged = {
            let catalog = &mut facts
                .source_computations
                .as_deref_mut()
                .ok_or_else(|| Error::at(super::super::FileId(0), 0, "missing source catalog"))?
                .0;
            let foreign = catalog
                .caller(
                    catalog::CallerKey {
                        source: other,
                        ..caller
                    },
                    catalog::Publication::new([0..0, 0..0])
                        .map_err(|e| Error::at(super::super::FileId(0), 0, e))?,
                )
                .map_err(|e| Error::at(super::super::FileId(0), 0, e))?;
            [
                catalog::Use {
                    caller: foreign,
                    ..entry
                },
                catalog::Use {
                    original: other,
                    ..entry
                },
                catalog::Use {
                    selected: Instance {
                        arguments: crate::types::GenericArgs::EMPTY,
                        ..entry.selected
                    },
                    ..entry
                },
            ]
            .map(|row| catalog.retain_use(row).map(SourceComputation))
        };
        for witness in forged {
            assert!(
                witness
                    .map_err(|e| Error::at(super::super::FileId(0), 0, e))?
                    .validate(facts, result)
                    .is_err()
            );
        }
        assert!(
            source
                .validate(facts, TypeId::primitive(crate::types::Primitive::U16))
                .is_err()
        );
        let region = facts
            .types
            .arguments(entry.selected.arguments)
            .and_then(|mut a| {
                a.find_map(|a| match a {
                    Argument::Lifetime(r) if r != RegionId::STATIC => Some(r),
                    _ => None,
                })
            })
            .ok_or_else(|| Error::at(super::super::FileId(0), 0, "missing source lifetime"))?;
        let record = &mut facts.body_regions[region.0 as usize - 1];
        let previous = *record;
        record.body = other;
        assert!(source.validate(facts, result).is_err());
        facts.body_regions[region.0 as usize - 1] = previous;
        assert!(source.validate(facts, result).is_ok());
        self.0.set(self.0.get() + 1);
        Err(Error::at(
            super::super::FileId(0),
            0,
            "captured checked source witness",
        ))
    }
    fn advance(
        &mut self,
        _: &crate::resolve::expansion::Report,
        _: &mut Declarations,
        _: &Request,
    ) -> Result<Progress, Error> {
        Ok(Progress::Complete(crate::sema::values::Constant::scalar(
            crate::sema::model::Scalar::Integer(1),
        )))
    }
}
#[test]
fn foreign_and_stale_source_witnesses_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa25/tests/spec/lifetime-initializer-execution/fresh-dispatcher.rs"),
    )?;
    let report = crate::test_source::resolve(&text, true)?;
    let count = Rc::new(Cell::new(0));
    let facts = crate::sema::declarations::analyze_with_executor(
        &report,
        Box::new(Capture(Rc::clone(&count))),
    )?;
    let result = crate::pipeline::analyze(&report, facts);
    let error = result.err().ok_or("source capture did not stop analysis")?;
    assert!(
        error
            .to_string()
            .contains("captured checked source witness"),
        "{error}"
    );
    assert_eq!(count.get(), 1);
    Ok(())
}
#[test]
fn common_request_layout_and_closed_argument_rules_are_unchanged() {
    assert_eq!(size_of::<SourceComputation>(), 8);
    assert_eq!(size_of::<Target>(), 28);
    assert_eq!(size_of::<Request>(), 56);
    assert_eq!(size_of::<(Target, usize)>(), 40);
    assert_eq!(size_of::<crate::const_eval::code::Catalog>(), 192);
    eprintln!(
        "caller-layout source={} target={} request={} code={} declarations={}",
        size_of::<SourceComputation>(),
        size_of::<Target>(),
        size_of::<Request>(),
        size_of::<crate::const_eval::code::Code>(),
        size_of::<Declarations>()
    );
    assert_eq!(size_of::<crate::const_eval::code::Code>(), 336);
    assert_eq!(size_of::<crate::body::ir::Rvalue>(), 20);
    assert_eq!(size_of::<crate::body::ir::Value>(), 24);
    assert_eq!(size_of::<crate::body::ir::Body>(), 464);
}

#[test]
fn closed_result_gate_does_not_admit_caller_regions_or_new_pointer_families()
-> Result<(), &'static str> {
    let mut types = crate::types::Store::default();
    let byte = TypeId::primitive(crate::types::Primitive::U8);
    for (region, mutable, accepted) in [
        (RegionId::STATIC, false, true),
        (RegionId::STATIC, true, false),
        (RegionId(1), false, false),
        (RegionId::bound(0, 0)?, false, false),
    ] {
        let ty = types.intern(Kind::Reference {
            mutable,
            region,
            referent: byte,
        })?;
        assert_eq!(closed_result(&types, ty), accepted);
        if !mutable {
            let fields = types.intern_list(&[ty, byte])?;
            let tuple = types.intern(Kind::Tuple(fields))?;
            assert_eq!(closed_result(&types, tuple), accepted);
        }
    }
    let pointer = types.intern(Kind::Pointer {
        mutable: false,
        pointee: byte,
    })?;
    assert!(!closed_result(&types, pointer));
    Ok(())
}

struct Observed {
    inner: Box<dyn Executor>,
    counts: Rc<Cell<[usize; 2]>>,
}
impl Executor for Observed {
    fn check(
        &mut self,
        report: &crate::resolve::expansion::Report,
        facts: &mut crate::sema::model::Declarations,
        target: Target,
    ) -> Result<(), crate::resolve::package::Error> {
        self.inner.check(report, facts, target)
    }
    fn advance(
        &mut self,
        report: &crate::resolve::expansion::Report,
        facts: &mut crate::sema::model::Declarations,
        request: &Request,
    ) -> Result<Progress, crate::resolve::package::Error> {
        let result = self.inner.advance(report, facts, request);
        if matches!(request.target, Target::SourceAssociated { .. }) {
            let mut counts = self.counts.get();
            counts[0] += 1;
            counts[1] += usize::from(matches!(result, Ok(Progress::Prepare(_))));
            self.counts.set(counts);
        }
        result
    }
}

#[test]
fn wide_source_requests_revalidate_after_external_pattern_preparation()
-> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    for width in [8, 32, 128] {
        for calls in [8, 32] {
            let params = (0..width)
                .map(|n| format!("'a{n}"))
                .collect::<Vec<_>>()
                .join(",");
            let args = vec!["'a"; width].join(",");
            let fields = (0..width)
                .map(|n| format!("&'a{n} u8"))
                .collect::<Vec<_>>()
                .join(",");
            let mut text = format!(
                "const M:u8=7;struct S<{params}>({fields});impl<{params}>S<{params}>{{const C:u8={{let n=q0();n}};}}\nfn f<'a>(_:&'a u8,x:u8)->bool{{match x{{S::<{args}>::C=>true,_=>false}}}}\n"
            );
            for n in 0..calls {
                let result = if n + 1 < calls {
                    format!("q{}()", n + 1)
                } else {
                    "7".into()
                };
                writeln!(text, "const fn q{n}()->u8{{match 7u8{{M=>{result},_=>0}}}}")?;
            }
            writeln!(text, "fn main(){{let n=7;let _=f(&n,7);}}")?;
            let report = crate::test_source::resolve(&text, true)?;
            let context = crate::pipeline::execution::Context::default();
            let counts = Rc::new(Cell::new([0; 2]));
            let facts = crate::sema::declarations::analyze_with_executor(
                &report,
                Box::new(Observed {
                    inner: context.executor(),
                    counts: Rc::clone(&counts),
                }),
            )?;
            let analysis = crate::pipeline::analyze(&report, facts)?;
            assert_eq!(counts.get(), [calls + 1, calls]);
            eprintln!(
                "source width={width} calls={calls} advance/prepare={:?} machine_resumes={}",
                counts.get(),
                analysis.facts.constant_work.resumes
            );
        }
    }
    Ok(())
}

#[test]
fn ordinary_body_source_computation_retains_its_real_caller()
-> Result<(), Box<dyn std::error::Error>> {
    let mut report = crate::test_source::resolve(
        "struct S<'a>(&'a u8);impl<'a>S<'a>{const C:u8={let n=7;n};}fn f<'a>(_:&'a u8)->u8{S::<'a>::C}fn main(){let n=7;let _=f(&n);}",
        true,
    )?;
    let prepared = crate::pipeline::analyze_source(&report)?;
    let mut facts = prepared.facts;
    let entry = crate::sema::entry::select(&mut report, &facts)?;
    crate::lower::instances::build_prepared(&report, &mut facts, entry, prepared.prepared)?;
    assert!(facts.constant_work.source_context_bytes > 0);
    Ok(())
}

#[test]
fn specialized_body_callers_keep_distinct_namespaces() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/associated-callers");
    let mut failures = Vec::new();
    for name in [
        "generic-body",
        "nested-initializer",
        "inline-caller",
        "generic-reference",
    ] {
        let text = std::fs::read_to_string(root.join(format!("{name}.rs")))?;
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            let mut report = crate::test_source::resolve(&text, true)?;
            let prepared = crate::pipeline::analyze_source(&report)?;
            let mut facts = prepared.facts;
            let entry = crate::sema::entry::select(&mut report, &facts)?;
            crate::lower::instances::build_prepared(&report, &mut facts, entry, prepared.prepared)?;
            Ok(())
        })();
        if let Err(error) = result {
            failures.push(format!("{name}: {error}"));
        } else {
            eprintln!("caller-source {name}: accepted");
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn legacy_ast_completed_operands_preserve_ordinary_body_selection()
-> Result<(), Box<dyn std::error::Error>> {
    for path in [
        "projections/definition-readiness/lowir/generic-callable-early.rs",
        "projections/definition-readiness/lowir/generic-callable-late.rs",
        "selection/lookup-readiness/lowir/interleaved-trait-lookup.rs",
        "constants/value/lowir/qualified_lifetime.rs",
    ] {
        let text = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../pa20/tests/spec")
                .join(path),
        )?;
        let mut report = crate::test_source::resolve(&text, true)?;
        let prepared = crate::pipeline::analyze_source(&report)?;
        let mut facts = prepared.facts;
        for (&input, &selected) in &facts.constant_targets {
            eprintln!(
                "legacy {path} input={input:?} selected={selected:?} cached={}",
                crate::sema::values::Reader::new(&facts)
                    .associated(selected)
                    .is_some()
            );
        }
        let entry = crate::sema::entry::select(&mut report, &facts)?;
        crate::lower::instances::build_prepared(&report, &mut facts, entry, prepared.prepared)?;
    }
    Ok(())
}
