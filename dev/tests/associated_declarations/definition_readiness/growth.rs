//! Initializer work grows with definitions and callers, retaining only owned proof segments.
use super::{native, semantic};
use std::fmt::Write as _;

fn program(width: usize, callers: usize) -> Result<String, std::fmt::Error> {
    let mut source = "struct X<const N:usize>;".to_owned();
    for index in 0..callers {
        write!(source, "fn f{index}()->X<{{<u8 as Total>::C}}>{{X}}")?;
    }
    source.push_str("struct R<'a>(&'a u8);trait B<'a>{const N:usize;}impl<'a>B<'a>for R<'a>{const N:usize=1;}struct V<'a,const N:usize>(&'a u8,[u8;N]);trait Total{const C:usize;}impl Total for u8{const C:usize=");
    for index in 0..width {
        if index != 0 {
            source.push('+');
        }
        write!(source, "*<R<'static>as Q{index}<'static>>::C.0 as usize")?;
    }
    source.push_str(";}");
    for index in 0..width {
        write!(
            source,
            "trait Q{index}<'a>{{type Out:'a;const C:Self::Out;}}impl<'a>Q{index}<'a>for R<'a>{{type Out=V<'a,{{<R<'static>as B<'static>>::N}}>;const C:Self::Out=V(&1,[0;1]);}}"
        )?;
    }
    write!(source, "fn main(){{check(<u8 as Total>::C=={width});")?;
    for index in 0..callers {
        write!(source, "let _:X<{width}>=f{index}();")?;
    }
    source.push('}');
    Ok(source)
}

#[test]
fn initializer_waits_share_work_across_repeated_callers() -> Result<(), Box<dyn std::error::Error>>
{
    use rustgm::{sema::model::BodyRegionOrigin, syntax::ast::Kind};
    for family in ["definitions", "callers", "coupled"] {
        let mut shared_work = None;
        for size in [4, 16, 48] {
            let (width, callers) = match family {
                "definitions" => (size, 1),
                "callers" => (4, size),
                _ => (size, size),
            };
            let source = format!("{}{}", native::CHECK, program(width, callers)?);
            let (report, facts) = semantic::analyze(&source)?;
            let work = &facts.body_work;
            let dispatch = facts.dispatch_work();
            println!(
                "dispatch {family}-{size}: selections={} shared={} proofs={} expansions={} member_edges={}",
                dispatch.selections,
                dispatch.shared_selections,
                dispatch.closed_proofs,
                dispatch.dependency_expansions,
                dispatch.member_dependencies
            );
            assert_eq!(dispatch.selections, width + 2);
            assert!(dispatch.shared_selections <= 8 * (width + callers) + 8);
            assert!(dispatch.member_dependencies <= 8 * (width + callers) + 8);
            assert!(dispatch.dependency_expansions <= 16 * (width + callers) + 32);
            let body = facts
                .initializers
                .iter()
                .find(|initializer| {
                    matches!(
                        report
                            .package
                            .unit(initializer.value.file)
                            .configured
                            .ast
                            .node(initializer.value.node)
                            .kind,
                        Kind::Binary { .. }
                    )
                })
                .ok_or("missing shared initializer")?
                .value;
            let mut foreign = 0;
            let mut proofs = 0;
            for entry in facts
                .quantified_proofs
                .iter()
                .filter(|entry| entry.body == body)
            {
                proofs += 1;
                for owned in entry.evidence.proof.owned_regions() {
                    for id in owned {
                        let region = &facts.body_regions[id as usize - 1];
                        assert_eq!(region.body, body);
                        assert!(matches!(region.origin, BodyRegionOrigin::QuantifiedProof));
                    }
                }
                foreign += entry
                    .evidence
                    .proof
                    .regions
                    .clone()
                    .filter(|&id| facts.body_regions[id as usize - 1].body != body)
                    .count();
            }
            println!(
                "{family}-{size}: waits={} selections={} fold={} body={} slots={} proofs={proofs} foreign={foreign}",
                work.projection_waits,
                work.projection_selections,
                work.projection_steps,
                work.steps,
                work.peak_suspended_slots
            );
            assert_eq!(work.projection_waits, width);
            assert_eq!(work.projection_selections, width);
            assert!(work.projection_steps <= 16 * width + 32);
            assert!(work.steps <= 128 * (width + callers) + 128);
            assert!(work.peak_suspended_slots <= 128 * width + 128);
            assert!(
                foreign > 0,
                "the initializer must cross other bodies' allocations"
            );
            assert!(proofs <= 4 * width + 8);
            if family == "callers" {
                let actual = (
                    work.projection_steps,
                    work.peak_suspended_slots,
                    proofs,
                    foreign,
                );
                assert_eq!(*shared_work.get_or_insert(actual), actual);
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn shared_initializer_dependencies_preserve_native_values() -> Result<(), Box<dyn std::error::Error>>
{
    let programs = [(4, 1), (4, 8), (8, 8)]
        .into_iter()
        .map(|(width, callers)| Ok((format!("{width}-{callers}"), program(width, callers)?)))
        .collect::<Result<Vec<_>, std::fmt::Error>>()?;
    let cases = programs
        .iter()
        .map(|(name, source)| (name.as_str(), source.as_str(), false))
        .collect::<Vec<_>>();
    native::check_cases("projection-initializers", &cases)
}
