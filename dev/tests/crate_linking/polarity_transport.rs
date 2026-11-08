//! Fresh CLI processes re-prove version-thirteen auto impls. Each impl of its
//! own packet's auto trait keeps its unsafe and negative flags, and every
//! consumer re-proves its safety, its empty members, the local auto Self rule
//! and a negative nominal header's coverage of its declared inputs.
use super::fresh_packets::{Packet, Result, declaration, relabel, spliced, with_id};
use super::marker_transport::{export, source, variant};
use std::process::Command;
const SPEC: &str = "pa29/tests/spec/auto-impls";
/// The output tree C1's marker controls share; each label here is distinct
/// from theirs, because tests of one process run concurrently.
const GROUP: &str = "auto-traits";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
const GENERIC: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";
const SAFETY: &str = "trait and implementation safety declarations disagree";
const ROW_SAFETY: &str = "selected impl safety/member kinds disagree with its actual trait";
const SIZED: &str = "automatic implementation generic subject must be Sized";
const STRENGTH: &str = "negative implementation header-strength proof is not implemented";
const SPECIALIZED: &str = "specialized negative header legality is not implemented";
const IMPL_FENCE: &str =
    "source const/negative/unsafe implementation requires a wider explicit profile";
/// Frozen outcomes from `pa29/tests/auto-impl-outcomes.json`.
const ACCEPTED: [&str; 7] = [
    "explicit-send",
    "loc-static",
    "unwind-structural",
    "cycle",
    "local-cell",
    "local-impl",
    "local-specialized",
];
const REJECTED: [(&str, &str); 19] = [
    ("cell-sync", GENERIC),
    ("raw-send", GENERIC),
    ("foo-other", GENERIC),
    ("rc-send", GENERIC),
    ("pinned", GENERIC),
    ("unique-mut", GENERIC),
    ("cell-ref-unwind", GENERIC),
    ("cell-static", "shared static type must implement Sync"),
    (
        "arguments-send",
        "conditional negative implementation requires scoped debt payment",
    ),
    ("raw-holder", GENERIC),
    ("pair-cell", GENERIC),
    ("local-specialized-other", GENERIC),
    ("local-negative", GENERIC),
    (
        "foreign-pointer",
        "library trait implementation requires a local type",
    ),
    (
        "foreign-reference",
        "foreign automatic trait requires an actual local nominal implementation target",
    ),
    ("safe-impl", SAFETY),
    ("unsafe-negative", SAFETY),
    ("unsafe-unpin-impl", SAFETY),
    (
        "freeze-impl",
        "explicit source Freeze implementations require freeze_impls",
    ),
];
/// An impl header row with a trait application: its Self definition, scope
/// and Self type, the application's tag and id, then its unsafe, constant and
/// negative bytes, its region captures and bounds, its inputs and declared
/// count, and its member range.
const IMPL_ROW: usize = 52;
const SELF_TYPE: usize = 8;
const APPLICATION: usize = 13;
const UNSAFE: usize = 17;
const NEGATIVE: usize = 19;
const MEMBERS_END: usize = 48;
/// A trait application row: its declaration address (artifact, then
/// declaration), its subject and its argument range.
const APPLICATION_ROW: usize = 20;
/// The provider tail's impls in source order, each `(unsafe, negative)`.
const TAIL: [(u8, u8); 22] = [
    (0, 1), // !Send for *const T
    (0, 1), // !Send for *mut T
    (0, 1), // !Sync for *const T
    (0, 1), // !Sync for *mut T
    (1, 0), // Send for &T
    (0, 1), // !UnwindSafe for &mut T
    (0, 0), // UnwindSafe for &T
    (0, 1), // !Send for Arguments<'_>
    (0, 1), // !Sync for Arguments<'_>
    (1, 0), // Send for Cell<T>
    (0, 1), // !Sync for Cell<T>
    (0, 1), // !RefUnwindSafe for Cell<T>
    (1, 0), // Send for Unique<T>
    (1, 0), // Sync for Unique<T>
    (1, 0), // Send for Loc<'_>
    (1, 0), // Sync for Loc<'_>
    (0, 1), // !Unpin for PhantomPinned
    (0, 1), // !UnsafeUnpin for PhantomPinned
    (0, 0), // Allocator for Global
    (0, 1), // !Send for Rc<T, A>
    (0, 1), // !Sync for Rc<T, A>
    (1, 0), // Send for Foo<u8>
];
const POINTER_SEND: usize = 0;
const REFERENCE_SEND: usize = 4;
const REFERENCE_UNWIND: usize = 6;
const CELL_SYNC: usize = 10;
const CELL_REF_UNWIND: usize = 11;
const LOC_SEND: usize = 14;
const GLOBAL: usize = 18;
const RC_SEND: usize = 19;
const FOO_SEND: usize = 21;
/// The auto-declarations provider's first 57 lines, which versions 12 and 10
/// export, follow this provider's own feature line.
const FOUNDATION: usize = 57;
pub(super) fn id(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(<[u8; 4]>::try_from(
        bytes.get(offset..offset + 4).ok_or("id")?,
    )?))
}
/// Each tail impl's row, found from the one byte that making
/// `UnwindSafe for &T` negative moves; every row's flags are then checked.
fn impl_rows(packet: &Packet, bytes: &[u8]) -> Result<[usize; 22]> {
    let negative = variant(
        packet,
        bytes,
        "negative-unwind",
        "impl<T: RefUnwindSafe + PointeeSized> UnwindSafe for &T {}",
        "impl<T: RefUnwindSafe + PointeeSized> !UnwindSafe for &T {}",
    )?;
    let first = (negative - NEGATIVE)
        .checked_sub(REFERENCE_UNWIND * IMPL_ROW)
        .ok_or("impl header table")?;
    let rows = std::array::from_fn(|index| first + index * IMPL_ROW);
    for (row, (unsafe_, negative)) in rows.iter().zip(TAIL) {
        assert_eq!(bytes[row + APPLICATION - 1], 1, "trait application tag");
        assert_eq!(
            bytes[row + UNSAFE..=row + NEGATIVE],
            [unsafe_, 0, negative],
            "impl row {row}"
        );
    }
    Ok(rows)
}
/// The trait application table: the one offset at which every tail impl's
/// application names that impl's own Self type.
pub(super) fn applications(bytes: &[u8], rows: &[usize]) -> Result<usize> {
    let pairs = rows
        .iter()
        .map(|&row| Ok((id(bytes, row + APPLICATION)?, id(bytes, row + SELF_TYPE)?)))
        .collect::<Result<Vec<_>>>()?;
    let mut tables = (0..bytes.len()).filter(|&table| {
        pairs.iter().all(|&(application, self_type)| {
            let row = table + application as usize * APPLICATION_ROW;
            id(bytes, row).is_ok_and(|artifact| artifact == 0)
                && id(bytes, row + 8).is_ok_and(|subject| subject == self_type)
        })
    });
    let table = tables.next().ok_or("trait application table")?;
    if tables.next().is_some() {
        return Err("trait application table is ambiguous".into());
    }
    Ok(table)
}
/// The application row of the impl at `row`, and the next row: the impl's
/// first input, whose subject is its own type parameter.
pub(super) fn application(bytes: &[u8], table: usize, row: usize) -> Result<(usize, usize)> {
    let at = table + id(bytes, row + APPLICATION)? as usize * APPLICATION_ROW;
    Ok((at, at + APPLICATION_ROW))
}
/// The impl at `row` with its Self and its application's subject both
/// changed to its first input's subject, its own type parameter.
fn generic_self(bytes: &[u8], table: usize, row: usize) -> Result<Vec<u8>> {
    let (header, input) = application(bytes, table, row)?;
    let parameter = id(bytes, input + 8)?;
    Ok(with_id(
        &with_id(bytes, row + SELF_TYPE, parameter),
        header + 8,
        parameter,
    ))
}
/// The one clause row that `Rc` owns whose trait application names
/// `Allocator`: an unbound, ordinary trait requirement without a Self slot.
fn rc_allocator(packet: &Packet, bytes: &[u8], table: usize, global: usize) -> Result<usize> {
    let repr = variant(
        packet,
        bytes,
        "rc-c",
        "pub struct Rc<",
        "#[repr(C)]\npub struct Rc<",
    )?;
    let rc = declaration(bytes, repr)?;
    let allocator = id(bytes, application(bytes, table, global)?.0 + 4)?;
    let mut clauses = (0..bytes.len().saturating_sub(16)).filter(|&row| {
        bytes[row..row + 4] == rc
            && bytes[row + 8..row + 10] == [0, 0]
            && bytes[row + 14..row + 16] == [0, 0]
            && id(bytes, row + 10).is_ok_and(|application| {
                id(bytes, table + application as usize * APPLICATION_ROW + 4)
                    .is_ok_and(|trait_| trait_ == allocator)
            })
    });
    let clause = clauses.next().ok_or("Rc's Allocator input")?;
    if clauses.next().is_some() {
        return Err("Rc's Allocator input is ambiguous".into());
    }
    let application = id(bytes, clause + 10)? as usize;
    Ok(table + application * APPLICATION_ROW)
}
fn generated(packet: &Packet, name: &str, text: &str) -> Result<Command> {
    let path = source(packet, &format!("{name}.rs"), text)?;
    Ok(packet.export(
        PROFILE,
        &path,
        &packet.folder.join(format!("{name}.rgcrate")),
    ))
}
#[test]
fn auto_impls_survive_wire_and_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "impls-frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(
        rustgm::support::crates::encode(&rustgm::support::crates::decode(&bytes)?)?,
        bytes
    );
    let rows = impl_rows(&packet, &bytes)?;
    let table = applications(&bytes, &rows)?;
    // Rc's own Allocator input is a single clause of its nominal header.
    rc_allocator(&packet, &bytes, table, rows[GLOBAL])?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    for (name, diagnostic) in REJECTED {
        packet.rejected(name, &mut packet.client(name), diagnostic)?;
    }
    Ok(())
}
#[test]
fn forged_auto_impls_reject_in_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "impls-forged", "provider", PROFILE)?;
    let bytes = &std::fs::read(&packet.metadata)?;
    let rows = impl_rows(&packet, bytes)?;
    let table = applications(bytes, &rows)?;
    let send = id(bytes, application(bytes, table, rows[POINTER_SEND])?.0 + 4)?;
    let allocator = rc_allocator(&packet, bytes, table, rows[GLOBAL])?;
    let foo = rows[FOO_SEND];
    let members = rows[RC_SEND] + MEMBERS_END;
    let rc_send = application(bytes, table, rows[RC_SEND])?.0;
    for (name, changed, diagnostic) in [
        (
            "unsafe-pointer",
            spliced(bytes, rows[POINTER_SEND] + UNSAFE, &[1]),
            ROW_SAFETY,
        ),
        (
            "safe-loc",
            spliced(bytes, rows[LOC_SEND] + UNSAFE, &[0]),
            ROW_SAFETY,
        ),
        (
            "positive-cell",
            spliced(bytes, rows[CELL_SYNC] + NEGATIVE, &[0]),
            ROW_SAFETY,
        ),
        (
            "negative-foo",
            spliced(&spliced(bytes, foo + UNSAFE, &[0]), foo + NEGATIVE, &[1]),
            SPECIALIZED,
        ),
        // A blanket Send meets the packet's other Send impls first.
        (
            "generic-send",
            generic_self(bytes, table, rows[REFERENCE_SEND])?,
            "overlapping trait implementation headers require coherence proof",
        ),
        (
            "generic-unwind",
            generic_self(bytes, table, rows[CELL_REF_UNWIND])?,
            SIZED,
        ),
        ("rc-bound", with_id(bytes, allocator + 4, send), STRENGTH),
        // Admission rejects a negative impl's range that takes a row.
        (
            "rc-member",
            with_id(bytes, members, id(bytes, members)? + 1),
            "auto or negative trait implementations cannot declare members",
        ),
        (
            "foreign-rc",
            with_id(bytes, rc_send, 1),
            "source implementation requires its exact supported trait header",
        ),
        (
            "version-12",
            relabel(bytes, 12),
            "unknown source declaration kind",
        ),
    ] {
        let mut consumer = packet.forged(name, changed, "explicit-send")?;
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    let mut honest = packet.forged("honest", bytes.clone(), "explicit-send")?;
    super::artifacts::checked("honest", &packet.folder, &mut honest)?;
    Ok(())
}
#[test]
fn generated_auto_impl_controls_keep_their_diagnostics() -> Result {
    let packet = Packet::new(GROUP, SPEC, "impls-gates", "provider")?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    for (name, text, diagnostic) in [
        (
            "strengthened",
            fixture.replacen(
                "impl<T: ?Sized, A: Allocator> !Send for Rc<T, A> {}",
                "impl<T: ?Sized + Sync, A: Allocator> !Send for Rc<T, A> {}",
                1,
            ),
            STRENGTH,
        ),
        (
            "specialized",
            format!(
                "{fixture}// Generated control.\npub struct Bar<T> {{ pub value: T }}\nimpl !Send for Bar<u8> {{}}\n"
            ),
            SPECIALIZED,
        ),
        (
            "blanket",
            format!(
                "{fixture}// Generated control.\npub auto trait Mark {{}}\nimpl<T: ?Sized> Mark for T {{}}\n"
            ),
            SIZED,
        ),
    ] {
        assert_ne!(text, fixture, "{name}");
        packet.rejected(name, &mut generated(&packet, name, &text)?, diagnostic)?;
    }
    let foundation: String = fixture
        .split_inclusive('\n')
        .skip(1)
        .take(FOUNDATION)
        .collect();
    assert!(foundation.ends_with("pub unsafe auto trait Freeze {}\n"));
    let older = source(
        &packet,
        "older.rs",
        &format!(
            "#![feature(negative_impls, freeze_impls)]\n{foundation}pub struct X {{ pub v: u8 }}\nimpl !Freeze for X {{}}\n"
        ),
    )?;
    for profile in OLDER {
        let metadata = packet.folder.join(format!("older-{profile}.rgcrate"));
        packet.rejected(
            profile,
            &mut packet.export(profile, &older, &metadata),
            IMPL_FENCE,
        )?;
    }
    let metadata = packet.folder.join("older.rgcrate");
    export(&packet, &older, &metadata)?;
    let answer = source(
        &packet,
        "answer.rs",
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { 42 }\n",
    )?;
    let mut consumer = packet.consumer(&answer, "answer", &metadata, &packet.owner());
    super::artifacts::checked("answer", &packet.folder, &mut consumer)?;
    // The honest counterpart of the generic-Self forgery: a consumer proves a
    // Sized parameter Self through the imported Sized role.
    let sized = source(
        &packet,
        "sized.rs",
        &format!(
            "{fixture}// Generated control.\npub auto trait Mark {{}}\nimpl<T> Mark for T {{}}\n"
        ),
    )?;
    let metadata = packet.folder.join("sized.rgcrate");
    export(&packet, &sized, &metadata)?;
    let client = source(
        &packet,
        "sized-client.rs",
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\nfn need<T: base::Mark>() {}\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { need::<u8>(); need::<base::Cell<u8>>(); 42 }\n",
    )?;
    let mut consumer = packet.consumer(&client, "sized-client", &metadata, &packet.owner());
    super::artifacts::checked("sized-client", &packet.folder, &mut consumer)?;
    Ok(())
}
/// A source client's library export carries no nominal, so it can declare no
/// foreign auto impl that another consumer could observe.
#[test]
fn client_libraries_cannot_export_auto_impls() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "impls-library", "provider", PROFILE)?;
    let prefix = "#![feature(no_core)]\n#![no_core]\nextern crate base;\n";
    for (name, holder) in [
        (
            "public",
            "pub struct Holder { pub p: *const u8 }\nunsafe impl base::Send for Holder {}\n",
        ),
        ("private", ""),
    ] {
        let path = source(
            &packet,
            &format!("{name}.rs"),
            &format!("{prefix}{holder}pub fn answer() -> u8 {{ 42 }}\n"),
        )?;
        let output = packet.folder.join(format!("{name}.rgcrate"));
        let mut command = packet.consumer(&path, name, &packet.metadata, &packet.owner());
        command.arg("--metadata-output").arg(&output);
        if holder.is_empty() {
            super::artifacts::checked(name, &packet.folder, &mut command)?;
            assert!(std::fs::metadata(&output)?.len() > 0);
        } else {
            packet.rejected(
                name,
                &mut command,
                "nominal trait, member or non-exhaustive contract is outside the metadata profile",
            )?;
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn auto_impl_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "impls-native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
