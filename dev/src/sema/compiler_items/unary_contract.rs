//! Demanded source unary contracts borrow real headers, never a bootstrap factory.
use super::{Declaration, DefId, Error, LangItem, Report, headers};
use crate::{
    resolve::{model::DefKind, unit::NodeKey},
    sema::{header_attributes::ContractKnowledge, model::Declarations},
    syntax::{ast::Kind as Syntax, item::ItemKind},
    types::{Kind, TypeId, callable::Convention},
};

/// Identity and signature only; ordinary trait, effect, body and call proof follows.
pub(in crate::sema) struct SourceUnary {
    pub definition: DefId,
    pub method: DefId,
    pub signature: TypeId,
}
// Preserve the existing Not entry's descriptor type and callers.
pub(in crate::sema) type SourceNot = SourceUnary;
#[derive(Clone, Copy)]
enum Operator {
    Not,
    Neg,
}
impl Operator {
    const fn role(self) -> LangItem {
        match self {
            Self::Not => LangItem::Not,
            Self::Neg => LangItem::Neg,
        }
    }
    const fn method(self) -> &'static str {
        match self {
            Self::Not => "not",
            Self::Neg => "neg",
        }
    }
}
enum Failure {
    Header(Error),
    Contract(&'static str),
}
impl From<Error> for Failure {
    fn from(error: Error) -> Self {
        Self::Header(error)
    }
}
impl From<&'static str> for Failure {
    fn from(message: &'static str) -> Self {
        Self::Contract(message)
    }
}
struct Work(usize);
impl Work {
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.0 = self.0.saturating_add(count);
        if self.0 > crate::support::crates::MAX_COMPONENTS {
            return Err("source unary declaration query exceeds its work budget");
        }
        Ok(())
    }
}
pub(in crate::sema) fn source_not(
    report: &Report,
    facts: &mut Declarations,
    site: NodeKey,
) -> Result<SourceNot, Error> {
    source_unary(report, facts, site, Operator::Not)
}
pub(in crate::sema) fn source_neg(
    report: &Report,
    facts: &mut Declarations,
    site: NodeKey,
) -> Result<SourceUnary, Error> {
    source_unary(report, facts, site, Operator::Neg)
}
fn source_unary(
    report: &Report,
    facts: &mut Declarations,
    site: NodeKey,
    operator: Operator,
) -> Result<SourceUnary, Error> {
    let mut work = Work(facts.body_work.canonical_steps);
    let result = checked(report, facts, operator, &mut work).map_err(|failure| match failure {
        Failure::Header(error) => error,
        Failure::Contract(message) => Declarations::error(report, site, message),
    });
    // Successful and failed attempts both belong to this existing work owner.
    facts.body_work.canonical_steps = work.0;
    result
}
fn checked(
    report: &Report,
    facts: &Declarations,
    operator: Operator,
    work: &mut Work,
) -> Result<SourceUnary, Failure> {
    work.charge(1)?;
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source unary has no actual source catalog")?;
    catalog.require_provider_report(report)?;
    let declaration = catalog.language(operator.role()).ok_or(match operator {
        Operator::Not => "source ! requires its actual Not language declaration",
        Operator::Neg => "source - requires its actual Neg language declaration",
    })?;
    current_header(report, facts, declaration.definition, declaration.origin)?;
    let header = headers::Trait::source(report, facts, declaration)?;
    header.profile(
        &facts.types,
        headers::Flags {
            unsafe_: false,
            auto_: false,
            constant: header.flags.constant,
        },
        0,
    )?;
    if header.written.parameters != 0
        || header.written.supertraits != 0
        || header.members.len() != 2
    {
        return Err("source unary requires its actual Output and operation members without extra parameters or supertraits".into());
    }
    let (output, method) = members(report, facts, declaration, &header, operator, work)?;
    let signature = method_signature(report, facts, &header, output, method, work)?;
    Ok(SourceUnary {
        definition: declaration.definition,
        method,
        signature,
    })
}
fn current_header(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
    origin: NodeKey,
) -> Result<(), Failure> {
    let current = facts.header_attribute_contracts(report, definition)?;
    if current
        .origin
        .source()
        .is_none_or(|site| site.declaration != origin)
        || matches!(current.must_use, ContractKnowledge::Unknown)
        || matches!(current.inline, ContractKnowledge::Unknown)
    {
        return Err("source unary requires positive completed source-header authentication".into());
    }
    Ok(())
}
fn members(
    report: &Report,
    facts: &Declarations,
    declaration: Declaration,
    header: &headers::Trait<'_>,
    operator: Operator,
    work: &mut Work,
) -> Result<(DefId, DefId), Failure> {
    let unit = report.package.unit(declaration.origin.file);
    let Syntax::Item(index) = unit.configured.ast.node(declaration.origin.node).kind else {
        return Err("source unary lost its actual trait item".into());
    };
    let ItemKind::Trait { items, .. } = unit.configured.ast.items[index as usize].kind else {
        return Err("source unary lost its actual trait declaration".into());
    };
    let raw = unit.configured.ast.list(items);
    work.charge(raw.len().saturating_add(1))?;
    if raw.len() > 64 {
        return Err("source unary member syntax exceeds its bounded declaration profile".into());
    }
    let mut actual = [None; 2];
    let mut count = 0;
    for &node in raw {
        if unit.configured.active.get(node.get() as usize) != Some(&true) {
            continue;
        }
        let mut bare = node;
        for _ in 0..64 {
            work.charge(1)?;
            match unit
                .configured
                .ast
                .nodes
                .get(bare.get() as usize)
                .map(|n| n.kind)
            {
                Some(Syntax::Attributed { value, .. }) => bare = value,
                Some(Syntax::Item(_)) => break,
                _ => return Err("source unary member lost its actual item".into()),
            }
        }
        if !matches!(
            unit.configured
                .ast
                .nodes
                .get(bare.get() as usize)
                .map(|node| node.kind),
            Some(Syntax::Item(_))
        ) || count == actual.len()
        {
            return Err("source unary has an unsupported or cyclic active member list".into());
        }
        actual[count] = Some(NodeKey {
            file: declaration.origin.file,
            node: bare,
        });
        count += 1;
    }
    if count != 2 {
        return Err("source unary has an incomplete actual member list".into());
    }
    let mut output = None;
    let mut method = None;
    for &member in header.members {
        work.charge(3)?;
        let definition = report
            .resolution
            .definitions
            .get(member.0 as usize)
            .ok_or("source unary has a missing member definition")?;
        if !actual.contains(&definition.node)
            || facts.associated.trait_owner(member) != Some(declaration.definition)
        {
            return Err("source unary member belongs to another current source owner".into());
        }
        match (definition.kind, report.package.names.get(definition.name)) {
            (DefKind::TypeAlias, "Output") if output.is_none() => {
                output_item(
                    report,
                    definition
                        .node
                        .ok_or("source unary Output lost its source item")?,
                )?;
                output = Some(member);
            }
            (DefKind::Function, name) if name == operator.method() && method.is_none() => {
                method = Some(member);
            }
            _ => {
                return Err(
                    "source unary requires its actual Output type and operation function".into(),
                );
            }
        }
    }
    Ok((
        output.ok_or("source unary has no Output")?,
        method.ok_or("source unary has no operation method")?,
    ))
}
fn output_item(report: &Report, origin: NodeKey) -> Result<(), Failure> {
    let unit = report.package.unit(origin.file);
    let Syntax::Item(index) = unit.configured.ast.node(origin.node).kind else {
        return Err("source unary Output is not its actual type item".into());
    };
    let item = unit.configured.ast.items[index as usize];
    if !matches!(item.kind, ItemKind::TypeAlias { ty: None, .. })
        || !unit.configured.ast.list(item.generics).is_empty()
    {
        return Err("source unary Output requires its actual nongeneric required type".into());
    }
    Ok(())
}
fn method_signature(
    report: &Report,
    facts: &Declarations,
    header: &headers::Trait<'_>,
    output: DefId,
    method: DefId,
    work: &mut Work,
) -> Result<TypeId, Failure> {
    work.charge(4)?;
    let origin = report.resolution.definitions[method.0 as usize]
        .node
        .ok_or("source unary method has no source origin")?;
    current_header(report, facts, method, origin)?;
    method_syntax(report, facts, method, origin)?;
    let function = facts
        .declaration_header(method)?
        .ok_or("source unary has no completed method header")?;
    if function.declaration.source() != Some(origin)
        || function.body.is_some()
        || function.universals != 0
        || facts
            .generics
            .parameters
            .get(&method)
            .is_some_and(|p| !p.is_empty())
    {
        return Err(
            "source unary requires its actual required method without extra binders".into(),
        );
    }
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err(
            "source unary method requires a safe Rust signature without late binders".into(),
        );
    };
    if facts.types.list(parameters) != Some([header.self_type].as_slice()) {
        return Err("source unary method requires its exact by-value Self receiver".into());
    }
    let expected = facts
        .associated
        .types
        .get(&output)
        .ok_or("source unary Output lacks its actual projection contract")?
        .projection;
    let Some(Kind::Projection(application)) = facts.types.kind(expected) else {
        return Err("source unary Output is not its declared projection".into());
    };
    let owner = facts
        .types
        .trait_application(application.trait_application)
        .ok_or("source unary Output has no actual trait application")?;
    if result != expected
        || application.item.0 != output.0
        || application.subject != header.self_type
        || owner.item.0 != header.definition.0
        || facts
            .types
            .arguments(owner.arguments)
            .is_none_or(|a| a.len() != 0)
        || facts
            .types
            .arguments(application.arguments)
            .is_none_or(|a| a.len() != 0)
    {
        return Err("source unary method changed its exact Self::Output projection".into());
    }
    let ty =
        facts.definitions[method.0 as usize].ok_or("source unary method lost its callable item")?;
    if !matches!(facts.types.kind(ty), Some(Kind::FunctionItem {item, signature, ..}) if item.0 == method.0 && signature == function.signature)
    {
        return Err("source unary callable item differs from its completed method header".into());
    }
    Ok(function.signature)
}

fn method_syntax(
    report: &Report,
    facts: &Declarations,
    method: DefId,
    origin: NodeKey,
) -> Result<(), Failure> {
    let unit = report.package.unit(origin.file);
    let Syntax::Item(index) = unit.configured.ast.node(origin.node).kind else {
        return Err("source unary method lost its current function item".into());
    };
    let item = unit
        .configured
        .ast
        .items
        .get(index as usize)
        .ok_or("source unary method has a stale current item index")?;
    let ItemKind::Function {
        flags,
        abi: crate::syntax::item::Abi::Rust,
        parameters,
        body: None,
        ..
    } = item.kind
    else {
        return Err("source unary method requires its current required Rust function".into());
    };
    let function = facts
        .function_indices
        .get(&method)
        .and_then(|&index| facts.functions.get(index))
        .ok_or("source unary method has no actual source function")?;
    if flags != crate::syntax::item::FunctionFlags::default()
        || function.definition != method
        || function.node != origin
        || function.parameters != parameters
        || !unit.configured.ast.list(item.generics).is_empty()
    {
        return Err("source unary method changed its current safe synchronous header".into());
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/not_contracts.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../tests/invariants/neg_contracts.rs"]
mod neg_tests;
