//! Initializer borrows have an owning body; actual storage determines escape validity.
use crate::native_references as native;
use crate::resolve as source;
use std::{path::Path, process::Command};

const CASES: &[(&str, &str, bool)] = &[
    (
        "transient",
        "const VALUE:()={let value=7u32;let _reference=&value;};fn main(){}",
        true,
    ),
    (
        "static-local",
        "static VALUE:u8={let value=7u8;let reference=&value;*reference};fn main(){}",
        true,
    ),
    (
        "mutable",
        "const VALUE:u8={let mut value=7u8;let reference=&mut value;*reference+=1;value};fn main(){}",
        true,
    ),
    (
        "projection",
        "struct S{value:u8}const VALUE:u8={let value=S{value:7};let reference=&value.value;*reference};fn main(){}",
        true,
    ),
    (
        "reborrow",
        "const VALUE:u8={let value=7u8;let first=&value;let second=&*first;*second};fn main(){}",
        true,
    ),
    (
        "method",
        "struct S(u8);impl S{const fn get(&self)->u8{self.0}}const VALUE:u8={let value=S(7);value.get()};fn main(){}",
        true,
    ),
    ("literal", "const VALUE:&u8=&7;fn main(){}", true),
    (
        "static",
        "static STORAGE:u8=7;const VALUE:&u8=&STORAGE;fn main(){}",
        true,
    ),
    (
        "constant",
        "const STORAGE:u8=7;const VALUE:&u8=&STORAGE;fn main(){}",
        true,
    ),
    (
        "escape-const",
        "const VALUE:&u8={let value=7u8;&value};fn main(){}",
        false,
    ),
    (
        "escape-static",
        "static VALUE:&u8={let value=7u8;&value};fn main(){}",
        false,
    ),
    (
        "escape-function",
        "const fn value()->&'static u8{let local=7u8;&local}const VALUE:&u8=value();fn main(){}",
        false,
    ),
];

#[test]
fn initializer_storage_outcomes_match_independent_qualification()
-> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!(
        "obj/parallel/pa25-initializer-lifetimes/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for &(name, text, accepted) in CASES {
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, text)?;
        let independent = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-name=lifetime_control",
                "--cap-lints=allow",
                "--emit=obj",
                "-o",
            ])
            .arg(folder.join(format!("{name}.o")))
            .arg(&path)
            .output()?;
        std::fs::write(
            folder.join(format!("{name}.rustc.stderr")),
            &independent.stderr,
        )?;
        assert_eq!(
            independent.status.code(),
            Some(i32::from(!accepted)),
            "independent {name}"
        );
        for mode in ["--emit-types", "--emit-lowir"] {
            let actual = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                .arg(mode)
                .arg(&path)
                .output()?;
            std::fs::write(folder.join(format!("{name}.{mode}.stdout")), actual.stdout)?;
            std::fs::write(folder.join(format!("{name}.{mode}.stderr")), &actual.stderr)?;
            if actual.status.code() != Some(i32::from(!accepted)) {
                failures.push(format!(
                    "{name} {mode}: {:?}: {}",
                    actual.status,
                    String::from_utf8_lossy(&actual.stderr)
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn local_borrow_region_retains_actual_initializer_owner() -> Result<(), Box<dyn std::error::Error>>
{
    let report = source::resolve(CASES[0].1, true)?;
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    let initializer = analysis.facts.initializers.first().ok_or("initializer")?;
    let mut borrows = analysis.facts.body_regions.iter().filter(|region| {
        matches!(
            region.origin,
            rustgm::sema::model::BodyRegionOrigin::ExpressionBorrow
        )
    });
    let borrow = borrows.next().ok_or("missing owned borrow region")?;
    assert_eq!(borrow.body, initializer.value);
    assert!(borrows.next().is_none());
    Ok(())
}

#[test]
#[ignore = "requires configured pinned CPPGM backend"]
fn initializer_local_reborrows_methods_and_static_results_execute()
-> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "parallel/pa25-initializer-lifetimes",
        &[
            (
                "locals",
                "struct S(u8);impl S{const fn get(&self)->u8{self.0}}const UNIT:()={let value=7u32;let _reference=&value;};const FIRST:u8={let mut value=7;let reference=&mut value;*reference+=1;value};const SECOND:u8={let value=S(9);value.get()};static THIRD:u8={let value=7u8;let first=&value;let second=&*first;*second};fn main(){check(FIRST==8&&SECOND==9&&THIRD==7);}",
                false,
            ),
            (
                "static-results",
                "static STORAGE:u8=7;const OTHER:u8=9;const FIRST:&u8=&7;const SECOND:&u8=&STORAGE;const THIRD:&u8=&OTHER;fn main(){check(*FIRST==7&&*SECOND==7&&*THIRD==9);}",
                false,
            ),
        ],
    )
}
