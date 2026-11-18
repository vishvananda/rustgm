//! Declared mixed-edition source controls in PA34's guest-editions packet.
//! rustc compiles each guest separately; only the bring-up transport is specific
//! to `RustGM`. Each crate retains its implemented edition rules.
use std::{
    path::PathBuf,
    process::{Command, Output},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa34/tests/spec/guest-editions";

fn session(name: &str, guests: &[(&str, &str)]) -> Result<Output> {
    let (folder, mut command) = prepare(name, guests)?;
    Ok(command.arg(folder.join("core.rs")).output()?)
}

fn prepare(name: &str, guests: &[(&str, &str)]) -> Result<(PathBuf, Command)> {
    let (folder, mut command) = crate::guest_crates::prepare(name, &[])?;
    let inputs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(PACKET);
    for source in ["shared.rs", "included.rs"] {
        std::fs::copy(inputs.join(source), folder.join(source))?;
    }
    std::fs::copy(inputs.join("host.rs"), folder.join("core.rs"))?;
    let skip = folder.join("skip.tsv");
    std::fs::write(&skip, "")?;
    command.args(["--bringup-skip-list"]).arg(skip);
    for (index, &(guest, source)) in guests.iter().enumerate() {
        let root = folder.join(format!("guest-{index}.rs"));
        std::fs::copy(inputs.join(source), &root)?;
        command
            .arg("--bringup-guest")
            .arg(format!("{guest}={}", root.display()));
    }
    Ok((folder, command))
}

#[test]
fn each_guest_keeps_its_edition_prelude_and_macro_definition_tokens() -> Result {
    for (name, guests) in [
        ("root-2021", vec![("legacy@2021", "legacy.rs")]),
        ("module-2021", vec![("legacy@2021", "module.rs")]),
        ("module-2015", vec![("legacy@2015", "module.rs")]),
        ("module-2018", vec![("legacy@2018", "module.rs")]),
        ("include-2021", vec![("legacy@2021", "include.rs")]),
        ("reserved-2021", vec![("legacy@2021", "reserved.rs")]),
        (
            "mixed-preludes-macro",
            vec![("legacy@2021", "legacy.rs"), ("modern@2024", "modern.rs")],
        ),
    ] {
        let output = session(name, &guests)?;
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

#[test]
fn a_2021_guest_never_lends_its_edition_to_a_2024_guest() -> Result {
    for (name, guests, message) in [
        (
            "root-2024",
            vec![("legacy@2024", "plain.rs")],
            "external blocks require unsafe",
        ),
        (
            "root-inherited",
            vec![("legacy", "plain.rs")],
            "external blocks require unsafe",
        ),
        (
            "module-cache-isolation",
            vec![("legacy@2021", "module.rs"), ("modern@2024", "module.rs")],
            "external blocks require unsafe",
        ),
        (
            "include-cache-isolation",
            vec![("legacy@2021", "include.rs"), ("modern@2024", "include.rs")],
            "external blocks require unsafe",
        ),
        (
            "reserved-2024",
            vec![("modern@2024", "reserved.rs")],
            "expected identifier",
        ),
    ] {
        let output = session(name, &guests)?;
        let errors = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{name}: {errors}");
        assert!(errors.contains(message), "{name}: {errors}");
    }
    Ok(())
}

#[test]
fn guest_edition_tags_are_checked_before_loading_sources() -> Result {
    for guest in ["legacy@2017", "legacy@", "legacy@2021@2024"] {
        let output = session("edition-usage", &[(guest, "legacy.rs")])?;
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("edition must be"));
    }
    Ok(())
}

#[test]
fn guest_editions_do_not_change_the_host_or_allow_repeated_roots() -> Result {
    let (folder, mut command) = prepare("host-edition", &[("legacy@2021", "reserved.rs")])?;
    let host = folder.join("core.rs");
    let source = std::fs::read_to_string(&host)?;
    std::fs::write(&host, format!("{source}\npub fn gen() {{}}\n"))?;
    let output = command.arg(host).output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("file 0:"));

    let (folder, mut command) = prepare("root-edition", &[("legacy@2021", "reserved.rs")])?;
    command.arg("--bringup-guest").arg(format!(
        "modern@2024={}",
        folder.join("guest-0.rs").display()
    ));
    let output = command.arg(folder.join("core.rs")).output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("shares its root source"));
    Ok(())
}

#[test]
fn rust_2015_guests_inject_a_private_crate_root_item() -> Result {
    const STD: &str = "#![no_std]\n#![no_main]\n\
        pub mod prelude { pub mod rust_2015 {} }\n\
        pub mod sample { pub struct Selected; }";
    let cases = [
        (
            "legacy-core",
            2015,
            "#![no_std]\n#![no_implicit_prelude]",
            "mod nested {\n\
                use core::prelude::rust_2024 as plain;\n\
                use ::core::prelude::rust_2024 as absolute;\n\
                use crate::core::prelude::rust_2024 as rooted;\n\
                pub fn value() -> rooted::From2024 {\n\
                    let _: plain::From2024 = absolute::From2024;\n\
                    rooted::From2024\n\
                }\n\
            }",
            None,
        ),
        (
            "legacy-std",
            2015,
            "",
            "mod nested { use std::sample; pub fn value() -> sample::Selected { sample::Selected } }",
            None,
        ),
        (
            "legacy-private",
            2015,
            "#![no_std]",
            "pub use crate::core as exposed;",
            Some("private import or widened visibility"),
        ),
        (
            "legacy-duplicate",
            2015,
            "#![no_std]",
            "extern crate core;",
            Some("duplicate type name `core`"),
        ),
        (
            "legacy-no-core",
            2015,
            "#![feature(no_core)]\n#![no_core]",
            "use core::prelude;",
            Some("unresolved import"),
        ),
        (
            "modern-extern-prelude",
            2018,
            "#![no_std]",
            "mod nested { use core::prelude::rust_2024; pub fn value() -> rust_2024::From2024 { rust_2024::From2024 } }",
            None,
        ),
        (
            "modern-no-root-item",
            2018,
            "#![no_std]",
            "use crate::core::prelude;",
            Some("unresolved import"),
        ),
    ];
    for (name, edition, attrs, contents, expected) in cases {
        let (folder, mut command) = prepare(name, &[])?;
        let std = folder.join("std.rs");
        std::fs::write(&std, STD)?;
        let guest = folder.join("guest.rs");
        std::fs::write(&guest, format!("#![no_main]\n{attrs}\n{contents}\n"))?;
        let output = command
            .arg("--bringup-guest")
            .arg(format!("std@2018={}", std.display()))
            .arg("--bringup-guest")
            .arg(format!("app@{edition}={}", guest.display()))
            .arg(folder.join("core.rs"))
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        if let Some(message) = expected {
            assert_eq!(output.status.code(), Some(1), "{name}: {stderr}");
            assert!(stderr.contains(message), "{name}: {stderr}");
        } else {
            assert!(output.status.success(), "{name}: {stderr}");
        }
    }
    Ok(())
}
