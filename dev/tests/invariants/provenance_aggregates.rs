//! Actual parsed field consumers with supplied opaque token provenance.
//! These do not enable or qualify end-to-end scoped macro expansion.
use crate::{
    expand::marks::{ItemContext, SyntaxContextId, Transparency},
    resolve::{
        expansion::Report,
        model::DefKind,
        unit::{FileId, NodeKey},
    },
    sema::{members::Selection, model::Operation, patterns::Projection},
    support::crates::CrateIdentity,
    syntax::{
        ast::{Kind, NodeId},
        token_tree::Span,
    },
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const ROUNDTRIP: &str = "macro_rules! make { ($record:ident,$answer:ident) => {
pub struct $record { pub value:u32 }
pub fn $answer()->u32 { let record=$record{value:7};
let $record{value:selected}=record;let again=$record{value:selected};again.value }
} } make!(Record,answer);";

fn identifiers(report: &Report) -> TestResult<Vec<(NodeKey, Span)>> {
    let mut fields = Vec::new();
    for (file, unit) in report.package.units.iter().enumerate() {
        for (index, node) in unit.configured.ast.nodes.iter().enumerate() {
            let (Kind::FieldDefinition {
                name: Some(span), ..
            }
            | Kind::FieldInit { name: span, .. }
            | Kind::PatternField { name: span, .. }
            | Kind::Field { name: span, .. }) = node.kind
            else {
                continue;
            };
            if unit.configured.active[index] {
                fields.push((
                    NodeKey {
                        file: FileId(u32::try_from(file)?),
                        node: NodeId::new(u32::try_from(index)?),
                    },
                    span,
                ));
            }
        }
    }
    Ok(fields)
}

fn reads(report: &Report) -> TestResult<Vec<(NodeKey, Span)>> {
    Ok(identifiers(report)?
        .into_iter()
        .filter(|(key, _)| {
            matches!(
                report
                    .package
                    .unit(key.file)
                    .configured
                    .ast
                    .node(key.node)
                    .kind,
                Kind::Field { .. }
            )
        })
        .collect())
}

fn supply_context(
    report: &mut Report,
    fields: &[(NodeKey, Span)],
    invocation: usize,
) -> TestResult<ItemContext> {
    let invocation = report
        .invocations
        .get(invocation)
        .ok_or("actual invocation")?;
    let call_site = report
        .package
        .syntax_context(invocation.node.file, invocation.original.start)?;
    let raw = report.package.marks.apply(
        SyntaxContextId::ROOT,
        invocation.context,
        Transparency::Opaque,
        call_site,
    )?;
    for &(key, span) in fields {
        let unit = &mut report.package.units[key.file.0 as usize];
        assert!(span.start as usize >= unit.source.physical_len());
        let origin = unit
            .origins
            .iter_mut()
            .find(|origin| origin.generated == span)
            .ok_or("actual generated field token origin")?;
        origin.context = Some(raw);
        assert_eq!(report.package.syntax_context(key.file, span.start)?, raw);
    }
    Ok(report.package.marks.item(raw)?)
}

#[test]
fn actual_opaque_field_uses_share_one_selected_identity_and_reject_metadata() -> TestResult {
    let mut report = crate::test_source::resolve(ROUNDTRIP, true)?;
    let fields = identifiers(&report)?;
    assert_eq!(fields.len(), 5);
    let context = supply_context(&mut report, &fields, 0)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let owner = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, definition)| {
            definition.kind == DefKind::Struct
                && report.package.names.get(definition.name) == "Record"
        })
        .map(|(index, _)| u32::try_from(index))
        .ok_or("actual record owner")??;
    let [variant] = facts.members.variants(crate::types::ItemId(owner)) else {
        return Err("actual record variant".into());
    };
    let variant = *variant;
    let field = facts
        .members
        .fields(variant)
        .and_then(|fields| fields.first())
        .ok_or("actual declared field")?;
    let selected = facts
        .members
        .field_in(variant, field.name, context)
        .ok_or("actual contextual field index")?;
    assert_eq!(facts.members.field(variant, field.name), None);
    let mut initializers = 0;
    let mut reads = 0;
    for &(key, _) in &fields {
        match facts.node(key).and_then(|node| node.operation) {
            Some(Operation::FieldInit { field }) => {
                assert_eq!(field, selected);
                initializers += 1;
            }
            Some(Operation::Field { field }) => {
                assert_eq!(field, Selection::Nominal(selected));
                reads += 1;
            }
            _ => {}
        }
    }
    assert_eq!((initializers, reads), (2, 1));
    assert!(!facts.patterns.fields.is_empty());
    for field in &facts.patterns.fields {
        assert_eq!(
            field.selection,
            Projection::Field(Selection::Nominal(selected))
        );
    }
    let error =
        crate::sema::crates::export(&report, &mut facts, CrateIdentity::new("fields", "opaque")?)
            .err()
            .ok_or("contextual fields were silently exported")?;
    assert_eq!(
        error.message,
        "contextual aggregate field metadata is not implemented"
    );
    Ok(())
}

#[test]
fn actual_field_uses_cannot_select_an_unrelated_opaque_container_identity() -> TestResult {
    let source = format!("{ROUNDTRIP} macro_rules! unrelated {{()=>{{}}}} unrelated!();");
    let mut report = crate::test_source::resolve(&source, true)?;
    let fields = identifiers(&report)?;
    supply_context(&mut report, &fields, 0)?;
    let reads = reads(&report)?;
    assert_eq!(reads.len(), 1);
    supply_context(&mut report, &reads, 1)?;
    let error = crate::pipeline::analyze_source(&report)
        .err()
        .ok_or("unrelated field use selected a same-spelling opaque field")?;
    assert_eq!(error.message, "unknown aggregate field");
    Ok(())
}

#[test]
fn actual_literal_field_uses_receive_definition_site_privacy() -> TestResult {
    let source = "mod provider {pub struct Record {hidden:u32}
pub fn make()->Record{Record{hidden:23}}
#[macro_export] macro_rules! read {($record:expr)=>{$record.hidden}}}
pub fn answer()->u32{read!(provider::make())}";
    let mut report = crate::test_source::resolve(source, true)?;
    let fields = reads(&report)?;
    assert_eq!(fields.len(), 1);
    supply_context(&mut report, &fields, 0)?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    assert!(!facts.members.has_contextual_fields());
    assert!(matches!(
        facts.node(fields[0].0).and_then(|node| node.operation),
        Some(Operation::Field {
            field: Selection::Nominal(_)
        })
    ));
    Ok(())
}

#[test]
fn actual_captured_field_tokens_retain_caller_privacy() -> TestResult {
    let source = "mod provider {pub struct Record {hidden:u32}
pub fn make()->Record{Record{hidden:23}}
#[macro_export] macro_rules! read {($record:expr,$field:ident)=>{$record.$field}}}
pub fn answer()->u32{read!(provider::make(),hidden)}";
    let report = crate::test_source::resolve(source, true)?;
    let fields = reads(&report)?;
    assert_eq!(fields.len(), 1);
    assert_eq!(
        report
            .package
            .item_context(fields[0].0.file, fields[0].1.start)?,
        ItemContext::ROOT
    );
    let error = crate::pipeline::analyze_source(&report)
        .err()
        .ok_or("captured field token acquired definition-site privacy")?;
    assert_eq!(error.message, "aggregate field is private");
    Ok(())
}
