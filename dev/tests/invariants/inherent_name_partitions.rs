//! Inherent name partitions preserve lookup work and caller-specific privacy.
use super::*;
use crate::expand::marks::{ExpansionId, Marks, SyntaxContextId, Transparency};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn mark(
    marks: &mut Marks,
    parent: SyntaxContextId,
    id: u32,
) -> Result<SyntaxContextId, &'static str> {
    marks.apply(
        parent,
        ExpansionId(id),
        Transparency::Opaque,
        SyntaxContextId::ROOT,
    )
}

#[test]
fn ordinary_members_keep_the_existing_storage_and_invalidate_only_their_index() -> TestResult {
    let mut methods = Methods::default();
    for index in 0..128 {
        let key = (ItemId(index), NameId(0));
        methods.publish_inherent(key, ItemContext::ROOT, DefId(index + 1));
        methods.retain_candidate_index(key, ItemContext::ROOT, Candidates::default())?;
    }
    assert!(methods.contextual.is_none());
    assert_eq!(methods.inherent_groups().count(), 128);
    let key = (ItemId(0), NameId(0));
    methods.publish_inherent(key, ItemContext::ROOT, DefId(1000));
    assert!(methods.candidate_index(key, ItemContext::ROOT).is_none());
    assert!(
        methods
            .candidate_index((ItemId(1), NameId(0)), ItemContext::ROOT)
            .is_some()
    );
    assert_eq!(
        methods.inherent_members(key, ItemContext::ROOT),
        Some(&[DefId(1), DefId(1000)][..])
    );
    Ok(())
}

#[test]
fn sibling_opaque_names_do_not_multiply_candidate_queries_by_declaration_width() -> TestResult {
    let key = (ItemId(0), NameId(0));
    for width in [8_u32, 32, 128] {
        let mut methods = Methods::default();
        let mut report = crate::test_source::resolve("", true)?;
        let mut contexts = Vec::new();
        for index in 0..width {
            let marks = &mut report.package.marks;
            let syntax = mark(marks, SyntaxContextId::ROOT, index + 1)?;
            let context = marks.item(syntax)?;
            methods.publish_inherent(key, context, DefId(index));
            contexts.push(context);
        }
        methods.freeze_inherent_names();
        assert!(methods.inherent.get(&key).is_some_and(Vec::is_empty));
        for (index, context) in contexts.into_iter().enumerate() {
            let mut cursor = Cursor::new(context);
            assert_eq!(cursor.next(&mut methods, &report, key)?, Some(context));
            assert_eq!(
                methods.inherent_members(key, context),
                Some(&[DefId(u32::try_from(index)?)][..])
            );
            assert_eq!(cursor.next(&mut methods, &report, key)?, None);
        }
        let work = methods
            .contextual
            .as_ref()
            .ok_or("missing contextual index")?
            .steps;
        assert_eq!(work, width as usize);
        println!("member-name-partitions width={width} queries={width} ancestor-visits={work}");
    }
    Ok(())
}

#[test]
fn nested_use_walks_only_its_own_ancestry_and_cold_shapes_follow_exact_contexts() -> TestResult {
    let key = (ItemId(0), NameId(0));
    let mut methods = Methods::default();
    let mut report = crate::test_source::resolve("", true)?;
    let marks = &mut report.package.marks;
    let parent = mark(marks, SyntaxContextId::ROOT, 1)?;
    let child = mark(marks, parent, 2)?;
    let sibling = mark(marks, SyntaxContextId::ROOT, 3)?;
    let (parent, child, sibling) = (
        marks.item(parent)?,
        marks.item(child)?,
        marks.item(sibling)?,
    );
    for (context, member) in [(parent, 3), (child, 2), (sibling, 1)] {
        methods.publish_inherent(key, context, DefId(member));
        methods.retain_candidate_index(key, context, Candidates::default())?;
    }
    let mut cursor = Cursor::new(child);
    assert_eq!(cursor.next(&mut methods, &report, key)?, Some(child));
    assert_eq!(cursor.next(&mut methods, &report, key)?, Some(parent));
    assert_eq!(cursor.next(&mut methods, &report, key)?, None);
    methods.publish_inherent(key, child, DefId(4));
    assert!(methods.candidate_index(key, child).is_none());
    assert!(methods.candidate_index(key, parent).is_some());
    assert!(methods.candidate_index(key, sibling).is_some());
    assert_eq!(methods.contextual.as_ref().ok_or("missing index")?.steps, 2);
    Ok(())
}

#[test]
fn identity_lookup_exhaustion_counts_the_attempt_before_returning() -> TestResult {
    let key = (ItemId(0), NameId(0));
    let mut methods = Methods::default();
    let mut report = crate::test_source::resolve("", true)?;
    let marks = &mut report.package.marks;
    let syntax = mark(marks, SyntaxContextId::ROOT, 1)?;
    let context = marks.item(syntax)?;
    methods.publish_inherent(key, context, DefId(1));
    methods.contextual.as_mut().ok_or("missing index")?.steps = 1_000_000;
    assert!(
        Cursor::new(context)
            .next(&mut methods, &report, key)
            .is_err()
    );
    assert_eq!(
        methods.contextual.as_ref().ok_or("missing index")?.steps,
        1_000_001
    );
    Ok(())
}

#[test]
fn actual_source_receiver_and_path_sites_retain_their_identifier_context() -> TestResult {
    let report = crate::test_source::resolve(
        "struct S; impl S { fn method<T>(&self, x:T)->T{x} } fn use_it(s:S)->u32{s.method::<u32>(7u32)}",
        true,
    )?;
    let (&key, &call) = report
        .resolution
        .method_calls
        .iter()
        .next()
        .ok_or("actual method use")?;
    let site = Use::receiver(&report, key, call)?;
    let segment = call.method.ok_or("actual generic method segment")?;
    let path = Use::path(
        &report,
        NodeKey {
            node: segment,
            ..key
        },
        call.scope,
    )?;
    assert_eq!(site.context, path.context);
    assert_eq!(site.caller, path.caller);
    assert_eq!(site.context, ItemContext::ROOT);
    let member = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "method")
        .ok_or("actual method")?;
    assert_eq!(
        report.definition_item_context(DefId(u32::try_from(member)?))?,
        ItemContext::ROOT
    );
    assert!(report.definition_item_context(DefId(u32::MAX)).is_err());
    println!(
        "member-use={} methods={}",
        size_of::<Use>(),
        size_of::<Methods>()
    );
    Ok(())
}

#[test]
fn actual_impl_containers_adjust_privacy_without_reusing_another_callers_access() -> TestResult {
    use crate::resolve::model::DefKind;
    let mut report = crate::test_source::resolve(
        "mod definition { pub struct S; impl S { fn private() {} pub fn public() {} }
         macro_rules! make { () => { impl S { fn generated() {} } } } make!(); }
         mod caller {}",
        true,
    )?;
    let expansion = report
        .invocations
        .first()
        .ok_or("actual invocation")?
        .context;
    let definition = report.definitions[report.invocations[0].definition].module;
    let caller = report
        .package
        .modules
        .iter()
        .position(|module| report.package.names.get(module.name) == "caller")
        .ok_or("actual caller")?;
    let caller = ModuleId(u32::try_from(caller)?);
    let member = |name: &str| -> Result<(DefId, DefId), Box<dyn std::error::Error>> {
        let (index, member) = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find(|(_, item)| {
                item.kind == DefKind::Function && report.package.names.get(item.name) == name
            })
            .ok_or("actual member")?;
        let owner = report
            .resolution
            .definitions
            .iter()
            .position(|item| item.kind == DefKind::SelfType && item.scope == member.scope)
            .ok_or("actual impl owner")?;
        Ok((DefId(u32::try_from(owner)?), DefId(u32::try_from(index)?)))
    };
    let (owner, private) = member("private")?;
    let (_, public) = member("public")?;
    let (generated_owner, generated) = member("generated")?;
    // Supplied opaque context tests the consumer API; it does not admit scoped invocation.
    let syntax = report.package.marks.apply(
        SyntaxContextId::ROOT,
        expansion,
        Transparency::Opaque,
        SyntaxContextId::ROOT,
    )?;
    let literal = Use {
        context: report.package.marks.item(syntax)?,
        caller,
    };
    let captured = Use {
        context: ItemContext::ROOT,
        caller,
    };
    for _ in 0..2 {
        assert_eq!(literal.matching(&report, owner, private)?, Some(definition));
        assert!(literal.permits(&report, owner, private)?);
        assert!(!captured.permits(&report, owner, private)?);
        assert!(captured.permits(&report, owner, public)?);
        assert_eq!(literal.matching(&report, generated_owner, generated)?, None);
        assert!(!captured.permits(&report, generated_owner, generated)?);
    }
    assert!(literal.permits(&report, DefId(u32::MAX), private).is_err());
    assert!(literal.permits(&report, owner, DefId(u32::MAX)).is_err());
    report.resolution.definitions[public.0 as usize].access = Access::ExternalPrivate;
    assert!(!literal.permits(&report, owner, public)?);
    Ok(())
}
