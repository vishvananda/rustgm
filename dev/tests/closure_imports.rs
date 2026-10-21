//! Lexical type inputs remain separate from the typed uses that authorize captures.
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
#[expect(
    dead_code,
    reason = "this lane checks resolution before semantic inference"
)]
mod semantic;
use rustgm::{
    resolve::model::{DefId, Resolution, Scope, ScopeId},
    sema::closures::imports::Imports,
};

fn names(source: &str) -> Result<Vec<Vec<String>>, String> {
    let report = semantic::resolve(source)?;
    let imports = Imports::collect(&report.resolution)?;
    let mut result = Vec::new();
    for (index, scope) in report.resolution.scopes.iter().enumerate() {
        if scope.label_barrier && !scope.barrier {
            let mut names: Vec<_> = imports
                .bindings(ScopeId(u32::try_from(index).map_err(|e| e.to_string())?))
                .iter()
                .map(|id| {
                    report
                        .package
                        .names
                        .get(report.resolution.definitions[id.0 as usize].name)
                        .to_owned()
                })
                .collect();
            names.sort_unstable();
            result.push(names);
        }
    }
    result.sort_unstable();
    Ok(result)
}

#[test]
fn nested_boundaries_share_only_resolved_free_bindings() -> Result<(), Box<dyn std::error::Error>> {
    for (name, source, expected) in [
        (
            "nested",
            "fn main(){let x=7;let f=||{let g=||x;g()};let _=f();}",
            vec![vec!["x"], vec!["x"]],
        ),
        (
            "parameter",
            "fn run(x:u8){let f=||x;let _=f();}fn main(){}",
            vec![vec!["x"]],
        ),
        (
            "shadow",
            "fn main(){let x=7;let f=|x:u8|x;let g=||{let x=9;x};let _=(x,f(1),g());}",
            vec![vec![], vec![]],
        ),
        (
            "inner",
            "fn main(){let x=7;let f=||{let y=8;let g=||x+y;g()};let _=f();}",
            vec![vec!["x"], vec!["x", "y"]],
        ),
        (
            "items",
            "const X:u8=7;fn add(x:u8)->u8{x+1}fn main(){let f=||add(X);let _=f();}",
            vec![vec![]],
        ),
        (
            "wildcard",
            "fn main(){let x=7;let f=||{let _=x;};f();}",
            vec![vec!["x"]],
        ),
        (
            "macro",
            "fn main(){let x=7;macro_rules! get{()=>{x}}let f=||get!();let _=f();}",
            vec![vec!["x"]],
        ),
    ] {
        rustc::qualify(&format!("closure-imports-{name}"), source, true)?;
        assert_eq!(names(source)?, expected, "{name}");
    }
    Ok(())
}

#[test]
fn same_spelling_and_repeated_mentions_keep_definition_identity() -> Result<(), String> {
    let report = semantic::resolve(
        "fn main(){let x=1;let f=||x+x+x;{let x=2;let g=||x+x;let _=g();}let _=f();}",
    )?;
    let imports = Imports::collect(&report.resolution)?;
    let mut targets = Vec::<DefId>::new();
    for (index, scope) in report.resolution.scopes.iter().enumerate() {
        if scope.label_barrier && !scope.barrier {
            let bindings =
                imports.bindings(ScopeId(u32::try_from(index).map_err(|e| e.to_string())?));
            assert_eq!(bindings.len(), 1);
            targets.push(bindings[0]);
        }
    }
    assert_eq!(targets.len(), 2);
    assert_ne!(targets[0], targets[1]);
    Ok(())
}

#[test]
fn unrelated_scopes_and_repeated_deep_mentions_have_bounded_work() -> Result<(), String> {
    for count in [8, 32, 128, 512] {
        let mut source = String::from("fn main(){let x=7;");
        for _ in 0..count {
            source.push_str("let f=||{let _=x;let _=x;let _=x;};");
        }
        source.push('}');
        let report = semantic::resolve(&source)?;
        let imports = Imports::collect(&report.resolution)?;
        assert!(
            imports.work()
                <= 6 * report.resolution.scopes.len() + 3 * report.resolution.references.len()
        );
    }
    for depth in [8, 16, 32] {
        let source = format!(
            "fn main(){{let x=7;let f={}{{{}}}{};}}",
            "||{".repeat(depth),
            "let _=x;".repeat(128),
            "}".repeat(depth)
        );
        let report = semantic::resolve(&source)?;
        let imports = Imports::collect(&report.resolution)?;
        assert!(
            imports.work()
                <= 6 * report.resolution.scopes.len()
                    + 3 * report.resolution.references.len()
                    + depth
        );
    }
    Ok(())
}

fn scope(parent: Option<ScopeId>, closure: bool) -> Scope {
    Scope {
        parent,
        module: rustgm::resolve::package::ModuleId(0),
        barrier: false,
        associated: false,
        trait_access: None,
        nominal_self: None,
        label_barrier: closure,
        bindings: std::collections::HashMap::default(),
        order: Vec::new(),
    }
}

#[test]
fn forward_parent_links_are_supported_and_malformed_links_fail() -> Result<(), &'static str> {
    let mut resolution = Resolution {
        scopes: vec![scope(Some(ScopeId(1)), true), scope(None, false)],
        ..Resolution::default()
    };
    let imports = Imports::collect(&resolution)?;
    assert!(imports.bindings(ScopeId(0)).is_empty());
    resolution.scopes[1].parent = Some(ScopeId(0));
    assert!(Imports::collect(&resolution).is_err());
    resolution.scopes[1].parent = Some(ScopeId(2));
    assert!(Imports::collect(&resolution).is_err());
    Ok(())
}

#[test]
fn deep_scope_storage_and_parent_resolution_are_iterative() -> Result<(), &'static str> {
    for count in [64_u32, 256, 1024, 16_384] {
        let resolution = Resolution {
            scopes: (0..count)
                .map(|index| {
                    scope(
                        (index + 1 < count).then_some(ScopeId(index + 1)),
                        index == 0,
                    )
                })
                .collect(),
            ..Resolution::default()
        };
        let imports = Imports::collect(&resolution)?;
        assert!(imports.work() <= 4 * count as usize);
        assert!(imports.bindings(ScopeId(0)).is_empty());
    }
    Ok(())
}
