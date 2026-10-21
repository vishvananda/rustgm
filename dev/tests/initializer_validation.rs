//! Ordinary unused initializers are required, while associated values remain lazy.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/resolve.rs"]
mod source;
use std::{path::Path, process::Command};

const INVALID: &[(&str, &str)] = &[
    ("const-divide", "const C:u8=1/0;fn main(){}"),
    ("static-divide", "static S:u8=1/0;fn main(){}"),
    ("mutable-static-divide", "static mut S:u8=1/0;fn main(){}"),
    ("overflow", "const C:u8=255u8+1;fn main(){}"),
    (
        "closed-call",
        "const fn f(n:u8)->u8{1/n}const C:u8=f(0);fn main(){}",
    ),
    (
        "union-invalid-bool",
        "union U{x:u8,b:bool}const X:bool=unsafe{U{x:3}.b};fn main(){}",
    ),
    (
        "union-uninitialized",
        "union U{x:u8,y:u32}const X:u32=unsafe{U{x:1}.y};fn main(){}",
    ),
];
const LAZY: &[(&str, &str)] = &[
    ("unused-callee", "const fn f(n:u8)->u8{1/n}fn main(){}"),
    ("lazy-branch", "const C:u8=if true{7}else{1/0};fn main(){}"),
    (
        "associated-closed",
        "struct S;const fn f(n:u8)->u8{1/n}impl S{const C:u8=f(0);}fn main(){}",
    ),
    (
        "associated-generic",
        "struct S<T>(T);const fn f(n:u8)->u8{1/n}impl<T>S<T>{const C:u8=f(0);}fn main(){}",
    ),
];

#[test]
fn unused_initializer_outcomes_match_independent_object_qualification()
-> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let folder = root.join(format!(
        "obj/parallel/pa25-initializer-outcomes/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let mut errors = Vec::new();
    for (cases, accepted) in [(INVALID, false), (LAZY, true)] {
        for &(name, text) in cases {
            let input = folder.join(format!("{name}.rs"));
            std::fs::write(&input, text)?;
            let independent = Command::new("rustc")
                .args([
                    "+1.98.0",
                    "--edition=2024",
                    "--crate-name=initializer_outcome",
                    "--emit=obj",
                    "-o",
                ])
                .arg(folder.join(format!("{name}.o")))
                .arg(&input)
                .output()?;
            std::fs::write(
                folder.join(format!("{name}.rustc.stderr")),
                &independent.stderr,
            )?;
            assert_eq!(
                independent.status.code(),
                Some(i32::from(!accepted)),
                "{name}"
            );
            for mode in ["--emit-types", "--emit-lowir"] {
                let candidate = Command::new(env!("CARGO_BIN_EXE_rustgm"))
                    .arg(mode)
                    .arg(&input)
                    .output()?;
                std::fs::write(
                    folder.join(format!("{name}.{mode}.stderr")),
                    &candidate.stderr,
                )?;
                if candidate.status.code() != Some(i32::from(!accepted)) {
                    errors.push(format!(
                        "{name} {mode}: {:?}: {}",
                        candidate.status,
                        String::from_utf8_lossy(&candidate.stderr)
                    ));
                }
            }
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the configured CPPGM native backend"]
fn required_initializers_preserve_lazy_paths_and_runtime_values()
-> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "parallel/pa25-initializer-values",
        &[
            (
                "ordinary-static",
                "const fn f(n:u8)->u8{n+1}const C:u8=f(6);static S:u8=f(8);fn main(){check(C==7&&S==9);}",
                false,
            ),
            (
                "mutable-static",
                "const fn f(n:u8)->u8{n+1}static mut S:u8=f(8);fn main(){check(unsafe{S}==9);}",
                false,
            ),
            (
                "lazy-branch",
                "const C:u8=if true{7}else{1/0};fn main(){check(C==7);}",
                false,
            ),
            (
                "forward-value",
                "const C:u8=f();const fn f()->u8{N+1}const N:u8=6;fn main(){check(C==7);}",
                false,
            ),
        ],
    )
}

#[test]
fn ordinary_initializer_dispatch_is_linear_and_reuses_checked_callees()
-> Result<(), Box<dyn std::error::Error>> {
    for width in [8, 32, 128] {
        use std::fmt::Write;
        let mut text = String::from("const fn f(n:u32)->u32{n+1}");
        for index in 0..width {
            write!(
                text,
                "const C{index}:u32=f({index});static S{index}:u32=f({index});static mut M{index}:u32=f({index});"
            )?;
        }
        text.push_str(
            "struct Lazy;impl Lazy{const C:u32=f(0);}const fn unused(n:u32)->u32{1/n}fn main(){}",
        );
        let mut report = source::resolve(&text, true)?;
        let rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        } = rustgm::pipeline::analyze_source(&report)?;
        assert_eq!(facts.initializers.len(), 3 * width + 1);
        assert_eq!(facts.constant_work.initializer_visits, 3 * width + 1);
        assert_eq!(facts.constant_work.initializer_validations, 3 * width);
        assert_eq!(facts.constant_work.initializer_indexed, 3 * width + 1);
        assert_eq!(facts.constant_work.expressions, 3 * width);
        assert_eq!(facts.body_work.cfg_functions.get(), 1);
        assert!(facts.constant_work.initializer_lookups.get() <= 8 * (3 * width + 1));
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
        assert_eq!(facts.constant_work.expressions, 3 * width);
        assert_eq!(facts.constant_work.initializer_validations, 3 * width);
        // Runtime preparation retains f and builds main plus the unused const
        // function's ordinary CFG; it does not execute that unused function.
        assert_eq!(facts.body_work.cfg_functions.get(), 3);
    }
    Ok(())
}
