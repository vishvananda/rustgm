//! Exact-name transport preserves bytes and never evaluates the Rust-name callback.
use rustgm::{
    lower::linkage::{Identity, select},
    resolve::model::DefId,
    support::{
        linkage::{Naming, SymbolNames},
        names::Names,
    },
    types::{GenericArgs, ItemId, Kind, Store, TypeId, callable::Convention, instance::Instance},
};
use std::cell::Cell;

#[test]
fn exact_symbols_preserve_prefixes_and_unicode_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let mut source_names = Names::default();
    let semantic = source_names.intern("r#e\u{301}")?;
    assert_eq!(source_names.get(semantic), "é");
    let mut symbols = SymbolNames::default();
    let spellings = ["r#name", "'name", "é", "e\u{301}", "name"];
    let mut identities = Vec::new();
    for spelling in spellings {
        let id = symbols.intern(spelling)?;
        assert_eq!(symbols.get(id), Some(spelling));
        assert_eq!(symbols.intern(spelling)?, id);
        identities.push(id);
    }
    assert_ne!(identities[0], identities[4]);
    assert_ne!(identities[2], identities[3]);
    assert_eq!(symbols.len(), spellings.len());
    assert!(!symbols.is_empty());
    assert_eq!(symbols.byte_len(), spellings.iter().map(|s| s.len()).sum());
    assert!(symbols.intern("").is_err());
    assert!(symbols.intern("nul\0name").is_err());
    assert_eq!(symbols.len(), spellings.len());
    // Metadata remaps bytes into its destination pool, never the source's ID.
    let mut imported = SymbolNames::default();
    let _ = imported.intern("different-first-entry")?;
    let mapped = imported.intern(symbols.get(identities[0]).ok_or("missing name")?)?;
    assert_ne!(mapped, identities[0]);
    assert_eq!(imported.get(mapped), Some("r#name"));
    Ok(())
}

#[test]
fn exact_selection_never_invokes_rust_naming() -> Result<(), Box<dyn std::error::Error>> {
    let mut names = SymbolNames::default();
    let id = names.intern("ffi.$0")?;
    let calls = Cell::new(0);
    let selected = select(
        Some(&names),
        Naming::Exact(id),
        Identity::Static(DefId(9)),
        |_| {
            calls.set(calls.get() + 1);
            Err("Rust naming must not run")
        },
    )?;
    assert_eq!(selected, "ffi.$0");
    assert_eq!(calls.get(), 0);
    assert!(
        select(None, Naming::Exact(id), Identity::Static(DefId(9)), |_| {
            calls.set(calls.get() + 1);
            Err("Rust naming must not run")
        })
        .is_err()
    );
    assert_eq!(calls.get(), 0);
    Ok(())
}

#[test]
fn exact_backend_limits_preserve_the_original_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let mut names = SymbolNames::default();
    let calls = Cell::new(0);
    for spelling in ["r#name", "'name", "é", "e\u{301}", "a b", "a-b"] {
        let id = names.intern(spelling)?;
        let result = select(
            Some(&names),
            Naming::Exact(id),
            Identity::Static(DefId(9)),
            |_| {
                calls.set(calls.get() + 1);
                Err("Rust naming must not run")
            },
        );
        assert_eq!(
            result,
            Err("exact symbol spelling is outside the pinned LowIR vocabulary")
        );
        assert_eq!(names.get(id), Some(spelling));
    }
    assert_eq!(calls.get(), 0);
    Ok(())
}

#[test]
fn rust_selection_receives_the_canonical_callable_once() -> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let parameters = types.intern_list(&[])?;
    let signature = types.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: Convention::RUST,
    })?;
    let item = types.intern(Kind::FunctionItem {
        item: ItemId(7),
        arguments: GenericArgs::EMPTY,
        signature,
    })?;
    let identity = Identity::Callable(Instance::new(&types, item)?);
    let calls = Cell::new(0);
    let selected = select(None, Naming::Rust, identity, |actual| {
        assert_eq!(actual, identity);
        calls.set(calls.get() + 1);
        Ok("rg.7".into())
    })?;
    assert_eq!(selected, "rg.7");
    assert_eq!(calls.get(), 1);
    assert_eq!(
        select(None, Naming::Rust, identity, |_| Err(
            "missing crate identity"
        )),
        Err("missing crate identity")
    );
    Ok(())
}
