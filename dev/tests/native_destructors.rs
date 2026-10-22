use crate::cleanup_lowering as lowering;
use crate::destructor_cases as cases;
use crate::qualify as rustc;
use std::{
    path::Path,
    process::{Command, Output},
};

fn build(
    name: &str,
    source: &str,
) -> Result<rustgm::lowir::program::Program, Box<dyn std::error::Error>> {
    if name != "edge-conditional" {
        return lowering::build(source);
    }
    lowering::build_with(source, |_, program| {
        use rustgm::body::{
            ir::{Action, BlockId, LocalKind, Rvalue, Terminator},
            storage::Location,
        };
        let body = program
            .bodies
            .iter_mut()
            .find(|b| {
                b.locals
                    .iter()
                    .filter(|l| matches!(l.kind, LocalKind::Parameter(_)))
                    .count()
                    == 3
            })
            .ok_or("edge test body")?;
        let (parent, target) = body
            .blocks
            .iter()
            .enumerate()
            .find_map(|(i, b)| {
                if let Terminator::Branch { yes, .. } = b.terminator
                    && matches!(
                        body.blocks[yes.0 as usize].terminator,
                        Terminator::Return(_)
                    )
                {
                    Some((i, yes))
                } else {
                    None
                }
            })
            .ok_or("edge test branch")?;
        let block = &body.blocks[target.0 as usize];
        let mut moved = 0;
        for exit in &mut body.storage.as_mut().ok_or("edge test storage")?.exits {
            if let Location::At { block: id, before } = exit.location
                && id == target
            {
                assert!(body.statements[block.start as usize..before as usize].iter().all(|s|
                    matches!(s.action,Action::Evaluate(v) if matches!(body.values[v.0 as usize].kind,Rvalue::Unit)) || matches!(s.action,Action::StorageEnd {..})));
                exit.location = Location::Edge {
                    block: BlockId(u32::try_from(parent)?),
                    target,
                };
                moved += 1;
            }
        }
        assert!(moved > 0);
        Ok(())
    })
}

#[test]
fn checked_cleanup_programs_lower_without_reopening_invalid_borrows()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, path, accepted) in cases::CASES {
        let source = cases::source(path)?;
        rustc::qualify(&format!("lower-cleanup-{name}"), &source, accepted)?;
        let result = build(name, &source);
        assert_eq!(result.is_ok(), accepted, "{name}: {:?}", result.err());
    }
    Ok(())
}

#[test]
fn cli_requires_completed_destructor_proofs_and_matches_library_lowering()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/cleanup-cli-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    for &(name, path, accepted) in cases::CASES {
        let source = cases::source(path)?;
        let input = folder.join(format!("{name}.rs"));
        std::fs::write(&input, &source)?;
        let program = lowering::build(&source);
        for (mode, count) in [("--emit-lowir", false), ("--count-lowir", true)] {
            let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(mode)
                .arg(&input)
                .output()?;
            assert_eq!(
                output.status.code(),
                Some(i32::from(!accepted)),
                "{name}: {mode}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            if accepted {
                assert!(output.stderr.is_empty(), "{name}: {mode}");
                let mut expected = Vec::new();
                let program = program.as_ref().map_err(|e| format!("{name}: {e}"))?;
                if count {
                    rustgm::lowir::writer::count(program, &mut expected)?;
                } else {
                    rustgm::lowir::writer::write(program, &mut expected)?;
                }
                assert_eq!(output.stdout, expected, "{name}: {mode}");
            } else {
                assert!(output.stdout.is_empty(), "{name}: {mode}");
            }
        }
    }
    Ok(())
}

#[test]
fn billion_zero_sized_drops_have_bounded_glue_and_stable_backing()
-> Result<(), Box<dyn std::error::Error>> {
    let mut counts = Vec::new();
    for length in [8, 1_000_000_000] {
        let source = format!(
            "#[repr(align(32))]struct Z;impl Drop for Z{{fn drop(&mut self){{let _borrow=&*self;}}}}const ZED:Z=Z;fn main(){{let _array=[ZED;{length}];}}"
        );
        rustc::qualify(&format!("zero-sized-drop-{length}"), &source, true)?;
        let program = lowering::build(&source)?;
        let mut bytes = Vec::new();
        rustgm::lowir::writer::write(&program, &mut bytes)?;
        assert!(bytes.len() < 12_000, "ZST drop output expanded with length");
        assert!(std::str::from_utf8(&bytes)?.contains("obj<32x32>"));
        counts.push(
            program
                .symbols
                .iter()
                .filter_map(|s| s.body.as_ref())
                .map(|b| b.instructions.len())
                .sum::<usize>(),
        );
    }
    assert_eq!(counts[0], counts[1]);
    Ok(())
}
fn success(name: &str, output: Output) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(format!(
            "{name}: {:?}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(output.stdout)
}
#[test]
#[ignore = "requires the pinned CPPGM backend; run make test-maintainer-native"]
fn destructor_counters_execute_like_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!("obj/native-destructors-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    for &(name, path, accepted) in cases::CASES {
        let source = cases::source(path)?;
        if !accepted {
            continue;
        }
        let input = folder.join(format!("{name}.rs"));
        std::fs::write(&input, &source)?;
        let oracle = folder.join(format!("{name}.rustc"));
        success(
            name,
            Command::new("rustc")
                .args([
                    "+1.98.0",
                    "--edition=2024",
                    "--crate-name=probe",
                    "--cap-lints=allow",
                    "-C",
                    "overflow-checks=yes",
                    "-C",
                    "panic=abort",
                ])
                .arg(&input)
                .arg("-o")
                .arg(&oracle)
                .output()?,
        )?;
        let expected = success(name, Command::new(&oracle).output()?)?;
        let program = build(name, &source).map_err(|e| format!("{name}: {e}"))?;
        let mut bytes = Vec::new();
        rustgm::lowir::writer::write(&program, &mut bytes)?;
        let again = build(name, &source)?;
        let mut repeated = Vec::new();
        rustgm::lowir::writer::write(&again, &mut repeated)?;
        assert_eq!(bytes, repeated, "{name}: nondeterministic lowering");
        let lowir = folder.join(format!("{name}.lowir"));
        std::fs::write(&lowir, bytes)?;
        let native = folder.join(format!("{name}.native"));
        success(
            name,
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg(&lowir)
                .arg("-o")
                .arg(&native)
                .output()?,
        )?;
        assert_eq!(
            success(name, Command::new(native).output()?)?,
            expected,
            "{name}"
        );
    }
    Ok(())
}
