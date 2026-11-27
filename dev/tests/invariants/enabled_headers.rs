//! Version thirteen's function header bit 3 carries a function's declared
//! target features. The reader owns their canonical syntax; every consumer
//! re-proves a row on its own installed header and resolves the names against
//! its own table, and the import ledger pays one fixed charge per row.
use super::*;
use crate::{
    lower::instances::SourcePublication,
    module_packets, source_packets,
    support::crates::{self, generics as g, owner_catalog::DeclarationRef},
};
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
type Row<T> = std::result::Result<T, &'static str>;

const _: [(); 64] = [(); size_of::<s::HeaderContract>()];
const _: [(); 24] = [(); size_of::<s::DeclaredFeatures>()];

const BODIES: s::Profile = s::Profile::Source2024ModuleBodies;
const BRANCHES: s::Profile = s::Profile::Source2024ModuleBranches;
const SPEC: &str = "pa42/tests/spec/target-feature-transport";
const VERSION: &str = "source target-feature rows require the version-thirteen header profile";
const UNKNOWN: &str = "unknown source target feature";
const CANONICAL: &str = "non-canonical source target-feature set";
/// The provider's first 73 lines, the pinned foundation.
const FOUNDATION: usize = 73;

fn contract(inline: Option<s::InlineHint>, names: Option<&[&str]>) -> s::HeaderContract {
    s::HeaderContract {
        declaration: DeclarationRef(1),
        declaration_site: 0,
        must_use: None,
        inline: inline.map(|hint| s::InlineContract {
            owner_site: 0,
            hint,
            applicability: s::InlineApplicability::FunctionBody,
        }),
        advisories: s::AdvisoryFlags::default(),
        caller_tracking: false,
        format_panic: false,
        promotable: None,
        callable: names.map(|names| {
            Box::new(s::CallableAttributes {
                target_features: Some(declared(names)),
                ..s::CallableAttributes::default()
            })
        }),
        optimization: None,
    }
}
fn declared(names: &[&str]) -> s::DeclaredFeatures {
    s::DeclaredFeatures {
        names: names.iter().map(|&name| Box::from(name)).collect(),
        safe: false,
    }
}
fn written(row: &s::HeaderContract) -> Result<Vec<u8>> {
    let mut out = Writer::default();
    header_facts::row_write(&mut out, row)?;
    Ok(out.finish())
}
/// One row read under `profile`; a successful read consumes every byte.
fn read(bytes: &[u8], profile: s::Profile) -> Row<s::HeaderContract> {
    let mut reader = Reader::new(bytes);
    let row = header_facts::row_read(&mut Input {
        reader: &mut reader,
        bytes: 0,
        base: 0,
        profile,
    })?;
    if !reader.done() {
        return Err("header contract row left trailing bytes");
    }
    Ok(row)
}
fn name(text: &str) -> Vec<u8> {
    let mut bytes = u32::try_from(text.len())
        .unwrap_or(u32::MAX)
        .to_le_bytes()
        .to_vec();
    bytes.extend_from_slice(text.as_bytes());
    bytes
}

#[test]
fn optimization_intent_round_trips_only_in_the_v14_header_profile() -> Result {
    let ordinary = contract(None, None);
    let bytes = written(&ordinary)?;
    assert_eq!(read(&bytes, s::Profile::Source2024ModuleBodies)?, ordinary);
    for preference in [
        s::Optimization::Disabled,
        s::Optimization::Size,
        s::Optimization::Speed,
    ] {
        let mut marked = ordinary.clone();
        marked.optimization = Some(preference);
        let bytes = written(&marked)?;
        assert_eq!(
            read(&bytes, s::Profile::Source2024ModuleBodyOperations)?,
            marked
        );
        assert_eq!(
            read(&bytes, s::Profile::Source2024ModuleBodies),
            Err("unknown transported inline or callable advisory flags")
        );
    }
    Ok(())
}
#[test]
fn pointer_lint_and_must_use_flags_share_the_diagnostic_byte() -> Result {
    let mut row = contract(None, None);
    row.advisories = s::AdvisoryFlags::new(true, true);
    for reason in [None, Some("retain the returned pointer")] {
        row.must_use = reason.map(|reason| s::MustUseContract {
            owner_site: 0,
            reason: Some(reason.into()),
        });
        let bytes = written(&row)?;
        assert_eq!(bytes[8], 2 + u8::from(reason.is_some()));
        assert_eq!(
            read(&bytes, s::Profile::Source2024ModuleBodyOperations)?,
            row
        );
        assert_eq!(read(&bytes, BODIES), Err("invalid source boolean"));
        let mut unknown = bytes;
        unknown[8] |= 32;
        assert_eq!(
            read(&unknown, s::Profile::Source2024ModuleBodyOperations),
            Err("invalid source boolean")
        );
    }
    Ok(())
}

#[test]
fn final_trait_method_round_trips_only_in_the_extended_header_profile() -> Result {
    let mut row = contract(None, None);
    row.advisories = row.advisories.with_final_trait_method(true);
    let bytes = written(&row)?;
    assert_eq!(bytes[8], 4);
    assert_eq!(
        read(&bytes, s::Profile::Source2024ModuleBodyOperations)?,
        row
    );
    assert_eq!(read(&bytes, BODIES), Err("invalid source boolean"));
    Ok(())
}

#[test]
fn default_impl_method_round_trips_only_in_the_extended_header_profile() -> Result {
    let mut row = contract(None, None);
    row.advisories = row.advisories.with_default_impl_method(true);
    let bytes = written(&row)?;
    let profile = s::Profile::Source2024ModuleBodyOperations;
    assert_eq!(bytes[8], 8);
    assert_eq!(read(&bytes, profile)?, row);
    assert_eq!(read(&bytes, BODIES), Err("invalid source boolean"));
    let mut conflicting = bytes;
    conflicting[8] |= 4;
    assert_eq!(
        read(&conflicting, profile),
        Err("source method cannot be both final and default")
    );
    Ok(())
}

fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!("{SPEC}/{name}.rs"))?)
}
fn provider(source: &str) -> Result<crates::Interface> {
    module_packets::exported_profile(source, SourcePublication::ModuleBodies, "target_features")
}

#[test]
fn declared_feature_rows_write_bit_three_and_read_only_under_body_nodes() -> Result {
    let prefix = [1, 0, 0, 0, 0, 0, 0, 0, 0];
    // Without bit 3 a row keeps its exact earlier bytes.
    let plain = written(&contract(None, None))?;
    assert_eq!(plain, [&prefix[..], &[0]].concat());
    assert_eq!(read(&plain, BRANCHES)?, contract(None, None));
    let keep = contract(None, Some(&["sse", "sse2"]));
    let bytes = written(&keep)?;
    let expected = [&prefix[..], &[8, 2, 0, 0, 0], &name("sse"), &name("sse2")].concat();
    assert_eq!(bytes, expected);
    assert_eq!(read(&bytes, BODIES)?, keep);
    assert_eq!(read(&bytes, BRANCHES).err(), Some(VERSION));
    // `_mm_stream_ps`'s shape: the inline payload precedes the feature set.
    let stream = contract(Some(s::InlineHint::Hint), Some(&["sse"]));
    let bytes = written(&stream)?;
    let expected = [
        &prefix[..],
        &[9, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0],
        &name("sse"),
    ]
    .concat();
    assert_eq!(bytes, expected);
    assert_eq!(read(&bytes, BODIES)?, stream);
    let mut safe_keep = contract(None, Some(&["sse", "sse2"]));
    if let Some(features) = safe_keep
        .callable
        .as_deref_mut()
        .and_then(|attributes| attributes.target_features.as_mut())
    {
        features.safe = true;
    }
    let safe_bytes = written(&safe_keep)?;
    assert_eq!(
        read(&safe_bytes, s::Profile::Source2024ModuleBodyOperations)?,
        safe_keep
    );
    assert_eq!(
        read(&safe_bytes, BODIES).err(),
        Some("source safe target-feature function requires the version-fourteen header profile")
    );
    Ok(())
}

#[test]
fn forged_feature_rows_reject_by_name_in_the_reader() -> Result {
    let keep = written(&contract(None, Some(&["sse", "sse2"])))?;
    let patched = |offset: usize, value: &[u8]| {
        let mut changed = keep.clone();
        changed[offset..offset + value.len()].copy_from_slice(value);
        changed
    };
    assert_eq!(
        read(&patched(9, &[0x18]), BODIES).err(),
        Some("unknown transported inline or callable advisory flags")
    );
    // The target-feature gate precedes the cold and caller-tracking gate.
    assert_eq!(read(&patched(9, &[0x0b]), BRANCHES).err(), Some(VERSION));
    assert_eq!(
        read(&patched(10, &129_u32.to_le_bytes()), BODIES).err(),
        Some("source target-feature set exceeds transport profile limit")
    );
    let mut invalid = written(&contract(None, Some(&["sse"])))?;
    if let Some(last) = invalid.last_mut() {
        *last = 0xff;
    }
    assert_eq!(read(&invalid, BODIES).err(), Some(UNKNOWN));
    let long = "a".repeat(33);
    for (names, inline, diagnostic) in [
        (&[][..], None, "source target-feature set is empty"),
        (
            &[long.as_str()][..],
            None,
            "source target-feature name exceeds transport profile limit",
        ),
        (&[""][..], None, UNKNOWN),
        (&["sse2", "sse"][..], None, CANONICAL),
        (&["sse", "sse"][..], None, CANONICAL),
        (
            &["sse"][..],
            Some(s::InlineHint::Always),
            "source target-feature function cannot use an always-inline hint",
        ),
    ] {
        let bytes = written(&contract(inline, Some(names)))?;
        assert_eq!(read(&bytes, BODIES).err(), Some(diagnostic), "{names:?}");
    }
    // A 32-byte name is within the profile; the table owns its meaning.
    let widest = "a".repeat(32);
    let bytes = written(&contract(None, Some(&[widest.as_str()])))?;
    assert!(read(&bytes, BODIES).is_ok());
    Ok(())
}

/// The provider's function rows with declared names, by declaration name.
fn feature_rows(interface: &crates::Interface) -> Result<Vec<(String, Vec<String>)>> {
    let rows = interface
        .source_descriptor()
        .ok_or("actual module descriptor")?
        .rows();
    let attributes = rows
        .header_attributes()
        .ok_or("complete advisory columns")?;
    let mut found = Vec::new();
    for contract in &attributes.functions {
        let declaration = rows
            .declarations
            .get(contract.declaration.0 as usize)
            .ok_or("function declaration")?;
        if let Some(features) = contract.target_features() {
            found.push((
                declaration.name.clone().unwrap_or_default(),
                features.names.iter().map(ToString::to_string).collect(),
            ));
        } else if contract.inline.is_some()
            || contract.advisories.cold()
            || contract.caller_tracking
        {
            return Err(format!("unexpected advisory on {:?}", declaration.name).into());
        }
    }
    Ok(found)
}

#[test]
fn exported_rows_carry_declared_names_and_validation_owns_their_bodies() -> Result {
    let interface = provider(&fixture("provider")?)?;
    let strings = |names: &[&str]| names.iter().map(|&name| name.to_owned()).collect();
    assert_eq!(
        feature_rows(&interface)?,
        [
            ("second".to_owned(), strings(&["sse2"])),
            ("keep".to_owned(), strings(&["sse", "sse2"])),
            ("wide".to_owned(), strings(&["avx"])),
        ]
    );
    let (rows, attributes, _) = module_packets::raw(&interface)?;
    attributes.validate(&rows, 0, BODIES, &mut g::Work::default())?;
    let mut forged = attributes.clone();
    forged
        .traits
        .first_mut()
        .ok_or("a foundation trait row")?
        .callable
        .get_or_insert_with(Box::default)
        .target_features = Some(declared(&["sse"]));
    assert_eq!(
        forged
            .validate(&rows, 0, BODIES, &mut g::Work::default())
            .err(),
        Some("source trait header cannot carry a callable advisory")
    );
    // The foundation's required `Clone::clone` has no original body.
    let required = rows
        .functions
        .iter()
        .position(|header| header.body == s::FunctionBody::Absent)
        .ok_or("a required trait method row")?;
    let mut forged = attributes.clone();
    forged.functions[required]
        .callable
        .get_or_insert_with(Box::default)
        .target_features = Some(declared(&["sse"]));
    assert_eq!(
        forged
            .validate(&rows, 0, BODIES, &mut g::Work::default())
            .err(),
        Some("source target features require an exported original body")
    );
    assert_eq!(
        forged
            .validate(&rows, 0, BRANCHES, &mut g::Work::default())
            .err(),
        Some(VERSION)
    );
    Ok(())
}

/// The packet with function row `index` changed: the header attribute
/// columns are re-encoded, spliced in place of the genuine encoding, the
/// declarative macro envelope's inner length is rewritten and the bytes are
/// decoded again.
fn forged(
    interface: &crates::Interface,
    index: usize,
    change: impl FnOnce(&mut s::HeaderContract),
) -> Result<crates::Interface> {
    let bytes = crates::encode(interface)?;
    let attributes = interface
        .source_descriptor()
        .ok_or("actual module descriptor")?
        .rows()
        .header_attributes()
        .ok_or("complete advisory columns")?
        .clone();
    let mut out = Writer::default();
    header_facts::encode(&mut out, &attributes)?;
    let genuine = out.finish();
    let mut changed = attributes;
    change(changed.functions.get_mut(index).ok_or("forged row")?);
    let mut out = Writer::default();
    header_facts::encode(&mut out, &changed)?;
    let replacement = out.finish();
    let found: Vec<usize> = bytes
        .windows(genuine.len())
        .enumerate()
        .filter(|(_, window)| *window == genuine.as_slice())
        .map(|(offset, _)| offset)
        .collect();
    let [at] = *found.as_slice() else {
        return Err(format!("header attribute columns at {found:?}").into());
    };
    let mut packet = [
        &bytes[..at],
        replacement.as_slice(),
        &bytes[at + genuine.len()..],
    ]
    .concat();
    let magic = b"RGMETA\r\n";
    if let Some(inner) = packet
        .windows(magic.len())
        .skip(magic.len())
        .position(|window| window == magic)
        .map(|position| position + magic.len())
    {
        let length = u32::try_from(packet.len() - inner)?;
        packet[inner - 4..inner].copy_from_slice(&length.to_le_bytes());
    }
    Ok(crates::decode(&packet)?)
}
/// The function rows named `name`, in declaration order.
fn named(interface: &crates::Interface, name: &str) -> Result<Vec<usize>> {
    let rows = interface
        .source_descriptor()
        .ok_or("actual module descriptor")?
        .rows();
    Ok(rows
        .functions
        .iter()
        .enumerate()
        .filter(|(_, header)| {
            rows.declarations
                .get(header.declaration.0 as usize)
                .and_then(|declaration| declaration.name.as_deref())
                == Some(name)
        })
        .map(|(index, _)| index)
        .collect())
}
fn only(interface: &crates::Interface, name: &str) -> Result<usize> {
    match *named(interface, name)?.as_slice() {
        [index] => Ok(index),
        ref found => Err(format!("{name} rows {found:?}").into()),
    }
}
fn lowir(interface: crates::Interface, client: &str) -> Result<String> {
    let program = source_packets::client(interface, client)?;
    let mut out = Vec::new();
    crate::lowir::writer::write(&program, &mut out)?;
    Ok(String::from_utf8(out)?)
}

#[test]
fn forged_feature_rows_reject_on_the_consumers_installed_header() -> Result {
    let _check = crate::support::sealed::SelfCheck::hold();
    let genuine = provider(&fixture("provider")?)?;
    let call = fixture("call")?;
    let second = only(&genuine, "second")?;
    let clone = {
        let rows = genuine
            .source_descriptor()
            .ok_or("actual module descriptor")?
            .rows();
        *named(&genuine, "clone")?
            .iter()
            .rfind(|&&index| matches!(rows.functions[index].body, s::FunctionBody::Original(_)))
            .ok_or("the u32 Clone impl's clone")?
    };
    let cases = [
        (second, &["ssx"][..], UNKNOWN),
        (second, &["soft-float"][..], UNKNOWN),
        (
            clone,
            &["sse2"][..],
            "source trait member target features require their trait implementation transport",
        ),
        (
            only(&genuine, "zero")?,
            &["sse2"][..],
            "source target-feature function form requires its wider profile",
        ),
        (
            only(&genuine, "plain")?,
            &["sse2"][..],
            "source target-feature function is not declared unsafe",
        ),
    ];
    for (index, names, diagnostic) in cases {
        let interface = forged(&genuine, index, |row| {
            row.callable
                .get_or_insert_with(Box::default)
                .target_features = Some(declared(names));
        })?;
        let error = source_packets::client_analysis(interface, &call)
            .err()
            .ok_or("forged row was accepted")?
            .to_string();
        assert!(error.contains(diagnostic), "{names:?}: {error}");
    }
    // The bit-3 limit: neither forgery is distinguishable on this target.
    let bare = forged(&genuine, only(&genuine, "bare")?, |row| {
        row.callable
            .get_or_insert_with(Box::default)
            .target_features = Some(declared(&["avx"]));
    })?;
    assert_eq!(
        lowir(bare, &call)?,
        lowir(crates::decode(&crates::encode(&genuine)?)?, &call)?
    );
    let generic = fixture("generic")?;
    let stripped = forged(&genuine, only(&genuine, "keep")?, |row| {
        row.callable = None;
    })?;
    assert_eq!(lowir(stripped, &generic)?, lowir(genuine, &generic)?);
    Ok(())
}

/// A forged row on a safe inherent method reaches the consumer's inherent
/// admission and fails as safe: generated/inherent-forgery.rs, the provider
/// with a safe inherent `Lane::get`. An unsafe one is imported honestly since
/// the guarded-methods item (`enabled_callables.rs`, mover M2).
#[test]
fn forged_rows_on_inherent_functions_pass_the_member_check_and_fail_as_safe() -> Result {
    let _check = crate::support::sealed::SelfCheck::hold();
    let lane = "pub struct Lane;\nimpl Lane {\n    pub fn get(value: u32) -> u32 { value }\n}\n";
    let genuine = provider(&format!("{}{lane}", fixture("provider")?))?;
    let interface = forged(&genuine, only(&genuine, "get")?, |row| {
        row.callable
            .get_or_insert_with(Box::default)
            .target_features = Some(declared(&["sse"]));
    })?;
    let error = source_packets::client_analysis(interface, &fixture("call")?)
        .err()
        .ok_or("forged inherent row was accepted")?
        .to_string();
    assert!(
        error.contains("source target-feature function is not declared unsafe"),
        "{error}"
    );
    Ok(())
}

/// The foundation, then `count` functions, each with or without one feature.
fn generated(count: usize, targeted: bool) -> Result<String> {
    let provider = fixture("provider")?;
    let mut source = String::new();
    for line in provider.lines().take(FOUNDATION) {
        writeln!(source, "{line}")?;
    }
    source.push_str("// Authored target-feature transport section.\n");
    for index in 0..count {
        if targeted {
            source.push_str("#[target_feature(enable = \"sse2\")] ");
        }
        writeln!(source, "pub unsafe fn f{index}() {{}}")?;
    }
    Ok(source)
}
fn work(count: usize, targeted: bool) -> Result<usize> {
    let client = "#![feature(no_core, asm_native_fixture)]\n#![no_core]\nextern crate base;\n\
                  #[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { 42 }\n";
    let interface = provider(&generated(count, targeted)?)?;
    let (_, analysis, _) = source_packets::client_analysis(interface, client)?;
    Ok(
        crate::sema::header_attributes::imported_header_work(&analysis.facts)
            .ok_or("the client's import ledger")?,
    )
}

#[test]
fn the_import_ledger_pays_one_fixed_charge_per_feature_row() -> Result {
    let targeted = [work(1, true)?, work(2, true)?, work(8, true)?];
    let plain = [work(1, false)?, work(2, false)?, work(8, false)?];
    eprintln!("targeted={targeted:?} untargeted={plain:?}");
    assert_eq!(
        targeted[2] - targeted[0],
        7 * (targeted[1] - targeted[0]),
        "t(8) - t(1) = 7 (t(2) - t(1))"
    );
    let charge = targeted[0] - plain[0];
    assert!(charge > 0, "a bit-3 row pays its ledger charge");
    assert_eq!(
        targeted[2] - plain[2],
        8 * charge,
        "t(8) - u(8) = 8 (t(1) - u(1))"
    );
    Ok(())
}
