//! Canonical declarations share one symbol and admit exactly one body.
use rustgm::lowir::{
    builder::Builder,
    program::{Function, Literal, Program, SignatureId, Symbol, SymbolId, Terminator},
    vocabulary::Type,
};
fn body(value: u128) -> Result<Function, &'static str> {
    let mut builder = Builder::default();
    let block = builder.block()?;
    builder.enter(block)?;
    let value = builder.integer(Type::I32, value)?;
    builder.terminate(Terminator::Return(Some(value)))?;
    builder.finish()
}
#[test]
fn aliases_share_the_first_declaration_and_body() -> Result<(), Box<dyn std::error::Error>> {
    for reverse in [false, true] {
        let mut program = Program::default();
        let signature = program.signature(&[Type::I32], Type::I32, false)?;
        let equal = program.signature(&[Type::I32], Type::I32, false)?;
        assert_ne!(signature, equal);
        let a = program.declare("ffi".into(), signature, false)?;
        let b = program.declare("ffi".into(), equal, false)?;
        assert_eq!(a, b);
        assert_eq!(program.symbols.len(), 1);
        assert_eq!(program.symbols[0].signature, signature);
        let (first, second) = if reverse { (b, a) } else { (a, b) };
        program.define(first, body(11)?)?;
        assert_eq!(
            program.define(second, body(22)?),
            Err("multiple function definitions for one LowIR symbol")
        );
        assert!(matches!(
            program.symbols[0]
                .body
                .as_ref()
                .and_then(|b| b.literals.first()),
            Some(Literal::Integer(11))
        ));
        assert_eq!(program.declare("ffi".into(), equal, false)?, a);
        let other = program.declare("other".into(), signature, false)?;
        assert_eq!(other, SymbolId(1));
        program.define(other, body(44)?)?;
    }
    Ok(())
}
#[test]
fn incompatible_declarations_leave_the_canonical_symbol_unchanged()
-> Result<(), Box<dyn std::error::Error>> {
    let mut program = Program::default();
    let signature = program.signature(&[Type::I32], Type::I32, false)?;
    let selected = program.declare("ffi".into(), signature, false)?;
    for (parameters, result, noreturn, entry) in [
        (vec![Type::I64], Type::I32, false, false),
        (vec![], Type::I32, false, false),
        (vec![Type::I32], Type::I64, false, false),
        (vec![Type::I32], Type::I32, true, false),
        (vec![Type::I32], Type::I32, false, true),
    ] {
        let incompatible = program.signature(&parameters, result, noreturn)?;
        assert_eq!(
            program.declare("ffi".into(), incompatible, entry),
            Err("incompatible LowIR symbol declarations")
        );
        assert_eq!(program.symbols.len(), 1);
        assert_eq!(program.symbols[0].signature, signature);
        assert!(!program.symbols[0].entry);
        assert!(program.symbols[0].body.is_none());
    }
    assert_eq!(
        program.declare("new".into(), SignatureId(u32::MAX), false),
        Err("invalid LowIR signature")
    );
    assert_eq!(program.declare("ffi".into(), signature, false)?, selected);
    assert_eq!(
        program.declare("new".into(), signature, false)?,
        SymbolId(1)
    );
    Ok(())
}
#[test]
fn invalid_definitions_do_not_change_bodies_or_canonical_names()
-> Result<(), Box<dyn std::error::Error>> {
    let mut program = Program::default();
    let signature = program.signature(&[], Type::I32, false)?;
    let selected = program.declare("ffi".into(), signature, false)?;
    assert_eq!(
        program.define(SymbolId(u32::MAX), body(1)?),
        Err("invalid LowIR definition symbol")
    );
    program.symbols.push(Symbol {
        name: "ffi".into(),
        signature,
        entry: false,
        body: None,
    });
    assert_eq!(
        program.define(SymbolId(1), body(2)?),
        Err("noncanonical LowIR definition symbol")
    );
    program.symbols[0].name = "changed".into();
    assert_eq!(
        program.define(selected, body(3)?),
        Err("noncanonical LowIR definition symbol")
    );
    assert!(program.symbols.iter().all(|symbol| symbol.body.is_none()));
    program.symbols[0].name = "ffi".into();
    assert_eq!(program.declare("ffi".into(), signature, false)?, selected);
    program.define(selected, body(4)?)?;
    Ok(())
}

#[test]
fn function_and_storage_names_share_one_kind_checked_namespace()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::lowir::program::MemorySpan;
    for global_first in [false, true] {
        let mut program = Program::default();
        let signature = program.signature(&[], Type::I32, false)?;
        let span = MemorySpan { bytes: 4, align: 4 };
        if global_first {
            let global = program.declare_global("shared".into(), span, true)?;
            assert_eq!(
                program.declare("shared".into(), signature, false),
                Err("LowIR symbol kind conflicts with prior declaration")
            );
            assert!(program.symbols.is_empty());
            assert_eq!(program.global("shared".into(), span, true)?, global);
            program.initialize(global, &[rustgm::lowir::data::Item::Zero(4)])?;
        } else {
            let function = program.declare("shared".into(), signature, false)?;
            assert_eq!(
                program.declare_global("shared".into(), span, true),
                Err("LowIR symbol kind conflicts with prior declaration")
            );
            assert!(program.globals.is_empty());
            program.define(function, body(7)?)?;
        }
    }
    Ok(())
}
#[test]
fn storage_aliases_validate_before_upgrading_or_installing_data()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::lowir::{
        data::{GlobalId, GlobalKind, Item},
        program::MemorySpan,
    };
    let mut program = Program::default();
    let span = MemorySpan { bytes: 8, align: 4 };
    let imported = program.declare_global("storage".into(), span, true)?;
    for (incompatible, readonly) in [
        (MemorySpan { bytes: 4, align: 4 }, true),
        (MemorySpan { bytes: 8, align: 8 }, true),
        (span, false),
    ] {
        assert_eq!(
            program.global("storage".into(), incompatible, readonly),
            Err("incompatible LowIR global declarations")
        );
        assert_eq!(program.globals[0].kind, GlobalKind::Declaration);
        assert!(program.globals[0].items.is_none());
    }
    assert_eq!(
        program.initialize(imported, &[Item::Zero(8)]),
        Err("external LowIR global cannot have a local initializer")
    );
    assert_eq!(
        program.initialize(GlobalId(u32::MAX), &[Item::Zero(8)]),
        Err("invalid LowIR global identity")
    );
    assert!(program.data.is_empty());
    let definition = program.global("storage".into(), span, true)?;
    assert_eq!(imported, definition);
    assert_eq!(
        program.initialize(definition, &[Item::Zero(9)]),
        Err("LowIR global initializer exceeds storage span")
    );
    assert!(program.data.is_empty());
    assert!(program.globals[0].items.is_none());
    program.initialize(definition, &[Item::Zero(8)])?;
    assert_eq!(
        program.initialize(imported, &[Item::Zero(4)]),
        Err("LowIR global initialized twice")
    );
    assert_eq!(program.data.len(), 1);
    assert_eq!(
        program.declare_global("storage".into(), span, true)?,
        definition
    );
    assert_eq!(program.globals[0].kind, GlobalKind::Definition);
    assert_eq!(program.globals.len(), 1);
    Ok(())
}
