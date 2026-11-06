//! Selected cleanup qualifies actual payloads while preserving ordinary body checks.
use crate::resolve as source;
use rustgm::{lower, pipeline};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn compile(name: &str) -> Result {
    demand_work(name).map(drop)
}
fn demand_work(name: &str) -> Result<usize> {
    let text = source::fixture(&format!("pa31/tests/spec/source-const-cleanup/{name}.rs"))?;
    let report = source::source_provider(&text, "selected_const_cleanup")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    Ok(facts.body_work.canonical_steps)
}
#[test]
fn sparse_array_qualification_work_does_not_expand_runtime_length() -> Result {
    let small = demand_work("arysm")?;
    let large = demand_work("arybig")?;
    eprintln!("array qualification canonical work: length8={small}, length1048576={large}");
    assert!(
        large.abs_diff(small) <= 128,
        "array length expanded work: {small} vs {large}"
    );
    Ok(())
}
#[test]
fn source_copy_fields_preserve_no_drop_and_explicit_whole_obligations() -> Result {
    let mut mismatches = Vec::new();
    for (name, positive) in [
        ("cpfld", true),
        ("cpmiss", false),
        ("cpbad", false),
        ("cpwhol", false),
    ] {
        match compile(name) {
            Ok(()) if !positive => {
                mismatches.push(format!("{name}: omitted actual required obligation"));
            }
            Err(error) if positive => mismatches.push(format!("{name}: {error}")),
            Err(error) => {
                let cause = error.to_string();
                eprintln!("{name}: rejected (Rust positive=false): {cause}");
                if !(cause.contains("const Destruct")
                    || cause.contains("const Drop")
                    || cause.contains("can't drop"))
                {
                    mismatches.push(format!("{name}: unexpected rejection cause: {cause}"));
                }
            }
            Ok(()) => eprintln!("{name}: accepted (Rust positive=true)"),
        }
    }
    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(mismatches.join("\n").into())
    }
}
#[test]
fn source_scoped_obligations_and_field_borrows_preserve_qualification_laws() -> Result {
    let mut errors = Vec::new();
    for (name, positive) in [
        ("rfield", true),
        ("gcon", true),
        ("rlocal", true),
        ("mfield", false),
        ("gmiss", false),
    ] {
        match compile(name) {
            Ok(()) => {
                eprintln!("{name}: accepted (Rust positive={positive})");
                if !positive {
                    errors.push(format!("{name}: const cleanup omitted"));
                }
            }
            Err(error) => {
                let cause = error.to_string();
                eprintln!("{name}: rejected (Rust positive={positive}): {cause}");
                if positive || !(cause.contains("const Drop") || cause.contains("const Destruct")) {
                    errors.push(format!("{name}: {cause}"));
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n").into())
    }
}
#[test]
fn value_history_matches_pinned_local_projection_law() -> Result {
    let mut mismatches = Vec::new();
    for (name, positive) in [
        ("hmov", true),
        ("hmvk", true),
        ("hmvn", false),
        ("hcop", true),
        ("hcpn", false),
        ("hjoin", true),
        ("hjbad", false),
        ("hmraw", false),
        ("hmshr", true),
        ("hrnew", true),
        ("hrold", false),
        ("hrpre", false),
        ("hcont", false),
        ("hcntp", false),
        ("hsub", false),
        ("hsubp", false),
        ("hwrap", true),
        ("harray", true),
    ] {
        match compile(name) {
            Ok(()) => {
                eprintln!("{name}: accepted (Rust positive={positive})");
                if !positive {
                    mismatches.push(format!("{name}: accepted invalid cleanup"));
                }
            }
            Err(error) => {
                let cause = error.to_string();
                eprintln!("{name}: rejected (Rust positive={positive}): {cause}");
                if positive
                    || !(cause.contains("const Drop")
                        || cause.contains("const Destruct")
                        || cause.contains("can't drop"))
                {
                    mismatches.push(format!("{name}: {cause}"));
                }
            }
        }
    }
    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(mismatches.join("\n").into())
    }
}
#[test]
fn scoped_qualifier_preserves_actual_constructor_move_and_lifetime_values() -> Result {
    for name in ["prknow", "prmove", "taggen", "taglif"] {
        compile(name)?;
    }
    Ok(())
}
#[test]
fn actual_known_payloads_moves_and_wrappers_keep_eligibility_provenance() -> Result {
    let mut errors = Vec::new();
    for name in [
        "prknow", "prmove", "tagmv", "epart", "epart2", "snest", "dwrap", "dpass", "dstrct",
        "dowmv", "taglif", "taggen", "tagshr", "aryone", "arybig", "arysm",
    ] {
        match compile(name) {
            Ok(()) => eprintln!("{name}: accepted"),
            Err(error) => errors.push(format!("{name}: {error}")),
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n").into());
    }
    Ok(())
}
#[test]
fn independent_fresh_unknown_alias_and_partial_struct_values_stay_demanded() -> Result {
    for name in [
        "prunk", "tagbad", "tagarg", "eunk", "dfresh", "denum", "dfarr", "dsib", "dother", "dself",
        "drep", "dunk2", "tagmut", "tagraw", "fulcpy", "cppart", "tagpar", "arybad", "arymov",
    ] {
        let error = compile(name)
            .err()
            .ok_or_else(|| format!("{name}: required const cleanup omitted"))?;
        let message = error.to_string();
        assert!(
            message.contains("const Drop")
                || message.contains("const Destruct")
                || message.contains("can't drop"),
            "{name}: {message}"
        );
        assert!(!message.contains("not implemented"), "{name}: {message}");
    }
    Ok(())
}
#[test]
fn contradictory_tags_never_waive_body_typing_reads_moves_loans_or_calls() -> Result {
    for (name, cause) in [
        ("dborrow", "loan"),
        ("dread", "initializ"),
        ("dmove", "move"),
        ("dunsafe", "unsafe"),
        ("dcall", "const"),
        ("dbound", "can't drop"),
    ] {
        let error = compile(name)
            .err()
            .ok_or_else(|| format!("{name}: invalid ordinary body accepted"))?;
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    Ok(())
}

#[test]
fn typed_constructor_calls_retain_actual_member_identity() -> Result {
    for name in ["prknow", "taggen", "taglif"] {
        trace(name)?;
    }
    Ok(())
}
fn emitted(
    name: &str,
) -> Result<(
    rustgm::sema::model::Declarations,
    rustgm::body::templates::Template,
)> {
    use rustgm::{body, sema};
    let text = source::fixture(&format!("pa31/tests/spec/source-const-cleanup/{name}.rs"))?;
    let report = source::source_provider(&text, "selected_constructor_trace")?;
    let facts = sema::declarations::analyze(&report)?;
    // This is a typed emission observer, not a completed const-definition check.
    let facts = sema::bodies::analyze(&report, facts)?;
    let function = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "probe"
        })
        .ok_or("probe")?;
    let template = body::templates::Prepared::default().function(&report, &facts, function)?;
    Ok((facts, template))
}
fn trace(name: &str) -> Result {
    use rustgm::{body, types::Kind};
    let (facts, template) = emitted(name)?;
    let mut calls = 0;
    for (index, value) in template.body.values.iter().enumerate() {
        eprintln!(
            "value{index} {:?} {:?}",
            facts.types.kind(value.ty),
            value.kind
        );
        if let body::ir::Rvalue::Call {
            callee,
            signature,
            arguments,
        } = value.kind
        {
            let callee = template
                .body
                .values
                .get(callee.0 as usize)
                .ok_or("callee")?;
            if let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(callee.ty)
                && let Some(variant) = facts
                    .members
                    .constructor(rustgm::resolve::model::DefId(item.0))
            {
                let member = facts
                    .members
                    .variants
                    .get(variant.0 as usize)
                    .ok_or("variant")?;
                eprintln!(
                    "{name}: constructor item={} variant={} ordinal={} signature={} arguments={}..{}",
                    item.0, variant.0, member.ordinal, signature.0, arguments.start, arguments.end
                );
                assert!(
                    matches!(facts.types.kind(value.ty), Some(Kind::Adt(owner, _)) if owner == member.owner)
                );
                calls += 1;
            }
        }
    }
    assert!(calls > 0, "constructor aggregate/call profile changed");
    Ok(())
}

#[test]
fn qualification_input_storage_and_move_coordinates_are_actual_typed_ir() -> Result {
    for name in ["hcop", "taglif", "rfield", "mfield", "gcon", "rlocal"] {
        let (facts, template) = emitted(name)?;
        eprintln!("CASE {name}");
        let mut needed = rustgm::types::destruction::NeedsDrop::in_environment(
            facts.destructors.implementations().iter().map(|d| d.item),
            std::sync::Arc::clone(&template.environment),
        );
        for clause in template.environment.trait_clauses()? {
            eprintln!("environment {clause:?}");
            if let rustgm::types::parameters::clauses::Clause::Predicate(predicate) = clause {
                eprintln!("subject {:?}", facts.types.kind(predicate.ty));
            }
        }
        for (index, local) in template.body.locals.iter().enumerate() {
            eprintln!(
                "local{index} needs_drop={:?}",
                needed.demand(&facts.types, &facts.aggregates, local.ty)
            );
            eprintln!(
                "local{index} {:?} mutable={} ty={:?} {:?}",
                local.kind,
                local.mutable,
                local.ty,
                facts.types.kind(local.ty)
            );
        }
        for (index, projection) in template.body.projections.iter().enumerate() {
            eprintln!(
                "projection{index} base={:?} {:?} ty={:?}",
                projection.base,
                projection.kind,
                facts.types.kind(projection.ty)
            );
        }
        for (index, statement) in template.body.statements.iter().enumerate() {
            eprintln!("statement{index} {:?}", statement.action);
        }
        for (index, value) in template.body.values.iter().enumerate() {
            eprintln!(
                "value{index} destruction={:?}",
                template.destruction.values.get(index)
            );
            eprintln!(
                "value{index} {:?} ty={:?}",
                value.kind,
                facts.types.kind(value.ty)
            );
        }
    }
    Ok(())
}
