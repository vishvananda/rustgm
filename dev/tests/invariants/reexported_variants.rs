//! A re-exported variant travels as one value-namespace module binding that
//! names its constructor declaration; older profiles reject it by name.
use crate::{
    lower::instances::SourcePublication,
    module_packets::{Result, exported_profile, raw},
    support::crates::{
        self, Namespace,
        generics::{self as g, source as s},
    },
};
use std::fmt::Write as _;

const PROVIDER: &str = "pa29/tests/spec/variant-reexports/provider.rs";
const VARIANTS: [&str; 4] = ["None", "Some", "Absent", "Present"];

fn provider(source: &str, disambiguator: &str) -> Result<crates::Interface> {
    exported_profile(source, SourcePublication::ModuleBodies, disambiguator)
}

type Restage = fn(
    &crates::CrateIdentity,
    s::SourceRows,
    s::HeaderAttributeColumns,
    s::ModuleContractRows,
    s::AvailabilityRows,
    usize,
    &mut g::Work,
) -> std::result::Result<g::Payload, &'static str>;

fn restage(
    interface: &crates::Interface,
    (rows, modules): (s::SourceRows, s::ModuleContractRows),
    stage: Restage,
    work: &mut g::Work,
) -> Result<std::result::Result<g::Payload, &'static str>> {
    let descriptor = interface.source_descriptor().ok_or("actual descriptor")?;
    let (_, attributes, _) = raw(interface)?;
    let availability = descriptor.availability()?.clone();
    Ok(stage(
        interface.identity(),
        rows,
        attributes,
        modules,
        availability,
        0,
        work,
    ))
}

fn variant_rows(
    rows: &s::SourceRows,
    modules: &s::ModuleContractRows,
) -> Vec<(usize, s::BindingSource)> {
    modules
        .namespace
        .bindings
        .iter()
        .enumerate()
        .filter(|(_, row)| VARIANTS.contains(&row.name.as_str()))
        .map(|(index, row)| {
            let kind = rows.declarations[row.target.declaration.0 as usize]
                .definition
                .kind;
            assert_eq!(kind, s::DefinitionKind::Variant, "{}", row.name);
            assert_eq!(row.namespace, Namespace::Value, "{}", row.name);
            (index, row.source)
        })
        .collect()
}

#[test]
fn variant_reexports_travel_once_by_value_and_reject_older_profiles_and_forgeries() -> Result {
    let source = crate::test_source::fixture(PROVIDER)?;
    let interface = provider(&source, "variant_reexports")?;
    let bytes = crates::encode(&interface)?;
    let decoded = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&decoded)?, bytes);
    let (rows, _, modules) = raw(&decoded)?;
    let variants = variant_rows(&rows, &modules);
    // `v1` imports None and Some; each edition prelude globs them again.
    let imports = variants
        .iter()
        .filter(|(_, source)| *source == s::BindingSource::Import)
        .count();
    assert_eq!(imports, 4);
    assert!(
        variants
            .iter()
            .any(|(_, source)| *source == s::BindingSource::Glob)
    );
    let mut work = g::Work::default();
    restage(
        &decoded,
        (rows.clone(), modules.clone()),
        g::Payload::from_source_module_bodies,
        &mut work,
    )??;
    let (first, _) = variants[0];
    parent_relabeled(&decoded, &rows, &modules, first)?;
    for (case, expected) in [
        (0, "module binding changed its target namespace"),
        (
            1,
            "direct module binding changed its actual declaration/owner/access",
        ),
        (2, "module binding changed its target namespace"),
    ] {
        let mut changed = modules.clone();
        match case {
            0 => changed.namespace.bindings[first].namespace = Namespace::Type,
            1 => changed.namespace.bindings[first].source = s::BindingSource::Declaration,
            _ => {
                // A type-namespace alias beside the constructor is never transported.
                let mut alias = changed.namespace.bindings[first].clone();
                alias.namespace = Namespace::Type;
                changed.namespace.bindings.insert(first, alias);
            }
        }
        let staged = restage(
            &decoded,
            (rows.clone(), changed),
            g::Payload::from_source_module_bodies,
            &mut g::Work::default(),
        )?;
        assert_eq!(staged.err(), Some(expected), "variant forgery {case}");
    }
    assert_eq!(
        restage(
            &decoded,
            (rows, modules),
            g::Payload::from_source_module_branches,
            &mut g::Work::default(),
        )?
        .err(),
        // Enums themselves are version-13 rows, so their gate fires first.
        Some("source enum headers require the version-thirteen body IR profile")
    );
    // Variant declarations are version-13 rows; that gate fires before the
    // re-export's own profile check.
    let older = exported_profile(&source, SourcePublication::ModuleBranches, "variant_v12");
    assert_eq!(
        older.err().map(|error| error.to_string()).as_deref(),
        Some("input: source declaration kind is outside the initial source profile")
    );
    strict_prefixes(&bytes)?;
    shadowed_constructor(&source);
    scaling(&source)
}

/// The consumer re-derives a constructor binding from its installed parent.
/// A variant whose enum is relabeled as a struct is rejected by name: today the
/// enum header check fires first, before the module binding's parent dispatch
/// (whose struct branch waits for transported struct constructors).
fn parent_relabeled(
    interface: &crates::Interface,
    rows: &s::SourceRows,
    modules: &s::ModuleContractRows,
    binding: usize,
) -> Result {
    let target = modules.namespace.bindings[binding].target.declaration;
    let parent = rows.declarations[target.0 as usize]
        .definition
        .parent
        .ok_or("installed variant parent")?;
    assert_eq!(
        rows.declarations[parent.0 as usize].definition.kind,
        s::DefinitionKind::Enum
    );
    let mut changed = rows.clone();
    changed.declarations[parent.0 as usize].definition.kind = s::DefinitionKind::Struct;
    let staged = restage(
        interface,
        (changed, modules.clone()),
        g::Payload::from_source_module_bodies,
        &mut g::Work::default(),
    )?;
    assert_eq!(
        staged.err(),
        Some("source enum nominal header carries record fields or representation")
    );
    Ok(())
}

/// Every strict prefix that ends inside a renamed variant row is rejected.
fn strict_prefixes(bytes: &[u8]) -> Result {
    let at = bytes
        .windows(b"Present".len())
        .position(|window| window == b"Present")
        .ok_or("renamed variant row")?;
    for end in at.saturating_sub(16)..at + b"Present".len() + 16 {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    Ok(())
}

/// A glob can leave a variant's type-namespace alias whose constructor a local
/// function shadows. That alias cannot travel without its value binding.
fn shadowed_constructor(source: &str) {
    let shadowed = format!(
        "{source}\npub mod shadowed {{\n    pub use crate::option::Option::*;\n    \
         #[allow(non_snake_case)]\n    pub fn Some() {{}}\n}}\n"
    );
    let rejected = provider(&shadowed, "variant_shadowed");
    assert_eq!(
        rejected.err().map(|error| error.to_string()).as_deref(),
        Some("input: type-namespace variant re-export requires its value constructor binding")
    );
}

fn scaling(source: &str) -> Result {
    let mut baseline = None;
    for width in [6_usize, 24, 96] {
        let mut wide = format!("{source}\npub enum Wide {{");
        for index in 0..width {
            write!(&mut wide, " V{index},")?;
        }
        wide.push_str(" }\npub mod wide {\n    pub use crate::Wide::{");
        for index in 0..width {
            write!(&mut wide, "V{index}, ")?;
        }
        wide.push_str("};\n}\n");
        let interface = provider(&wide, "variant_scaling")?;
        let (rows, _, modules) = raw(&interface)?;
        let reexported = modules
            .namespace
            .bindings
            .iter()
            .filter(|row| row.name.starts_with('V') && row.name[1..].parse::<usize>().is_ok())
            .inspect(|row| {
                assert_eq!(row.namespace, Namespace::Value);
                assert_eq!(
                    rows.declarations[row.target.declaration.0 as usize]
                        .definition
                        .kind,
                    s::DefinitionKind::Variant
                );
            })
            .count();
        assert_eq!(reexported, width);
        let mut work = g::Work::default();
        restage(
            &interface,
            (rows, modules),
            g::Payload::from_source_module_bodies,
            &mut work,
        )??;
        if let Some(steps) = baseline {
            assert!(work.steps <= steps + (width - 6) * 512, "width {width}");
        } else {
            baseline = Some(work.steps);
        }
        println!(
            "variant re-export width={width} consumerSteps={}",
            work.steps
        );
    }
    Ok(())
}
