//! Closed goals share one crate-wide answer under callers whose premises all
//! name a parameter, formation prefixes included; a self-check re-proves every
//! hit instead of replaying it.
use std::fmt::Write;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn check_family(premise: &str) -> Result<super::Declarations> {
    let mut source = String::from(
        "trait Leaf {} trait Ready {} trait Outer {} trait Noise {}\n\
         struct Tag; struct Permit; struct Wrapper<T>(T);\n\
         impl Leaf for Tag {} impl Noise for Permit {}\n\
         impl<T: Leaf> Ready for Wrapper<T> {}\n\
         fn need<T: Outer>() {}\n",
    );
    for index in 0..8 {
        writeln!(
            source,
            "struct Root{index}<T>(T);\n\
             impl<T> Outer for Root{index}<T> where Wrapper<Tag>: Ready {{}}\n\
             fn site{index}<P: Clone>(_: P) {premise} {{ need::<Root{index}<u8>>(); }}",
        )?;
    }
    let (report, facts) = super::super::tests::declarations(&source)?;
    Ok(crate::pipeline::analyze(&report, facts)?.facts)
}

/// A source catalog, whose sizing goals on a nominal type first prove the
/// type's own requirements as a formation prefix, and `sites` callers of one
/// such goal. `Holder<Inner<bool>>` forms `Inner<bool>` as its argument.
fn formed_family(sites: usize) -> Result<super::Declarations> {
    let actual = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    let (foundation, _) = actual
        .split_once("pub fn identity<T>")
        .ok_or("actual prerequisite foundation")?;
    let mut source = format!(
        "{foundation}\nimpl Clone for bool{{fn clone(&self)->Self{{*self}}}}impl Copy for bool{{}}\n\
         pub struct Inner<T: Copy>(T);\npub struct Holder<T>(T);\npub fn need<T>() {{}}\n"
    );
    for index in 0..sites {
        writeln!(
            source,
            "pub fn site{index}() {{ need::<Holder<Inner<bool>>>(); }}"
        )?;
    }
    let report = crate::test_source::source_provider(&source, "answer_formation")?;
    Ok(crate::pipeline::analyze_source(&report)?.facts)
}

fn formation_entries(answers: &super::Answers) -> usize {
    answers
        .entries
        .values()
        .filter(|entry| {
            matches!(entry, super::Entry::Proven { steps, .. }
                if steps.iter().any(|step| step.proof == super::Proof::Formation))
        })
        .count()
}

#[test]
fn a_formed_nominal_goal_enters_with_its_formation_and_later_callers_replay_it() -> Result {
    let mut facts = formed_family(8)?;
    let answers = facts.generics.selection_memo.answers();
    let formed = formation_entries(answers);
    assert!(formed >= 1, "formation entries={formed}");
    assert!(answers.counts.hits >= 5, "hits={}", answers.counts.hits);
    Ok(())
}

#[test]
fn a_self_check_proves_each_formed_hit_again_and_agrees() -> Result {
    let _checking = crate::support::sealed::SelfCheck::hold();
    let mut facts = formed_family(8)?;
    let answers = facts.generics.selection_memo.answers();
    assert!(formation_entries(answers) >= 1);
    assert!(
        answers.counts.checked >= 5,
        "checked={}",
        answers.counts.checked
    );
    Ok(())
}

#[test]
fn parametric_callers_replay_one_closed_child_proof() -> Result {
    let mut facts = check_family("")?;
    let answers = facts.generics.selection_memo.answers();
    // Eight roots share `Wrapper<Tag>: Ready` and its `Tag: Leaf` child; the
    // first search of each records it and the later ones replay it.
    assert!(answers.counts.hits >= 5, "hits={}", answers.counts.hits);
    assert!(
        answers.counts.recorded >= 2,
        "recorded={}",
        answers.counts.recorded
    );
    assert_eq!(answers.counts.checked, 0);
    Ok(())
}

#[test]
fn a_closed_caller_premise_keeps_its_search_out_of_the_table() -> Result {
    let open = check_family("")?
        .generics
        .selection_memo
        .answers()
        .counts
        .hits;
    let closed = check_family("where Permit: Noise")?
        .generics
        .selection_memo
        .answers()
        .counts
        .hits;
    assert!(closed < open, "closed={closed} open={open}");
    Ok(())
}

#[test]
fn a_self_check_proves_each_hit_again_and_agrees() -> Result {
    let _checking = crate::support::sealed::SelfCheck::hold();
    let mut facts = check_family("")?;
    let answers = facts.generics.selection_memo.answers();
    assert_eq!(answers.counts.hits, 0);
    assert!(
        answers.counts.checked >= 5,
        "checked={}",
        answers.counts.checked
    );
    Ok(())
}

#[test]
fn an_entry_holds_the_promise_set_its_key_names_so_no_later_set_takes_its_address() -> Result {
    use std::sync::Arc;
    let promised = |item| {
        let mut set = super::Promises::default();
        set.insert(
            crate::types::ItemId(item),
            crate::types::projection::Builtins::default(),
        );
        Arc::new(set)
    };
    let mut facts = check_family("")?;
    let recorded = &mut facts.generics.selection_memo.answers().entries;
    let (key, entry) = recorded.drain().next().ok_or("no recorded answer")?;
    let mut answers = super::Answers::default();
    let first = promised(0);
    let keyed = super::Key {
        promises: Arc::as_ptr(&first).addr(),
        ..key
    };
    assert!(answers.insert(keyed, entry, Some(&first)));
    // The caller's last handle goes away while the entry lives. A freed block
    // is what the allocator hands the next set of the same size, so without
    // the table's hold one of these sets would name the first set's entry.
    drop(first);
    for item in 1..=16 {
        let later = promised(item);
        let probe = super::Key {
            promises: Arc::as_ptr(&later).addr(),
            ..key
        };
        assert!(!answers.entries.contains_key(&probe));
    }
    assert_eq!(answers.promises.len(), 1);
    Ok(())
}

#[test]
fn a_catalog_written_in_place_empties_the_table_at_its_next_lookup() -> Result {
    use crate::resolve::model::DefId;
    use crate::sema::compiler_items::SelectionState;
    let mut facts = check_family("")?;
    let recorded = &mut facts.generics.selection_memo.answers().entries;
    let (key, entry) = recorded.drain().next().ok_or("no recorded answer")?;
    let mut answers = super::Answers::default();
    // The catalog keeps its address, so only its content can tell the two
    // lookups apart.
    let before = SelectionState::default();
    assert!(answers.current(Some(before)).insert(key, entry, None));
    assert!(answers.current(Some(before)).entries.contains_key(&key));
    let after = before.with_destruction(DefId(7));
    assert!(answers.current(Some(after)).entries.is_empty());
    Ok(())
}
