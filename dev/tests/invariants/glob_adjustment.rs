//! The pinned synchronized-glob example and width-independent suffix reuse.
use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn opaque(
    marks: &mut Marks,
    parent: ItemContext,
    expansion: u32,
) -> Result<ItemContext, &'static str> {
    let raw = marks.apply(
        parent.0,
        ExpansionId(expansion),
        Transparency::Opaque,
        SyntaxContextId::ROOT,
    )?;
    marks.item(raw)
}

#[test]
fn physical_globs_do_not_allocate_and_unknown_containers_fail() -> TestResult {
    let mut marks = Marks::default();
    let ancestry = Ancestry::default();
    let answer = marks
        .reverse_glob(
            ItemContext::ROOT,
            ExpansionId(0),
            ItemContext::ROOT,
            &ancestry,
        )?
        .ok_or("missing physical answer")?;
    assert_eq!(answer.context, ItemContext::ROOT);
    assert_eq!(answer.definition, None);
    assert!(marks.0.is_none());
    assert!(
        marks
            .reverse_glob(
                ItemContext::ROOT,
                ExpansionId(1),
                ItemContext::ROOT,
                &ancestry
            )
            .is_err()
    );
    assert!(marks.0.is_none());
    Ok(())
}

#[test]
fn nested_globs_preserve_literal_and_captured_names_separately() -> TestResult {
    let mut marks = Marks::default();
    let mut ancestry = Ancestry::default();
    ancestry.append(ExpansionId(1), ExpansionId(0))?;
    ancestry.append(ExpansionId(2), ExpansionId(1))?;
    ancestry.append(ExpansionId(3), ExpansionId(0))?;
    let outer = opaque(&mut marks, ItemContext::ROOT, 1)?;
    let glob = opaque(&mut marks, outer, 2)?;
    let inner = opaque(&mut marks, ItemContext::ROOT, 2)?;
    let sibling = opaque(&mut marks, ItemContext::ROOT, 3)?;
    let literal = marks
        .reverse_glob(outer, ExpansionId(1), glob, &ancestry)?
        .ok_or("literal hidden")?;
    let captured = marks
        .reverse_glob(ItemContext::ROOT, ExpansionId(1), glob, &ancestry)?
        .ok_or("capture hidden")?;
    assert_eq!(literal.context, glob);
    assert_eq!(captured.context, inner);
    assert_eq!(literal.definition, Some(ExpansionId(2)));
    assert_eq!(captured.definition, Some(ExpansionId(2)));
    assert_ne!(literal.context, outer);
    assert_ne!(captured.context, outer);
    assert!(
        marks
            .reverse_glob(sibling, ExpansionId(1), glob, &ancestry)?
            .is_none()
    );
    Ok(())
}

#[test]
fn malformed_provenance_cannot_publish_a_glob_answer() -> TestResult {
    let mut marks = Marks::default();
    let ancestry = Ancestry::default();
    let unknown = opaque(&mut marks, ItemContext::ROOT, 99)?;
    assert!(
        marks
            .reverse_glob(ItemContext::ROOT, ExpansionId(0), unknown, &ancestry)
            .is_err()
    );
    assert!(
        marks
            .reverse_glob(unknown, ExpansionId(0), ItemContext::ROOT, &ancestry)
            .is_err()
    );
    assert!(
        marks
            .0
            .as_ref()
            .ok_or("missing arena")?
            .reversed_globs
            .is_empty()
    );
    Ok(())
}

#[test]
fn shared_glob_suffixes_cost_depth_plus_names_without_per_name_storage() -> TestResult {
    for depth in [8_u32, 32, 128, 512] {
        for width in [8, 32, 128] {
            let mut marks = Marks::default();
            let mut ancestry = Ancestry::default();
            let mut glob = ItemContext::ROOT;
            for id in 1..=depth {
                ancestry.append(ExpansionId(id), ExpansionId(id - 1))?;
                glob = opaque(&mut marks, glob, id)?;
            }
            let nodes = marks.context_count();
            let mut steps = 0;
            for _ in 0..width {
                let answer = marks
                    .reverse_observed(ItemContext::ROOT, ExpansionId(0), glob, &ancestry, || {
                        steps += 1;
                    })?
                    .ok_or("valid glob hidden")?;
                assert_eq!(answer.context, glob);
                assert_eq!(answer.definition, Some(ExpansionId(1)));
            }
            assert!(steps <= 2 * usize::try_from(depth)? + width);
            assert_eq!(marks.context_count(), nodes);
            assert_eq!(
                marks
                    .0
                    .as_ref()
                    .ok_or("missing arena")?
                    .reversed_globs
                    .len(),
                1
            );
            eprintln!("glob depth={depth} width={width} steps={steps} contexts={nodes}");
        }
    }
    Ok(())
}
