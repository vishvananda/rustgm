//! Independent chain expectations and depth/width bounds for actual adjustment.
use super::*;
use crate::expand::marks::{SyntaxContextId, Transparency};
type TestResult = Result<(), &'static str>;

fn mark(
    marks: &mut Marks,
    parent: SyntaxContextId,
    expansion: u32,
) -> Result<SyntaxContextId, &'static str> {
    marks.apply(
        parent,
        ExpansionId(expansion),
        Transparency::Opaque,
        SyntaxContextId::ROOT,
    )
}

#[test]
fn ordinary_identity_keeps_the_arena_unallocated() -> TestResult {
    let marks = Marks::default();
    let ancestry = Ancestry::default();
    assert_eq!(
        marks.adjust_item(ItemContext::ROOT, ExpansionId(0), &ancestry)?,
        Adjusted {
            context: ItemContext::ROOT,
            definition: None
        }
    );
    assert!(marks.0.is_none());
    assert!(
        marks
            .adjust_item(ItemContext::ROOT, ExpansionId(1), &ancestry)
            .is_err()
    );
    assert!(marks.0.is_none());
    Ok(())
}

#[test]
fn actual_container_ancestry_selects_the_last_removed_mark() -> TestResult {
    let mut marks = Marks::default();
    let mut ancestry = Ancestry::default();
    for (id, parent) in [(1, 0), (2, 1), (3, 0), (4, 2)] {
        ancestry.append(ExpansionId(id), ExpansionId(parent))?;
    }
    let first = mark(&mut marks, SyntaxContextId::ROOT, 1)?;
    let second = mark(&mut marks, first, 2)?;
    let context = marks.item(second)?;
    for container in [2, 4] {
        assert_eq!(
            marks.adjust_item(context, ExpansionId(container), &ancestry)?,
            Adjusted {
                context,
                definition: None
            }
        );
    }
    assert_eq!(
        marks.adjust_item(context, ExpansionId(1), &ancestry)?,
        Adjusted {
            context: marks.item(first)?,
            definition: Some(ExpansionId(2))
        }
    );
    for container in [0, 3] {
        assert_eq!(
            marks.adjust_item(context, ExpansionId(container), &ancestry)?,
            Adjusted {
                context: ItemContext::ROOT,
                definition: Some(ExpansionId(1))
            }
        );
    }
    // Transparent/semiopaque wrappers do not become an item privacy owner.
    let mixed = marks.apply(second, ExpansionId(4), Transparency::SemiOpaque, second)?;
    assert_eq!(
        marks.adjust_item(marks.item(mixed)?, ExpansionId(0), &ancestry)?,
        marks.adjust_item(context, ExpansionId(0), &ancestry)?
    );
    Ok(())
}

#[test]
fn rejected_unknown_expansions_do_not_publish_partial_answers() -> TestResult {
    let mut marks = Marks::default();
    let mut ancestry = Ancestry::default();
    ancestry.append(ExpansionId(1), ExpansionId(0))?;
    let unknown = mark(&mut marks, SyntaxContextId::ROOT, 2)?;
    let outer = mark(&mut marks, unknown, 1)?;
    let context = marks.item(outer)?;
    assert!(
        marks
            .adjust_item(context, ExpansionId(0), &ancestry)
            .is_err()
    );
    let data = marks.0.as_ref().ok_or("missing marked arena")?;
    assert!(
        data.adjusted
            .try_borrow()
            .map_err(|_| "borrowed cache")?
            .is_empty()
    );
    ancestry.append(ExpansionId(2), ExpansionId(0))?;
    assert_eq!(
        marks.adjust_item(context, ExpansionId(0), &ancestry)?,
        Adjusted {
            context: ItemContext::ROOT,
            definition: Some(ExpansionId(2))
        }
    );
    Ok(())
}

#[test]
fn shared_suffix_queries_charge_depth_once_and_each_use_once() -> TestResult {
    for depth in [8, 32, 128, 512] {
        for width in [8, 32, 128] {
            let mut marks = Marks::default();
            let mut ancestry = Ancestry::default();
            let mut context = SyntaxContextId::ROOT;
            for id in 1..=depth {
                ancestry.append(ExpansionId(id), ExpansionId(id - 1))?;
                context = mark(&mut marks, context, id)?;
            }
            let mut steps = 0;
            for query in 1..=width {
                let id = depth + query;
                ancestry.append(ExpansionId(id), ExpansionId(depth))?;
                let leaf = mark(&mut marks, context, id)?;
                let answer =
                    marks.adjust_observed(marks.item(leaf)?, ExpansionId(0), &ancestry, || {
                        steps += 1;
                    })?;
                assert_eq!(
                    answer,
                    Adjusted {
                        context: ItemContext::ROOT,
                        definition: Some(ExpansionId(1))
                    }
                );
                assert_eq!(
                    marks.adjust_observed(marks.item(leaf)?, ExpansionId(0), &ancestry, || {
                        steps += 1;
                    })?,
                    answer
                );
            }
            assert_eq!(steps, depth + 3 * width);
            let data = marks.0.as_ref().ok_or("missing marked arena")?;
            assert_eq!(
                data.adjusted
                    .try_borrow()
                    .map_err(|_| "borrowed cache")?
                    .len(),
                (depth + width) as usize
            );
            eprintln!(
                "item-adjustment depth={depth} width={width} steps={steps} entries={}",
                depth + width
            );
        }
    }
    Ok(())
}
