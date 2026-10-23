//! Native answers use the exact independently settled source cutouts.
use super::{CALL_EXECUTION, CLONE_EXECUTION, COPY_EXECUTION, TestResult, lower};
use std::{path::Path, process::Command};
pub(crate) fn run(folder: &Path, name: &str, command: &mut Command) -> TestResult {
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

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn actual_source_freeze_original_native_answers() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/freeze-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for &(name, source, _, native) in super::freeze_cases::CASES {
        if !native {
            continue;
        }
        let source = super::fixture(source)?;
        let program = lower(&source)?;
        let mut text = Vec::new();
        rustgm::lowir::writer::write(&program, &mut text)?;
        let ir = folder.join(format!("{name}.lowir"));
        let object = folder.join(format!("{name}.o"));
        let binary = folder.join(format!("{name}.native"));
        let client = folder.join(format!("{name}.c"));
        std::fs::write(&ir, text)?;
        std::fs::write(
            &client,
            "extern unsigned char answer(void);int main(void){return answer()!=7;}\n",
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
            &mut Command::new(&binary),
        )?;
        std::fs::remove_file(&binary)?;
        std::fs::remove_file(&object)?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn actual_freeze_and_glue_original_cleanup_native_seven() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/freeze-cleanup-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let program = lower(&super::fixture(super::freezing::AUTOMATIC_CLEANUP)?)?;
    let mut text = Vec::new();
    rustgm::lowir::writer::write(&program, &mut text)?;
    let ir = folder.join("cleanup.lowir");
    let object = folder.join("cleanup.o");
    let binary = folder.join("cleanup.native");
    let client = folder.join("cleanup.c");
    std::fs::write(&ir, text)?;
    std::fs::write(
        &client,
        "extern unsigned char observe(void);int main(void){return observe()!=7;}\n",
    )?;
    run(
        &folder,
        "cleanup-backend",
        Command::new("python3")
            .arg(root.join("scripts/backend.py"))
            .arg("--emit-object")
            .arg(&ir)
            .arg("-o")
            .arg(&object),
    )?;
    run(
        &folder,
        "cleanup-link",
        Command::new("cc")
            .arg(&client)
            .arg(&object)
            .arg("-o")
            .arg(&binary),
    )?;
    run(&folder, "cleanup-execute", &mut Command::new(&binary))?;
    std::fs::remove_file(&binary)?;
    std::fs::remove_file(&object)?;
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn source_nominal_formation_native_answers() -> TestResult {
    for &(name, path, accepted) in super::freeze_formation::CASES {
        if accepted {
            native_observe_fixture(name, path, 7)?;
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn actual_drop_capture_native_answers() -> TestResult {
    for &(name, path, accepted, expected) in super::borrowed_destructors::CASES {
        if accepted {
            native_observe_fixture(name, path, expected)?;
        }
    }
    Ok(())
}
fn native_observe_fixture(name: &str, path: &str, expected: u8) -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/formation-native")
        .join(std::process::id().to_string())
        .join(name);
    std::fs::create_dir_all(&folder)?;
    let program = lower(&super::fixture(path)?)?;
    let mut text = Vec::new();
    rustgm::lowir::writer::write(&program, &mut text)?;
    let ir = folder.join("input.lowir");
    let object = folder.join("input.o");
    let binary = folder.join("input.native");
    let client = folder.join("input.c");
    std::fs::write(&ir, text)?;
    std::fs::write(
        &client,
        format!(
            "extern unsigned char observe(void);int main(void){{return observe()!={expected};}}\n"
        ),
    )?;
    run(
        &folder,
        "backend",
        Command::new("python3")
            .arg(root.join("scripts/backend.py"))
            .arg("--emit-object")
            .arg(&ir)
            .arg("-o")
            .arg(&object),
    )?;
    run(
        &folder,
        "link",
        Command::new("cc")
            .arg(&client)
            .arg(&object)
            .arg("-o")
            .arg(&binary),
    )?;
    run(&folder, "execute", &mut Command::new(&binary))?;
    std::fs::remove_file(&binary)?;
    std::fs::remove_file(&object)?;
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn original_full_typed_source_sizing_depths_keep_native_seven() -> TestResult {
    for depth in [8, 32, 128] {
        for noise in [0, 128] {
            let name = format!("typed-depth-{depth}-noise-{noise}");
            native_observe_fixture(&name, &format!("formation/{name}.rs"), 7)?;
        }
    }
    Ok(())
}
