//! Version thirteen admits an unsafe associated function under direct Call's
//! rule. Its original and public safety agree, an implementation member's
//! safety is its trait member's, and a call inside or to one follows the
//! matrix a free function's call does. Forged bits and lexical contexts are
//! rejected by name, versions twelve and ten keep both fences, and an unsafe
//! method costs a constant over its safe twin.
use crate::{
    lower::instances::SourcePublication,
    module_packets::{Result, exported_profile, raw},
    sema::crates::Linkage,
    support::crates::{
        self,
        generics::{self as g, source as s},
        owner_catalog::DeclarationRef,
    },
};
use std::fmt::Write as _;

const PROVIDER: &str = "pa29/tests/spec/guarded-methods/provider.rs";
const NOMINALS: &str = "pa29/tests/spec/source-nominals/provider.rs";
const AGREEMENT: &str = "source associated callable changed its actual original/public safety";
const MEMBER: &str = "source implementation member changed its trait member's safety";
const CALLER: &str = "source unsafe-function Call changed its actual caller declaration";
const CALLABLE: &str = "source Call changed its actual callable safety";
const LEXICAL: &str = "source unsafe Call omitted its actual lexical context";
const CLAIM: &str = "source safe Call cannot claim an unsafe obligation";
const NONORDINARY: &str =
    "source nonordinary callable effect/safety/convention requires a wider profile";
const ASSOCIATED: &str = "source unsafe associated Calls require their wider actual profile";
/// The safe twins the older-version forgeries append to the nominals provider.
const OLDER: &str = "// Generated associated-unsafe control.\npub fn raw() {}\npub trait Via { fn via(&self); }\nimpl Via for u8 { fn via(&self) { unsafe { raw() } } }\n";
const COUNTS: [usize; 3] = [8, 16, 32];
const PROBES: [usize; 3] = [1, 2, 4];
const CLIENT: &str = "#![feature(no_core)]\n#![no_core]\nextern crate base;\n";
const ENTRY: &str = "#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { 42 }\n";

type Restage = fn(
    &crates::CrateIdentity,
    s::SourceRows,
    s::HeaderAttributeColumns,
    s::ModuleContractRows,
    s::AvailabilityRows,
    usize,
    &mut g::Work,
) -> std::result::Result<g::Payload, &'static str>;

/// An exported interface whose rows are restaged, forged or not, by one
/// version's validator.
struct Staging<'a> {
    interface: &'a crates::Interface,
    version: Restage,
}
impl Staging<'_> {
    fn restage(
        &self,
        rows: s::SourceRows,
        work: &mut g::Work,
    ) -> Result<std::result::Result<g::Payload, &'static str>> {
        let (_, attributes, modules) = raw(self.interface)?;
        let descriptor = self
            .interface
            .source_descriptor()
            .ok_or("actual descriptor")?;
        let availability = descriptor.availability()?.clone();
        let owner = self.interface.identity();
        Ok((self.version)(
            owner,
            rows,
            attributes,
            modules,
            availability,
            0,
            work,
        ))
    }
    /// The diagnostic `rows` restage with, if any.
    fn rejection(&self, rows: s::SourceRows) -> Result<Option<&'static str>> {
        Ok(self.restage(rows, &mut g::Work::default())?.err())
    }
}
/// The one declaration of `candidates` named `name`.
fn only(
    rows: &s::SourceRows,
    candidates: impl Iterator<Item = DeclarationRef>,
    name: &str,
) -> Result<DeclarationRef> {
    let mut found = candidates.filter(|candidate| {
        rows.declarations
            .get(candidate.0 as usize)
            .and_then(|row| row.name.as_deref())
            == Some(name)
    });
    match (found.next(), found.next()) {
        (Some(one), None) => Ok(one),
        _ => Err(format!("no single declaration {name}").into()),
    }
}
fn nominal(rows: &s::SourceRows, name: &str) -> Result<DeclarationRef> {
    only(rows, rows.nominals.iter().map(|row| row.declaration), name)
}
fn trait_owner(rows: &s::SourceRows, name: &str) -> Result<DeclarationRef> {
    only(rows, rows.traits.iter().map(|row| row.declaration), name)
}
/// The one inherent impl of the local nominal named `name`.
fn inherent(rows: &s::SourceRows, name: &str) -> Result<DeclarationRef> {
    let owner = nominal(rows, name)?;
    let mut found = rows.implementations.iter().filter(|row| {
        row.trait_application.is_none()
            && matches!(
                rows.types.get(row.self_type.0 as usize),
                Some(s::Type::Nominal { declaration, .. })
                    if declaration.artifact.0 == 0 && declaration.declaration == owner
            )
    });
    match (found.next(), found.next()) {
        (Some(one), None) => Ok(one.self_definition),
        _ => Err(format!("no single inherent impl of {name}").into()),
    }
}
/// The index of the function header named `name` whose parent is `parent`:
/// an impl for an inherent or impl member, a trait for a trait member.
fn header(rows: &s::SourceRows, parent: Option<DeclarationRef>, name: &str) -> Result<usize> {
    let siblings = rows.functions.iter().filter(|row| row.parent == parent);
    let declaration = only(rows, siblings.map(|row| row.declaration), name)?;
    rows.functions
        .iter()
        .position(|row| row.declaration == declaration)
        .ok_or_else(|| "function header".into())
}
/// The original of the function at `function` and its one direct or pointer
/// call value.
fn site(rows: &s::SourceRows, function: usize, pointer: bool) -> Result<(usize, usize)> {
    let declaration = rows.functions[function].declaration;
    let body = rows
        .originals
        .iter()
        .position(|original| original.declaration == declaration)
        .ok_or("original")?;
    let values = &rows.originals[body].values;
    let calls: Vec<usize> = (0..values.len())
        .filter(|&at| match values[at].kind {
            s::ValueKind::PointerCall { .. } => pointer,
            s::ValueKind::Call { .. } => !pointer,
            _ => false,
        })
        .collect();
    match calls[..] {
        [one] => Ok((body, one)),
        _ => Err(format!("{} call values", calls.len()).into()),
    }
}
/// Sets the public and original `unsafe` bits of the function at `function`.
fn marked(rows: &mut s::SourceRows, function: usize, (public, original): (bool, bool)) {
    let header = &mut rows.functions[function];
    header.callable.unsafe_ = public;
    header.original.unsafe_ = original;
}
/// Relabels the lexical safety of the call at `site`: a pointer call's own,
/// or its direct call's request's.
fn relabeled(rows: &mut s::SourceRows, (body, at): (usize, usize), to: s::CallSafety) -> Result {
    let original = &mut rows.originals[body];
    if let s::ValueKind::PointerCall { safety, .. } = &mut original.values[at].kind {
        *safety = to;
        return Ok(());
    }
    let value = u32::try_from(at)?;
    let request = original
        .calls
        .iter_mut()
        .find(|request| request.value == value)
        .ok_or("call request")?;
    request.safety = to;
    Ok(())
}
/// The fixture rows the forgeries retarget.
#[derive(Clone, Copy)]
struct Members {
    get: usize,
    twice: usize,
    relay: usize,
    checked: usize,
    invoke: usize,
    slot_call: usize,
    key_fetch: usize,
    get_fetch: usize,
}
fn members(rows: &s::SourceRows) -> Result<Members> {
    let lane = Some(inherent(rows, "Lane")?);
    Ok(Members {
        get: header(rows, lane, "get")?,
        twice: header(rows, lane, "twice")?,
        relay: header(rows, None, "relay")?,
        checked: header(rows, lane, "checked")?,
        invoke: header(rows, Some(inherent(rows, "Table")?), "invoke")?,
        slot_call: header(rows, Some(inherent(rows, "Slot")?), "call")?,
        key_fetch: implementing(rows, "Get", "fetch")?,
        get_fetch: header(rows, Some(trait_owner(rows, "Get")?), "fetch")?,
    })
}
/// One forged row and the diagnostic every consumer gives it.
fn forged(rows: &mut s::SourceRows, chosen: Members, mutation: usize) -> Result<&'static str> {
    use s::CallSafety::{ExplicitUnsafeBlock, Safe, UnsafeFunction};
    Ok(match mutation {
        0 => {
            marked(rows, chosen.get, (false, true));
            AGREEMENT
        }
        1 => {
            marked(rows, chosen.get, (true, false));
            AGREEMENT
        }
        // `twice` calls `get` bare, under `unsafe_op_in_unsafe_fn`.
        2 => {
            marked(rows, chosen.twice, (false, false));
            CALLER
        }
        // `relay` calls `get` in an unsafe block.
        3 => {
            marked(rows, chosen.get, (false, false));
            CALLABLE
        }
        4 => {
            relabeled(rows, site(rows, chosen.relay, false)?, Safe)?;
            LEXICAL
        }
        5 => {
            relabeled(rows, site(rows, chosen.checked, false)?, UnsafeFunction)?;
            CALLER
        }
        6 => {
            relabeled(rows, site(rows, chosen.invoke, true)?, Safe)?;
            LEXICAL
        }
        7 => {
            relabeled(
                rows,
                site(rows, chosen.slot_call, true)?,
                ExplicitUnsafeBlock,
            )?;
            CLAIM
        }
        8 => {
            marked(rows, chosen.key_fetch, (false, false));
            MEMBER
        }
        _ => {
            marked(rows, chosen.get_fetch, (false, false));
            MEMBER
        }
    })
}
#[test]
fn unsafe_members_restage_and_forged_safety_rejects_by_name() -> Result {
    let source = crate::test_source::fixture(PROVIDER)?;
    let interface = exported_profile(&source, SourcePublication::ModuleBodies, "guarded")?;
    let bytes = crates::encode(&interface)?;
    let decoded = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&decoded)?, bytes);
    let (rows, _, _) = raw(&decoded)?;
    let chosen = members(&rows)?;
    let staging = Staging {
        interface: &decoded,
        version: g::Payload::from_source_module_bodies,
    };
    assert_eq!(staging.rejection(rows.clone())?, None);
    for mutation in 0..10 {
        let mut forgery = rows.clone();
        let expected = forged(&mut forgery, chosen, mutation)?;
        assert_eq!(
            staging.rejection(forgery)?,
            Some(expected),
            "mutation {mutation}"
        );
    }
    Ok(())
}
/// The index of the implementation member of trait `name`'s `member`.
fn implementing(rows: &s::SourceRows, name: &str, member: &str) -> Result<usize> {
    let required = header(rows, Some(trait_owner(rows, name)?), member)?;
    let declared = rows.functions[required].declaration;
    let actual = rows
        .member_links
        .iter()
        .find_map(|link| match link.trait_member {
            Some(address) if address.declaration == declared => link.implementation,
            _ => None,
        })
        .ok_or("implementation member")?;
    rows.functions
        .iter()
        .position(|row| row.declaration == actual)
        .ok_or_else(|| "implementation header".into())
}
/// Marks the direct call at `site` unsafe in its own signature row and
/// relabels it as in an unsafe block.
fn unsafe_call(rows: &mut s::SourceRows, (body, at): (usize, usize)) -> Result {
    let s::ValueKind::Call { signature, .. } = rows.originals[body].values[at].kind else {
        return Err("direct call value".into());
    };
    let Some(s::Type::Function(row)) = rows.types.get_mut(signature.0 as usize) else {
        return Err("call signature row".into());
    };
    row.unsafe_ = true;
    relabeled(rows, (body, at), s::CallSafety::ExplicitUnsafeBlock)
}
/// Safe twins exported under each version, then forged: rows 1 and 7 on the
/// u8 impl's member give the nonordinary text below version thirteen and its own
/// texts at thirteen, and an unsafe call in the member gives the associated
/// call text under versions twelve and ten.
#[test]
fn older_versions_keep_member_and_call_fences_by_name() -> Result {
    let source = format!("{}{OLDER}", crate::test_source::fixture(NOMINALS)?);
    let versions: [(SourcePublication, Restage, [&str; 2], Option<&str>); 3] = [
        (
            SourcePublication::ModuleNominals,
            g::Payload::from_source_module_nominals,
            [NONORDINARY, NONORDINARY],
            Some(ASSOCIATED),
        ),
        (
            SourcePublication::ModuleBranches,
            g::Payload::from_source_module_branches,
            [NONORDINARY, NONORDINARY],
            Some(ASSOCIATED),
        ),
        (
            SourcePublication::ModuleBodies,
            g::Payload::from_source_module_bodies,
            [AGREEMENT, MEMBER],
            None,
        ),
    ];
    for (profile, version, [agreement, member], call) in versions {
        let interface = exported_profile(&source, profile, "guarded_older")?;
        let (rows, _, _) = raw(&interface)?;
        let staging = Staging {
            interface: &interface,
            version,
        };
        assert_eq!(staging.rejection(rows.clone())?, None, "{profile:?}");
        let via = implementing(&rows, "Via", "via")?;
        let mut public = rows.clone();
        marked(&mut public, via, (true, false));
        assert_eq!(staging.rejection(public)?, Some(agreement), "{profile:?}");
        let mut both = rows.clone();
        marked(&mut both, via, (true, true));
        assert_eq!(staging.rejection(both)?, Some(member), "{profile:?}");
        if let Some(expected) = call {
            let mut called = rows.clone();
            let raw_fn = header(&called, None, "raw")?;
            marked(&mut called, raw_fn, (true, true));
            let call = site(&called, via, false)?;
            unsafe_call(&mut called, call)?;
            assert_eq!(staging.rejection(called)?, Some(expected), "{profile:?}");
        }
    }
    Ok(())
}
/// The nominals provider and `count` unit structs, each with a `get` method
/// declared `qualifier` that a provider function calls in an unsafe block.
fn family(count: usize, qualifier: &str, callers: bool) -> Result<String> {
    let mut text = crate::test_source::fixture(NOMINALS)?;
    text.push_str("// Generated associated-unsafe scaling items.\n");
    for index in 0..count {
        writeln!(text, "pub struct S{index:03};")?;
        writeln!(
            text,
            "impl S{index:03} {{ pub {qualifier}fn get(&self) {{}} }}"
        )?;
        if callers {
            writeln!(
                text,
                "pub fn v{index:03}(s: &S{index:03}) {{ unsafe {{ s.get() }} }}"
            )?;
        }
    }
    Ok(text)
}
/// A client that calls each struct's `get` in an unsafe block.
fn caller(count: usize) -> Result<String> {
    let mut text = String::from(CLIENT);
    for index in 0..count {
        writeln!(
            text,
            "pub fn r{index:03}(s: &base::S{index:03}) {{ unsafe {{ s.get() }} }}"
        )?;
    }
    text.push_str(ENTRY);
    Ok(text)
}
/// The packet's length, its restage's steps, and the client's installed
/// original-header steps, retained bytes, observed bytes and visits.
fn measured(provider: &str, client: &str) -> Result<[i64; 6]> {
    let interface = exported_profile(provider, SourcePublication::ModuleBodies, "guarded_scaling")?;
    let bytes = crates::encode(&interface)?;
    let decoded = crates::decode(&bytes)?;
    let (rows, _, _) = raw(&decoded)?;
    let staging = Staging {
        interface: &decoded,
        version: g::Payload::from_source_module_bodies,
    };
    let mut work = g::Work::default();
    staging.restage(rows, &mut work)??;
    let (_, analysis, _) = crate::source_packets::client_analysis(decoded, client)?;
    let installed = analysis
        .facts
        .crates
        .as_deref()
        .and_then(Linkage::original_work);
    let (steps, retained, observed, visits) = installed.ok_or("installed header budget")?;
    let mut out = [0; 6];
    for (slot, value) in
        out.iter_mut()
            .zip([bytes.len(), work.steps, steps, retained, observed, visits])
    {
        *slot = i64::try_from(value)?;
    }
    Ok(out)
}
/// Each metric's excess over the twins at three doubling counts grows at
/// most linearly: the second step is at most twice the first plus a constant.
fn linear<const N: usize>(label: &str, excess: [[i64; N]; 3]) {
    for metric in 0..N {
        let [small, middle, large] = excess.map(|row| row[metric]);
        assert!(
            large - middle <= 2 * (middle - small) + 64,
            "{label}: {excess:?}"
        );
    }
}
/// Unsafe methods each called once in an unsafe block cost the bytes of safe
/// twins called in the same block, and a constant excess per method in the
/// restage and in an empty client's header work.
#[test]
fn unsafe_methods_cost_constant_bytes_and_work() -> Result {
    let client = format!("{CLIENT}{ENTRY}");
    let mut excess = [[0; 2]; 3];
    let mut twins = [[0; 3]; 3];
    for ((slot, twin), count) in excess.iter_mut().zip(&mut twins).zip(COUNTS) {
        let [bytes, restage, steps, ..] = measured(&family(count, "unsafe ", true)?, &client)?;
        let [safe_bytes, safe_restage, safe_steps, ..] =
            measured(&family(count, "", true)?, &client)?;
        assert_eq!(bytes, safe_bytes, "{count} unsafe methods");
        *slot = [restage - safe_restage, steps - safe_steps];
        *twin = [safe_bytes, safe_restage, safe_steps];
    }
    eprintln!("unsafe methods {COUNTS:?}: excess {excess:?}; safe twins {twins:?}");
    linear("unsafe-method excess", excess);
    Ok(())
}
/// The self-demand probe: a client calling `count` unsafe methods installs
/// their headers at the safe twins' retained bytes, observed bytes and visits,
/// and at most a constant per method more steps.
#[test]
fn unsafe_methods_install_at_their_safe_twins_cost() -> Result {
    let mut excess = [[0; 1]; 3];
    let mut twins = [[0; 4]; 3];
    for ((slot, twin), count) in excess.iter_mut().zip(&mut twins).zip(PROBES) {
        let client = caller(count)?;
        let [.., steps, retained, observed, visits] =
            measured(&family(count, "unsafe ", false)?, &client)?;
        let [.., safe_steps, safe_retained, safe_observed, safe_visits] =
            measured(&family(count, "", false)?, &client)?;
        assert_eq!(
            [retained, observed, visits],
            [safe_retained, safe_observed, safe_visits],
            "{count} unsafe methods"
        );
        *slot = [steps - safe_steps];
        *twin = [safe_steps, safe_retained, safe_observed, safe_visits];
    }
    eprintln!("unsafe-method probe {PROBES:?}: step excess {excess:?}; safe twins {twins:?}");
    linear("unsafe-method probe", excess);
    Ok(())
}
