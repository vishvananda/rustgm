//! An imported nominal's transported where-clauses are formation requirements
//! in a client, and each unit of their work has one payer. An uncharged walk
//! pays the installed header budget exactly the substitution it attempted,
//! once per distinct instance. A charged walk, as specialization admission
//! makes, pays only its own closure, as a source nominal's walk does. A cache
//! hit pays what a source hit pays on both paths, whichever path filled it.
use super::*;
use crate::{
    lower::instances::SourcePublication,
    resolve::{expansion::Report, model::DefKind},
    sema::crates::Linkage,
    support::crates,
    types::Primitive,
};
use std::cell::Cell;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const SPEC: &str = "pa29/tests/spec/imported-formation";
/// A client with a local twin of the provider's `Rc<T: ?Sized, A: Allocator>`.
const CLIENT: &str = "#![feature(no_core)]\n#![no_core]\nextern crate base;\ntrait LocalAlloc {}\nimpl LocalAlloc for base::Global {}\nstruct LocalRc<T: ?base::Sized, A: LocalAlloc> { alloc: A, value: T }\nfn hold(_r: base::Rc<u8, base::Global>, _l: LocalRc<u8, base::Global>) {}\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { 42 }\n";
const EMPTY: &str = "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { 42 }\n";
const COUNTS: [usize; 3] = [8, 16, 32];
/// The two walks, as their payers see them.
#[derive(Clone, Copy, Debug)]
enum Walk {
    Uncharged,
    Charged,
}
/// One completed client analysis over the packet `bytes`.
struct Session {
    report: Report,
    facts: Declarations,
}
fn packet(provider: &str) -> TestResult<Vec<u8>> {
    let interface = crate::module_packets::exported_profile(
        provider,
        SourcePublication::ModuleBodies,
        "imported_formation",
    )?;
    Ok(crates::encode(&interface)?)
}
fn session(bytes: &[u8], client: &str) -> TestResult<Session> {
    let (report, analysis, _) =
        crate::source_packets::client_analysis(crates::decode(bytes)?, client)?;
    Ok(Session {
        report,
        facts: analysis.facts,
    })
}
fn owner(report: &Report, name: &str) -> TestResult<DefId> {
    let (index, _) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| {
            report.package.names.get(row.name) == name
                && matches!(row.kind, DefKind::Struct | DefKind::Trait)
        })
        .ok_or("actual nominal or trait")?;
    Ok(DefId(u32::try_from(index)?))
}
/// The installed header budget's work and retained bytes.
fn budget(facts: &Declarations) -> TestResult<(usize, usize)> {
    let (work, bytes, _, _) = facts
        .crates
        .as_deref()
        .and_then(Linkage::original_work)
        .ok_or("installed header budget")?;
    Ok((work, bytes))
}
fn global(session: &mut Session) -> TestResult<TypeId> {
    let item = ItemId(owner(&session.report, "Global")?.0);
    Ok(session
        .facts
        .types
        .intern(Kind::Adt(item, GenericArgs::EMPTY))?)
}
/// `name<u16, base::Global>`, an instance no analysis has formed.
fn instance(session: &mut Session, name: &str) -> TestResult<TypeId> {
    let global = global(session)?;
    let arguments = session.facts.types.intern_arguments(&[
        Argument::Type(TypeId::primitive(Primitive::U16)),
        Argument::Type(global),
    ])?;
    let item = ItemId(owner(&session.report, name)?.0);
    Ok(session.facts.types.intern(Kind::Adt(item, arguments))?)
}
/// The steps and bytes of substituting `name`'s own non-outlives inputs at the
/// instance's arguments, the work a payer of that instance's clauses owes.
fn substitution(bytes: &[u8], name: &str) -> TestResult<(usize, usize)> {
    let mut session = session(bytes, CLIENT)?;
    let ty = instance(&mut session, name)?;
    let Some(Kind::Adt(item, arguments)) = session.facts.types.kind(ty) else {
        return Err("nominal instance".into());
    };
    let owner = DefId(item.0);
    let facts = &mut session.facts;
    let mut clauses: Vec<Clause> = facts
        .generics
        .predicates
        .get(&owner)
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .map(|predicate| predicate.value)
        .collect();
    if let Some(linkage) = facts.crates.as_deref() {
        clauses.extend(
            linkage
                .original_inputs(&facts.generics, owner, false)?
                .iter()
                .map(|input| input.value),
        );
    }
    let total = Cell::new((0_usize, 0_usize));
    for clause in clauses {
        if matches!(
            clause,
            Clause::Predicate(Predicate {
                bound: Bound::Outlives(_),
                ..
            })
        ) {
            continue;
        }
        facts.generics.substitute_clause_charged(
            &mut facts.types,
            owner,
            arguments,
            clause,
            &|steps, retained| {
                let (work, kept) = total.get();
                total.set((work + steps, kept + retained));
                Ok(())
            },
        )?;
    }
    Ok(total.get())
}
/// One walk's clauses, header-budget delta and closure total.
struct Walked {
    clauses: Vec<Clause>,
    paid: (usize, usize),
    total: (usize, usize),
}
/// One walk over `ty`, as its payers see it.
fn walk(facts: &mut Declarations, ty: TypeId, path: Walk) -> TestResult<Walked> {
    let before = budget(facts)?;
    let total = Cell::new((0_usize, 0_usize));
    let clauses = match path {
        Walk::Uncharged => requirements(facts, ty, &mut Requirements::default())?,
        Walk::Charged => formation_requirements_counted(
            facts,
            ty,
            &mut FormationRequirements::default(),
            &|steps, retained| {
                let (work, kept) = total.get();
                total.set((work + steps, kept + retained));
                Ok(())
            },
        )?,
    };
    let after = budget(facts)?;
    Ok(Walked {
        clauses,
        paid: (after.0 - before.0, after.1 - before.1),
        total: total.get(),
    })
}
/// Whether `clauses` require `Global` to implement the trait named `name`.
fn requires(session: &mut Session, clauses: &[Clause], name: &str) -> TestResult<bool> {
    let global = global(session)?;
    let item = ItemId(owner(&session.report, name)?.0);
    Ok(clauses.iter().any(|clause| match clause {
        Clause::Predicate(predicate) if predicate.ty == global => predicate
            .bound
            .trait_parts()
            .and_then(|(application, _)| session.facts.types.trait_application(application))
            .is_some_and(|application| application.item == item),
        _ => false,
    }))
}
#[test]
fn uncharged_formation_pays_its_imported_substitution_once_per_instance() -> TestResult {
    let bytes = packet(&crate::test_source::fixture(&format!(
        "{SPEC}/provider.rs"
    ))?)?;
    let expected = substitution(&bytes, "Rc")?;
    assert!(expected.0 > 0, "the imported inputs were substituted");
    let mut session = session(&bytes, CLIENT)?;
    let ty = instance(&mut session, "Rc")?;
    let Walked { clauses, paid, .. } = walk(&mut session.facts, ty, Walk::Uncharged)?;
    assert!(requires(&mut session, &clauses, "Allocator")?);
    assert_eq!(paid, expected, "the miss pays its attempted substitution");
    let Walked {
        clauses: again,
        paid: hit,
        ..
    } = walk(&mut session.facts, ty, Walk::Uncharged)?;
    assert_eq!(again, clauses);
    assert_eq!(hit, (0, 0), "a hit pays the header budget nothing");
    let local = instance(&mut session, "LocalRc")?;
    for _ in 0..2 {
        let Walked { clauses, paid, .. } = walk(&mut session.facts, local, Walk::Uncharged)?;
        assert!(requires(&mut session, &clauses, "LocalAlloc")?);
        assert_eq!(
            paid,
            (0, 0),
            "a source nominal never pays the header budget"
        );
    }
    Ok(())
}
#[test]
fn charged_formation_pays_only_its_closure_as_its_source_twin_does() -> TestResult {
    let bytes = packet(&crate::test_source::fixture(&format!(
        "{SPEC}/provider.rs"
    ))?)?;
    let mut totals = Vec::new();
    for (name, trait_name) in [("Rc", "Allocator"), ("LocalRc", "LocalAlloc")] {
        let mut session = session(&bytes, CLIENT)?;
        let ty = instance(&mut session, name)?;
        let Walked {
            clauses,
            paid,
            total,
        } = walk(&mut session.facts, ty, Walk::Charged)?;
        assert!(requires(&mut session, &clauses, trait_name)?);
        assert_eq!(
            paid,
            (0, 0),
            "{name}: the charged path leaves the header budget"
        );
        let substituted = substitution(&bytes, name)?;
        assert!(
            total.0 >= substituted.0 && total.1 >= substituted.1,
            "{name}"
        );
        totals.push((total.0 - substituted.0, total.1 - substituted.1));
    }
    assert_eq!(
        totals[0], totals[1],
        "beyond its substitution, an imported nominal's walk pays what its source twin's does"
    );
    Ok(())
}
#[test]
fn cache_hits_pay_as_source_hits_whichever_path_filled_them() -> TestResult {
    let bytes = packet(&crate::test_source::fixture(&format!(
        "{SPEC}/provider.rs"
    ))?)?;
    for (fill, hit) in [
        (Walk::Uncharged, Walk::Charged),
        (Walk::Charged, Walk::Uncharged),
    ] {
        let mut costs = Vec::new();
        for name in ["Rc", "LocalRc"] {
            let mut session = session(&bytes, CLIENT)?;
            let ty = instance(&mut session, name)?;
            let filled = walk(&mut session.facts, ty, fill)?.clauses;
            let Walked {
                clauses,
                paid,
                total,
            } = walk(&mut session.facts, ty, hit)?;
            assert_eq!(clauses, filled, "{name}: {fill:?} then {hit:?}");
            assert_eq!(paid, (0, 0), "{name}: a hit pays the header budget nothing");
            costs.push(total);
        }
        assert_eq!(
            costs[0], costs[1],
            "{fill:?} then {hit:?}: an imported hit pays what a source hit pays"
        );
    }
    Ok(())
}
/// The provider and N generated nominals, each with an imported where-clause.
fn wrappers(provider: &str, count: usize) -> TestResult<String> {
    use std::fmt::Write;
    let mut text = format!("{provider}// Generated scaling nominals.\n");
    for index in 0..count {
        writeln!(text, "pub struct W{index:03}<A: Allocator> {{ pub a: A }}")?;
    }
    Ok(text)
}
fn uses(count: usize) -> TestResult<String> {
    use std::fmt::Write;
    let mut text = String::from("#![feature(no_core)]\n#![no_core]\nextern crate base;\n");
    for index in 0..count {
        writeln!(
            text,
            "fn h{index:03}(_w: base::W{index:03}<base::Global>) {{}}"
        )?;
    }
    text.push_str("#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { 42 }\n");
    Ok(text)
}
/// Each distinct instance's uncharged walk, in order, in a fresh session.
fn instance_walks(bytes: &[u8], count: usize) -> TestResult<Vec<(usize, usize)>> {
    let mut fresh = session(bytes, EMPTY)?;
    let global = global(&mut fresh)?;
    let arguments = fresh
        .facts
        .types
        .intern_arguments(&[Argument::Type(global)])?;
    let mut walks = Vec::with_capacity(count);
    for index in 0..count {
        let item = ItemId(owner(&fresh.report, &format!("W{index:03}"))?.0);
        let ty = fresh.facts.types.intern(Kind::Adt(item, arguments))?;
        let walked = walk(&mut fresh.facts, ty, Walk::Uncharged)?;
        assert!(
            requires(&mut fresh, &walked.clauses, "Allocator")?,
            "W{index:03}"
        );
        walks.push(walked.paid);
    }
    Ok(walks)
}
/// One instance's formation pays the same constant at every N. A walk pays
/// more only when it resizes the shared substitution cache, which a doubling
/// map does at most floor(log2 N) + 1 times, so the total stays linear.
#[test]
fn imported_formation_work_grows_by_a_constant_per_instance() -> TestResult {
    let provider = crate::test_source::fixture(&format!("{SPEC}/provider.rs"))?;
    let mut previous: Option<usize> = None;
    let mut constant: Option<(usize, usize)> = None;
    for count in COUNTS {
        let bytes = packet(&wrappers(&provider, count)?)?;
        if let Some(before) = previous {
            assert!(
                bytes.len() <= 2 * before + 64,
                "{count}: {} packet bytes after {before}",
                bytes.len()
            );
        }
        previous = Some(bytes.len());
        session(&bytes, &uses(count)?)?;
        let walks = instance_walks(&bytes, count)?;
        let least = walks.iter().copied().min().ok_or("instance walks")?;
        assert_eq!(
            *constant.get_or_insert(least),
            least,
            "{count}: an instance's formation work is independent of N"
        );
        let resized = walks.iter().filter(|paid| **paid != least).count();
        assert!(
            walks
                .iter()
                .all(|paid| *paid == least || (paid.0 > least.0 && paid.1 > least.1)),
            "{count}: {walks:?}"
        );
        let doublings = usize::try_from(usize::BITS - count.leading_zeros())?;
        assert!(resized <= doublings, "{count}: {resized} resizes");
        let steps: usize = walks.iter().map(|paid| paid.0).sum();
        assert!(steps <= 2 * count * least.0, "{count}: {steps} steps");
    }
    Ok(())
}
