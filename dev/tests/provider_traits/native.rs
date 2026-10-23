//! Native answers use the exact independently settled source cutouts.
use super::{CALL_EXECUTION, CLONE_EXECUTION, COPY_EXECUTION, TestResult, lower};
use std::{path::Path, process::Command};
pub(super) fn run(folder: &Path, name: &str, command: &mut Command) -> TestResult {
    let output = command.output()?;
    std::fs::write(folder.join(format!("{name}.stdout")), &output.stdout)?;
    std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
    std::fs::write(
        folder.join(format!("{name}.status")),
        format!("{:?}\n", output.status),
    )?;
    assert!(
        output.status.success(),
        "{name}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn source_copy_clone_and_function_item_native_answers() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing root")?;
    let folder = root
        .join("obj/parallel/pa31/source-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let destruct = super::destructibility::ordinary_source();
    let ordinary_member = [
        super::destructibility::BASE,
        super::forbidden_cleanup::MEMBER_NATIVE,
    ]
    .concat();
    let ordinary_free = [
        super::destructibility::BASE,
        super::forbidden_cleanup::FREE_NATIVE,
    ]
    .concat();
    for (name, source, expected) in [
        ("ordinary-destruct", destruct.as_str(), 7),
        ("ordinary-member-pointer", ordinary_member.as_str(), 7),
        ("ordinary-free-pointer", ordinary_free.as_str(), 11),
        ("copy", COPY_EXECUTION, 7),
        ("clone", CLONE_EXECUTION, 11),
        ("callable", CALL_EXECUTION, 23),
        (
            "unused-conditional-runtime",
            super::cases::UNUSED_CONDITIONAL_RUNTIME,
            7,
        ),
    ] {
        let program = lower(source)?;
        let ir = folder.join(format!("{name}.lowir"));
        let object = folder.join(format!("{name}.o"));
        let binary = folder.join(format!("{name}.native"));
        let client = folder.join(format!("{name}.c"));
        let mut text = Vec::new();
        rustgm::lowir::writer::write(&program, &mut text)?;
        std::fs::write(&ir, text)?;
        std::fs::write(
            &client,
            format!(
                "extern unsigned char answer(void);int main(void){{return answer()!={expected};}}\n"
            ),
        )?;
        run(
            &folder,
            &format!("{name}-backend"),
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg("--emit-object")
                .arg(&ir)
                .arg("-o")
                .arg(&object),
        )?;
        run(
            &folder,
            &format!("{name}-link"),
            Command::new("cc")
                .arg(&client)
                .arg(&object)
                .arg("-o")
                .arg(&binary),
        )?;
        run(
            &folder,
            &format!("{name}-execute"),
            &mut Command::new(binary),
        )?;
    }
    Ok(())
}
