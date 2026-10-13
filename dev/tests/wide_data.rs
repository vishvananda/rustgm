#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{
    body, borrow, lower,
    lowir::{
        data::{Item, Target},
        program::Program,
        vocabulary::Type,
    },
    resolve::model::DefId,
};

fn compile(source: &str) -> Result<Program, String> {
    let (report, mut facts) = semantic::analyze(source)?;
    let mut program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    body::constants::analyze(&report, &mut facts, &program).map_err(|e| e.to_string())?;
    body::promote::analyze(&report, &mut facts, &mut program).map_err(|e| e.to_string())?;
    let contracts = body::regions::collect(&facts)?;
    let regions = borrow::regions::check_bodies(
        &facts.types,
        &facts.aggregates,
        &facts.lifetimes.variances,
        &contracts,
        &program,
    )
    .map_err(|e| e.to_string())?;
    let checked = borrow::verify(
        &mut borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            copies: &mut facts.copies,
        },
        &regions,
        &program,
    )
    .map_err(|e| e.to_string())?;
    let entry = report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == "main")
        .ok_or("main")?;
    lower::build(
        &mut facts,
        &checked,
        DefId(u32::try_from(entry).map_err(|e| e.to_string())?),
    )
    .map_err(|e| e.message.to_owned())
}

#[test]
fn persistent_text_metadata_counts_bytes_and_points_to_exact_utf8()
-> Result<(), Box<dyn std::error::Error>> {
    for (literal, expected) in [
        (r#""héllo""#, &b"h\xc3\xa9llo"[..]),
        (r#""🦀""#, &b"\xf0\x9f\xa6\x80"[..]),
        (r#""e\u{301}""#, &b"e\xcc\x81"[..]),
        (r#""a\0b""#, &b"a\0b"[..]),
        (r#""""#, &b""[..]),
    ] {
        let program = compile(&format!("static S:&str={literal};fn main(){{let _=S;}}"))?;
        assert_eq!(program.globals.len(), 2, "{literal}");
        let value = program
            .globals
            .iter()
            .find(|g| g.span.bytes == 16)
            .ok_or("text value")?;
        assert_eq!(value.span.align, 8);
        let range = value.items.ok_or("text initializer")?;
        let [
            Item::Address {
                target: Target::Global(id),
                addend: 0,
            },
            metadata,
        ] = program.data[range.start as usize..range.end as usize]
        else {
            return Err("text value must store a data address followed by its byte length".into());
        };
        let length = match metadata {
            Item::Bits {
                ty: Type::I64,
                bits,
            } => bits,
            Item::Zero(8) => 0,
            _ => return Err("text length must occupy exactly eight bytes".into()),
        };
        assert_eq!(length, expected.len() as u128);
        let bytes = &program.globals[id.0 as usize];
        assert_eq!(
            (bytes.span.bytes, bytes.span.align),
            (expected.len().max(1) as u64, 1)
        );
        assert!(bytes.readonly);
        let range = bytes.items.ok_or("byte initializer")?;
        let mut actual = Vec::new();
        for item in &program.data[range.start as usize..range.end as usize] {
            match *item {
                Item::Bits { ty: Type::U8, bits } => actual.push(u8::try_from(bits)?),
                Item::Zero(n) => actual.resize(actual.len() + usize::try_from(n)?, 0),
                _ => return Err("invalid text byte storage".into()),
            }
        }
        assert_eq!(&actual[..expected.len()], expected);
        assert_eq!(actual.len(), expected.len().max(1));
    }
    Ok(())
}
