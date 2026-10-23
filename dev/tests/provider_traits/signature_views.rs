//! Public call binders and opened body universals keep their distinct identities.
use super::{TestResult, lower, native::run};
use std::{path::Path, process::Command};

const CASES: &[(&str, &str)] = &[
    (
        "unused-reference",
        "fn inspect<T>()->u8{7}#[unsafe(no_mangle)]pub extern \"C\" fn answer()->u8{inspect::<&u8>()}",
    ),
    (
        "unused-reference-pair",
        "fn inspect<T>()->u8{7}#[unsafe(no_mangle)]pub extern \"C\" fn answer()->u8{inspect::<(&u8,&u16)>()}",
    ),
    (
        "nested-callable",
        "fn inspect<T>()->u8{7}#[unsafe(no_mangle)]pub extern \"C\" fn answer()->u8{inspect::<for<'a> fn(&'a u8)->&'a u8>()}",
    ),
    (
        "early-and-external",
        "fn choose<'a,T>(x:&'a u8)->&'a u8{x}#[unsafe(no_mangle)]pub extern \"C\" fn answer()->u8{*choose::<&u16>(&7)}",
    ),
];

pub(super) fn source(body: &str) -> TestResult<String> {
    let (header, _) = super::cases::GENERIC_COPY
        .split_once("fn twice")
        .ok_or("source header")?;
    Ok(format!("{header}{body}\n"))
}

#[test]
fn actual_source_copy_handles_public_and_owned_callable_binders() -> TestResult {
    for &(name, body) in CASES {
        let program = lower(&source(body)?).map_err(|error| format!("{name}: {error}"))?;
        let mut bytes = Vec::new();
        rustgm::lowir::writer::write(&program, &mut bytes)?;
        assert!(!bytes.is_empty(), "{name}");
    }
    Ok(())
}

#[test]
#[ignore = "requires pinned rustc, CPPGM backend and the host C linker"]
fn actual_source_signature_views_match_independent_native_answers() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing root")?;
    let folder = root
        .join("obj/parallel/source-signature-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let client = folder.join("client.c");
    std::fs::write(
        &client,
        "extern unsigned char answer(void);int main(void){return answer()!=7;}\n",
    )?;
    for &(name, body) in CASES {
        let source = source(body)?;
        let input = folder.join(format!("{name}.rs"));
        let reference = folder.join(format!("{name}.reference.o"));
        let candidate = folder.join(format!("{name}.candidate.o"));
        let ir = folder.join(format!("{name}.lowir"));
        std::fs::write(&input, &source)?;
        run(
            &folder,
            &format!("{name}-reference-build"),
            Command::new("rustc")
                .env("RUSTC_BOOTSTRAP", "1")
                .args([
                    "+1.98.0",
                    "--edition=2024",
                    "--crate-type=lib",
                    "--crate-name=probe",
                    "--emit=obj",
                    "-C",
                    "panic=abort",
                ])
                .arg(&input)
                .arg("-o")
                .arg(&reference),
        )?;
        let mut bytes = Vec::new();
        rustgm::lowir::writer::write(&lower(&source)?, &mut bytes)?;
        std::fs::write(&ir, bytes)?;
        run(
            &folder,
            &format!("{name}-backend"),
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg("--emit-object")
                .arg(ir)
                .arg("-o")
                .arg(&candidate),
        )?;
        for (label, object) in [("reference", reference), ("candidate", candidate)] {
            let binary = folder.join(format!("{name}-{label}.native"));
            run(
                &folder,
                &format!("{name}-{label}-link"),
                Command::new("cc")
                    .arg(&client)
                    .arg(object)
                    .arg("-o")
                    .arg(&binary),
            )?;
            run(
                &folder,
                &format!("{name}-{label}-run"),
                &mut Command::new(binary),
            )?;
        }
    }
    Ok(())
}
