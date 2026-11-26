//! Version thirteen's assembly rows re-validate from the provider's own
//! exported rows before any proof. Every forged row is rejected by name, the
//! columns cost affine work in their size, asm-free bodies allocate none, and
//! the IR and row widths stay pinned.
use crate::{
    lower::instances::SourcePublication,
    module_packets::{Result, exported_profile, raw},
    support::{
        constness::Constness,
        crates::{
            self,
            generics::{self as g, source as s},
        },
        initial_assembly as profile,
    },
};
use std::fmt::Write as _;

const PROVIDER: &str = "pa41/tests/spec/asm-transport/provider.rs";
const MARKER: &str = "// Authored asm transport section.\n";
const MALFORMED: &str = "source assembly placeholder is malformed";
const COUNTS: [usize; 3] = [8, 16, 32];

// The site replaces the instruction's node key and the payload's origin
// widens to a body origin; value rows keep their widths.
const _: [(); 24] = [(); size_of::<crate::body::assembly::Instruction>()];
const _: [(); 112] = [(); size_of::<crate::body::assembly::Payload>()];
const _: [(); 20] = [(); size_of::<s::ValueKind>()];
const _: [(); 24] = [(); size_of::<s::Value>()];
// 400 bytes at 28e2b19fb, plus one cold column pointer.
const _: [(); 408] = [(); size_of::<s::Original>()];

fn stage(
    interface: &crates::Interface,
    rows: s::SourceRows,
    work: &mut g::Work,
) -> Result<std::result::Result<g::Payload, &'static str>> {
    let descriptor = interface.source_descriptor().ok_or("actual descriptor")?;
    let (_, attributes, modules) = raw(interface)?;
    Ok(g::Payload::from_source_module_bodies(
        interface.identity(),
        rows,
        attributes,
        modules,
        descriptor.availability()?.clone(),
        0,
        work,
    ))
}
fn decoded(source: &str, tag: &str) -> Result<(crates::Interface, usize)> {
    let interface = exported_profile(source, SourcePublication::ModuleBodies, tag)?;
    let bytes = crates::encode(&interface)?;
    let decoded = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&decoded)?, bytes);
    Ok((decoded, bytes.len()))
}
/// The one original whose first template starts with `prefix`.
fn original(rows: &s::SourceRows, prefix: &str) -> Result<usize> {
    let found: Vec<usize> = (0..rows.originals.len())
        .filter(|&index| {
            rows.originals[index]
                .assembly
                .as_deref()
                .and_then(|columns| columns.templates.first())
                .is_some_and(|template| template.text.starts_with(prefix))
        })
        .collect();
    let [index] = *found.as_slice() else {
        return Err(format!("{} originals start with {prefix:?}", found.len()).into());
    };
    Ok(index)
}
/// The one assembly value of an original, as `(value, argument)`.
fn invocation(original: &s::Original) -> Result<(usize, usize)> {
    let found: Vec<(usize, usize)> = original
        .values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| match value.kind {
            s::ValueKind::Assembly { arguments, .. } => Some((index, arguments.start as usize)),
            _ => None,
        })
        .collect();
    let [only] = *found.as_slice() else {
        return Err("one assembly value".into());
    };
    Ok(only)
}
fn columns(original: &mut s::Original) -> &mut s::Assembly {
    original.assembly.get_or_insert_default()
}
/// `tagged`'s one template rewritten to `text`, its placeholder at 3..6.
fn commented(rows: &mut s::SourceRows, text: &str) -> Option<()> {
    let tagged = original(rows, "# ").ok()?;
    let columns = columns(&mut rows.originals[tagged]);
    let first = columns.templates[0].pieces.start as usize;
    columns.templates[0].text = text.into();
    let piece = &mut columns.pieces[first];
    (piece.start, piece.end) = (3, 6);
    Some(())
}
type Forgery = fn(&mut s::SourceRows, usize, (usize, usize)) -> Option<()>;
/// Each forgery of `store` (or `tagged`, for rows 10, 23 and 24) and its
/// rejection.
const FORGED: [(&str, Forgery, &str); 15] = [
    (
        "4 xmm class on a pointer",
        |rows, store, _| {
            columns(&mut rows.originals[store]).operands[0].class = s::AssemblyClass::Vector;
            Some(())
        },
        profile::XMM,
    ),
    (
        "8 late input",
        |rows, store, _| {
            columns(&mut rows.originals[store]).operands[0].late = true;
            Some(())
        },
        "source assembly input cannot be late",
    ),
    (
        "11 piece outside its text",
        |rows, store, _| {
            let columns = columns(&mut rows.originals[store]);
            let length = u32::try_from(columns.templates[0].text.len()).ok()?;
            columns.pieces[0].end = length + 1;
            Some(())
        },
        MALFORMED,
    ),
    (
        "12 out-of-order pieces",
        |rows, store, _| {
            let columns = columns(&mut rows.originals[store]);
            columns.pieces.push(columns.pieces[0]);
            columns.templates[0].pieces.end += 1;
            Some(())
        },
        MALFORMED,
    ),
    (
        "13 piece names a missing operand",
        |rows, store, _| {
            columns(&mut rows.originals[store]).pieces[0].operand = 1;
            Some(())
        },
        MALFORMED,
    ),
    (
        "15 two values name one instruction",
        |rows, store, (value, _)| {
            let original = &mut rows.originals[store];
            let copy = original.values[value];
            if original.values.len() == value + 1 {
                original.values.push(copy);
            } else {
                let last = original.values.len() - 1;
                original.values[last].kind = copy.kind;
            }
            Some(())
        },
        "source assembly ordinal leaves its value order",
    ),
    (
        "16 column longer than its values",
        |rows, store, _| {
            let columns = columns(&mut rows.originals[store]);
            let mut extra = columns.instructions[0];
            extra.templates = g::Range { start: 1, end: 1 };
            extra.operands = g::Range { start: 1, end: 1 };
            columns.instructions.push(extra);
            Some(())
        },
        "source assembly column differs from its assembly values",
    ),
    (
        "17 inputs differ from arguments",
        |rows, store, (value, _)| {
            columns(&mut rows.originals[store]).operands[0].input =
                Some(u32::try_from(value).ok()?);
            Some(())
        },
        "assembly input order differs from retained operands",
    ),
    (
        "18 operand type differs from its input",
        |rows, store, _| {
            let original = &mut rows.originals[store];
            let result = original.result;
            columns(original).operands[0].ty = result;
            Some(())
        },
        "assembly input type differs from retained operand",
    ),
    (
        "19 argument and input name the asm itself",
        |rows, store, (value, argument)| {
            let original = &mut rows.originals[store];
            let value = u32::try_from(value).ok()?;
            original.arguments[argument] = value;
            columns(original).operands[0].input = Some(value);
            Some(())
        },
        "source value recipe is cyclic, forward or unknown",
    ),
    (
        "20 constant context on both sides",
        |rows, store, _| {
            let declaration = rows.originals[store].declaration;
            rows.originals[store].context = s::CallContext::ConstDefinition;
            rows.functions
                .iter_mut()
                .find(|header| header.declaration == declaration)?
                .constness = Constness::Const;
            Some(())
        },
        profile::CONSTANT,
    ),
    (
        "21 template ranges leave their column",
        |rows, store, _| {
            columns(&mut rows.originals[store]).instructions[0].templates =
                g::Range { start: 0, end: 0 };
            Some(())
        },
        "source assembly template ranges do not partition their column",
    ),
    (
        "10 unsafe-function context in a safe function",
        |rows, _, _| {
            let tagged = original(rows, "# ").ok()?;
            columns(&mut rows.originals[tagged]).instructions[0].safety =
                s::CallSafety::UnsafeFunction;
            Some(())
        },
        "source inline assembly has no matching unsafe context",
    ),
    (
        "23 block comment in tagged's template",
        |rows, _, _| commented(rows, "/* {p} */"),
        profile::COMMENT,
    ),
    (
        "24 line comment in tagged's template",
        |rows, _, _| commented(rows, "// {p}"),
        profile::COMMENT,
    ),
];
#[test]
fn provider_assembly_rows_restage_and_forged_rows_reject_by_name() -> Result {
    let source = crate::test_source::fixture(PROVIDER)?;
    let (interface, _) = decoded(&source, "asm_transport")?;
    let (rows, _, _) = raw(&interface)?;
    stage(&interface, rows.clone(), &mut g::Work::default())??;
    let store = original(&rows, "mov ")?;
    let bump = original(&rows, "add ")?;
    let tagged = original(&rows, "# ")?;
    assert_eq!(
        columns(&mut rows.originals[bump].clone()).instructions[0].safety,
        s::CallSafety::UnsafeFunction
    );
    assert_eq!(
        rows.originals[bump]
            .assembly
            .as_deref()
            .map(|columns| columns.templates.len()),
        Some(2)
    );
    assert_ne!(tagged, store);
    // Asm-free bodies allocate no assembly columns.
    for original in &rows.originals {
        let assembly = original
            .values
            .iter()
            .any(|value| matches!(value.kind, s::ValueKind::Assembly { .. }));
        assert_eq!(original.assembly.is_some(), assembly);
    }
    let site = invocation(&rows.originals[store])?;
    for (label, forge, expected) in FORGED {
        let mut forgery = rows.clone();
        forge(&mut forgery, store, site).ok_or(label)?;
        let staged = stage(&interface, forgery, &mut g::Work::default())?;
        assert_eq!(staged.err(), Some(expected), "row {label}");
    }
    Ok(())
}
/// The frozen foundation and one generic body with `count` zero-padded
/// pointer inputs, each used by its own `#` comment template (`/* */` until
/// the comment fence).
fn family(provider: &str, count: usize) -> Result<String> {
    let foundation = provider.split(MARKER).next().ok_or("foundation")?;
    let mut parameters = String::new();
    let mut templates = String::new();
    let mut operands = String::new();
    for index in 0..count {
        write!(parameters, "a{index:03}: *mut u32, ")?;
        write!(templates, "\"# {{a{index:03}}}\", ")?;
        write!(operands, "a{index:03} = in(reg) a{index:03}, ")?;
    }
    Ok(format!(
        "{foundation}// Generated assembly scaling body.\npub fn wide<T>({parameters}) {{ unsafe {{ crate::arch::asm!({templates}{operands}options(nostack)) }} }}\n"
    ))
}
/// The packet's bytes and the restage's charged work.
fn measured(provider: &str, count: usize) -> Result<[usize; 2]> {
    let (interface, bytes) = decoded(&family(provider, count)?, "asm_scaling")?;
    let (rows, _, _) = raw(&interface)?;
    let index = original(&rows, "# {a000}")?;
    let columns = rows.originals[index].assembly.as_deref().ok_or("columns")?;
    assert_eq!(
        (columns.templates.len(), columns.operands.len()),
        (count, count)
    );
    let mut work = g::Work::default();
    stage(&interface, rows, &mut work)??;
    Ok([bytes, work.steps])
}
#[test]
fn assembly_columns_cost_affine_bytes_and_validation_work() -> Result {
    let provider = crate::test_source::fixture(PROVIDER)?;
    let mut observed = Vec::with_capacity(COUNTS.len());
    for count in COUNTS {
        observed.push(measured(&provider, count)?);
    }
    eprintln!("assembly operands {COUNTS:?}: {observed:?}");
    let [small, middle, large] = <[[usize; 2]; 3]>::try_from(observed).map_err(|_| "counts")?;
    let first = middle[0] - small[0];
    assert!(first > 0, "packet bytes did not grow: {small:?} {middle:?}");
    assert_eq!(
        large[0] - middle[0],
        2 * first,
        "packet bytes: {small:?} {middle:?} {large:?}"
    );
    // Cache and allocation thresholds need not replay identical debits. Keep
    // the original doubling factor as an upper bound on total validation work.
    for (before, after) in [(small[1], middle[1]), (middle[1], large[1])] {
        assert!(
            after <= 2 * before,
            "validation work: {small:?} {middle:?} {large:?}"
        );
    }
    Ok(())
}

#[test]
fn integer_outputs_keep_exact_effect_place_type_and_version_boundaries() -> Result {
    let foundation = crate::test_source::fixture(PROVIDER)?;
    let (foundation, _) = foundation.split_once(MARKER).ok_or("assembly foundation")?;
    let source = format!(
        "{foundation} impl Clone for u64{{fn clone(&self)->Self{{*self}}}}impl Copy for u64{{}} pub fn integer<T>(mut value:u64)->u64{{unsafe{{crate::arch::asm!(\"add {{value}}, 1\",value=inout(reg)value,options(nomem,nostack));}}value}}"
    );
    let interface = exported_profile(
        &source,
        SourcePublication::ModuleBodyOperations,
        "integer_assembly",
    )?;
    let bytes = crates::encode(&interface)?;
    let decoded = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&decoded)?, bytes);
    let (rows, _, _) = raw(&decoded)?;
    let index = original(&rows, "add ")?;
    let body = &rows.originals[index];
    let output = body
        .values
        .iter()
        .position(|value| matches!(value.kind, s::ValueKind::AssemblyOutput { .. }))
        .ok_or("actual output node")?;
    assert!(
        body.assembly.as_deref().ok_or("actual assembly")?.operands[0]
            .output
            .is_some()
    );
    for forge in 0..5 {
        let mut forged = rows.clone();
        let body = &mut forged.originals[index];
        match forge {
            0 => columns(body).operands[0].output = None,
            1 => columns(body).operands[0].output = Some(s::Place::Local(u32::MAX)),
            2 => {
                body.values[output].kind = s::ValueKind::AssemblyOutput {
                    effect: 0,
                    operand: u32::MAX,
                }
            }
            3 => body.values[output].ty = g::TypeRef(u32::MAX),
            _ => body.values[output].kind = s::ValueKind::Unit,
        }
        let (_, attributes, modules) = raw(&decoded)?;
        let descriptor = decoded.source_descriptor().ok_or("actual descriptor")?;
        assert!(
            g::Payload::from_source_module_body_operations(
                decoded.identity(),
                forged,
                attributes,
                modules,
                descriptor.availability()?.clone(),
                0,
                &mut g::Work::default()
            )
            .is_err(),
            "forgery {forge}"
        );
    }
    let old = exported_profile(&source, SourcePublication::ModuleBodies, "integer_old");
    assert!(
        old.err()
            .ok_or("v13 must refuse")?
            .to_string()
            .contains("nomem option")
    );
    Ok(())
}
