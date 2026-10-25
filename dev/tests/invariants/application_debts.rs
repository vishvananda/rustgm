//! Direct generic calls derive debts without sharing caller proof.
use super::*;

#[test]
fn imported_call_goals_are_derived_even_for_unused_originals() -> Result {
    use crates::generics as g;
    let bytes = provider_source(
        "const fn helper<T:Copy>(x:T)->T{x}pub const fn through<T:Copy>(x:T)->T{helper(x)}",
    )?;
    let interface = crates::decode(&bytes)?;
    let mut rows = interface
        .generic_payload()
        .ok_or("payload")?
        .legacy_rows()?
        .clone();
    let clauses = std::mem::take(&mut rows.clauses);
    for function in &mut rows.functions {
        let begin = u32::try_from(rows.clauses.len())?;
        let g::Declaration::Function { identity, .. } =
            &rows.declarations[function.declaration.0 as usize]
        else {
            return Err("function".into());
        };
        for clause in function.clauses.slice(&clauses)? {
            if identity.path().last().is_some_and(|s| s == "through")
                && matches!(clause.bound, g::Bound::Copy(_))
            {
                continue;
            }
            rows.clauses.push(clause.clone());
        }
        function.clauses = g::Range {
            start: begin,
            end: u32::try_from(rows.clauses.len())?,
        };
        function.declared_clauses = function.clauses.end - function.clauses.start;
    }
    let payload = g::Payload::new(interface.identity(), rows, &mut g::Work::default())?;
    let forged = crates::Interface::new(interface.identity().clone(), vec![], vec![])?
        .with_generic_payload(payload)?;
    let Err(error) = client(&crates::encode(&forged)?) else {
        return Err("omitted helper Copy goal authorized unused caller original".into());
    };
    let diagnostic = error
        .downcast_ref::<Error>()
        .ok_or("not a frontend proof diagnostic")?;
    assert!(
        matches!(
            &diagnostic.file,
            crate::resolve::package::DiagnosticFile::Imported(_)
        ),
        "{diagnostic:?}"
    );
    Ok(())
}
#[test]
fn application_arguments_cannot_capture_another_function_in_the_same_artifact() -> Result {
    use crates::generics as g;
    let bytes = provider_source(
        "const fn helper<T:Copy>(x:T)->T{x}pub const fn through<T:Copy>(x:T)->T{helper(x)}",
    )?;
    let interface = crates::decode(&bytes)?;
    let mut rows = interface
        .generic_payload()
        .ok_or("payload")?
        .legacy_rows()?
        .clone();
    let foreign = rows.types.iter().position(|ty|matches!(ty,g::Type::Parameter(reference) if matches!(rows.declarations[reference.0 as usize],g::Declaration::TypeParameter{owner,..} if owner.0 == 0))).ok_or("foreign root")?;
    rows.calls.as_mut().ok_or("calls")?.arguments[0] = g::TypeRef(u32::try_from(foreign)?);
    let mut work = g::Work::default();
    assert!(g::Payload::new(interface.identity(), rows, &mut work).is_err());
    assert!(work.steps > 0);
    Ok(())
}

#[test]
fn direct_call_preparation_tracks_calls_and_distinct_helpers_independently() -> Result {
    for axis in ["calls", "helpers"] {
        let mut previous = None;
        for width in [8, 32, 128] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
                "../pa29/tests/spec/generic-contracts/call-scaling-{axis}-{width}.rs"
            ));
            let bytes = provider_source(&std::fs::read_to_string(path)?)?;
            let interface = crates::decode(&bytes)?;
            let rows = interface
                .generic_payload()
                .ok_or("payload")?
                .legacy_rows()?;
            let calls = rows.calls.as_ref().ok_or("calls")?;
            assert_eq!(
                calls.applications.len(),
                if axis == "helpers" { width } else { 1 }
            );
            let mut retained = None;
            for uses in [1, 8] {
                let source = format!("fn main(){{{}}}", "let _=base::through(7u8);".repeat(uses));
                let (_, analysis) = client_source(&bytes, &source)?;
                let headers = analysis
                    .facts
                    .crates
                    .as_deref()
                    .and_then(crate::sema::crates::Linkage::original_work)
                    .ok_or("header work")?;
                let originals = analysis.prepared.imported.as_ref().ok_or("originals")?;
                let counts = (
                    headers.0,
                    headers.1,
                    originals.steps(),
                    originals.retained_bytes(),
                );
                if let Some(first) = retained {
                    assert_eq!(
                        counts, first,
                        "repeated consumer demand rebuilt application preparation"
                    );
                } else {
                    retained = Some(counts);
                }
                assert!(headers.2 <= headers.1);
                assert!(counts.0 <= width * 1000 + 2000 && counts.2 <= width * 1000 + 2000);
            }
            let counts = retained.ok_or("empty demand axis")?;
            if let Some((h, b, w, p)) = previous {
                assert!(
                    counts.0 > h
                        && counts.0 <= 4 * h
                        && counts.1 > b
                        && counts.1 <= 4 * b
                        && counts.2 > w
                        && counts.2 <= 4 * w
                        && counts.3 > p
                        && counts.3 <= 4 * p
                );
            }
            eprintln!(
                "axis={axis} width={width} applications={} header_work={} bound={} original_work={} payload={}",
                calls.applications.len(),
                counts.0,
                counts.1,
                counts.2,
                counts.3
            );
            previous = Some(counts);
        }
    }
    Ok(())
}

#[test]
fn imported_call_results_cannot_replace_independent_caller_parameters() -> Result {
    use crates::generics as g;
    let bytes = provider_source(
        "const fn helper<T:Copy>(x:T)->T{x}pub const fn through<T:Copy,U:Copy>(x:T)->T{helper(x)}",
    )?;
    let interface = crates::decode(&bytes)?;
    let mut rows = interface
        .generic_payload()
        .ok_or("payload")?
        .legacy_rows()?
        .clone();
    let unrelated = rows.types.iter().position(|ty|matches!(ty,g::Type::Parameter(reference) if matches!(rows.declarations[reference.0 as usize],g::Declaration::TypeParameter{owner,ordinal:1,..} if owner.0 == 1))).ok_or("unrelated actual parameter")?;
    let value = rows.originals[1]
        .values
        .iter_mut()
        .find(|v| matches!(v.kind, g::ValueKind::Call { .. }))
        .ok_or("call")?;
    value.ty = g::TypeRef(u32::try_from(unrelated)?);
    let payload = g::Payload::new(interface.identity(), rows, &mut g::Work::default())?;
    let interface = crates::Interface::new(interface.identity().clone(), vec![], vec![])?
        .with_generic_payload(payload)?;
    let Err(error) = client(&crates::encode(&interface)?) else {
        return Err("call result mismatch authorized an unused original".into());
    };
    let diagnostic = error
        .downcast_ref::<Error>()
        .ok_or("not a frontend proof diagnostic")?;
    assert!(
        matches!(
            &diagnostic.file,
            crate::resolve::package::DiagnosticFile::Imported(_)
        ),
        "{diagnostic:?}"
    );
    Ok(())
}
