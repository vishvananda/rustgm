//! Cleanup demand must neither instantiate unused recipes nor bypass original proofs.
use crate::qualify as rustc;
use crate::semantic;
use std::{path::Path, process::Command};

const GROWING: &str = "#![recursion_limit=\"64\"]struct D<T>(T);impl<T> Drop for D<T>{fn drop(&mut self){grow::<T>();}}fn grow<T>(){grow::<(T,T)>();}";

fn compile(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    rustgm::lower::instances::build(&report, &mut facts, entry)?;
    Ok(())
}
fn codegen(name: &str, source: &str, accepted: bool) -> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/drop-demand-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    let input = folder.join(format!("{name}.rs"));
    std::fs::write(&input, source)?;
    let output = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-name=drop_demand",
            "--emit=obj",
            "-Cpanic=abort",
            "-o",
        ])
        .arg(folder.join(format!("{name}.o")))
        .arg(input)
        .output()?;
    std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
    assert_eq!(
        output.status.code(),
        Some(i32::from(!accepted)),
        "{name}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    if !accepted {
        assert!(String::from_utf8_lossy(&output.stderr).contains("recursion limit"));
    }
    Ok(())
}

#[test]
fn only_selected_runtime_cleanup_roots_demand_destructors() -> Result<(), Box<dyn std::error::Error>>
{
    for (name, body) in [
        ("diverging-local", "fn main(){let _d=D(7u8);loop{}}"),
        (
            "diverging-owned-call",
            "fn keep<T>(x:T)->!{loop{}}fn main(){keep(D(7u8));}",
        ),
        (
            "transferred-then-diverge",
            "fn id<T>(x:T)->T{x}fn main(){let _d=id(D(7u8));loop{}}",
        ),
        (
            "unused-ordinary-owner",
            "fn unused(){let _d=D(7u8);}fn main(){}",
        ),
    ] {
        let source = format!("{GROWING}{body}");
        // Object generation is essential: rustc metadata does not check monomorphization.
        // The diverging programs must never be executed.
        codegen(name, &source, true)?;
        compile(&source).map_err(|e| format!("{name}: {e}"))?;
    }
    for (name, source) in [
        (
            "demanded-growing-drop",
            format!("{GROWING}fn main(){{let _d=D(7u8);}}"),
        ),
        (
            "growing-drop-from-drop",
            String::from(
                "#![recursion_limit=\"64\"]struct D<T>([T;0]);impl<T> Drop for D<T>{fn drop(&mut self){let _d=D::<(T,)>([]);}}fn main(){let _d=D::<u8>([]);}",
            ),
        ),
    ] {
        codegen(name, &source, false)?;
        let error = compile(&source)
            .err()
            .ok_or("unbounded destructor demand was accepted")?;
        assert!(
            error.to_string().contains("128 active instances"),
            "{name}: {error}"
        );
    }
    Ok(())
}

#[test]
fn destructor_instances_preserve_moves_loans_and_unused_template_checks()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        (
            "unused-drop-field-move",
            "struct D<T>(T);impl<T> Drop for D<T>{fn drop(&mut self){}}fn bad<T>(x:D<T>)->T{x.0}fn main(){}",
        ),
        (
            "unused-double-move",
            "fn bad<T>(x:T){let a=x;let b=x;}fn main(){}",
        ),
        (
            "drop-local-reference",
            "struct D<T>(T);impl<T> Drop for D<T>{fn drop(&mut self){}}fn main(){let d;{let n=7u8;d=D(&n);}}",
        ),
        (
            "drop-active-loan",
            "struct D<T>(T);impl<T> Drop for D<T>{fn drop(&mut self){}}fn main(){let mut n=7u8;let d=D(&n);n=8;}",
        ),
        (
            "partial-wrapper-live-loan",
            "struct D<'a,T>(&'a mut u8,T);impl<T> Drop for D<'_,T>{fn drop(&mut self){*self.0+=1;}}struct W<T>(T,T);fn take<T>(x:T){}fn main(){let mut a=0;let mut b=0;{let w=W(D(&mut a,7u8),D(&mut b,8u8));take(w.0);let _read=a;}}",
        ),
    ];
    for (name, source) in cases {
        rustc::qualify(name, source, false)?;
        assert!(compile(source).is_err(), "{name}");
    }
    Ok(())
}
