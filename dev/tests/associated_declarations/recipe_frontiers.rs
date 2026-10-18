//! Shared structural work and per-caller evidence have separate growth bounds.
use super::{native, object_outcome, semantic};
use std::fmt::Write as _;

fn program(count: usize, callers: usize) -> Result<String, std::fmt::Error> {
    let mut source = "struct X<const N:usize>;".to_owned();
    for caller in 0..callers {
        write!(
            source,
            "fn f{caller}()->X<{{(<u8 as A>::C,3usize).1}}>{{X}}"
        )?;
    }
    source.push_str("struct R<'a>(&'a u8);trait B<'a>{const N:usize;}impl<'a>B<'a>for R<'a>{const N:usize=1;}struct V<'a,const N:usize>(&'a u8,[u8;N]);enum O<T>{None,Some(T)}");
    let mut fields = String::new();
    let mut values = String::new();
    for index in 0..count {
        write!(
            source,
            "trait Q{index}<'a>{{type Out:'a;}}impl<'a>Q{index}<'a>for R<'a>{{type Out=V<'a,{{<R<'static>as B<'static>>::N}}>;}}struct W{index}<'a,T:Q{index}<'a>>{{v:T::Out}}"
        )?;
        write!(fields, "O<W{index}<'static,R<'static>>>,")?;
        values.push_str("O::None,");
    }
    write!(
        source,
        "trait A{{const C:({fields});}}impl A for u8{{const C:({fields})=({values});}}fn main(){{"
    )?;
    for caller in 0..callers {
        write!(source, "let _:X<3>=f{caller}();")?;
    }
    source.push_str("check(true);}");
    Ok(source)
}

#[test]
fn stored_field_waits_share_structure_and_retain_each_callers_proof()
-> Result<(), Box<dyn std::error::Error>> {
    for count in [4, 16, 48] {
        for callers in [1, count] {
            let source = format!("{}{}", native::CHECK, program(count, callers)?);
            object_outcome(
                &format!("recipe-frontiers-{count}-{callers}"),
                &source,
                true,
            )?;
            let (_, facts) = semantic::analyze(&source)?;
            let body = &facts.body_work;
            let graph = facts.stored_field_work();
            println!(
                "stored {count}-{callers}: waits={} definitions={} selections={} fold={} graph={} recipes={} retained={}",
                body.storage_waits,
                body.storage_definition_waits,
                body.storage_selections,
                body.storage_steps,
                graph.steps,
                graph.recipes,
                graph.retained_bytes
            );
            assert_eq!(body.storage_definition_waits, count);
            // V's raw field template is also unfinished at the first result.
            assert_eq!(body.storage_waits, count + 1);
            assert!(body.storage_selections <= count * (callers + 1));
            assert!(body.storage_steps <= 64 * count * (callers + 2));
            assert!(graph.steps <= 40 * count + 4 * callers + 64);
            assert_eq!(graph.recipes, count + 1);
            assert!(graph.retained_bytes <= 256 * count + 64 * callers + 1024);
            let mut foreign = 0;
            for entry in &facts.quantified_proofs {
                for interval in entry.evidence.proof.owned_regions() {
                    for id in interval {
                        assert_eq!(facts.body_regions[id as usize - 1].body, entry.body);
                    }
                }
                foreign += entry
                    .evidence
                    .proof
                    .regions
                    .clone()
                    .filter(|&id| facts.body_regions[id as usize - 1].body != entry.body)
                    .count();
            }
            assert!(
                foreign > 0,
                "storage proof must cross a child declaration's allocations"
            );
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn storage_recipe_waits_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let programs = [(4, 1), (4, 4), (16, 1), (16, 16), (48, 1), (48, 48)]
        .into_iter()
        .map(|(count, callers)| Ok((format!("{count}-{callers}"), program(count, callers)?)))
        .collect::<Result<Vec<_>, std::fmt::Error>>()?;
    let cases = programs
        .iter()
        .map(|(name, source)| (name.as_str(), source.as_str(), false))
        .collect::<Vec<_>>();
    native::check_cases("recipe-frontiers", &cases)
}
