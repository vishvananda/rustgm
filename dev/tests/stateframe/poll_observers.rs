//! Persistent native observers exercise generated state, real source and completion.
use super::{Result, source};
use rustgm::pipeline;

#[test]
#[ignore = "requires the locked CPPGM object backend and a POSIX C compiler"]
fn scalar_poll_executes_source_without_user_copy_and_rejects_completed_reentry() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/scalar-poll-join-v1/native");
    std::fs::create_dir_all(&folder)?;
    for (name, observer, expected) in [
        ("pollready", "poll-observer.c", "3\n"),
        ("pollbyte", "poll-observer.c", "3\n"),
        ("pollrepeat", "poll-repeat-observer.c", ""),
    ] {
        let text = source::fixture(&format!(
            "pa43/tests/spec/future-runtime-v1/{name}-source.rs"
        ))?;
        let report = source::source_provider(&text, "scalar_poll_native")?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report)?;
        let program = rustgm::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            rustgm::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        crate::ffi_program::check(
            &root,
            &folder,
            name,
            &program,
            &source::fixture(&format!("pa43/tests/spec/future-runtime-v1/{observer}"))?,
            expected,
        )?;
    }
    Ok(())
}
