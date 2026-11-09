//! Version thirteen's record variants: variant shape byte 2 and a dense name
//! column that travels exactly when some variant row is a record variant.
//! Each consumer re-derives a variant's namespace from its shape. Forged
//! names, shapes, name columns, constructor applications and binding
//! namespaces are rejected by name, and every strict prefix of the packet
//! fails to decode.
use crate::{
    lower::instances::SourcePublication,
    module_packets::{Result, exported_profile, raw},
    support::crates::{
        self, Namespace,
        generics::{self as g, source as s},
        owner_catalog::DeclarationRef,
    },
};

const PROVIDER: &str = "pa29/tests/spec/record-variants/provider.rs";
const REEXPORTS: &str =
    "// Generated re-export control.\npub use Shape::Pair;\npub use Shape::Rect;\n";
const INVALID: &str = "invalid canonical metadata path component";
const NAMESPACE: &str = "module binding changed its target namespace";
type Staged = std::result::Result<g::Payload, &'static str>;

fn provider(source: &str) -> Result<crates::Interface> {
    exported_profile(source, SourcePublication::ModuleBodies, "record_variants")
}
fn stage(
    interface: &crates::Interface,
    (rows, modules): (s::SourceRows, s::ModuleContractRows),
) -> Result<Staged> {
    let descriptor = interface.source_descriptor().ok_or("actual descriptor")?;
    let (_, attributes, _) = raw(interface)?;
    Ok(g::Payload::from_source_module_bodies(
        interface.identity(),
        rows,
        attributes,
        modules,
        descriptor.availability()?.clone(),
        0,
        &mut g::Work::default(),
    ))
}
/// The declaration row of the variant named `name`.
fn declaration(rows: &s::SourceRows, name: &str) -> Result<DeclarationRef> {
    let index = rows
        .declarations
        .iter()
        .position(|row| {
            row.definition.kind == s::DefinitionKind::Variant && row.name.as_deref() == Some(name)
        })
        .ok_or("actual variant declaration")?;
    Ok(DeclarationRef(u32::try_from(index)?))
}
/// The variant row of the variant named `name`.
fn variant(rows: &s::SourceRows, name: &str) -> Result<usize> {
    let declaration = declaration(rows, name)?;
    Ok(rows
        .enums()
        .variants
        .iter()
        .position(|row| row.declaration == declaration)
        .ok_or("actual variant row")?)
}
/// The name slots of variant `name`'s fields.
fn names(rows: &s::SourceRows, name: &str) -> Result<Vec<String>> {
    let row = rows.enums().variants[variant(rows, name)?];
    Ok(row.fields.slice(&rows.enums().names)?.to_vec())
}

#[test]
fn record_variants_travel_by_shape_and_names_and_reject_forgeries() -> Result {
    let source = crate::test_source::fixture(PROVIDER)?;
    let interface = provider(&source)?;
    let bytes = crates::encode(&interface)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    let decoded = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&decoded)?, bytes);
    let (rows, _, modules) = raw(&decoded)?;
    let shapes: Vec<_> = ["Unit", "Pair", "Rect", "Empty"]
        .into_iter()
        .map(|name| Ok(rows.enums().variants[variant(&rows, name)?].shape))
        .collect::<Result<_>>()?;
    assert_eq!(
        shapes,
        [
            s::VariantShape::Unit,
            s::VariantShape::Tuple,
            s::VariantShape::Record,
            s::VariantShape::Record
        ]
    );
    // One name slot per field row; positional slots are empty.
    assert_eq!(rows.enums().names.len(), rows.enums().fields.len());
    assert_eq!(names(&rows, "Rect")?, ["w", "h"]);
    assert_eq!(names(&rows, "InteriorNul")?, ["position"]);
    assert_eq!(names(&rows, "Pair")?, ["", ""]);
    stage(&decoded, (rows.clone(), modules.clone()))??;
    shape_bytes(&bytes, &rows)?;
    for (case, expected) in forgeries(&rows)? {
        let staged = stage(&decoded, (case, modules.clone()))?;
        assert_eq!(staged.err(), Some(expected));
    }
    namespaces(&source)?;
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    Ok(())
}

/// Rect's row is `ordinal | shape | discriminant`; any shape byte from 3 up is
/// unknown.
fn shape_bytes(bytes: &[u8], rows: &s::SourceRows) -> Result {
    let row = rows.enums().variants[variant(rows, "Rect")?];
    let mut needle = row.ordinal.to_le_bytes().to_vec();
    needle.push(2);
    needle.extend_from_slice(&row.discriminant.to_le_bytes());
    let found: Vec<_> = bytes
        .windows(needle.len())
        .enumerate()
        .filter(|(_, window)| *window == needle.as_slice())
        .map(|(at, _)| at + 4)
        .collect();
    let [at] = found.as_slice() else {
        return Err("one Rect variant row".into());
    };
    for value in [3, 4, u8::MAX] {
        let mut forged = bytes.to_vec();
        forged[*at] = value;
        assert_eq!(
            crates::decode(&forged).err(),
            Some("unknown source variant shape"),
            "{value}"
        );
    }
    Ok(())
}

fn forgeries(rows: &s::SourceRows) -> Result<Vec<(s::SourceRows, &'static str)>> {
    let rect = rows.enums().variants[variant(rows, "Rect")?];
    let pair = variant(rows, "Pair")?;
    let first = rect.fields.start as usize;
    let mut cases = Vec::new();
    for name in ["", "0", "a-b"] {
        let mut changed = rows.clone();
        name.clone_into(&mut changed.cold_mut().enums.names[first]);
        cases.push((changed, INVALID));
    }
    let mut changed = rows.clone();
    "w".clone_into(&mut changed.cold_mut().enums.names[first + 1]);
    cases.push((changed, "source record variant has a duplicate field name"));
    let mut changed = rows.clone();
    changed.cold_mut().enums.variants[pair].shape = s::VariantShape::Record;
    cases.push((changed, INVALID));
    let mut changed = rows.clone();
    changed.cold_mut().enums.variants[variant(rows, "Rect")?].shape = s::VariantShape::Tuple;
    cases.push((changed, "source positional variant field carries a name"));
    let mut changed = rows.clone();
    changed.cold_mut().enums.names.pop();
    cases.push((
        changed,
        "source record variant names have incomplete field coverage",
    ));
    // `pair`'s called constructor retargeted at the record variant.
    let (from, to) = (declaration(rows, "Pair")?, declaration(rows, "Rect")?);
    let mut changed = rows.clone();
    let application = changed
        .function_applications
        .iter_mut()
        .find(|row| row.declaration.declaration == from)
        .ok_or("pair's constructor application")?;
    application.declaration.declaration = to;
    cases.push((changed, "source record variant has no called constructor"));
    Ok(cases)
}

/// A re-exported record variant binds its type-namespace declaration and a
/// re-exported tuple variant its value constructor; either moved to the other
/// namespace is rejected by name.
fn namespaces(source: &str) -> Result {
    let interface = provider(&format!("{source}{REEXPORTS}"))?;
    let (rows, _, modules) = raw(&interface)?;
    let binding = |name: &str| {
        modules
            .namespace
            .bindings
            .iter()
            .position(|row| row.name == name && row.source != s::BindingSource::Declaration)
            .ok_or("re-exported variant binding")
    };
    let (pair, rect) = (binding("Pair")?, binding("Rect")?);
    assert_eq!(modules.namespace.bindings[pair].namespace, Namespace::Value);
    assert_eq!(modules.namespace.bindings[rect].namespace, Namespace::Type);
    for (index, name) in [(pair, "Pair"), (rect, "Rect")] {
        assert_eq!(
            modules.namespace.bindings[index].target.declaration,
            declaration(&rows, name)?
        );
    }
    stage(&interface, (rows.clone(), modules.clone()))??;
    for (index, namespace) in [(rect, Namespace::Value), (pair, Namespace::Type)] {
        let mut changed = modules.clone();
        changed.namespace.bindings[index].namespace = namespace;
        let staged = stage(&interface, (rows.clone(), changed))?;
        assert_eq!(staged.err(), Some(NAMESPACE), "{namespace:?}");
    }
    Ok(())
}
