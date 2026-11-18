//! Ordinary semantic consumers retain real source Field calls before CFG/loan completion.
use crate::resolve as source;
use rustgm::{sema, syntax::ast::Kind};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[path = "access_sources/dereferenced_indexes.rs"]
mod dereferenced_indexes;

#[test]
fn field_semantic_producers_keep_actual_calls_and_complete_target_applications() -> Result {
    let mut failures = Vec::new();
    for name in [
        "shared",
        "mutable",
        "late-mut",
        "nested",
        "shadow",
        "raw-reference",
        "lifetime",
    ] {
        let text = source::fixture(&format!("pa31/tests/spec/source-field-deref/{name}.rs"))?;
        let report = source::source_provider(&text, "field_semantics")?;
        let result = sema::declarations::analyze(&report)
            .and_then(|facts| sema::bodies::analyze(&report, facts));
        let facts = match result {
            Ok(facts) => facts,
            Err(error) => {
                failures.push(format!("{name}: {error}"));
                continue;
            }
        };
        let plans = facts
            .nodes
            .iter()
            .flatten()
            .filter(|node| {
                matches!(
                    node.operation,
                    Some(sema::model::Operation::Field { plan: Some(_), .. })
                )
            })
            .count();
        if name == "shadow" {
            assert_eq!(plans, 0);
        } else {
            assert!(plans > 0, "{name}: no actual field plan");
        }
        for site in facts.methods.calls.keys() {
            assert!(
                !matches!(
                    report
                        .package
                        .unit(site.file)
                        .configured
                        .ast
                        .node(site.node)
                        .kind,
                    Kind::Field { .. }
                ),
                "{name}: a Field step overwrote a single-call fact"
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn field_calls_preserve_mutability_lifetime_move_and_const_requirements() -> Result {
    let mut failures = Vec::new();
    let mut count = 0;
    for line in source::fixture("pa31/tests/spec/source-field-deref/cases.tsv")?.lines() {
        let (name, expected) = line.split_once('\t').ok_or("field case columns")?;
        let text = source::fixture(&format!("pa31/tests/spec/source-field-deref/{name}"))?;
        let result = source::source_provider(&text, "field_consumer").and_then(|report| {
            let rustgm::pipeline::Analysis {
                mut facts,
                prepared,
            } = rustgm::pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
            rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())
                .map_err(|e| e.to_string())
        });
        match (expected, result) {
            ("0", Ok(())) => {}
            ("1", Err(error))
                if ![
                    "not implemented",
                    "invalid or exhausted",
                    "budget exceeded",
                    "lost its actual",
                    "changed its actual",
                    "field construction",
                    "field call lost",
                    "field output view",
                    "missing associated",
                    "unknown feature",
                    "source retention lacks",
                    "attribute retention lacks",
                    "caller inputs exceed source work/storage budget",
                    "source contract work or retention exceeds its input-graph bound",
                    "source header contract query work exceeds its input-graph bound",
                ]
                .iter()
                .any(|fence| error.contains(fence)) => {}
            (_, result) => failures.push(format!(
                "{name}: {}",
                result.map_or_else(|e| e, |()| "accepted".into())
            )),
        }
        count += 1;
    }
    assert_eq!(count, 19);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn field_call_order_temporary_once_shadow_and_const_mutability_reach_native() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/async-iterator-v1/field-native");
    std::fs::create_dir_all(&folder)?;
    for (name, observer, expected) in [
        (
            "native",
            "observer.c",
            "call=1\nshared=13\ncall=2\nlate=23\ncall=3\ncall=1\ntemporary=13\ncall=4\ncall=1\nnested=13\nshadow=41\n",
        ),
        ("const-mut", "const-observer.c", "const=23\n"),
    ] {
        let text = source::fixture(&format!("pa31/tests/spec/source-field-deref/{name}.rs"))?;
        let report = source::source_provider(&text, "field_native")?;
        let rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        } = rustgm::pipeline::analyze_source(&report)?;
        let program = rustgm::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            rustgm::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        crate::ffi_program::check(
            &root,
            &folder,
            name,
            &program,
            &source::fixture(&format!("pa31/tests/spec/source-field-deref/{observer}"))?,
            expected,
        )?;
    }
    Ok(())
}

#[test]
fn source_field_receiver_borrow_reads_union_storage_in_its_lexical_context() -> Result {
    for (name, accepted) in [("union-borrow-safe", false), ("union-borrow-unsafe", true)] {
        let text = source::fixture(&format!("pa31/tests/spec/source-field-deref/{name}.rs"))?;
        let report = source::source_provider(&text, "union_field_receiver")?;
        let result = rustgm::pipeline::analyze_source(&report);
        if accepted {
            let rustgm::pipeline::Analysis {
                mut facts,
                prepared,
            } = result?;
            rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())?;
        } else {
            let error = result
                .err()
                .ok_or("unsafe union receiver borrow accepted")?;
            assert!(
                error
                    .to_string()
                    .contains("union field access requires an unsafe context"),
                "{error}"
            );
        }
    }
    Ok(())
}

const BOX_FIELD_OWNER: &str = r#"
mod owning {
    use super::{Global, Sized, Unique};
    #[lang="owned_box"] pub struct Box<T:?Sized,A=Global>(Unique<T>,A);
    pub fn near(x: &Box<super::Bytes>) -> &Unique<super::Bytes> { &x.0 }
}
#[repr(transparent)] pub struct Bytes(pub [u8]);
"#;
const DEREF_FIELD_OWNER: &str = r"
mod owning {
    use super::{Bytes, Deref};
    pub struct Wrap(u8, Bytes);
    impl Deref for Wrap {
        type Target = Bytes;
        fn deref(&self) -> &Bytes { &self.1 }
    }
    pub fn near(x: &Wrap) -> &u8 { &x.0 }
}
pub struct Bytes(pub u8);
";

#[test]
fn field_lookup_passes_private_candidates_without_granting_access() -> Result {
    use rustgm::{
        resolve::unit::{FileId, NodeKey},
        sema::{members::Selection, model::Operation},
        syntax::ast::NodeId,
    };

    let box_source = source::fixture("pa31/tests/spec/explicit-dynamic-call-members/prefix.rs")?;
    let (box_prefix, _) = box_source
        .split_once("#[lang=\"owned_box\"]")
        .ok_or("owned Box fixture prefix")?;
    let deref_source = source::fixture("pa31/tests/spec/source-field-deref/shadow.rs")?;
    let (deref_prefix, _) = deref_source
        .split_once("pub struct Leaf")
        .ok_or("Deref fixture prefix")?;
    for (prefix, owner, near, tail, has_plan) in [
        (
            box_prefix,
            BOX_FIELD_OWNER,
            "Box",
            "pub fn far(x: &owning::Box<Bytes>) -> &[u8] { &x.0 }",
            false,
        ),
        (
            deref_prefix,
            DEREF_FIELD_OWNER,
            "Wrap",
            "pub fn far(x: &owning::Wrap) -> &u8 { &x.0 }",
            true,
        ),
    ] {
        let text = format!("{prefix}{owner}{tail}");
        let report = source::source_provider(&text, "private_field_candidates")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let ast = &report.package.unit(FileId(0)).configured.ast;
        let mut selected = Vec::new();
        let first = text.find("pub fn near").ok_or("near control")?;
        for (index, node) in ast.nodes.iter().enumerate() {
            if (node.span.start as usize) < first || !matches!(node.kind, Kind::Field { .. }) {
                continue;
            }
            let key = NodeKey {
                file: FileId(0),
                node: NodeId::new(u32::try_from(index)?),
            };
            if let Some(Operation::Field {
                field: Selection::Nominal(field),
                plan,
            }) = facts.node(key).and_then(|fact| fact.operation)
            {
                let id = facts
                    .members
                    .identity(field)
                    .ok_or("selected field identity")?;
                let owner = &report.resolution.definitions[id.owner.0 as usize];
                selected.push((
                    report.package.names.get(owner.name),
                    id.field,
                    plan.is_some(),
                ));
            }
        }
        assert_eq!(selected, [(near, 0, false), ("Bytes", 0, has_plan)]);
    }

    for tail in [
        "pub fn reject(x: &owning::Box<u8>) { let _ = &x.0; }",
        "mod hidden { pub struct Both(u8); }\n\
         pub fn reject(x: &owning::Box<hidden::Both>) { let _ = &x.0; }",
    ] {
        let text = format!("{box_prefix}{BOX_FIELD_OWNER}{tail}");
        let report = source::source_provider(&text, "private_field_rejection")?;
        let error = rustgm::pipeline::analyze_source(&report)
            .err()
            .ok_or("private field was accepted")?;
        let site = text.rfind("x.0").ok_or("private field source")?;
        assert_eq!(error.file.source(), Some(FileId(0)));
        assert!(
            (site..site + 3).contains(&(error.span.start as usize)),
            "{error}"
        );
        assert_eq!(error.message, "aggregate field is private");
    }
    Ok(())
}
