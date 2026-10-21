//! Unused generic closures still owe independent ownership, loan and lifetime proofs.
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "identity",
        "fn owner<T>(){let _f=|value:T|value;}fn main(){}",
        true,
    ),
    (
        "copy",
        "fn owner<T:Copy>(){let _f=|value:T|(value,value);}fn main(){}",
        true,
    ),
    (
        "double-move",
        "fn owner<T>(){let _f=|value:T|(value,value);}fn main(){}",
        false,
    ),
    (
        "clone-is-not-copy",
        "fn owner<T:Clone>(){let _f=|value:T|(value,value);}fn main(){}",
        false,
    ),
    (
        "conditional-move",
        "fn owner<T>(){let _f=|value:T,yes:bool|{if yes{let _moved=value;}value};}fn main(){}",
        false,
    ),
    (
        "reinitialize",
        "fn owner<T>(){let _f=|mut value:T|{let moved=value;value=moved;value};}fn main(){}",
        true,
    ),
    (
        "borrow-conflict",
        "fn owner<T>(){let _f=||{let mut value=1u8;let a=&mut value;let b=&mut value;*a+*b};}fn main(){}",
        false,
    ),
    (
        "local-borrow",
        "fn owner<T>(){let _f=||{let mut value=1u8;let a=&mut value;*a+=1;value};}fn main(){}",
        true,
    ),
    (
        "borrowed-move",
        "fn owner<T>(){let _f=|value:&mut T|->T{*value};}fn main(){}",
        false,
    ),
    (
        "borrowed-copy",
        "fn owner<T:Copy>(){let _f=|value:&T|->T{*value};}fn main(){}",
        true,
    ),
    (
        "contextual-borrow",
        "fn owner<T>(){let _f:fn(&T)->&T=|value|value;}fn main(){}",
        true,
    ),
    (
        "dangling-local",
        "fn owner<T>(){let _f=||{let value=1u8;&value};}fn main(){}",
        false,
    ),
    (
        "const-parameter",
        "fn owner<const N:usize>(){let _f=||N;}fn main(){}",
        true,
    ),
    (
        "nested-valid",
        "fn owner<T:Copy>(){let _f=||{let _g=|value:T|(value,value);};}fn main(){}",
        true,
    ),
    (
        "nested-invalid",
        "fn owner<T>(){let _f=||{let _g=|value:T|(value,value);};}fn main(){}",
        false,
    ),
];

fn lower(source: &str) -> Result<rustgm::lowir::program::Program, String> {
    let (mut report, mut facts) =
        semantic::analyze(source).map_err(|e| format!("semantics: {e}"))?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry)
        .map_err(|e| format!("checked lowering: {e}"))
}

#[test]
fn unused_generic_closures_require_all_body_proofs() {
    for &(name, source, accepted) in CASES {
        let result = lower(source);
        assert_eq!(
            result.is_ok(),
            accepted,
            "{name}: {}",
            result.err().unwrap_or_default()
        );
    }
}

#[test]
fn independent_generic_closure_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("pa24-template-{name}"), source, accepted)?;
    }
    Ok(())
}

#[test]
fn templates_keep_separate_closure_owners_and_share_the_generic_environment()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::body::ir::Owner;
    use std::sync::Arc;

    let source =
        "fn owner<T:Copy>(){let _a=|value:T|(value,value);let _b=|value:T|value;}fn main(){}";
    let (report, facts) = semantic::analyze(source)?;
    let templates = rustgm::body::templates::build(&report, &facts)?;
    assert_eq!(templates.bodies.len(), 3);
    let parent = templates
        .bodies
        .iter()
        .find(|t| matches!(t.body.owner, Owner::Function { .. }))
        .ok_or("missing generic owner template")?;
    let mut identities = std::collections::HashSet::new();
    for template in &templates.bodies {
        if let Owner::Closure { identity, .. } = template.body.owner {
            assert!(identities.insert(identity));
            assert!(Arc::ptr_eq(&template.environment, &parent.environment));
            assert_ne!(template.body.key(), parent.body.key());
        }
    }
    assert_eq!(identities.len(), 2);
    let program = lower(source)?;
    assert!(
        program
            .symbols
            .iter()
            .all(|symbol| !symbol.name.starts_with("rg.closure."))
    );
    Ok(())
}

#[test]
fn formal_closure_signatures_bind_their_own_lifetimes() -> Result<(), String> {
    let source = "fn owner<T:Copy>(){let _f=|value:&T|->T{*value};}fn main(){let f:fn(&u8)->&u8=|value|value;let x=7;let _=f(&x);}";
    let (_, mut facts) = semantic::analyze(source)?;
    let signatures: Vec<_> = facts
        .closures
        .all()
        .iter()
        .map(|body| body.signature)
        .collect();
    assert_eq!(signatures.len(), 2);
    let mut closed =
        rustgm::types::substitute::Bound::restore_regions(std::collections::HashMap::new());
    for signature in signatures {
        assert_eq!(facts.types.needs_region_context(signature), Some(false));
        assert_eq!(closed.apply(&mut facts.types, signature)?, signature);
    }
    Ok(())
}
