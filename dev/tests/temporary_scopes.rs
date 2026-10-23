use crate::semantic;
use rustgm::{
    body::scopes::{self, Facts, Kind, Lifetime},
    resolve::{expansion::Report, unit::NodeKey},
    sema::model::Declarations,
    syntax::ast::{Kind as Syntax, NodeId},
};

fn analyze(source: &str) -> Result<(Report, Declarations, Facts, NodeKey), String> {
    let (report, types) = semantic::analyze(source)?;
    let function = types.functions.last().ok_or("missing function")?;
    let body = function.body.ok_or("missing body")?;
    let scopes = scopes::function(&report, &types, function).map_err(|e| e.to_string())?;
    Ok((report, types, scopes, body))
}
fn expression(
    report: &Report,
    scopes: &Facts,
    body: NodeKey,
    source: &str,
    spelling: &str,
) -> Result<NodeId, String> {
    let ast = &report.package.unit(body.file).configured.ast;
    let nodes = ast
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            let id = NodeId::new(u32::try_from(index).ok()?);
            (scopes.node(id).is_some()
                && source.get(node.span.start as usize..node.span.end as usize) == Some(spelling))
            .then_some(id)
        })
        .collect::<Vec<_>>();
    match nodes.as_slice() {
        [node] => Ok(*node),
        _ => Err(format!(
            "expected one scoped expression {spelling:?}, got {nodes:?}"
        )),
    }
}
fn lifetime_kind(scopes: &Facts, node: NodeId) -> Result<Kind, String> {
    let info = scopes.node(node).ok_or("missing expression scopes")?;
    let Lifetime::Scope(id) = info.temporary else {
        return Err("unexpected static temporary".into());
    };
    Ok(scopes.scopes()[id.0 as usize].kind)
}

#[test]
fn extending_expressions_retain_the_enclosing_block() -> Result<(), String> {
    let cases = [
        ("&temp()", Kind::Block),
        ("(&temp(),)", Kind::Block),
        ("[&temp()]", Kind::Block),
        ("[&temp();1]", Kind::Statement),
        ("id(&temp())", Kind::Statement),
        ("{&temp()}", Kind::Block),
        ("{id(&temp())}", Kind::Tail),
        ("if true{&temp()}else{&0}", Kind::Block),
        ("match true{true=>&temp(),false=>&0}", Kind::Block),
        ("W(&temp())", Kind::Block),
        ("(W)(&temp())", Kind::Block),
        ("S{a:&temp()}", Kind::Block),
        ("S{..S{a:&temp()}}", Kind::Statement),
    ];
    for (initializer, expected) in cases {
        let source = format!(
            "struct W<'a>(&'a u8);struct S<'a>{{a:&'a u8}}fn temp()->u8{{1}}fn id(x:&u8)->&u8{{x}}fn f(){{let x={initializer};}}"
        );
        let (report, _, facts, body) =
            analyze(&source).map_err(|e| format!("{initializer}: {e}"))?;
        let node = expression(&report, &facts, body, &source, "temp()")?;
        assert_eq!(lifetime_kind(&facts, node)?, expected, "{initializer}");
        assert_eq!(
            facts.node(node).ok_or("missing scopes")?.extended,
            expected == Kind::Block
        );
    }
    Ok(())
}

#[test]
fn constructor_aliases_do_not_inherit_syntactic_extension() -> Result<(), String> {
    for callee in ["ctor", "{W}"] {
        let source = format!(
            "struct W<'a>(&'a u8);fn temp()->u8{{1}}fn f(){{let ctor=W;let x={callee}(&temp());}}"
        );
        let (report, _, facts, body) = analyze(&source)?;
        let node = expression(&report, &facts, body, &source, "temp()")?;
        assert_eq!(lifetime_kind(&facts, node)?, Kind::Statement);
    }
    Ok(())
}

#[test]
fn place_extension_follows_the_base_but_not_the_index() -> Result<(), String> {
    let source = "fn array()->[u8;1]{[1]}fn index()->usize{0}fn f(){let x=&array()[index()];}";
    let (report, _, facts, body) = analyze(source)?;
    for (name, expected) in [("array()", Kind::Block), ("index()", Kind::Statement)] {
        let node = expression(&report, &facts, body, source, name)?;
        assert_eq!(lifetime_kind(&facts, node)?, expected);
    }
    let source = "fn temp()->u8{1}fn f(){let x=&{temp()};}";
    let (report, _, facts, body) = analyze(source)?;
    let result = expression(&report, &facts, body, source, "{temp()}")?;
    let tail = expression(&report, &facts, body, source, "temp()")?;
    assert_eq!(lifetime_kind(&facts, result)?, Kind::Block);
    assert_eq!(lifetime_kind(&facts, tail)?, Kind::Tail);
    Ok(())
}

#[test]
fn materialized_block_results_keep_their_own_scope_provenance() -> Result<(), String> {
    use rustgm::body::{self, ir::Action};
    let source = "fn temp()->u8{1}fn f(){let x=&{temp()};}";
    let (report, types, scopes, body_source) = analyze(source)?;
    let program = body::build(&report, &types).map_err(|e| e.to_string())?;
    let body = program
        .bodies
        .iter()
        .find(|b| b.source == body_source.into())
        .ok_or("missing body")?;
    let [temporary] = body.temporaries.as_slice() else {
        return Err("expected one temporary".into());
    };
    let result = expression(&report, &scopes, body_source, source, "{temp()}")?;
    let tail = expression(&report, &scopes, body_source, source, "temp()")?;
    assert_eq!(temporary.source, result.into());
    assert_eq!(lifetime_kind(&scopes, result)?, Kind::Block);
    let evaluation = body
        .statements
        .iter()
        .enumerate()
        .find(|(_, s)| {
            matches!(s.action,
        Action::Evaluate(id) if id == temporary.value)
        })
        .ok_or("missing evaluation")?;
    assert_eq!(evaluation.1.source, tail.into());
    assert!(evaluation.0 < temporary.before as usize);
    let block = &body.blocks[temporary.block.0 as usize];
    assert!(block.start <= temporary.before && temporary.before <= block.end);
    Ok(())
}

#[test]
fn explicit_pattern_extension_does_not_use_match_ergonomics() -> Result<(), String> {
    let cases = [
        ("ref x", "temp()", Kind::Block),
        ("(ref x,)", "(temp(),)", Kind::Statement),
        ("&ref x", "*&&temp()", Kind::Statement),
        ("&ref x", "&*&temp()", Kind::Block),
        ("x @ ref y", "temp()", Kind::Statement),
    ];
    for (pattern, initializer, expected) in cases {
        let source = format!("fn temp()->u8{{1}}fn f(){{let {pattern}={initializer};}}");
        let (report, _, facts, body) = analyze(&source)?;
        let call = expression(&report, &facts, body, &source, "temp()")?;
        assert_eq!(
            lifetime_kind(&facts, call)?,
            expected,
            "{pattern}={initializer}"
        );
        let root = expression(&report, &facts, body, &source, initializer)?;
        assert_eq!(
            facts.node(root).ok_or("missing scopes")?.extended,
            matches!(pattern, "ref x" | "(ref x,)")
        );
    }
    Ok(())
}

#[test]
fn conditions_and_match_scrutinees_have_distinct_endpoints() -> Result<(), String> {
    let cases = [
        ("if see(&temp()){ }", Kind::Condition),
        ("if let x=&temp(){ }else{ }", Kind::IfThen),
        ("if let x=&temp() && see(x){ }", Kind::IfThen),
        ("while see(&temp()){break;}", Kind::Condition),
        ("while let x=&temp(){break;}", Kind::Iteration),
        ("match &temp(){x=>{}}", Kind::Statement),
        ("match 1u8{x if see(&temp())=>{},_=>{}}", Kind::Condition),
        ("let x=see(&temp())&&true;", Kind::Condition),
    ];
    for (statement, expected) in cases {
        let source =
            format!("fn temp()->u8{{1}}fn see(_: &u8)->bool{{true}}fn f(){{{statement};}}");
        let (report, _, facts, body) = analyze(&source)?;
        let call = expression(&report, &facts, body, &source, "temp()")?;
        assert_eq!(lifetime_kind(&facts, call)?, expected, "{statement}");
    }
    let source = "fn temp()->u8{1}fn f(){match &temp(){x=>{}}}";
    let (report, _, facts, body) = analyze(source)?;
    let call = expression(&report, &facts, body, source, "temp()")?;
    assert_eq!(lifetime_kind(&facts, call)?, Kind::Tail);
    Ok(())
}

#[test]
fn binding_endpoints_follow_blocks_and_condition_success() -> Result<(), String> {
    let source = "fn f(p:u8){let a=1u8;if let b=&a{let c=b;}match p{d=>{}}}";
    let (report, types, scopes, body) = analyze(source)?;
    let ast = &report.package.unit(body.file).configured.ast;
    for (index, node) in ast.nodes.iter().enumerate() {
        let Syntax::Binding { .. } = node.kind else {
            continue;
        };
        let key = NodeKey {
            node: NodeId::new(u32::try_from(index).map_err(|e| e.to_string())?),
            ..body
        };
        let definition = types
            .node(key)
            .and_then(|n| n.binding)
            .ok_or("missing binding")?;
        let scope = scopes.binding(definition).ok_or("missing binding scope")?;
        let expected = match &source[node.span.start as usize..node.span.end as usize] {
            "p" => Kind::Body,
            "a" | "c" => Kind::Block,
            "b" => Kind::IfThen,
            "d" => Kind::Arm,
            other => return Err(format!("unexpected binding {other}")),
        };
        assert_eq!(scopes.scopes()[scope.0 as usize].kind, expected);
    }
    Ok(())
}

#[test]
fn const_and_static_extensions_are_distinct_from_stack_storage() -> Result<(), String> {
    let source = "const C:&u8=&1;static S:&[u8;1]=&[2];";
    let (report, facts) = semantic::analyze(source)?;
    for initializer in &facts.initializers {
        let scopes =
            scopes::initializer(&report, &facts, initializer).map_err(|e| e.to_string())?;
        let ast = &report.package.unit(initializer.value.file).configured.ast;
        let Syntax::Unary { operand, .. } = ast.node(initializer.value.node).kind else {
            return Err("expected borrow initializer".into());
        };
        assert_eq!(
            scopes.node(operand).ok_or("missing scopes")?.temporary,
            Lifetime::Static
        );
    }
    Ok(())
}

#[test]
fn constant_patterns_have_no_local_storage() -> Result<(), String> {
    let source = "const C:u8=1;struct U;fn f(x:u8){let U=U;match x{C=>{},_=>{}}}";
    analyze(source)?;
    Ok(())
}

#[test]
fn inactive_fields_and_arms_neither_bind_nor_extend_temporaries() -> Result<(), String> {
    for (pattern, expected) in [("a", Kind::Statement), ("a:ref x", Kind::Block)] {
        let source = format!(
            "struct S{{a:u8}}fn temp()->S{{S{{a:1}}}}fn f(){{let S{{#[cfg(any())]ref missing,{pattern}}}=temp();}}"
        );
        let (report, _, scopes, body) = analyze(&source)?;
        let call = expression(&report, &scopes, body, &source, "temp()")?;
        assert_eq!(lifetime_kind(&scopes, call)?, expected);
    }
    analyze("fn f(x:&[bool]){match x{#[cfg(any())][missing,..]=>(),[..]=>()}}")?;
    let source = "struct S{a:u8}const C:S=S{#[cfg(any())]unknown:0,#[cfg(all())]a:1};";
    let (report, types) = semantic::analyze(source)?;
    for initializer in &types.initializers {
        scopes::initializer(&report, &types, initializer).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[test]
fn nested_borrow_extension_has_linear_work() -> Result<(), String> {
    let mut prior = 0;
    for depth in [16, 32, 64] {
        let source = format!(
            "fn temp()->u8{{1}}fn f(){{let x={}temp();}}",
            "& ".repeat(depth)
        );
        let (report, _, facts, body) = analyze(&source)?;
        let node = expression(&report, &facts, body, &source, "temp()")?;
        assert_eq!(lifetime_kind(&facts, node)?, Kind::Block);
        assert!(
            facts.work < 8 * depth + 20,
            "{} visits at depth {depth}",
            facts.work
        );
        if prior != 0 {
            assert!(facts.work <= prior * 2);
        }
        prior = facts.work;
    }
    Ok(())
}

#[test]
fn statement_expansions_keep_their_enclosing_binding_scope() -> Result<(), String> {
    let source = "macro_rules! bind{()=>{let x=&temp();}}fn temp()->u8{1}fn f(){bind!();}";
    let (report, _, scopes, body) = analyze(source)?;
    let ast = &report.package.unit(body.file).configured.ast;
    let mut temporaries = 0;
    for (index, node) in ast.nodes.iter().enumerate() {
        let id = NodeId::new(u32::try_from(index).map_err(|e| e.to_string())?);
        let Some(info) = scopes.node(id) else {
            continue;
        };
        if matches!(node.kind, Syntax::Call { .. }) {
            assert_eq!(lifetime_kind(&scopes, id)?, Kind::Block);
            assert!(info.extended);
            temporaries += 1;
        }
    }
    assert_eq!(temporaries, 1);
    Ok(())
}

#[test]
fn expression_expansions_preserve_places_and_extending_constructor_paths() -> Result<(), String> {
    use rustgm::body::{
        self,
        ir::{Place, Rvalue},
    };
    for initializer in ["&id!(*&temp())", "id!(W)(&temp())"] {
        let source =
            format!("macro_rules! id{{($e:expr)=>{{$e}}}}{PREFIX}fn f(){{let x={initializer};}}");
        let (report, _, scopes, body) = analyze(&source)?;
        let ast = &report.package.unit(body.file).configured.ast;
        let mut extended = 0;
        for (index, node) in ast.nodes.iter().enumerate() {
            let id = NodeId::new(u32::try_from(index).map_err(|e| e.to_string())?);
            if matches!(node.kind, Syntax::Call { .. })
                && scopes.node(id).is_some_and(|n| n.extended)
            {
                assert_eq!(lifetime_kind(&scopes, id)?, Kind::Block);
                extended += 1;
            }
        }
        assert_eq!(extended, 1, "{source}");
    }
    let source = "macro_rules! id{($e:expr)=>{$e}}fn f(){let x=1u8;let r=&id!(x);}";
    let (report, types, _, _) = analyze(source)?;
    let program = body::build(&report, &types).map_err(|e| e.to_string())?;
    let body = program.bodies.first().ok_or("missing body")?;
    assert!(body.temporaries.is_empty());
    assert_eq!(
        body.values
            .iter()
            .filter(|v| matches!(
                v.kind,
                Rvalue::Borrow {
                    place: Place::Local(_),
                    ..
                }
            ))
            .count(),
        1
    );
    Ok(())
}

const PREFIX: &str = "struct W<'a>(&'a u8);struct S<'a>{a:&'a u8}fn temp()->u8{1}fn id(x:&u8)->&u8{x}fn sink(_: &u8){}";

#[test]
fn extension_boundaries_match_pinned_rustc_acceptance() -> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("&temp()", "x", Kind::Block, true),
        ("(&temp(),)", "x.0", Kind::Block, true),
        ("[&temp()]", "x[0]", Kind::Block, true),
        ("[&temp();1]", "x[0]", Kind::Statement, false),
        ("id(&temp())", "x", Kind::Statement, false),
        ("{&temp()}", "x", Kind::Block, true),
        ("{id(&temp())}", "x", Kind::Tail, false),
        ("if true{&temp()}else{&0}", "x", Kind::Block, true),
        (
            "match true{true=>&temp(),false=>&0}",
            "x",
            Kind::Block,
            true,
        ),
        ("match &temp(){p=>p}", "x", Kind::Statement, false),
        ("W(&temp())", "x.0", Kind::Block, true),
        ("(W)(&temp())", "x.0", Kind::Block, true),
        ("ctor(&temp())", "x.0", Kind::Statement, false),
        ("{W}(&temp())", "x.0", Kind::Statement, false),
        ("S{a:&temp()}", "x.a", Kind::Block, true),
        ("S{..S{a:&temp()}}", "x.a", Kind::Statement, false),
    ];
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/temporary-scope-contracts-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory)?;
    for (index, (initializer, use_site, expected, accepted)) in cases.into_iter().enumerate() {
        let source = format!("{PREFIX}fn f(){{let ctor=W;let x={initializer};sink({use_site});}}");
        let (report, _, scopes, body) = analyze(&source)?;
        let node = expression(&report, &scopes, body, &source, "temp()")?;
        assert_eq!(lifetime_kind(&scopes, node)?, expected, "{source}");
        let path = directory.join(format!("case-{index}.rs"));
        std::fs::write(&path, &source)?;
        let rustc = std::process::Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-type=lib",
                "--crate-name=probe",
                "--emit=metadata",
                "--cap-lints=allow",
                "-o",
            ])
            .arg(directory.join("probe.rmeta"))
            .arg(path)
            .output()?;
        std::fs::write(
            directory.join(format!("case-{index}.stderr")),
            &rustc.stderr,
        )?;
        assert_eq!(
            rustc.status.code(),
            Some(i32::from(!accepted)),
            "{source}: {}",
            String::from_utf8_lossy(&rustc.stderr)
        );
        if !accepted {
            assert!(String::from_utf8_lossy(&rustc.stderr).contains("E0716"));
        }
    }
    Ok(())
}

const DROP_CASES: &[(&str, &str, &str, Kind, &str)] = &[
    (
        "{let x=&temp(1);mark(2);}",
        "()",
        "temp(1)",
        Kind::Block,
        "m2\nd1\n",
    ),
    (
        "{see(&temp(1));mark(2);}",
        "()",
        "temp(1)",
        Kind::Statement,
        "d1\nm2\n",
    ),
    (
        "{let local=temp(1);see(&temp(2))}",
        "bool",
        "temp(2)",
        Kind::Tail,
        "d2\nd1\n",
    ),
    (
        "{if see(&temp(1)){mark(2);}mark(3);}",
        "()",
        "temp(1)",
        Kind::Condition,
        "d1\nm2\nm3\n",
    ),
    (
        "{if let D(0)=temp(1){mark(2);}else{mark(3);}mark(4);}",
        "()",
        "temp(1)",
        Kind::IfThen,
        "d1\nm3\nm4\n",
    ),
    (
        "{if let D(0)=temp(0){mark(2);}else{mark(3);}mark(4);}",
        "()",
        "temp(0)",
        Kind::IfThen,
        "m2\nd0\nm4\n",
    ),
    (
        "{match temp(1){D(_)=>mark(2)};mark(3);}",
        "()",
        "temp(1)",
        Kind::Statement,
        "m2\nd1\nm3\n",
    ),
    (
        "{match temp(1){D(_) if see(&temp(2))=>mark(3),_=>mark(4)};mark(5);}",
        "()",
        "temp(2)",
        Kind::Condition,
        "d2\nm3\nd1\nm5\n",
    ),
    (
        "{let _=see(&temp(1))&&{mark(2);see(&temp(3))};mark(4);}",
        "()",
        "temp(1)",
        Kind::Condition,
        "d1\nm2\nd3\nm4\n",
    ),
    (
        "{let x=if true{&temp(1)}else{&temp(2)};mark(3);}",
        "()",
        "temp(1)",
        Kind::Block,
        "m3\nd1\n",
    ),
    (
        "{let x=&{temp(1)};mark(2);}",
        "()",
        "{temp(1)}",
        Kind::Block,
        "m2\nd1\n",
    ),
    (
        "{let x={&temp(1)};mark(2);}",
        "()",
        "temp(1)",
        Kind::Block,
        "m2\nd1\n",
    ),
    (
        "{while let D(1)=temp(1){mark(2);break;}mark(3);}",
        "()",
        "temp(1)",
        Kind::Iteration,
        "m2\nd1\nm3\n",
    ),
];

#[test]
fn pinned_rustc_drop_order_confirms_scope_endpoints() -> Result<(), Box<dyn std::error::Error>> {
    // The shape is checked by RustGM; Drop instrumentation is executed only by rustc.
    // This is independent scope evidence, not a claim that RustGM lowers Drop yet.

    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/temporary-drop-scopes-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory)?;
    let prefix = "struct D(u8);fn temp(n:u8)->D{D(n)}fn see(_: &D)->bool{true}";
    for (index, (body_source, result, tracked, expected_scope, expected)) in
        DROP_CASES.iter().copied().enumerate()
    {
        let function = format!("fn f()->{result}{body_source}");
        let source = format!("{prefix}fn mark(_:u8){{}}{function}");
        let (report, _, scopes, body) = analyze(&source)?;
        let node = expression(&report, &scopes, body, &source, tracked)?;
        assert_eq!(lifetime_kind(&scopes, node)?, expected_scope, "{source}");
        let instrumented = format!(
            "{prefix}impl Drop for D{{fn drop(&mut self){{println!(\"d{{}}\",self.0);}}}}fn mark(n:u8){{println!(\"m{{n}}\");}}{function}fn main(){{let _=f();}}"
        );
        let path = directory.join(format!("case-{index}.rs"));
        let executable = directory.join(format!("case-{index}"));
        std::fs::write(&path, instrumented)?;
        let rustc = std::process::Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-name=probe",
                "--cap-lints=allow",
                "-Cpanic=abort",
                "-Coverflow-checks=yes",
                "-o",
            ])
            .arg(&executable)
            .arg(path)
            .output()?;
        std::fs::write(
            directory.join(format!("case-{index}.stderr")),
            &rustc.stderr,
        )?;
        assert_eq!(
            rustc.status.code(),
            Some(0),
            "{source}: {}",
            String::from_utf8_lossy(&rustc.stderr)
        );
        let output = std::process::Command::new(executable).output()?;
        std::fs::write(
            directory.join(format!("case-{index}.stdout")),
            &output.stdout,
        )?;
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout, expected.as_bytes(), "{source}");
    }
    Ok(())
}
