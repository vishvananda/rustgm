//! Scoped grammar reuses the retained matcher; semantic hygiene is a separate gate.
use rustgm::{
    expand::macros::{compile, input::Input, matcher::Compiled},
    support::{edition::Edition, names::Names},
    syntax::{
        expression::parse_crate,
        item::ItemKind,
        token_tree::{Forest, Span},
        transfer,
    },
};
type Result<T = ()> = std::result::Result<T, String>;

fn fixture(name: &str) -> Result<String> {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/declarative-macros")
            .join(format!("{name}.rs")),
    )
    .map_err(|error| error.to_string())
}

fn compiled(source: &str) -> Result<(Compiled, Names)> {
    let ast = parse_crate(source).map_err(|error| format!("{error:?}"))?;
    let item = ast
        .items
        .iter()
        .find_map(|item| match item.kind {
            ItemKind::DeclarativeMacro {
                parameters,
                capture,
            } => Some((parameters, capture, true)),
            ItemKind::MacroRules { capture } => Some((None, capture, false)),
            _ => None,
        })
        .ok_or("missing macro declaration")?;
    let mut names = Names::default();
    let forest = &ast.captures[item.1 as usize];
    let definition = if item.2 {
        compile::declarative_in_edition(
            source,
            item.0.map(|index| &ast.captures[index as usize]),
            forest,
            &mut names,
            Edition::Rust2024,
        )
    } else {
        compile::in_edition(source, forest, &mut names, Edition::Rust2024)
    }
    .map_err(|error| format!("{error:?}"))?;
    Ok((Compiled::new(definition), names))
}

#[test]
fn concise_and_ordered_scoped_rules_use_the_existing_fragment_matcher() -> Result {
    for (name, input, expected_rule) in [("concise-parameter", "7", 0), ("ordered-arms", "9", 1)] {
        let source = fixture(name)?;
        let (mut definition, mut names) = compiled(&source)?;
        let forest = Forest::parse(input).map_err(|error| format!("{error:?}"))?;
        let input_tokens =
            Input::from_forest(&forest, forest.roots()).map_err(|error| format!("{error:?}"))?;
        let matched = definition
            .invoke(&source, input, &input_tokens, &mut names)
            .map_err(|error| format!("{error:?}"))?;
        assert_eq!(matched.rule, expected_rule);
        assert_eq!(
            definition.definition.rules[expected_rule].variables.len(),
            1
        );
    }
    Ok(())
}

#[test]
fn scoped_and_legacy_separators_keep_distinct_grammars() -> Result {
    compiled(&fixture("legacy-semicolon-control")?)?;
    for (name, message) in [
        (
            "comma-required",
            "scoped macro rules require comma separation",
        ),
        (
            "legacy-comma-rejected",
            "macro rules require semicolon separation",
        ),
    ] {
        let Err(error) = compiled(&fixture(name)?) else {
            return Err("opposite macro separator was accepted".into());
        };
        assert!(error.contains(message), "{error}");
    }
    for source in ["macro m($x:expr);", "macro m[]", "macro m($x:expr)[$x]"] {
        assert!(parse_crate(source).is_err(), "{source}");
    }
    Ok(())
}

#[test]
fn transferred_scoped_macro_keeps_both_capture_owners_and_original_spans() -> Result {
    let source = fixture("concise-parameter")?;
    let ast = parse_crate(&source).map_err(|error| format!("{error:?}"))?;
    let prefix = "macro_rules! previous {()=>{}}";
    let mut target = parse_crate(prefix).map_err(|error| format!("{error:?}"))?;
    let captures = target.captures.len();
    let item_base = target.items.len();
    let shift = u32::try_from(prefix.len()).map_err(|error| error.to_string())?;
    transfer::append(
        &ast,
        &mut target,
        |span| {
            Ok(Span {
                start: span.start + shift,
                end: span.end + shift,
            })
        },
        |value| Ok(value.clone()),
    )
    .map_err(|error| format!("{error:?}"))?;
    for (index, item) in ast.items.iter().enumerate() {
        if let ItemKind::DeclarativeMacro {
            parameters: Some(parameters),
            capture,
        } = item.kind
        {
            assert_eq!(
                target.items[item_base + index].kind,
                ItemKind::DeclarativeMacro {
                    parameters: Some(
                        parameters + u32::try_from(captures).map_err(|error| error.to_string())?
                    ),
                    capture: capture + u32::try_from(captures).map_err(|error| error.to_string())?,
                }
            );
        }
    }
    for (old, new) in ast.captures.iter().zip(&target.captures[captures..]) {
        assert_eq!(old.nodes().len(), new.nodes().len());
        for (old, new) in old.nodes().iter().zip(new.nodes()) {
            assert_eq!(
                new.span,
                Span {
                    start: old.span.start + shift,
                    end: old.span.end + shift
                }
            );
        }
    }
    Ok(())
}

#[path = "declarative_macros/unavailable_items.rs"]
mod retained_items;

#[path = "declarative_macros/chosen_sources.rs"]
mod chosen_sources;

#[path = "declarative_macros/formatted_values.rs"]
mod formatted_values;

#[path = "declarative_macros/concatenated_values.rs"]
mod concatenated_values;

#[path = "declarative_macros/attributed_values.rs"]
mod attributed_values;
