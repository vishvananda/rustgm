use rustgm::{
    expand::{
        config::Config,
        configured::{Attributes, Effect},
        render,
    },
    syntax::{
        attribute,
        token_tree::{Forest, Tree, TreeKind},
    },
};

#[test]
fn hand_checked_preorder_dump() -> Result<(), String> {
    let mut output = Vec::new();
    render::token_trees("(x)", &Config::default(), &mut output).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        output,
        b"group 0 0 paren 0 3 2\ntoken 1 1 identifier 1 2 \"x\"\neof 3\n"
    );
    Ok(())
}

#[test]
fn deep_forests_build_traverse_and_drop_iteratively() -> Result<(), String> {
    let text = format!("{}x{}", "(".repeat(30_000), ")".repeat(30_000));
    let forest = Forest::parse(&text).map_err(|e| format!("{e:?}"))?;
    assert_eq!(forest.nodes().len(), 30_001);
    assert_eq!(forest.after(0), 30_001);
    assert!(size_of::<Tree>() <= 32);
    for index in 0..30_000 {
        assert_eq!(forest.after(index), 30_001);
        assert_eq!(
            forest.children(index).map(|range| range.start),
            Some(index + 1)
        );
    }
    drop(forest);
    Ok(())
}

#[test]
fn doc_desugaring_preserves_body_bytes() -> Result<(), String> {
    let text = "//! one\n/** two /*nested*/ */ fn f() {}";
    let report = render::analyze(text, &Config::default()).map_err(|e| format!("{e:?}"))?;
    assert_eq!(report.attributes.entries.len(), 2);
    let Some(Tree {
        kind: TreeKind::Doc { inner, body },
        ..
    }) = report.forest.tree(0)
    else {
        return Err("expected doc tree".into());
    };
    assert!(inner);
    assert_eq!(&text[body.start as usize..body.end as usize], " one");
    assert_eq!(
        report.attributes.entries[0]
            .attribute
            .name(&report.forest, text),
        Some("doc")
    );
    Ok(())
}

#[test]
fn arbitrary_macro_contents_are_not_eagerly_interpreted() -> Result<(), String> {
    let text = "m!(#[cfg(bogus())] \"x\"suffix 0b9);";
    let report = render::analyze(text, &Config::default()).map_err(|e| format!("{e:?}"))?;
    assert!(report.attributes.entries.is_empty());
    assert!(report.forest.nodes().len() > 5);
    Ok(())
}

#[test]
fn syntax_owner_can_demand_nested_attributes() -> Result<(), String> {
    let source = "{#[cfg(false)] fn f() { undefined(); }}";
    let forest = Forest::parse(source).map_err(|e| format!("{e:?}"))?;
    let children = forest.children(0).ok_or("missing block")?;
    let (attribute, next) = attribute::at(&forest, source, children.start)
        .map_err(|e| format!("{e:?}"))?
        .ok_or("missing attribute")?;
    let mut configured = Attributes::default();
    configured
        .push(&forest, source, attribute, &Config::default())
        .map_err(|e| format!("{e:?}"))?;
    assert!(!configured.enabled());
    assert_eq!(forest.name(source, next), Some("fn"));
    Ok(())
}

#[test]
fn cfg_attr_preserves_order_and_expansion_parentage() -> Result<(), String> {
    let source = "#[cfg_attr(unix, doc=\"x\",cfg_attr(true,cfg(false)),allow(unused))] fn f() {}";
    let report = render::analyze(source, &Config::default()).map_err(|e| format!("{e:?}"))?;
    let entries = &report.attributes.entries;
    assert_eq!(
        entries.iter().map(|entry| entry.effect).collect::<Vec<_>>(),
        [
            Effect::CfgAttr(true),
            Effect::Keep,
            Effect::CfgAttr(true),
            Effect::Cfg(false),
            Effect::Keep
        ]
    );
    assert_eq!(
        entries.iter().map(|entry| entry.parent).collect::<Vec<_>>(),
        [None, Some(0), Some(0), Some(2), Some(0)]
    );
    assert!(!report.attributes.enabled());
    Ok(())
}

#[test]
fn configuration_is_explicit_and_repeatable() -> Result<(), String> {
    let mut config = Config::default();
    for setting in ["flag", "feature=\"a\"", "feature=r#\"b\"#", "feature=\"a\""] {
        config.add(setting).map_err(|e| format!("{e:?}"))?;
    }
    assert!(config.contains("flag", None));
    assert!(config.contains("feature", Some("a")));
    assert!(config.contains("feature", Some("b")));
    assert!(!config.contains("feature", None));
    assert!(!config.contains("debug_assertions", None));
    assert!(!config.contains("test", None));
    config.add("test").map_err(|error| format!("{error:?}"))?;
    config.add("test").map_err(|error| format!("{error:?}"))?;
    assert!(config.contains("test", None));
    for invalid in [
        "target_os=\"windows\"",
        "unix",
        "foo()",
        "x=1",
        "x=\"a\"wat",
        "",
    ] {
        assert!(config.add(invalid).is_err());
    }
    Ok(())
}

#[test]
fn configuration_validates_operands_and_bounds_recursion() {
    for text in ["#[cfg(any(true,not()))]", "#[cfg(all(false,other()))]"] {
        assert!(render::analyze(text, &Config::default()).is_err());
    }
    let nested = format!(
        "#[cfg({}true{})]",
        "not(".repeat(10_000),
        ")".repeat(10_000)
    );
    assert!(render::analyze(&nested, &Config::default()).is_err());
    let attributes = format!(
        "#[{}doc=\"x\"{}]",
        "cfg_attr(true,".repeat(1000),
        ")".repeat(1000)
    );
    assert!(render::analyze(&attributes, &Config::default()).is_err());
}

#[test]
fn all_source_failures_are_atomic() {
    for source in ["x ([)]", "x (", "/// bare\rCR", "#[cfg(not())] fn f() {}"] {
        let mut output = Vec::new();
        assert!(render::token_trees(source, &Config::default(), &mut output).is_err());
        assert!(output.is_empty());
    }
}
