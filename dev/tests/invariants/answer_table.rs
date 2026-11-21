//! Closed goals share one crate-wide answer under callers whose premises all
//! name a parameter; a self-check re-proves every hit instead of replaying it.
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
