#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{
    body,
    borrow::{
        self,
        regions::{self, constraints::Constraints},
    },
    resolve::unit::{FileId, NodeKey},
    syntax::ast::NodeId,
    types::{RegionId, Store, variance::Variances},
};
use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

#[test]
fn contract_view_distinguishes_assumptions_variables_and_retained_requirements()
-> Result<(), Box<dyn std::error::Error>> {
    let types = Store::default();
    let variances = Variances::default();
    let mut graph = Constraints::new(&types, &variances, 2, 1000)?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    graph.assume(a, b)?;
    graph.variable(RegionId(7))?;
    graph.outlives(a, RegionId(7), 0)?;
    graph.outlives(RegionId(7), b, 1)?;
    graph.outlives(a, RegionId(7), 2)?;
    let proof = graph.prove()?;
    let analysis = regions::Analysis {
        bodies: vec![regions::Body {
            key: body::ir::BodyKey::Anonymous(NodeKey {
                file: FileId(0),
                node: NodeId(1),
            }),
            hidden_regions: Vec::new(),
            builtins: None,
            source: NodeKey {
                file: FileId(0),
                node: NodeId(1),
            },
            solution: proof.solution,
            instance: None,
            identities: proof.identities,
        }],
    };
    let mut view = Vec::new();
    regions::view::write(&analysis, &mut view)?;
    let expected = "regions-crate 1
body 0 source=0:1
  universal b0=g2
  universal b1=g3
  variable r7=g4
  region g0 static
  region g1 local
  region g2 universal universe=0
  region g3 universal universe=0
  region g4 variable universe=0
  assume g2:g3
  require g2:g4
  require g2:g4
  require g4:g1
  require g4:g3
end-body
eof bodies=1 universals=2 variables=1 regions=5 assumptions=1 requirements=4
";
    assert_eq!(String::from_utf8(view)?, expected);
    let mut count = Vec::new();
    regions::view::count(&analysis, &mut count)?;
    assert_eq!(
        count,
        b"bodies=1 universals=2 variables=1 regions=5 assumptions=1 requirements=4\n"
    );
    Ok(())
}

fn run(mode: &str, source: &str) -> Result<Output, Box<dyn std::error::Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg(mode)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("compiler input")?
        .write_all(source.as_bytes())?;
    Ok(child.wait_with_output()?)
}

#[test]
fn contract_and_loan_modes_have_separate_proof_boundaries() -> Result<(), Box<dyn std::error::Error>>
{
    let cases = [
        ("fn f<'a:'b,'b>(p:&'a u8)->&'b u8{p}", true, true),
        ("fn f<'a,'b>(p:&'a u8)->&'b u8{p}", false, false),
        ("fn f()->&'static u8{let x=7;&x}", false, false),
        ("fn f(){let mut x=0;let p=&x;let y=*p;x=1;}", true, true),
        ("fn f(){let mut x=0;let p=&x;x=1;let y=*p;}", true, false),
        ("fn f(){let p;{let x=0;p=&x;}let y=*p;}", true, false),
        (
            "fn take(_: &mut u8,_:u8){}fn f(p:&mut u8){take(p,*p);}",
            true,
            true,
        ),
        (
            "fn take(_: &mut u8,_:u8){}fn f(p:&mut u8){take(&mut *p,*p);}",
            true,
            false,
        ),
        (
            "#[repr(C,packed)]struct S{a:u8,b:u64}fn f(s:&S){let p=&s.b;}",
            true,
            false,
        ),
        (
            "fn f<'a>(p:for<'b>fn(&'b u8)->&'b u8,x:&'a u8)->&'a u8{p(x)}",
            true,
            true,
        ),
        (
            "struct S<'a>(&'a u8);fn f<'a:'b,'b>(s:S<'a>)->S<'b>{s}",
            true,
            true,
        ),
        (
            "fn f<'a:'b,'b>(p:&'b mut &'a u8)->&'b mut &'b u8{p}",
            false,
            false,
        ),
    ];
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/proof-views-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    for (i, (source, contract, loans)) in cases.into_iter().enumerate() {
        let input = folder.join(format!("case-{i}.rs"));
        std::fs::write(&input, source)?;
        let independent = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-type=lib",
                "--crate-name=probe",
                "--cap-lints=allow",
                "--emit=metadata",
            ])
            .arg(&input)
            .arg("-o")
            .arg(input.with_extension("rmeta"))
            .output()?;
        std::fs::write(input.with_extension("rustc.stderr"), &independent.stderr)?;
        assert_eq!(
            independent.status.code(),
            Some(i32::from(!loans)),
            "rustc {source}: {}",
            String::from_utf8_lossy(&independent.stderr)
        );
        for (stage, accepted) in [("regions", contract), ("loans", loans)] {
            let rendered = run(&format!("--emit-{stage}"), source)?;
            let counted = run(&format!("--count-{stage}"), source)?;
            for output in [&rendered, &counted] {
                assert_eq!(
                    output.status.code(),
                    Some(i32::from(!accepted)),
                    "{stage} {source}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert_eq!(output.stdout.is_empty(), !accepted);
                assert_eq!(output.stderr.is_empty(), accepted);
            }
            assert_eq!(rendered.stderr, counted.stderr);
            if accepted {
                assert!(
                    rendered
                        .stdout
                        .ends_with(&[b"eof ", counted.stdout.as_slice()].concat())
                );
                let repeated = run(&format!("--emit-{stage}"), source)?;
                assert_eq!(repeated.status.code(), Some(0));
                assert_eq!(rendered.stdout, repeated.stdout);
            }
        }
    }
    Ok(())
}

#[test]
fn loan_inspection_preserves_proof_counts_and_records_call_activation()
-> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) =
        semantic::analyze("fn take(_: &mut u8,_:u8){}fn f(p:&mut u8){take(p,*p);}")?;
    let mut program = body::build(&report, &facts)?;
    body::promote::analyze(&report, &mut facts, &mut program)?;
    let contracts = body::regions::collect(&facts)?;
    let regions = regions::check_bodies(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        &program,
    )?;
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        copies: &mut facts.copies,
    };
    let plain = borrow::check(&mut environment, &regions, &program)?;
    let inspected = borrow::inspect(&mut environment, &regions, &program)?;
    assert!(plain.inspection.is_empty());
    for (a, b) in plain.loans.iter().zip(&inspected.loans) {
        assert_eq!(
            (a.loans, a.steps, a.points, a.region_words),
            (b.loans, b.steps, b.points, b.region_words)
        );
    }
    let mut output = Vec::new();
    assert!(borrow::loans::view::write(&plain, &mut output).is_err());
    assert!(output.is_empty());
    borrow::loans::view::write(&inspected, &mut output)?;
    let view = String::from_utf8(output)?;
    let body = program.bodies.last().ok_or("caller body")?;
    let mut base = 0;
    let mut activation = None;
    for block in &body.blocks {
        for at in block.start..block.end {
            if let body::ir::Action::Evaluate(value) = body.statements[at as usize].action
                && matches!(
                    body.values[value.0 as usize].kind,
                    body::ir::Rvalue::Call { .. }
                )
            {
                activation = Some(base + at - block.start);
            }
        }
        base += block.end - block.start + 1;
    }
    let activation = activation.ok_or("call activation")?;
    assert!(view.contains("kind=two-phase\n"));
    assert!(
        view.contains(&format!(
            "  state t{activation} word=0 live=0000000000000001 activated=0000000000000001\n"
        )),
        "{view}"
    );
    assert!(
        view.lines()
            .any(|l| l.starts_with("  state ") && l.ends_with("activated=0000000000000000")),
        "reservation must precede activation: {view}"
    );
    Ok(())
}
