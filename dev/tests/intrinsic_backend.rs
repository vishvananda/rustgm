//! Independently declared capabilities of the pinned external `LowIR` backend.
use std::{path::Path, process::Command};

const CASES: &[(&str, usize)] = &[
    ("volatile-i8", 2),
    ("volatile-i16", 2),
    ("volatile-i32", 2),
    ("volatile-i64", 2),
    ("volatile-i128", 2),
    ("volatile-f32", 2),
    ("volatile-f64", 2),
    ("volatile-ptr", 4),
    ("atomic-u32", 0),
];

#[test]
#[ignore = "requires the qualified CPPGM native backend and GNU objdump"]
fn scalar_volatile_and_u32_atomic_capabilities() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!("obj/intrinsic-backend-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    for &(name, volatile) in CASES {
        let input = root.join(format!("pa28/tests/backend/{name}.lowir"));
        let source = std::fs::read_to_string(&input)?;
        let executable = folder.join(name);
        let build = Command::new("python3")
            .arg(root.join("scripts/backend.py"))
            .arg(&input)
            .arg("-o")
            .arg(&executable)
            .output()?;
        assert!(
            build.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&build.stderr)
        );
        assert!(build.stdout.is_empty() && build.stderr.is_empty(), "{name}");
        let validated = std::fs::read_to_string(executable.with_extension("validated.lowir"))?;
        assert_eq!(source.matches("volatile").count(), volatile, "{name}");
        assert_eq!(validated.matches("volatile").count(), volatile, "{name}");
        let run = Command::new(&executable).output()?;
        assert_eq!(run.status.code(), Some(42), "{name}");
        assert!(run.stdout.is_empty() && run.stderr.is_empty(), "{name}");
        if name == "atomic-u32" {
            // The backend writes an ELF without section headers. Disassemble its
            // raw bytes so section-table absence cannot hide the actual opcodes.
            let disassembly = Command::new("objdump")
                .args(["-D", "-b", "binary", "-m", "i386:x86-64"])
                .arg(&executable)
                .output()?;
            assert!(disassembly.status.success());
            let text = String::from_utf8(disassembly.stdout)?;
            for operation in ["xchg", "lock cmpxchg", "lock xadd"] {
                assert!(text.contains(operation), "missing {operation}");
            }
            std::fs::write(executable.with_extension("disassembly"), text)?;
        }
    }
    Ok(())
}
