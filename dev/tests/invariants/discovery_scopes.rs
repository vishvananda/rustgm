//! Actual original block categories constrain definition-site trait discovery.
//! Supplied opaque uses isolate consumers while scoped invocation stays disabled.
use super::*;
use crate::{
    expand::marks::{ExpansionId, SyntaxContextId, Transparency},
    sema::methods::names::Use,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn source(body: &str, imported: &str) -> String {
    format!(
        "mod operators {{
        pub trait Value {{ fn value(&self) -> u8; }}
        pub trait Other {{ fn value(&self) -> u8; }}
    }}
    mod definition {{ {imported}
        #[macro_export] macro_rules! literal {{ ($x:expr) => {{ $x.value() }} }}
        #[macro_export] macro_rules! empty {{ () => {{ }} }}
    }}
    use operators::Value;
    fn answer() -> u8 {{ {body} }}"
    )
}

fn input(
    body: &str,
    imported: &str,
) -> TestResult<(Report, Declarations, crate::resolve::model::MethodCall, Use)> {
    let mut report = crate::test_source::resolve(&source(body, imported), true)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let (&key, &site) = report
        .resolution
        .method_calls
        .iter()
        .next()
        .ok_or("actual method")?;
    let mut identifier = Use::receiver(&report, key, site)?;
    let invocation = report
        .invocations
        .iter()
        .find(|invocation| {
            report
                .package
                .names
                .get(report.definitions[invocation.definition].name)
                == "literal"
        })
        .ok_or("actual literal invocation")?;
    let syntax = report.package.marks.apply(
        SyntaxContextId::ROOT,
        invocation.context,
        Transparency::Opaque,
        SyntaxContextId::ROOT,
    )?;
    identifier.context = report.package.marks.item(syntax)?;
    Ok((report, facts, site, identifier))
}

fn owner_names(
    report: &Report,
    facts: &Declarations,
    methods: &[DefId],
) -> TestResult<Vec<String>> {
    methods
        .iter()
        .map(|&member| {
            let owner = facts
                .associated
                .trait_owner(member)
                .ok_or("actual trait owner")?;
            Ok(report
                .package
                .names
                .get(report.resolution.definitions[owner.0 as usize].name)
                .to_owned())
        })
        .collect()
}

#[test]
fn actual_block_categories_preserve_caller_traits_or_rewind_before_the_parent() -> TestResult {
    for (body, expected) in [
        ("return literal!(0u8);", true),
        ("let result = literal!(0u8); result", true),
        ("literal!(0u8)", true),
        ("return { literal!(0u8) };", true),
        ("const UNUSED: u8 = 0; return literal!(0u8);", false),
        ("literal!{0u8}", false),
        ("literal!(0u8); 7", false),
        ("empty!(); return literal!(0u8);", false),
    ] {
        let (report, facts, site, identifier) = input(body, "")?;
        let mut index = Index::new(&report, &facts)?;
        assert!(
            index.contextual.is_none(),
            "ordinary declared members allocate no context partition"
        );
        for _ in 0..2 {
            let mut groups =
                index.groups(&report, site.scope, site.name, &[], identifier.context)?;
            identifier.retain_traits(&report, &facts, &mut groups)?;
            let names = owner_names(&report, &facts, &groups[1])?;
            assert_eq!(
                names,
                if expected { vec!["Value"] } else { vec![] },
                "{body}"
            );
        }
        // The same lexical site with a captured method name still reaches caller imports.
        let captured = Use {
            context: ItemContext::ROOT,
            ..identifier
        };
        let mut groups = index.groups(&report, site.scope, site.name, &[], captured.context)?;
        captured.retain_traits(&report, &facts, &mut groups)?;
        assert_eq!(
            owner_names(&report, &facts, &groups[1])?,
            ["Value"],
            "{body}"
        );
    }
    Ok(())
}

#[test]
fn definition_and_caller_imports_are_distinct_from_the_methods_container() -> TestResult {
    for (body, imported, expected) in [
        (
            "return literal!(0u8);",
            "use crate::operators::Other;",
            vec!["Other", "Value"],
        ),
        (
            "use operators::Other; return literal!(0u8);",
            "",
            vec!["Other"],
        ),
        (
            "const UNUSED: u8 = 0; return literal!(0u8);",
            "use crate::operators::Other;",
            vec!["Other"],
        ),
    ] {
        let (report, facts, site, identifier) = input(body, imported)?;
        let mut index = Index::new(&report, &facts)?;
        for _ in 0..2 {
            let mut groups =
                index.groups(&report, site.scope, site.name, &[], identifier.context)?;
            identifier.retain_traits(&report, &facts, &mut groups)?;
            let mut names = owner_names(&report, &facts, &groups[1])?;
            names.sort();
            assert_eq!(names, expected, "{body}");
        }
    }
    Ok(())
}

#[test]
fn scope_walk_rejects_unknown_definition_sites_and_counts_failed_work() -> TestResult {
    let (mut report, facts, site, identifier) = input("return literal!(0u8);", "")?;
    assert!(
        report
            .trait_scope_parent(ScopeId(u32::MAX), identifier.context)
            .is_err()
    );
    let syntax = report.package.marks.apply(
        SyntaxContextId::ROOT,
        ExpansionId(99),
        Transparency::Opaque,
        SyntaxContextId::ROOT,
    )?;
    let foreign = report.package.marks.item(syntax)?;
    let mut index = Index::new(&report, &facts)?;
    assert!(
        index
            .groups(&report, site.scope, site.name, &[], foreign)
            .is_err()
    );
    index.steps = 1_000_000;
    assert!(
        index
            .groups(&report, site.scope, site.name, &[], identifier.context)
            .is_err()
    );
    assert!(index.steps > 1_000_000);
    assert_eq!(size_of::<crate::sema::methods::Methods>(), 312);
    assert_eq!(size_of::<Declarations>(), 7032 + size_of::<usize>());
    assert_eq!(size_of::<crate::resolve::model::Scope>(), 32);
    Ok(())
}

#[test]
fn sibling_trait_member_queries_copy_only_their_own_context_partition() -> TestResult {
    for width in [8_u32, 32, 128] {
        let mut report = crate::test_source::resolve("", true)?;
        let name = report.package.names.intern("selected")?;
        let scope = ScopeId(0);
        let mut index = Index::default();
        let mut contexts = Vec::new();
        for number in 1..=width {
            let expansion = ExpansionId(number);
            report
                .resolution
                .introductions
                .invocation(expansion, ExpansionId(0), scope, None)?;
            let syntax = report.package.marks.apply(
                SyntaxContextId::ROOT,
                expansion,
                Transparency::Opaque,
                SyntaxContextId::ROOT,
            )?;
            let context = report.package.marks.item(syntax)?;
            let owner = DefId(number);
            index.scopes.entry(scope).or_default().insert(owner);
            index
                .contextual
                .get_or_insert_default()
                .members
                .entry(owner)
                .or_default()
                .insert((name, context), DefId(number + width));
            contexts.push(context);
        }
        for (number, context) in contexts.into_iter().enumerate() {
            let groups = index.groups(&report, scope, name, &[], context)?;
            assert_eq!(groups[1], [DefId(u32::try_from(number)? + 1 + width)]);
        }
        println!(
            "trait-member-partitions width={width} steps={}",
            index.steps()
        );
        assert!(index.steps() <= 20 * width as usize);
    }
    Ok(())
}

#[test]
fn opaque_trait_bindings_still_contribute_captured_member_names() -> TestResult {
    use crate::resolve::model::Namespace;
    let (mut report, facts, site, identifier) = input("return literal!(0u8);", "")?;
    let scope = report.resolution.module_scopes[0];
    let name = report.package.names.intern("Value")?;
    let key = (scope, name, Namespace::Type, ItemContext::ROOT);
    let binding = *report
        .resolution
        .bindings
        .get_key(key)
        .ok_or("actual caller import")?;
    report.resolution.bindings.remove_key(key);
    report
        .resolution
        .bindings
        .insert_key((scope, name, Namespace::Type, identifier.context), binding)?;
    let mut index = Index::new(&report, &facts)?;
    let captured = Use {
        context: ItemContext::ROOT,
        ..identifier
    };
    for _ in 0..2 {
        let mut groups = index.groups(&report, site.scope, site.name, &[], captured.context)?;
        captured.retain_traits(&report, &facts, &mut groups)?;
        assert_eq!(owner_names(&report, &facts, &groups[1])?, ["Value"]);
    }
    Ok(())
}
