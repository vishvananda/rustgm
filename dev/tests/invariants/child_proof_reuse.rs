//! Cross-root reuse retains the complete eligible child proof in body search.
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
             fn site{index}() {premise} {{ need::<Root{index}<u8>>(); }}",
        )?;
    }
    let (report, facts) = super::super::tests::declarations(&source)?;
    Ok(crate::pipeline::analyze(&report, facts)?.facts)
}

#[test]
fn distinct_body_roots_reuse_closed_children_and_their_descendants() -> Result {
    let facts = check_family("")?;
    let work = &facts.generics.selection_memo.child_work;
    assert!(work.hits >= 5, "{}", work.hits);
    let memo = facts
        .generics
        .selection_memo
        .children()
        .ok_or("missing child recipes")?;
    assert_eq!(memo.entries.len(), 2);
    let proof = memo
        .entries
        .values()
        .find(|proof| proof.steps.len() == 2)
        .ok_or("missing Ready proof and its Leaf child")?;
    assert_eq!(proof.steps[1].children, [0]);
    assert_ne!(proof.steps[0].owner, proof.steps[1].owner);
    assert!(proof.steps[0].children.is_empty());
    Ok(())
}

#[test]
fn an_unrelated_caller_premise_keeps_child_search_local() -> Result {
    let facts = check_family("where Permit: Noise")?;
    let memo = &facts.generics.selection_memo;
    assert_eq!(memo.child_work.hits, 0);
    assert!(
        memo.children()
            .is_none_or(|proofs| proofs.entries.is_empty())
    );
    Ok(())
}
