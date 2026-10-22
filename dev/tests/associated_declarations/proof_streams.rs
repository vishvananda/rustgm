//! Interleaved constant dependencies preserve one callable binder map per query.
use super::{native, object_outcome, semantic};
use std::fmt::Write as _;

fn program(count: usize, paid: bool) -> Result<String, std::fmt::Error> {
    let mut source = "struct X<const N:usize>([u8;N]);struct R<'a>(&'a u8);trait B<'a>{const N:usize;}impl<'a>B<'a>for R<'a>{const N:usize=1;}".to_owned();
    for index in 0..count {
        write!(source, "trait A{index}<'a>{{type Out;}}")?;
    }
    source.push_str("type F<T>=(");
    for index in 0..count {
        write!(source, "for<'a>fn(&'a u8)-><T as A{index}<'a>>::Out,")?;
    }
    source.push_str(");enum O<T>{None,Some(T)}fn first()->X<{(3usize,O::<F<u8>>::None).0}>{X([7;3])}fn main(){check(first().0[2]==7);let fs:F<u8>=(");
    for index in 0..count {
        write!(source, "f{index},")?;
    }
    source.push_str(");check((fs.0)(&3).0[0]==7);}");
    for index in 0..count {
        let restriction = if paid || index + 1 != count {
            ""
        } else {
            ":'static"
        };
        write!(
            source,
            "impl<'a{restriction}>A{index}<'a>for u8{{type Out=X<{{<R<'static>as B<'static>>::N}}>;}}fn f{index}(_:&u8)->X<1>{{X([7;1])}}"
        )?;
    }
    Ok(source)
}

#[test]
fn callable_projection_streams_preserve_qualified_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    for count in [1, 4, 16, 48] {
        for paid in [true, false] {
            let source = format!("{}{}", native::CHECK, program(count, paid)?);
            object_outcome(&format!("proof-stream-{count}-{paid}"), &source, paid)?;
            let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
                let entry =
                    rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
                rustgm::lower::instances::build(&report, &mut facts, entry)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            });
            assert_eq!(result.is_ok(), paid, "{count}-{paid}: {result:?}");
        }
    }
    Ok(())
}

#[test]
fn interleaved_definitions_retain_linear_callable_scope_storage()
-> Result<(), Box<dyn std::error::Error>> {
    for count in [4, 16, 48] {
        let source = format!("{}{}", native::CHECK, program(count, true)?);
        let (_, facts) = semantic::analyze(&source)?;
        let mut scopes = 0;
        let mut foreign = 0;
        for entry in &facts.quantified_proofs {
            scopes += entry.evidence.proof.scope_count();
            for id in entry.evidence.proof.regions.clone() {
                foreign += usize::from(
                    facts
                        .body_regions
                        .get(entry.body, rustgm::types::RegionId(id))
                        .is_none_or(|region| region.body != entry.body),
                );
            }
        }
        println!(
            "proof-stream-{count}: proofs={} scopes={scopes} foreign_gap_ids={foreign}",
            facts.quantified_proofs.len()
        );
        // Interleaved bodies number their regions in their own units.
        assert_eq!(
            foreign, 0,
            "a proof interval must not contain foreign bodies"
        );
        assert!(
            scopes <= 4 * count + 16,
            "{count}: {scopes} retained scopes"
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn interleaved_callable_projections_preserve_native_values()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [1, 4, 16, 48]
        .into_iter()
        .map(|count| Ok((format!("width-{count}"), program(count, true)?)))
        .collect::<Result<Vec<_>, std::fmt::Error>>()?;
    let borrowed = cases
        .iter()
        .map(|(name, source)| (name.as_str(), source.as_str(), false))
        .collect::<Vec<_>>();
    native::check_cases("proof-stream", &borrowed)
}
