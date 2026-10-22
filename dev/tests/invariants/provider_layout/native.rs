//! Real source layout values cross the existing C export boundary.
use super::*;

#[test]
#[ignore = "requires the pinned CPPGM object backend and a C linker"]
fn source_layout_provider_const_wrapper_reaches_native_code() -> Result {
    use std::process::Command;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/pa31/provider-native-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let client = folder.join("client.c");
    std::fs::write(
        &client,
        "extern unsigned long observe(void);int main(void){return observe()!=4;}",
    )?;
    for (name, tail) in [
        (
            "selected-size",
            "#[rustc_intrinsic]pub const fn size_of<T>()->usize;pub trait Properties:Sized{#[lang=\"mem_size_const\"]const SIZE:usize=size_of::<Self>();}impl<T>Properties for T{}#[unsafe(no_mangle)]pub extern \"C\" fn observe()->usize{<u32 as Properties>::SIZE}",
        ),
        (
            "generic-wrapper",
            "#[rustc_intrinsic]pub const fn size_of<T>()->usize;pub const fn wrapper<T>()->usize{const{size_of::<T>()}}#[unsafe(no_mangle)]pub extern \"C\" fn observe()->usize{wrapper::<u32>()}",
        ),
    ] {
        let input = folder.join(format!("{name}.rs"));
        std::fs::write(&input, SOURCE.to_owned() + tail)?;
        let rust_object = folder.join(format!("{name}.rustc.o"));
        command(
            Command::new("rustc")
                .env("RUSTC_BOOTSTRAP", "1")
                .args([
                    "+1.98.0",
                    "--edition=2024",
                    "--crate-type=lib",
                    "--emit=obj",
                    "--cap-lints=allow",
                    "-Cpanic=abort",
                ])
                .arg(&input)
                .arg("-o")
                .arg(&rust_object),
        )?;
        let report = resolve(tail)?;
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
        let lowir = folder.join(format!("{name}.lowir"));
        crate::lowir::writer::write(&program, &mut std::fs::File::create(&lowir)?)?;
        let object = folder.join(format!("{name}.rustgm.o"));
        command(
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg("--emit-object")
                .arg(&lowir)
                .arg("-o")
                .arg(&object),
        )?;
        for (kind, object) in [("rustc", rust_object), ("rustgm", object)] {
            let executable = folder.join(format!("{name}.{kind}"));
            command(
                Command::new("cc")
                    .arg(&client)
                    .arg(object)
                    .arg("-o")
                    .arg(&executable),
            )?;
            command(&mut Command::new(&executable))?;
        }
    }
    Ok(())
}
fn command(command: &mut std::process::Command) -> Result {
    let output = command.output()?;
    if !output.status.success() {
        return Err(format!(
            "{command:?}: {}\\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(())
}
