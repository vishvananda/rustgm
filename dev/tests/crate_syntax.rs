use rustgm::{
    expand::{config::Config, crate_syntax},
    syntax::{
        ast::{Kind, Node, NodeId},
        expression::parse_crate,
        item::{Item, ItemKind},
    },
};

#[test]
fn closure_parameters_own_attributes_and_configuration() -> Result<(), String> {
    let source =
        "fn f(){let c=|#[cfg(false)] gone:Missing, #[allow(unused)] (x,y):(u8,u8)|x+y;c((1,2));}";
    let report = crate_syntax::analyze(source, &Config::default()).map_err(|e| format!("{e:?}"))?;
    let parameters = report
        .ast
        .nodes
        .iter()
        .find_map(|node| match node.kind {
            Kind::Closure { parameters, .. } => Some(report.ast.list(parameters)),
            _ => None,
        })
        .ok_or("missing closure")?;
    assert_eq!(parameters.len(), 2);
    for (index, &node) in parameters.iter().enumerate() {
        let Kind::Attributed { value, attributes } = report.ast.node(node).kind else {
            return Err("closure parameter lost its attribute owner".into());
        };
        assert_eq!(attributes.end - attributes.start, 1);
        assert!(matches!(
            report.ast.node(value).kind,
            Kind::Parameter { .. }
        ));
        assert_eq!(report.active[node.0 as usize], index == 1);
        assert_eq!(report.active[value.0 as usize], index == 1);
    }
    assert_eq!(report.owners.len(), 2);
    assert_eq!(report.owners[0].node, parameters[0]);
    assert_eq!(report.owners[1].node, parameters[1]);
    assert!(report.ast.lookahead_high_water <= 3);
    Ok(())
}

#[test]
fn empty_crate_has_a_hand_checked_dump() -> Result<(), String> {
    let mut output = Vec::new();
    crate_syntax::render("", &Config::default(), &mut output).map_err(|e| format!("{e:?}"))?;
    assert_eq!(output, b"active [0]\nnode 0 0 0 crate []\nroot 0\neof 0\n");
    Ok(())
}

#[test]
fn generic_delimiters_split_without_relexing() -> Result<(), String> {
    let text = "struct A<const N:i32>; type X=Vec<Vec<A<-3>>>; fn f(){let _:Vec<u8>=Vec::new();}";
    let ast = parse_crate(text).map_err(|e| format!("{e:?}"))?;
    assert!(ast.lookahead_high_water <= 3);
    assert_eq!(
        ast.nodes
            .iter()
            .filter(|n| matches!(n.kind, Kind::Applied { .. }))
            .count(),
        4
    );
    assert!(ast.captures.is_empty());
    assert!(ast.literals.is_empty());
    Ok(())
}

#[test]
fn configuration_elides_subtrees_before_literal_or_nested_cfg_work() -> Result<(), String> {
    let text = "#[cfg(false)] mod hidden { #[cfg(bad())] fn f(){1invalid;} } fn g(){3;}";
    let report = crate_syntax::analyze(text, &Config::default()).map_err(|e| format!("{e:?}"))?;
    assert_eq!(report.owners.len(), 1);
    assert!(!report.owners[0].enabled);
    assert_eq!(report.ast.literals.len(), 1);
    let mut inactive_literal = false;
    for (index, node) in report.ast.nodes.iter().enumerate() {
        if matches!(node.kind, Kind::DeferredLiteral(_)) {
            assert!(!report.active[index]);
            inactive_literal = true;
        }
    }
    assert!(inactive_literal);
    assert!(crate_syntax::analyze("#[cfg(true)] fn f(){1invalid;}", &Config::default()).is_err());
    Ok(())
}

#[test]
fn configuration_arguments_change_owners_without_reparsing() -> Result<(), String> {
    let source = "#[cfg(feature=\"x\")] fn f(){} fn g(){}";
    let absent = crate_syntax::analyze(source, &Config::default()).map_err(|e| format!("{e:?}"))?;
    let mut config = Config::default();
    config.add("feature=\"x\"").map_err(|e| format!("{e:?}"))?;
    let present = crate_syntax::analyze(source, &config).map_err(|e| format!("{e:?}"))?;
    assert_eq!(absent.ast.nodes, present.ast.nodes);
    assert!(!absent.owners[0].enabled);
    assert!(present.owners[0].enabled);
    assert!(
        present.active.iter().filter(|x| **x).count()
            > absent.active.iter().filter(|x| **x).count()
    );
    Ok(())
}

#[test]
fn owner_grammar_controls_associated_and_external_definitions() {
    for text in [
        "trait T{struct S;}",
        "impl i32{fn f();}",
        "unsafe extern \"C\"{fn f(){}}",
        "type T;",
        "fn f();",
    ] {
        assert!(parse_crate(text).is_err(), "{text}");
    }
    assert!(parse_crate("trait T{fn f();} unsafe extern \"C\"{fn g();}").is_ok());
}

#[test]
fn configured_effects_keep_global_parent_and_capture_identities() -> Result<(), String> {
    let text = "#[cfg_attr(true,allow(unused))] fn f(){} #[cfg_attr(true,allow(unused))] fn g(){}";
    let report = crate_syntax::analyze(text, &Config::default()).map_err(|e| format!("{e:?}"))?;
    assert_eq!(report.effects.len(), 4);
    assert_eq!(report.owners[0].attributes, 0..2);
    assert_eq!(report.owners[1].attributes, 2..4);
    assert_eq!(report.effects[1].attribute.parent, Some(0));
    assert_eq!(report.effects[3].attribute.parent, Some(2));
    assert_eq!(report.effects[1].capture, 0);
    assert_eq!(report.effects[3].capture, 1);
    Ok(())
}

#[test]
fn items_share_body_nodes_and_source_spans() -> Result<(), String> {
    let text = "pub fn f(x:i32)->i32{let y=x+1;y}";
    let ast = parse_crate(text).map_err(|e| format!("{e:?}"))?;
    assert_eq!(ast.items.len(), 1);
    let ItemKind::Function {
        parameters, body, ..
    } = ast.items[0].kind
    else {
        return Err("missing function".into());
    };
    assert_eq!(ast.list(parameters).len(), 1);
    let body = body.ok_or("missing body")?;
    let span = ast.node(body).span;
    assert_eq!(
        &text[span.start as usize..span.end as usize],
        "{let y=x+1;y}"
    );
    assert!(matches!(ast.node(body).kind, Kind::Block { .. }));
    assert!(size_of::<Node>() <= 36);
    assert!(size_of::<Item>() <= 96);
    Ok(())
}

#[test]
fn every_child_precedes_its_owner_in_typed_arenas() -> Result<(), String> {
    let sources = [
        "pub struct S<T>{pub x:T} impl<T> S<T>{fn f(self)->T{self.x}}",
        "trait A{type T; fn f(&self)->Self::T;} impl A for u8{type T=u8;fn f(&self)->u8{*self}}",
        "fn f<const N:usize>(){let a:[u8;N]=[0;N]; match a{[x,..]=>x,_=>0};}",
        "#[cfg(false)] mod m{#![allow(unused)] pub use core::{fmt::*,mem};}",
        "type F=for<'a> unsafe extern \"C\" fn(&'a u8)->u8;",
        "struct S{x:u8} fn f(s:S){let S{x:y}=s; if let 1..=3=y {} }",
    ];
    let mut children = Vec::new();
    for source in sources {
        let ast = parse_crate(source).map_err(|e| format!("{e:?}"))?;
        assert!(ast.lookahead_high_water <= 3);
        for index in 0..ast.nodes.len() {
            let id = NodeId(u32::try_from(index).map_err(|e| e.to_string())?);
            children.clear();
            ast.push_children(id, &mut children);
            assert!(children.iter().all(|child| child.0 < id.0));
        }
    }
    Ok(())
}

#[test]
fn recursive_item_type_use_and_pattern_inputs_are_bounded() {
    for source in [
        format!("{}{}", "mod m{".repeat(2000), "}".repeat(2000)),
        format!("type X={}u8{};", "Vec<".repeat(2000), ">".repeat(2000)),
        format!("use {}x{};", "a::{".repeat(2000), "}".repeat(2000)),
        format!(
            "fn f(){{let {}x{}=y;}}",
            "S{x:".repeat(2000),
            "}".repeat(2000)
        ),
    ] {
        assert!(parse_crate(&source).is_err());
    }
}

#[test]
fn large_item_lists_do_not_use_recursive_ownership() -> Result<(), String> {
    let source = "fn f(){}\n".repeat(10_000);
    let report =
        crate_syntax::analyze(&source, &Config::default()).map_err(|e| format!("{e:?}"))?;
    assert_eq!(report.ast.items.len(), 10_000);
    assert_eq!(report.ast.nodes.len(), 20_001);
    assert!(report.ast.captures.is_empty());
    drop(report);
    Ok(())
}

#[test]
fn crate_rendering_is_atomic_for_configuration_failures() {
    let mut output = Vec::new();
    assert!(
        crate_syntax::render(
            "fn a(){} #[cfg(bad())] fn b(){}",
            &Config::default(),
            &mut output
        )
        .is_err()
    );
    assert!(output.is_empty());
}
