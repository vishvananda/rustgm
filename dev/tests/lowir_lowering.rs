use rustgm::lowir::{
    builder::Builder,
    program::{BlockId, Operand, Terminator},
    vocabulary::{Predicate, Type},
};
use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

fn compile(source: &str) -> Result<Output, Box<dyn std::error::Error>> {
    mode(source, "--emit-lowir")
}
fn mode(source: &str, mode: &str) -> Result<Output, Box<dyn std::error::Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args([mode, "--edition=2024"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("missing compiler stdin")?
        .write_all(source.as_bytes())?;
    Ok(child.wait_with_output()?)
}

#[test]
fn count_mode_observes_the_same_lowered_program() -> Result<(), Box<dyn std::error::Error>> {
    let source = "fn main(){let mut x=0;while x<3{x+=1;}}";
    let full = compile(source)?;
    let count = mode(source, "--count-lowir")?;
    assert!(full.status.success());
    assert!(count.status.success());
    assert!(count.stderr.is_empty());
    let text = String::from_utf8(full.stdout)?;
    let fields = String::from_utf8(count.stdout)?;
    let fields: Vec<_> = fields.split_whitespace().collect();
    assert_eq!(
        fields[1].parse::<usize>()?,
        text.matches("function @").count()
    );
    assert_eq!(
        fields[5].parse::<usize>()?,
        text.matches("  block ^").count()
    );
    assert_eq!(
        fields[9].parse::<usize>()?,
        text.matches("  slot $").count()
    );
    assert_eq!(
        fields[11].parse::<usize>()?,
        text.lines()
            .filter(|line| line.starts_with("    %t"))
            .count()
    );
    Ok(())
}

#[test]
fn construction_rejects_unfinished_or_invalid_control_flow() -> Result<(), &'static str> {
    let mut builder = Builder::default();
    assert!(builder.slot(Type::Void).is_err());
    let entry = builder.block()?;
    let target = builder.block()?;
    builder.enter(entry)?;
    assert!(builder.enter(target).is_err());
    assert!(builder.terminate(Terminator::Jump(BlockId(100))).is_err());
    builder.terminate(Terminator::Jump(target))?;
    assert!(builder.enter(entry).is_err());
    assert!(builder.finish().is_err());
    Ok(())
}

#[test]
fn comparison_width_and_placement_order_are_typed() -> Result<(), &'static str> {
    let mut builder = Builder::default();
    let entry = builder.block()?;
    let exit = builder.block()?;
    let middle = builder.block()?;
    builder.enter(entry)?;
    let a = builder.integer(Type::I128, u128::MAX)?;
    let b = builder.integer(Type::I128, 0)?;
    let compare = builder.compare(Predicate::Ult, a, b)?;
    assert_eq!(compare.ty, Type::I64);
    builder.terminate(Terminator::Jump(middle))?;
    builder.enter(middle)?;
    builder.terminate(Terminator::Jump(exit))?;
    builder.enter(exit)?;
    builder.terminate(Terminator::Return(Some(compare)))?;
    let function = builder.finish()?;
    assert_eq!(function.order, [entry, middle, exit]);
    assert!(matches!(compare.operand, Operand::Temporary(_)));
    Ok(())
}

#[test]
fn executable_checks_all_bodies_but_emits_only_demanded_functions()
-> Result<(), Box<dyn std::error::Error>> {
    let result = compile("fn unused(x:u128,y:u128)->u128{x/y} fn main(){}")?;
    assert!(result.status.success(), "{:?}", result.stderr);
    let text = String::from_utf8(result.stdout)?;
    assert_eq!(text.matches("function @").count(), 2);
    assert!(!text.contains("abort"));
    let rejected = compile("fn unused(){let x:i32;let y=x;}fn main(){}")?;
    assert_eq!(rejected.status.code(), Some(1));
    assert!(rejected.stdout.is_empty());
    assert!(String::from_utf8(rejected.stderr)?.contains("uninitialized"));
    Ok(())
}

#[test]
fn initialization_joins_converge_before_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
    for source in [
        "fn main(){let x;loop{x=1;break;}let y=x;}",
        "fn f(b:bool)->i32{let x;if b{x=1;}else{x=2;}x}fn main(){let _=f(true);}",
        "fn main(){let mut n=0;while n<4{let x;x=n;n+=1;let y=x;}}",
        "fn main(){let x:i32;let _=(x);let y:();let _=y;}",
    ] {
        let result = compile(source)?;
        assert!(result.status.success(), "{source}: {:?}", result.stderr);
    }
    for source in [
        "fn f(b:bool){let x;loop{x=1;if b{break;}}}fn main(){}",
        "fn f(b:bool){let x:i32;if b{x=1;}let y=x;}fn main(){}",
        "fn main(){let x:();let y=x;}",
        "fn main(){let x=1;x=2;}",
    ] {
        let result = compile(source)?;
        assert_eq!(result.status.code(), Some(1), "{source}");
        assert!(result.stdout.is_empty(), "{source}");
    }
    Ok(())
}

#[test]
fn deep_scalar_lowering_is_iterative_and_deterministic() -> Result<(), Box<dyn std::error::Error>> {
    let source = format!("fn main(){{let _ = 0u128{};}}", "+1".repeat(10_000));
    let first = compile(&source)?;
    let second = compile(&source)?;
    assert!(first.status.success(), "{:?}", first.stderr);
    assert!(first.stderr.is_empty());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    let text = String::from_utf8(first.stdout)?;
    assert_eq!(text.matches("binary add i128").count(), 10_000);
    assert_eq!(text.matches("declare function @abort").count(), 1);
    assert!(!text.contains("nowrap"));
    assert!(!text.contains(" typed "));
    Ok(())
}
