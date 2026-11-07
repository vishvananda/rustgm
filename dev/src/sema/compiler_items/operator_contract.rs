//! One authenticated source operator walk supplies borrowed declaration facts.
mod indexing;
use super::{Declaration, DefId, Error, LangItem, Report, headers};
use crate::{
    resolve::{model::DefKind, unit::NodeKey},
    sema::{header_attributes::ContractKnowledge, model::Declarations},
    syntax::{ast::Kind as Syntax, item::ItemKind},
    types::{Kind, TypeId, callable::Convention},
};

/// Identity and signature only; ordinary trait, effect, body and call proof follows.
pub(in crate::sema) struct SourceOperator {
    pub definition: DefId,
    pub method: DefId,
    pub signature: TypeId,
    pub rhs: Option<TypeId>,
    pub output: Option<DefId>,
}
#[derive(Clone, Copy)]
pub(super) enum Operator {
    Not,
    Neg,
    Shl,
    BitOr,
    Add,
    Sub,
    Div,
    Mul,
    Rem,
    BitXor,
    BitAnd,
    Shr,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    BitXorAssign,
    BitAndAssign,
    BitOrAssign,
    ShlAssign,
    ShrAssign,
    Index,
    IndexMut,
}
impl Operator {
    const ALL: [Self; 24] = [
        Self::Not,
        Self::Neg,
        Self::Shl,
        Self::BitOr,
        Self::Add,
        Self::Sub,
        Self::Div,
        Self::Mul,
        Self::Rem,
        Self::BitXor,
        Self::BitAnd,
        Self::Shr,
        Self::AddAssign,
        Self::SubAssign,
        Self::MulAssign,
        Self::DivAssign,
        Self::RemAssign,
        Self::BitXorAssign,
        Self::BitAndAssign,
        Self::BitOrAssign,
        Self::ShlAssign,
        Self::ShrAssign,
        Self::Index,
        Self::IndexMut,
    ];
    const fn has_rhs(self) -> bool {
        matches!(
            self,
            Self::Index
                | Self::IndexMut
                | Self::Shl
                | Self::BitOr
                | Self::Add
                | Self::Sub
                | Self::Div
                | Self::Mul
                | Self::Rem
                | Self::BitXor
                | Self::BitAnd
                | Self::Shr
                | Self::AddAssign
                | Self::SubAssign
                | Self::MulAssign
                | Self::DivAssign
                | Self::RemAssign
                | Self::BitXorAssign
                | Self::BitAndAssign
                | Self::BitOrAssign
                | Self::ShlAssign
                | Self::ShrAssign
        )
    }
    const fn assignment(self) -> bool {
        matches!(
            self,
            Self::AddAssign
                | Self::SubAssign
                | Self::MulAssign
                | Self::DivAssign
                | Self::RemAssign
                | Self::BitXorAssign
                | Self::BitAndAssign
                | Self::BitOrAssign
                | Self::ShlAssign
                | Self::ShrAssign
        )
    }
    const fn mode(self) -> Mode {
        if self.assignment() {
            Mode::Assignment
        } else {
            match self {
                Self::Index => Mode::Index { mutable: false },
                Self::IndexMut => Mode::Index { mutable: true },
                _ => Mode::Value,
            }
        }
    }
    const fn indexing(self) -> bool {
        matches!(self.mode(), Mode::Index { .. })
    }
    const fn own_output(self) -> bool {
        matches!(self.mode(), Mode::Value | Mode::Index { mutable: false })
    }
    const fn role(self) -> LangItem {
        match self {
            Self::Index => LangItem::Index,
            Self::IndexMut => LangItem::IndexMut,
            Self::Not => LangItem::Not,
            Self::Neg => LangItem::Neg,
            Self::Shl => LangItem::Shl,
            Self::BitOr => LangItem::BitOr,
            Self::Add => LangItem::Add,
            Self::Sub => LangItem::Sub,
            Self::Mul => LangItem::Mul,
            Self::Rem => LangItem::Rem,
            Self::BitXor => LangItem::BitXor,
            Self::BitAnd => LangItem::BitAnd,
            Self::Shr => LangItem::Shr,
            Self::AddAssign => LangItem::AddAssign,
            Self::SubAssign => LangItem::SubAssign,
            Self::MulAssign => LangItem::MulAssign,
            Self::DivAssign => LangItem::DivAssign,
            Self::RemAssign => LangItem::RemAssign,
            Self::BitXorAssign => LangItem::BitXorAssign,
            Self::BitAndAssign => LangItem::BitAndAssign,
            Self::BitOrAssign => LangItem::BitOrAssign,
            Self::ShlAssign => LangItem::ShlAssign,
            Self::ShrAssign => LangItem::ShrAssign,
            Self::Div => LangItem::Div,
        }
    }
    const fn method(self) -> &'static str {
        match self {
            Self::Index => "index",
            Self::IndexMut => "index_mut",
            Self::Not => "not",
            Self::Neg => "neg",
            Self::Shl => "shl",
            Self::BitOr => "bitor",
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Rem => "rem",
            Self::BitXor => "bitxor",
            Self::BitAnd => "bitand",
            Self::Shr => "shr",
            Self::AddAssign => "add_assign",
            Self::SubAssign => "sub_assign",
            Self::MulAssign => "mul_assign",
            Self::DivAssign => "div_assign",
            Self::RemAssign => "rem_assign",
            Self::BitXorAssign => "bitxor_assign",
            Self::BitAndAssign => "bitand_assign",
            Self::BitOrAssign => "bitor_assign",
            Self::ShlAssign => "shl_assign",
            Self::ShrAssign => "shr_assign",
            Self::Div => "div",
        }
    }
}
#[derive(Clone, Copy)]
enum Mode {
    Value,
    Assignment,
    Index { mutable: bool },
}
struct Method {
    definition: DefId,
    output: Option<DefId>,
    projection_owner: DefId,
    rhs: Option<TypeId>,
    mode: Mode,
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
/// Query only readiness; the ordinary operator validator still checks the value.
pub(super) fn pending_default(
    report: &Report,
    facts: &Declarations,
    site: NodeKey,
    operator: Operator,
) -> Result<Option<DefId>, Error> {
    crate::sema::header_attributes::charge_source_contract_query(facts, 8)?;
    let error = |message| Declarations::error(report, site, message);
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| error("source binary default lost its actual catalog"))?;
    catalog.require_provider_report(report).map_err(error)?;
    let declaration = catalog
        .language(operator.role())
        .ok_or_else(|| error("source binary default requires its actual trait"))?;
    let header = headers::Trait::source(report, facts, declaration).map_err(error)?;
    let [parameter] = header.parameters else {
        return Ok(None);
    };
    if parameter.default.is_some() || parameter.anonymous {
        return Ok(None);
    }
    let original = report
        .resolution
        .definitions
        .get(parameter.binding.0 as usize)
        .and_then(|definition| definition.node)
        .ok_or_else(|| error("source binary default lost its actual parameter"))?;
    let ast = &report.package.unit(original.file).configured.ast;
    let Syntax::Generic(index) = ast.node(original.node).kind else {
        return Err(error(
            "source binary default changed its generic declaration",
        ));
    };
    Ok(ast.generics[index as usize]
        .default
        .map(|_| parameter.binding))
}
impl Work {
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.0 = self.0.saturating_add(count);
        if self.0 > crate::support::crates::MAX_COMPONENTS {
            return Err("source unary declaration query exceeds its work budget");
        }
        Ok(())
    }
}
/// Export authentication of a declared operator role: the provider's trait
/// passes the same contract its own operator uses. `None` means `role` is not
/// an operator role; the work count belongs to the caller's ledger.
pub(in crate::sema) fn exported(
    report: &Report,
    facts: &Declarations,
    role: LangItem,
) -> Result<Option<(Option<Declaration>, usize)>, &'static str> {
    let Some(operator) = Operator::ALL
        .into_iter()
        .find(|operator| operator.role() == role)
    else {
        return Ok(None);
    };
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("source operator export has no actual source catalog")?
        .language(role);
    let mut work = Work(0);
    if declaration.is_some() {
        checked(report, facts, operator, &mut work).map_err(|failure| match failure {
            Failure::Header(_) => {
                "source operator role requires positive completed source-header authentication"
            }
            Failure::Contract(message) => message,
        })?;
    }
    Ok(Some((declaration, work.0)))
}
pub(super) fn is_operator(role: LangItem) -> bool {
    Operator::ALL
        .into_iter()
        .any(|operator| operator.role() == role)
}
/// An imported operator role passes the same type-level contract as its
/// provider's declaration, read from the imported header instead of syntax.
/// Roles whose method returns `Output` need imported associated results.
pub(super) fn imported(
    report: &Report,
    facts: &Declarations,
    declaration: super::Role,
    role: LangItem,
    steps: &mut usize,
) -> Result<(), &'static str> {
    let operator = Operator::ALL
        .into_iter()
        .find(|operator| operator.role() == role)
        .ok_or("imported operator admission requested another language role")?;
    if !operator.assignment() {
        return Err("imported operator Output roles require imported associated results");
    }
    facts.associated.require_imported_role(declaration)?;
    let header = headers::Trait::current(report, facts, declaration, steps)?;
    let mut work = Work(*steps);
    let result = imported_assignment(report, facts, &header, operator, &mut work);
    *steps = work.0;
    result.map_err(|failure| match failure {
        Failure::Header(_) => "imported operator role lost its completed header",
        Failure::Contract(message) => message,
    })
}
fn imported_assignment(
    report: &Report,
    facts: &Declarations,
    header: &headers::Trait<'_>,
    operator: Operator,
    work: &mut Work,
) -> Result<(), Failure> {
    work.charge(8)?;
    header.profile(
        &facts.types,
        headers::Flags {
            unsafe_: false,
            auto_: false,
            constant: header.flags.constant,
        },
        1,
    )?;
    let rhs = rhs_parameter(facts, header, operator)?;
    let [method] = header.members else {
        return Err("imported assignment operator requires exactly its operation member".into());
    };
    let definition = report
        .resolution
        .definitions
        .get(method.0 as usize)
        .ok_or("imported operator member has no actual definition")?;
    if header.written.parameters != 1
        || header.written.supertraits != 0
        || definition.kind != DefKind::Function
        || report.package.names.get(definition.name) != operator.method()
        || facts.associated.trait_owner(*method) != Some(header.definition)
        || facts
            .generics
            .parameters
            .get(method)
            .is_some_and(|p| !p.is_empty())
    {
        return Err("imported assignment operator changed its actual member profile".into());
    }
    let function = facts
        .declaration_header(*method)?
        .ok_or("imported operator has no completed method header")?;
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err("imported operator method requires a safe Rust signature".into());
    };
    assignment_signature(facts, header, rhs, parameters, result, bound_regions)?;
    let ty = facts.definitions[method.0 as usize]
        .ok_or("imported operator method lost its callable item")?;
    if !matches!(facts.types.kind(ty), Some(Kind::FunctionItem {item, signature, ..}) if item.0 == method.0 && signature == function.signature)
    {
        return Err("imported operator callable item differs from its method header".into());
    }
    Ok(())
}
pub(super) fn query(
    report: &Report,
    facts: &mut Declarations,
    site: NodeKey,
    operator: Operator,
) -> Result<SourceOperator, Error> {
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
) -> Result<SourceOperator, Failure> {
    work.charge(1)?;
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source unary has no actual source catalog")?;
    catalog.require_provider_report(report)?;
    let declaration = catalog.language(operator.role()).ok_or(match operator {
        Operator::Not => "source ! requires its actual Not language declaration",
        Operator::Neg => "source - requires its actual Neg language declaration",
        Operator::Index
        | Operator::IndexMut
        | Operator::Shl
        | Operator::BitOr
        | Operator::Add
        | Operator::Sub
        | Operator::Div
        | Operator::Mul
        | Operator::Rem
        | Operator::BitXor
        | Operator::BitAnd
        | Operator::Shr
        | Operator::AddAssign
        | Operator::SubAssign
        | Operator::MulAssign
        | Operator::DivAssign
        | Operator::RemAssign
        | Operator::BitXorAssign
        | Operator::BitAndAssign
        | Operator::BitOrAssign
        | Operator::ShlAssign
        | Operator::ShrAssign => "source binary operator requires its actual language declaration",
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
        usize::from(operator.has_rhs()),
    )?;
    work.charge(6 * usize::from(operator.has_rhs()))?;
    let rhs = rhs_parameter(facts, &header, operator)?;
    if header.written.parameters != usize::from(operator.has_rhs())
        || header.written.supertraits != usize::from(matches!(operator, Operator::IndexMut))
        || header.members.len() != if operator.own_output() { 2 } else { 1 }
    {
        return Err("source unary requires its actual Output and operation members without extra parameters or supertraits".into());
    }
    let (output, method) = members(report, facts, declaration, &header, operator, work)?;
    let inherited = if operator.indexing() {
        indexing::parent(report, facts, &header, operator, rhs, work)?
    } else {
        None
    };
    let output = output.or(inherited.map(|(_, output)| output));
    let contract = Method {
        definition: method,
        output,
        projection_owner: inherited.map_or(header.definition, |(owner, _)| owner),
        rhs,
        mode: operator.mode(),
    };
    work.charge(4)?;
    let signature = method_signature(report, facts, &header, &contract, work)?;
    Ok(SourceOperator {
        definition: declaration.definition,
        method,
        signature,
        rhs,
        output,
    })
}
fn rhs_parameter(
    facts: &Declarations,
    header: &headers::Trait<'_>,
    operator: Operator,
) -> Result<Option<TypeId>, Failure> {
    if operator.indexing() {
        return indexing::parameter(facts, header).map(Some);
    }
    if !operator.has_rhs() {
        return Ok(None);
    }
    let [parameter] = header.parameters else {
        return Err("source binary operator requires its actual Rhs formal".into());
    };
    let crate::types::arguments::Argument::Type(rhs) = parameter.argument else {
        return Err("source binary Rhs must be its actual type formal".into());
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(rhs) else {
        return Err("source binary Rhs lost its declared parameter identity".into());
    };
    if parameter.anonymous
        || parameter.default != Some(crate::types::arguments::Argument::Type(header.self_type))
        || facts.types.parameter(id)
            != Some(crate::types::parameters::Parameter {
                owner: crate::types::ItemId(header.definition.0),
                index: 0,
                kind: crate::types::parameters::ParameterKind::Type { sized: true },
            })
    {
        return Err("source binary Rhs changed its genuine declaration or Self default".into());
    }
    Ok(Some(rhs))
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
) -> Result<(Option<DefId>, DefId), Failure> {
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
    if count != if operator.own_output() { 2 } else { 1 } {
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
            (DefKind::TypeAlias, "Output") if operator.own_output() && output.is_none() => {
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
        if operator.own_output() {
            Some(output.ok_or("source unary has no Output")?)
        } else {
            None
        },
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
    contract: &Method,
    work: &mut Work,
) -> Result<TypeId, Failure> {
    let Method {
        definition: method,
        output,
        rhs,
        mode,
        ..
    } = *contract;
    let origin = report.resolution.definitions[method.0 as usize]
        .node
        .ok_or("source unary method has no source origin")?;
    current_header(report, facts, method, origin)?;
    let safety = method_syntax(report, facts, method, origin, mode)?;
    if matches!(mode, Mode::Index { .. }) {
        indexing::target_policy(report, facts, method, safety)?;
    }
    let function = facts
        .declaration_header(method)?
        .ok_or("source unary has no completed method header")?;
    if function.declaration.source() != Some(origin)
        || (matches!(mode, Mode::Value) && (function.body.is_some() || function.universals != 0))
        || facts
            .generics
            .parameters
            .get(&method)
            .is_some_and(|p| !p.is_empty())
    {
        return Err(
            "source operator required-method profile excludes provided/default bodies and extra binders".into(),
        );
    }
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result,
        unsafe_,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err(
            "source unary method requires a safe Rust signature without late binders".into(),
        );
    };
    if unsafe_ != safety {
        return Err("source operator signature differs from its actual source safety".into());
    }
    if matches!(mode, Mode::Index { .. }) {
        indexing::signature(
            facts,
            header,
            contract,
            parameters,
            result,
            bound_regions,
            work,
        )?;
    } else if let Some(output) = output {
        let inputs = [header.self_type, rhs.unwrap_or(header.self_type)];
        if bound_regions != 0
            || facts.types.list(parameters) != Some(&inputs[..=usize::from(rhs.is_some())])
        {
            return Err("source unary method requires its exact by-value Self receiver".into());
        }
        output_projection(facts, header, output, rhs, result)?;
    } else {
        assignment_signature(facts, header, rhs, parameters, result, bound_regions)?;
    }
    let ty =
        facts.definitions[method.0 as usize].ok_or("source unary method lost its callable item")?;
    if !matches!(facts.types.kind(ty), Some(Kind::FunctionItem {item, signature, ..}) if item.0 == method.0 && signature == function.signature)
    {
        return Err("source unary callable item differs from its completed method header".into());
    }
    Ok(function.signature)
}

fn output_projection(
    facts: &Declarations,
    header: &headers::Trait<'_>,
    output: DefId,
    rhs: Option<TypeId>,
    result: TypeId,
) -> Result<(), Failure> {
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
            .is_none_or(|mut args| match rhs {
                Some(rhs) => {
                    args.next() != Some(crate::types::arguments::Argument::Type(rhs))
                        || args.next().is_some()
                }
                None => args.len() != 0,
            })
        || facts
            .types
            .arguments(application.arguments)
            .is_none_or(|a| a.len() != 0)
    {
        return Err("source unary method changed its exact Self::Output projection".into());
    }
    Ok(())
}

fn method_syntax(
    report: &Report,
    facts: &Declarations,
    method: DefId,
    origin: NodeKey,
    mode: Mode,
) -> Result<bool, Failure> {
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
        body,
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
    let expected = crate::syntax::item::FunctionFlags {
        safety: if matches!(mode, Mode::Index { .. }) {
            flags.safety
        } else {
            crate::syntax::item::Safety::Default
        },
        ..Default::default()
    };
    if (matches!(mode, Mode::Value) && body.is_some())
        || flags != expected
        || function.definition != method
        || function.node != origin
        || function.parameters != parameters
        || function.body
            != body.map(|node| NodeKey {
                file: origin.file,
                node,
            })
        || !unit.configured.ast.list(item.generics).is_empty()
    {
        return Err("source unary method changed its current safe synchronous header".into());
    }
    Ok(flags.safety == crate::syntax::item::Safety::Unsafe)
}

fn assignment_signature(
    facts: &Declarations,
    header: &headers::Trait<'_>,
    rhs: Option<TypeId>,
    parameters: crate::types::ListId,
    result: TypeId,
    bound_regions: u32,
) -> Result<(), Failure> {
    let Some([receiver, argument]) = facts.types.list(parameters) else {
        return Err("source assignment requires its actual receiver and Rhs".into());
    };
    if result != TypeId::UNIT
        || bound_regions != 1
        || Some(*argument) != rhs
        || !matches!(facts.types.kind(*receiver), Some(Kind::Reference {mutable:true,referent,region}) if referent==header.self_type && region.bound_parts()==Some((0,0)))
    {
        return Err("source assignment requires exact &mut Self, Rhs and unit result".into());
    }
    Ok(())
}
