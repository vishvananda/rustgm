//! Exercise the published destructor address independently of ordinary cleanup calls.
use rustgm::lowir::{
    builder::Builder,
    data::{GlobalId, Item, Target},
    program::{Operand, Program, SymbolId, Terminator},
    vocabulary::{Convert, Predicate, Type},
};

const SOURCE: &str = "trait A{}#[repr(transparent)]struct W<'a>(&'a mut u8);impl<'a>A for W<'a>{}impl<'a>Drop for W<'a>{fn drop(&mut self){*self.0+=2;}}fn main(){let mut v=7u8;{let w=W(&mut v);let p:&dyn A=&w;let _=p;}check(v==9);}";

fn table(program: &Program) -> Result<(GlobalId, SymbolId), Box<dyn std::error::Error>> {
    let (index, global) = program
        .globals
        .iter()
        .enumerate()
        .find(|(_, g)| g.name.starts_with("rg.vtable."))
        .ok_or("missing destructor table")?;
    assert!(global.readonly);
    assert_eq!((global.span.bytes, global.span.align), (24, 8));
    let items = global.items.ok_or("uninitialized destructor table")?;
    let entries = &program.data[items.start as usize..items.end as usize];
    let [
        Item::Address {
            target: Target::Function(drop),
            addend: 0,
        },
        Item::Bits {
            ty: Type::I64,
            bits: 8,
        },
        Item::Bits {
            ty: Type::I64,
            bits: 8,
        },
    ] = entries
    else {
        return Err("incorrect transparent destructor header".into());
    };
    let function = &program.symbols[drop.0 as usize];
    assert!(function.body.is_some());
    assert_eq!(program.parameters(function.signature), &[Type::Ptr]);
    assert_eq!(
        program.signatures[function.signature.0 as usize].result,
        Type::Void
    );
    Ok((GlobalId(u32::try_from(index)?), *drop))
}

fn through_table(source: &str) -> Result<Program, Box<dyn std::error::Error>> {
    let mut program = super::build(source)?;
    let (table, drop) = table(&program)?;
    let signature = program.symbols[drop.0 as usize].signature;
    let entry = program
        .symbols
        .iter_mut()
        .find(|s| s.entry)
        .ok_or("entry")?;
    // W is transparent over one reference. Construct its valid representation,
    // then call only the header's loaded destructor address.
    let mut body = Builder::default();
    let block = body.block()?;
    body.enter(block)?;
    let counter = body.slot(Type::U8)?;
    let initial = body.integer(Type::U8, 7)?;
    body.store(initial, Operand::Slot(counter))?;
    let pointer = body.address(Operand::Slot(counter))?;
    let object = body.slot(Type::Ptr)?;
    body.store(pointer, Operand::Slot(object))?;
    let object = body.address(Operand::Slot(object))?;
    let table = body.address(Operand::Global(table))?;
    let callee = body.load(Type::Ptr, table.operand)?;
    body.call(signature, Type::Void, callee.operand, &[object])?;
    let actual = body.load(Type::U8, Operand::Slot(counter))?;
    let expected = body.integer(Type::U8, 9)?;
    let wrong = body.compare(Predicate::Ne, actual, expected)?;
    let status = body.convert(Convert::Zext, Type::I64, wrong)?;
    body.terminate(Terminator::Return(Some(status)))?;
    entry.body = Some(body.finish()?);
    Ok(program)
}

#[test]
fn destructor_header_uses_the_proved_concrete_recipe() -> Result<(), Box<dyn std::error::Error>> {
    let source = format!("{}{SOURCE}", super::native::CHECK);
    super::rustc::qualify("object-destructor-header", &source, true)?;
    through_table(&source)?;
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn destructor_address_executes_through_the_table() -> Result<(), Box<dyn std::error::Error>> {
    super::native::check_cases_with(
        "pa24-pa25-work/native-object-destructor",
        &[("header-drop", SOURCE, false)],
        |source, _| {
            let program = through_table(source)?;
            let mut output = Vec::new();
            rustgm::lowir::writer::write(&program, &mut output)?;
            Ok(output)
        },
    )
}
