//! Normalization and prefix sharing must not collapse independent expansions.
use super::*;
type TestResult = Result<(), &'static str>;

#[test]
fn physical_contexts_need_no_allocation_and_normalized_records_stay_compact() -> TestResult {
    let marks = Marks::default();
    let root = SyntaxContextId::ROOT;
    assert_eq!(size_of::<Marks>(), 8);
    assert_eq!(size_of::<Node>(), 20);
    assert_eq!(size_of::<Key>(), 12);
    assert_eq!(size_of::<ItemContext>(), 4);
    assert_eq!(size_of::<LocalContext>(), 4);
    assert_eq!(marks.outer(root)?, None);
    assert_eq!(marks.item_parent(marks.item(root)?)?, None);
    assert_eq!(marks.local_parent(marks.local(root)?)?, None);
    assert!(marks.0.is_none());
    Ok(())
}

#[test]
fn projections_preserve_only_the_marks_that_own_each_namespace() -> TestResult {
    let mut marks = Marks::default();
    let root = SyntaxContextId::ROOT;
    let opaque = marks.apply(root, ExpansionId(1), Transparency::Opaque, root)?;
    let local = marks.apply(opaque, ExpansionId(2), Transparency::SemiOpaque, root)?;
    let transparent = marks.apply(local, ExpansionId(3), Transparency::Transparent, root)?;
    assert_eq!(marks.item(transparent)?, marks.item(opaque)?);
    assert_eq!(marks.local(transparent)?, marks.local(local)?);
    assert_eq!(marks.item(local)?, marks.item(opaque)?);
    assert_ne!(marks.local(local)?, marks.local(opaque)?);
    assert_eq!(marks.outer(transparent)?, Some(ExpansionId(3)));
    assert_eq!(
        marks.local_parent(marks.local(local)?)?,
        Some((marks.local(opaque)?, ExpansionId(2)))
    );
    assert_eq!(
        marks.item_parent(marks.item(transparent)?)?,
        Some((marks.item(root)?, ExpansionId(1)))
    );
    let count = marks.0.as_ref().ok_or("missing marks")?.nodes.len();
    assert_eq!(
        marks.apply(local, ExpansionId(3), Transparency::Transparent, root)?,
        transparent
    );
    assert_eq!(marks.0.as_ref().ok_or("missing marks")?.nodes.len(), count);
    Ok(())
}

#[test]
fn mixed_prefixes_are_shared_but_actual_invocations_remain_distinct() -> TestResult {
    let mut marks = Marks::default();
    let root = SyntaxContextId::ROOT;
    let call = marks.apply(root, ExpansionId(1), Transparency::Opaque, root)?;
    let mut definition = root;
    for index in 2..130 {
        definition = marks.apply(
            definition,
            ExpansionId(index),
            Transparency::SemiOpaque,
            root,
        )?;
    }
    let first = marks.apply(definition, ExpansionId(200), Transparency::SemiOpaque, call)?;
    let steps = marks.0.as_ref().ok_or("missing marks")?.rebase_steps;
    assert_eq!(steps, 128);
    for index in 201..329 {
        let next = marks.apply(
            definition,
            ExpansionId(index),
            Transparency::SemiOpaque,
            call,
        )?;
        assert_ne!(next, first);
        assert_ne!(marks.local(next)?, marks.local(first)?);
        assert_eq!(marks.item(next)?, marks.item(call)?);
    }
    assert_eq!(marks.0.as_ref().ok_or("missing marks")?.rebase_steps, steps);
    let other = marks.apply(root, ExpansionId(400), Transparency::Opaque, root)?;
    let second = marks.apply(
        definition,
        ExpansionId(401),
        Transparency::SemiOpaque,
        other,
    )?;
    assert_ne!(marks.item(second)?, marks.item(first)?);
    assert_eq!(
        marks.0.as_ref().ok_or("missing marks")?.rebase_steps,
        2 * steps
    );
    Ok(())
}

#[test]
fn literal_parents_and_call_site_policy_are_independent_of_macro_name_context() -> TestResult {
    let mut marks = Marks::default();
    let root = SyntaxContextId::ROOT;
    let outer = marks.apply(root, ExpansionId(1), Transparency::Opaque, root)?;
    let call = marks.apply(root, ExpansionId(2), Transparency::SemiOpaque, root)?;
    // One generated definition may have a captured root name and opaque body.
    let name = marks.apply(root, ExpansionId(3), Transparency::Opaque, call)?;
    let literal = marks.apply(outer, ExpansionId(3), Transparency::Opaque, call)?;
    assert_ne!(marks.item(name)?, marks.item(literal)?);
    assert_eq!(
        marks.item_parent(marks.item(name)?)?,
        Some((marks.item(root)?, ExpansionId(3)))
    );
    assert_eq!(
        marks.item_parent(marks.item(literal)?)?,
        Some((marks.item(outer)?, ExpansionId(3)))
    );
    let transparent = marks.apply(root, ExpansionId(4), Transparency::Transparent, call)?;
    let semi = marks.apply(root, ExpansionId(5), Transparency::SemiOpaque, call)?;
    assert_eq!(marks.local(transparent)?, marks.local(call)?);
    assert_eq!(
        marks.local_parent(marks.local(semi)?)?,
        Some((marks.local(root)?, ExpansionId(5)))
    );
    assert_eq!(marks.item(transparent)?, marks.item(root)?);
    assert_eq!(marks.item(semi)?, marks.item(root)?);
    Ok(())
}

#[test]
fn malformed_roots_and_contexts_do_not_create_partial_marks() {
    let mut marks = Marks::default();
    let root = SyntaxContextId::ROOT;
    assert!(
        marks
            .apply(root, ExpansionId(0), Transparency::Opaque, root)
            .is_err()
    );
    assert!(
        marks
            .apply(
                SyntaxContextId(1),
                ExpansionId(1),
                Transparency::Opaque,
                root
            )
            .is_err()
    );
    assert!(
        marks
            .apply(
                root,
                ExpansionId(1),
                Transparency::Opaque,
                SyntaxContextId(1)
            )
            .is_err()
    );
    assert!(marks.0.is_none());
}
