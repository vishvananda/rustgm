//! Actual terminal calls retain caller proofs and bypass owned cleanup only when reached.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const OPERATIONS: &str = r#"
pub mod intrinsics { #[rustc_intrinsic]pub fn abort()->!; }
#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
#[lang="drop_glue"]pub(crate) const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
unsafe extern "C"{fn marker();fn dropped();}
struct Guard;
impl Drop for Guard{fn drop(&mut self){unsafe{dropped();}}}
#[unsafe(export_name="abort")]pub extern "C" fn user_abort()->u8{unsafe{marker();}7}
#[unsafe(no_mangle)]pub extern "C" fn probe(trip:bool)->u8{
    let _guard=Guard;
    if trip{intrinsics::abort()}else{9}
}
#[unsafe(no_mangle)]pub extern "C" fn lazy()->u8{if false{intrinsics::abort()}else{11}}
fn not_reached()->!{intrinsics::abort()}
"#;
fn source(tail: &str) -> Result<String> {
    let foundation = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    Ok(format!("#![feature(intrinsics)]{foundation}{tail}"))
}
#[test]
fn actual_terminal_abort_preserves_owned_scope_and_prepares_native_controls() -> Result {
    let input = source(OPERATIONS)?;
    let report = crate::test_source::source_provider(&input, "source_abort")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    for name in ["probe", "lazy", "abort"] {
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|s| s.name == name && s.body.is_some())
                .count(),
            1,
            "{name}"
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/all-attribute-selected-output-v1/native/source-abort");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    assert_eq!(size_of::<crate::resolve::model::Resolution>(), 2056);
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    assert_eq!(size_of::<crate::sema::compiler_items::AbortSelection>(), 8);
    eprintln!(
        "source abort Catalog={} Resolution={} Package={} Body={} Selection={}",
        size_of::<crate::sema::compiler_items::Catalog>(),
        size_of::<crate::resolve::model::Resolution>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<ir::Body>(),
        size_of::<crate::sema::compiler_items::AbortSelection>()
    );
    Ok(())
}
#[test]
fn terminal_edge_and_const_reification_opponents_remain_closed() -> Result {
    let input = source(
        "#[rustc_intrinsic]pub fn abort()->!;#[unsafe(no_mangle)]pub fn reached()->!{abort()}",
    )?;
    let report = crate::test_source::source_provider(&input, "abort_edges")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    crate::lower::instances::inspect_prepared(
        &report,
        &mut facts,
        prepared,
        |_, proof| -> Result {
            let original = proof.program().bodies.iter().find(|body| matches!(body.owner, ir::Owner::Function{definition,..}
            if report.package.names.get(report.resolution.definitions[definition.0 as usize].name)=="reached")).ok_or("reached checked body")?;
            terminal_opponents(original)
        },
    )??;
    for (name, tail) in [
        (
            "const",
            "#[rustc_intrinsic]pub fn abort()->!;const fn bad()->!{abort()}",
        ),
        (
            "const_false",
            "#[rustc_intrinsic]pub fn abort()->!;const fn bad()->u8{if false{abort()}else{3}}",
        ),
        (
            "reify",
            "#[rustc_intrinsic]pub fn abort()->!;#[unsafe(no_mangle)]pub fn bad()->!{let call:fn()->! = abort;call()}",
        ),
    ] {
        let input = source(tail)?;
        let result =
            crate::test_source::source_provider(&input, "abort_opponents").and_then(|report| {
                let crate::pipeline::Analysis {
                    mut facts,
                    prepared,
                } = crate::pipeline::analyze_source(&report).map_err(|error| error.to_string())?;
                crate::lower::instances::build_roots_prepared(
                    &report,
                    &mut facts,
                    crate::lower::Roots {
                        entry: None,
                        exports: &[],
                    },
                    prepared,
                )
                .map_err(|error| error.to_string())
            });
        let error = result
            .err()
            .ok_or_else(|| format!("{name} acquired source Abort permission"))?;
        eprintln!("source Abort {name} rejected: {error}");
    }
    Ok(())
}

/// Observe actual prepared ownership/instance proof before opposing terminal facts.
fn terminal_opponents(original: &ir::Body) -> Result {
    let (index, block) = original
        .blocks
        .iter()
        .enumerate()
        .find(|(_, block)| {
            matches!(block.terminator, ir::Terminator::Unreachable) && block.end > block.start
        })
        .ok_or("terminal block")?;
    let before = block.end - 1;
    let ir::Action::Evaluate(value) = original.statements[before as usize].action else {
        return Err("terminal evaluate".into());
    };
    let block_id = ir::BlockId(u32::try_from(index)?);
    let location = Location::At {
        block: block_id,
        before,
    };
    terminal(original, value, location)?;
    let mut changed = original.clone();
    changed.blocks[index].terminator = ir::Terminator::Jump(block_id);
    assert!(terminal(&changed, value, location).is_err());
    assert!(
        terminal(
            original,
            value,
            Location::Edge {
                block: block_id,
                target: block_id
            }
        )
        .is_err()
    );
    assert!(
        terminal(
            original,
            value,
            Location::At {
                block: block_id,
                before: block.end
            }
        )
        .is_err()
    );
    changed = original.clone();
    changed.statements[before as usize].action = ir::Action::Mention(ir::Place::Temporary(value));
    assert!(terminal(&changed, value, location).is_err());
    Ok(())
}
