//! Backend encoding/native observations are separate from Rust frontend coverage.
use std::{error::Error, path::Path, process::Command};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

pub(super) fn disassemble(executable: &Path) -> Result<String> {
    let mut text = String::new();
    for (index, segment) in super::elf_code::extract(&std::fs::read(executable)?)?
        .iter()
        .enumerate()
    {
        let path = executable.with_extension(format!("code-{index}.bin"));
        std::fs::write(&path, &segment.bytes)?;
        let output = Command::new("objdump")
            .args(["-D", "-b", "binary", "-m", "i386:x86-64"])
            .arg(format!("--adjust-vma={}", segment.address))
            .arg(path)
            .output()?;
        assert!(output.status.success() && output.stderr.is_empty());
        text.push_str(&String::from_utf8(output.stdout)?);
    }
    Ok(text)
}

fn run_phase(phase: &str) -> Result<std::path::PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/pa28/atomic-matrix-{phase}-{}",
        std::process::id()
    ));
    let output = Command::new("python3")
        .arg(root.join("scripts/atomic_capabilities.py"))
        .arg(phase)
        .arg(&folder)
        .output()?;
    assert!(
        output.status.success(),
        "{phase}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    Ok(folder)
}

#[test]
#[ignore = "requires pinned CPPGM backend and GNU objdump; uncontended encoding only"]
fn atomic_width_pointer_bool_and_fence_backend_contracts() -> Result {
    let folder = run_phase("backend")?;
    let inputs = std::fs::read_to_string(folder.join("executables.tsv"))?;
    let mut receipt = String::new();
    let mut count = 0;
    for line in inputs.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 6);
        let executable = folder.join(fields[0]);
        let text = disassemble(&executable)?;
        let expected: Vec<u32> = fields[1..]
            .iter()
            .map(|value| value.parse())
            .collect::<std::result::Result<_, _>>()?;
        assert_machine(&text, &expected)?;
        std::fs::write(executable.with_extension("disassembly"), text)?;
        receipt.push_str(line);
        receipt.push('\n');
        count += 1;
    }
    assert_eq!(count, 19);
    std::fs::write(folder.join("machine-checked.tsv"), receipt)?;
    Ok(())
}

#[test]
#[ignore = "requires pinned rustc/native execution; never invokes RustGM frontend"]
fn independent_rust_atomic_outcomes_and_target_abi() -> Result {
    run_phase("rust")?;
    Ok(())
}

fn assert_machine(text: &str, expected: &[u32]) -> Result {
    let mut counts = [0; 4];
    for line in text.lines() {
        let instruction = line.rsplit('\t').next().ok_or("instruction")?.trim();
        let (index, operands) = if let Some(rest) = instruction.strip_prefix("lock xadd ") {
            (0, rest)
        } else if let Some(rest) = instruction.strip_prefix("lock cmpxchg ") {
            (1, rest)
        } else if let Some(rest) = instruction.strip_prefix("xchg ") {
            if !rest.contains('(') {
                continue;
            }
            (2, rest)
        } else if instruction == "mfence" {
            counts[3] += 1;
            continue;
        } else {
            continue;
        };
        if !operands.contains('(') {
            return Err("RMW must address memory".into());
        }
        let register = operands.trim().split(',').next().ok_or("RMW operand")?;
        if register_width(register)? != expected[0] {
            return Err(format!("unexpected RMW width: {instruction}").into());
        }
        counts[index] += 1;
    }
    if counts.as_slice() != &expected[1..] {
        return Err(format!("unexpected RMW/fence counts: {counts:?}").into());
    }
    Ok(())
}

fn register_width(register: &str) -> Result<u32> {
    let name = register.strip_prefix('%').ok_or("expected register")?;
    if matches!(
        name,
        "al" | "bl" | "cl" | "dl" | "sil" | "dil" | "spl" | "bpl"
    ) || name.ends_with('b')
    {
        Ok(8)
    } else if matches!(name, "ax" | "bx" | "cx" | "dx" | "si" | "di" | "sp" | "bp")
        || name.ends_with('w')
    {
        Ok(16)
    } else if name.starts_with('e') || name.ends_with('d') {
        Ok(32)
    } else if name.starts_with('r') {
        Ok(64)
    } else {
        Err("unknown RMW register".into())
    }
}

#[test]
fn machine_parser_checks_counts_and_operand_widths() -> Result {
    for (register, bits) in [("cl", 8), ("r8w", 16), ("edx", 32), ("rax", 64)] {
        let text = format!(
            "10:\t00\tlock xadd %{register},(%rax)\n11:\t00\tlock cmpxchg %{register},(%rax)\n12:\t00\txchg %{register},(%rax)\n13:\t00\tmfence\n"
        );
        assert_machine(&text, &[bits, 1, 1, 1, 1])?;
    }
    let text = "10:\t00\tlock xadd %cl,(%rax)\n";
    assert!(assert_machine(text, &[64, 1, 0, 0, 0]).is_err());
    assert!(assert_machine(text, &[8, 2, 0, 0, 0]).is_err());
    assert!(assert_machine("10:\t00\tlock xadd %cl,%al\n", &[8, 1, 0, 0, 0]).is_err());
    assert!(register_width("unrecognized").is_err());
    Ok(())
}
