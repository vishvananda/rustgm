//! Source-owned asm retains typed effects while actual emission stays capability-checked.
use crate::{body, syntax::ast::Kind};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const DECL: &str = "#[rustc_builtin_macro(asm)]macro_rules!actual{($($t:tt)*)=>{9u8};}";
fn source(program: &str) -> Result<String> {
    let mut prefix = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    prefix.push_str("impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{} ");
    prefix.push_str(DECL);
    prefix.push_str(program);
    Ok(prefix)
}
fn expanded(program: &str) -> Result<crate::resolve::expansion::Report> {
    Ok(crate::test_source::source_provider(
        &source(program)?,
        "typed_asm",
    )?)
}
#[test]
fn actual_templates_keep_one_parser_and_source_role_without_hot_node_growth() -> Result {
    let report = expanded(
        "#[rustc_builtin_macro(concat)]macro_rules!concat{($($t:tt)*)=>{};}macro_rules!forward{($s:expr,$x:expr)=>{actual!($s,x=in(reg)$x,options(nostack,preserves_flags))};}pub fn f(x:usize){unsafe{forward!(concat!(\"/* {x}\",\" */\"),x);}}",
    )?;
    let unit = &report.package.units[0];
    let mut roots = 0;
    for (index, node) in unit.configured.ast.nodes.iter().enumerate() {
        if unit.configured.active[index] && matches!(node.kind, Kind::InlineAssembly(_)) {
            roots += 1;
            report.prepare_assembly(
                crate::resolve::unit::NodeKey {
                    file: crate::resolve::unit::FileId(0),
                    node: crate::syntax::ast::NodeId::new(u32::try_from(index)?),
                },
                1,
                0,
            )?;
        }
    }
    assert_eq!(roots, 1);
    let assembly_calls: Vec<_> = report
        .invocations
        .iter()
        .filter(|invocation| {
            matches!(
                report.definitions[invocation.definition].body,
                crate::resolve::expansion::Body::Builtin(crate::expand::builtin::Builtin::Asm)
            )
        })
        .collect();
    assert_eq!(assembly_calls.len(), 1);
    assert_eq!(assembly_calls[0].parser_calls, 1);
    let reducer = crate::test_source::fixture("pa41/tests/spec/asm-frontend/stream-ps.rs")?;
    let mut actual = source("")?;
    // Crate attributes remain at the actual crate root, before source declarations.
    actual = actual.replacen("#![feature(", "#![feature(decl_macro,staged_api,", 1);
    actual.insert_str(0, "#![unstable(feature=\"asm_fixture\",issue=\"none\")]\n");
    // The independently authored foundation's const traits need their own policy.
    assert_eq!(actual.matches("pub const trait").count(), 2);
    actual = actual.replace(
        "pub const trait",
        "#[rustc_const_unstable(feature=\"asm_fixture\",issue=\"none\")]pub const trait",
    );
    actual.push_str("#[rustc_builtin_macro(concat)]macro_rules!concat{($($t:tt)*)=>{};}");
    actual.push_str(&reducer.lines().skip(2).collect::<Vec<_>>().join("\n"));
    let full = crate::test_source::source_provider(&actual, "actual_stream_asm")?;
    assert!(full.package.units.iter().any(|unit| {
        unit.configured
            .ast
            .nodes
            .iter()
            .enumerate()
            .any(|(i, n)| unit.configured.active[i] && matches!(n.kind, Kind::InlineAssembly(_)))
    }));

    assert_eq!(size_of::<Kind>(), 28);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    for bad in [
        "actual!();",
        "actual!(\"{missing}\");",
        "actual!(\"\",options(nomem,readonly));",
        "actual!(\"\",options(pure,nomem));",
        "actual!(\"\",x=in(\"rax\")0usize);",
        "actual!(\"\",label{});",
    ] {
        assert!(
            expanded(&format!("fn f(){{unsafe{{{bad}}}}}")).is_err(),
            "{bad}"
        );
    }
    Ok(())
}
#[test]
fn scalar_inputs_and_outputs_reach_real_original_proof_then_backend_denial() -> Result {
    let mut report = expanded(
        "pub fn main(){let mut x:usize=7;unsafe{actual!(\"add {x},1\",x=inout(reg)x,options(pure,nomem,nostack));}let _=x;}",
    )?;
    let mut analysis = crate::pipeline::analyze_source(&report)?;
    let entry = crate::sema::entry::select(&mut report, &analysis.facts)?;
    let mut program = body::build_concrete(&report, &analysis.facts)?;
    let selected = program
        .bodies
        .iter_mut()
        .find(|b| matches!(b.owner,body::ir::Owner::Function{definition,..}if definition==entry))
        .ok_or("missing genuine main body")?;
    let data = selected
        .assembly
        .as_ref()
        .ok_or("missing retained asm payload")?;
    assert_eq!(data.instructions.len(), 1);
    assert_eq!(data.operands.len(), 1);
    assert!(data.operands[0].input.is_some());
    assert!(data.operands[0].output.is_some());
    assert!(
        selected
            .values
            .iter()
            .any(|v| matches!(v.kind, body::ir::Rvalue::AssemblyOutput { .. }))
    );
    super::validate(selected)?;
    let retained = selected.retained_bytes();
    selected.assembly.as_mut().ok_or("payload")?.templates[0]
        .text
        .reserve(32);
    assert!(selected.retained_bytes() > retained);
    let prepared = std::mem::take(&mut analysis.prepared);
    crate::lower::instances::inspect_prepared(&report, &mut analysis.facts, prepared, |_, _| ())?;
    let mut analysis = crate::pipeline::analyze_source(&report)?;
    let error = crate::lower::instances::build_prepared(
        &report,
        &mut analysis.facts,
        entry,
        analysis.prepared,
    )
    .err()
    .ok_or("unsupported backend silently emitted asm")?;
    assert!(
        error
            .to_string()
            .contains("unavailable locked backend capability"),
        "{error}"
    );
    operand_evaluation_order()?;
    output_order_and_divergence()?;
    Ok(())
}
fn operand_evaluation_order() -> Result {
    use body::ir::{Action, Rvalue, Terminator};
    let report = expanded(
        "fn first()->usize{7}fn index()->usize{0}fn ordered(){let mut xs=[0usize,0usize];unsafe{actual!(\"mov {b},{a}; add {c},1\",a=in(reg)first(),b=out(reg)xs[index()],c=inout(reg)xs[index()]);}}",
    )?;
    let mut analysis = crate::pipeline::analyze_source(&report)?;
    let program = body::build_concrete(&report, &analysis.facts)?;
    let ordered = program
        .bodies
        .iter()
        .find(|b| b.assembly.is_some())
        .ok_or("ordered body")?;
    let (actions, terminator) = straight_trace(ordered)?;
    assert!(matches!(terminator, Terminator::Return(_)));
    let effect = actions
        .iter()
        .position(|action| assembly_effect(ordered, *action))
        .ok_or("ordered assembly effect")?;
    assert!(actions[effect + 1..].iter().all(|action| {
        !matches!(*action, Action::Evaluate(id) if matches!(
            ordered.values[id.0 as usize].kind, Rvalue::Call { .. }
        ))
    }));
    let calls: Vec<_> = actions
        .iter()
        .filter_map(|action| {
            let Action::Evaluate(value) = *action else {
                return None;
            };
            let Rvalue::Call { callee, .. } = ordered.values[value.0 as usize].kind else {
                return None;
            };
            let Rvalue::Function(definition) = ordered.values[callee.0 as usize].kind else {
                return None;
            };
            Some(
                report
                    .package
                    .names
                    .get(report.resolution.definitions[definition.0 as usize].name),
            )
        })
        .collect();
    assert_eq!(
        calls,
        ["first", "index", "index"],
        "inout place must be evaluated exactly once, after preceding operands"
    );
    crate::lower::instances::inspect_prepared(
        &report,
        &mut analysis.facts,
        analysis.prepared,
        |_, _| (),
    )?;
    Ok(())
}
fn output_order_and_divergence() -> Result {
    use body::ir::{Action, Rvalue, Terminator};
    let report = expanded(
        "fn overlap(){let mut x:usize=0;unsafe{actual!(\"mov {a},1; mov {b},2\",a=out(reg)x,b=out(reg)x);}let _=x;}fn diverges()->!{unsafe{actual!(\"ud2\",options(noreturn));}}",
    )?;
    let mut analysis = crate::pipeline::analyze_source(&report)?;
    let program = body::build_concrete(&report, &analysis.facts)?;
    let mut overlap_seen = false;
    let mut divergence_seen = false;
    for body in program.bodies.iter().filter(|b| b.assembly.is_some()) {
        let data = body.assembly.as_ref().ok_or("assembly data")?;
        let (actions, terminator) = straight_trace(body)?;
        let effect = actions
            .iter()
            .position(|action| assembly_effect(body, *action))
            .ok_or("assembly effect must execute")?;
        if data.operands.len() == 2 {
            overlap_seen = true;
            assert_eq!(data.operands[0].output, data.operands[1].output);
            assert!(matches!(terminator, Terminator::Return(_)));
            let Action::Evaluate(effect_value) = actions[effect] else {
                return Err("assembly effect statement".into());
            };
            let stores: Vec<_> = actions
                .iter()
                .enumerate()
                .filter_map(|(position, action)| {
                    let Action::Store { place, value } = *action else {
                        return None;
                    };
                    let Rvalue::AssemblyOutput { effect, operand } =
                        body.values[value.0 as usize].kind
                    else {
                        return None;
                    };
                    Some((position, operand, place, effect))
                })
                .collect();
            assert!(stores.iter().all(|&(position, operand, place, value)| {
                position > effect
                    && data.operands[operand as usize].output == Some(place)
                    && value == effect_value
            }));
            assert_eq!(
                stores
                    .iter()
                    .map(|&(_, operand, _, _)| operand)
                    .collect::<Vec<_>>(),
                [0, 1],
                "rightmost overlapping output is stored last"
            );
        } else {
            divergence_seen = true;
            assert_ne!(
                data.instructions[0].options & crate::syntax::assembly::NORETURN,
                0
            );
            assert!(matches!(terminator, Terminator::Unreachable));
            let block = body
                .blocks
                .iter()
                .find(|block| {
                    body.statements(block)
                        .iter()
                        .any(|statement| assembly_effect(body, statement.action))
                })
                .ok_or("noreturn assembly block")?;
            assert!(matches!(block.terminator, Terminator::Unreachable));
        }
    }
    assert!(overlap_seen && divergence_seen);
    crate::lower::instances::inspect_prepared(
        &report,
        &mut analysis.facts,
        analysis.prepared,
        |_, _| (),
    )?;
    Ok(())
}
fn straight_trace(body: &body::ir::Body) -> Result<(Vec<body::ir::Action>, body::ir::Terminator)> {
    use body::ir::Terminator;
    let mut visited = vec![false; body.blocks.len()];
    let mut current = body::ir::BlockId(0);
    let mut actions = Vec::new();
    loop {
        let seen = visited.get_mut(current.0 as usize).ok_or("trace block")?;
        if std::mem::replace(seen, true) {
            return Err("unexpected loop in assembly observer".into());
        }
        let block = &body.blocks[current.0 as usize];
        actions.extend(
            body.statements(block)
                .iter()
                .map(|statement| statement.action),
        );
        match block.terminator {
            Terminator::Jump(next) | Terminator::FalseEdge { real: next, .. } => current = next,
            Terminator::Branch { .. } => {
                return Err("unexpected branch in assembly observer".into());
            }
            terminal => return Ok((actions, terminal)),
        }
    }
}
fn assembly_effect(body: &body::ir::Body, action: body::ir::Action) -> bool {
    matches!(action, body::ir::Action::Evaluate(id) if matches!(
        body.values[id.0 as usize].kind, body::ir::Rvalue::Assembly { .. }
    ))
}
#[test]
fn asm_place_unsafe_const_and_type_failures_do_not_become_unit_effects() -> Result {
    for (program, message) in [
        (
            "fn f(){let x:usize;unsafe{actual!(\"/* {0} */\",in(reg)x);}}",
            "initialized",
        ),
        ("fn f(){actual!(\"nop\");}", "unsafe context"),
        ("const unsafe fn f(){unsafe{actual!(\"nop\");}}", "constant"),
        (
            "fn f(){unsafe{actual!(\"/* {0} */\",out(reg)1usize);}}",
            "place",
        ),
        (
            "fn f(x:bool){unsafe{actual!(\"/* {0} */\",in(reg)x);}}",
            "assembly register operand",
        ),
        (
            "fn f(){let x:usize=0;unsafe{actual!(\"/* {0} */\",out(reg)x);}}",
            "immutable",
        ),
    ] {
        let report = expanded(program)?;
        let result = crate::pipeline::analyze_source(&report).and_then(|mut analysis| {
            crate::lower::instances::inspect_prepared(
                &report,
                &mut analysis.facts,
                analysis.prepared,
                |_, _| (),
            )
        });
        let error = result
            .err()
            .ok_or("invalid asm became a proved empty effect")?;
        assert!(
            error.to_string().contains(message),
            "expected {message}: {error}"
        );
    }
    let report = expanded("fn f(p:*mut u8){unsafe{actual!(\"/* {0} */\",in(xmm_reg)p);}}")?;
    crate::pipeline::analyze_source(&report)?;
    Ok(())
}
#[test]
fn asm_storage_and_width_growth_remain_cold_and_body_owned() -> Result {
    assert_eq!(size_of::<body::ir::Body>(), 456);
    assert_eq!(size_of::<body::ir::Rvalue>(), 20);
    assert_eq!(size_of::<body::ir::Value>(), 24);
    let report = crate::test_source::resolve("fn f(){}", true)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let facts = crate::sema::bodies::analyze(&report, facts)?;
    let plain = body::build_concrete(&report, &facts)?;
    assert!(plain.bodies.iter().all(|b| b.assembly.is_none()));
    for width in [1usize, 8, 32] {
        let mut program = String::from("fn f(x:usize){unsafe{actual!(\"");
        for i in 0..width {
            use std::fmt::Write as _;
            write!(program, "{{a{i}}} ")?;
        }
        program.push('\"');
        for i in 0..width {
            use std::fmt::Write as _;
            write!(program, ",a{i}=in(reg)x")?;
        }
        program.push_str(",options(nomem,nostack));}}");
        let report = expanded(&program)?;
        let analysis = crate::pipeline::analyze_source(&report)?;
        let bodies = body::build_concrete(&report, &analysis.facts)?;
        let asm = bodies
            .bodies
            .iter()
            .find(|b| b.assembly.is_some())
            .ok_or("scaled body")?;
        let data = asm.assembly.as_ref().ok_or("data")?;
        assert_eq!(data.operands.len(), width);
        assert_eq!(data.pieces.len(), width);
        super::validate(asm)?;
        assert!(data.allocation_bytes() < 4096 + 1024 * width);
        eprintln!(
            "assembly width={width} payload={} body={} payload-owner={} instruction={} operand={}",
            data.allocation_bytes(),
            size_of_val(asm),
            size_of::<super::Payload>(),
            size_of::<super::Instruction>(),
            size_of::<super::Operand>()
        );
        let mut wrong = asm.clone();
        wrong.assembly.as_mut().ok_or("wrong data")?.instructions[0]
            .operands
            .end += 1;
        assert!(super::validate(&wrong).is_err());
        let mut foreign = asm.clone();
        foreign.assembly.as_mut().ok_or("foreign data")?.source = None;
        assert!(super::validate(&foreign).is_err());
    }
    let report = expanded("fn f(){unsafe{actual!(\"nop\");}}")?;
    let key = report.package.units[0]
        .configured
        .ast
        .nodes
        .iter()
        .enumerate()
        .find_map(|(index, node)| matches!(node.kind, Kind::InlineAssembly(_)).then_some(index))
        .ok_or("actual asm payer root")?;
    let key = crate::resolve::unit::NodeKey {
        file: crate::resolve::unit::FileId(0),
        node: crate::syntax::ast::NodeId::new(u32::try_from(key)?),
    };
    let before = report.assembly_work().ok_or("payer")?;
    assert!(report.prepare_assembly(key, 1, usize::MAX).is_err());
    let failed = report.assembly_work().ok_or("failed payer")?;
    assert!(failed.0 > before.0);
    assert!(failed.1 > before.1);
    assert!(report.prepare_assembly(key, 1, 0).is_err());
    Ok(())
}
