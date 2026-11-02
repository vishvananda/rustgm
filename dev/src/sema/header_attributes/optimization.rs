//! Source-owned optimization intent. No current lowering pass optimizes functions.
use super::{Declarations, DefId, Error, Header, NodeKey, Report, ledger};
use crate::{
    resolve::{
        bindings::FeatureName,
        expansion::MacroAllowanceSession,
        model::{DefKind, Namespace},
    },
    sema::{
        header_attributes::{ContractKnowledge, SourceContractSite},
        inlining::Hint,
    },
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::{Item, ItemKind},
        token_tree::{Delimiter, Forest, TreeKind},
    },
};
#[cfg(test)]
#[path = "../../../tests/invariants/optimization.rs"]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Optimization {
    Disabled,
    Size,
    Speed,
}
pub struct OptimizationView {
    pub source: SourceContractSite,
    pub preference: Optimization,
}
pub(super) struct Marker {
    owner_slot: usize,
    effect: usize,
    attribute: Attribute,
    item: Option<Item>,
    preference: Optimization,
    definition: Option<DefId>,
}
fn header(facts: &Declarations, key: NodeKey) -> Option<&Header> {
    facts
        .must_use_contracts
        .as_deref()?
        .headers
        .as_deref()?
        .local
        .declarations
        .get(&key)
}
fn marker(facts: &Declarations, key: NodeKey) -> Option<&Marker> {
    header(facts, key)?
        .promises
        .as_deref()?
        .optimization
        .as_deref()
}
pub(in crate::sema) fn consume(
    report: &Report,
    facts: &mut Declarations,
    (owner, key): (NodeKey, NodeKey),
    (attribute, effect): (Attribute, usize),
    forest: &Forest,
    allowances: &mut MacroAllowanceSession<'_>,
) -> Result<(), Error> {
    facts.require_lint_context(report)?;
    facts.reserve_must_use_source(key, attribute.span)?;
    super::reserve_source_contracts(facts, 1, 0)?;
    let error = |message| Error::at(key.file, attribute.span.start, message);
    let unit = report.package.unit(key.file);
    // Whole meta substitution is authorized by its generated attribute envelope;
    // forwarding a caller-authored item does not change that provenance.
    if !report
        .package
        .requested_optimize_attribute()
        .map_err(error)?
        && !allowances.allows_at(
            key.file,
            attribute.span,
            &FeatureName::new("optimize_attribute").map_err(error)?,
        )?
    {
        return Err(error("optimize requires its optimize_attribute request"));
    }
    let item = match unit.configured.ast.node(key.node).kind {
        Kind::Closure { .. } => {
            if !report
                .package
                .requested_expression_attributes()
                .map_err(error)?
                && !allowances.allows_at(
                    key.file,
                    attribute.span,
                    &FeatureName::new("stmt_expr_attributes").map_err(error)?,
                )?
            {
                return Err(error("optimize closure requires stmt_expr_attributes"));
            }
            None
        }
        Kind::Item(index)
            if matches!(
                unit.configured.ast.items[index as usize].kind,
                ItemKind::Function { body: Some(_), .. }
            ) =>
        {
            Some(unit.configured.ast.items[index as usize])
        }
        _ => {
            return Err(error(
                "optimize requires a function, method or closure body",
            ));
        }
    };
    super::charge_source_contract_query(facts, 3)?;
    let preference = parse(unit.source.text(), attribute, forest).map_err(error)?;
    if header(facts, key).is_none() {
        super::observe_selected_owner(report, facts, owner, key, true)?;
    }
    let actual = header(facts, key).ok_or_else(|| error("optimization source header missing"))?;
    super::charge_source_contract_query(facts, actual.owners.len())?;
    let owner_slot = actual
        .owners
        .iter()
        .position(|row| row.key == owner && row.effects.contains(&effect))
        .ok_or_else(|| error("optimization attribute belongs to another owner"))?;
    if marker(facts, key).is_some() {
        return Err(error("duplicate optimize attribute"));
    }
    let bytes = size_of::<Marker>()
        + if actual.promises.is_none() {
            size_of::<super::Promises>()
        } else {
            0
        };
    let local = &mut ledger(facts, bytes)?.local;
    local.first_optimization.get_or_insert(key);
    local
        .declarations
        .get_mut(&key)
        .ok_or_else(|| error("optimization header disappeared"))?
        .promises
        .get_or_insert_default()
        .optimization = Some(Box::new(Marker {
        owner_slot,
        effect,
        attribute,
        item,
        preference,
        definition: None,
    }));
    Ok(())
}
fn parse(
    source: &str,
    attribute: Attribute,
    forest: &Forest,
) -> Result<Optimization, &'static str> {
    let Input::Delimited(group) = attribute.input else {
        return Err("optimize requires exactly size, speed or none");
    };
    if attribute.unsafe_wrapper
        || !forest.tree(group).is_some_and(|tree| {
            matches!(
                tree.kind,
                TreeKind::Group {
                    delimiter: Delimiter::Parenthesis,
                    ..
                }
            )
        })
    {
        return Err("optimize requires an ordinary parenthesized argument");
    }
    let range = forest
        .children(group)
        .ok_or("optimize argument disappeared")?;
    let length = range.end - range.start;
    if !(length == 1
        || length == 2
            && forest.punctuation(range.start + 1, crate::lex::token::Punctuation::Comma))
    {
        return Err("optimize requires exactly one word argument");
    }
    match forest.name(source, range.start) {
        Some("size") => Ok(Optimization::Size),
        Some("speed") => Ok(Optimization::Speed),
        Some("none") => Ok(Optimization::Disabled),
        _ => Err("optimize expects size, speed or none"),
    }
}
fn current(
    report: &Report,
    facts: &Declarations,
    key: NodeKey,
) -> Result<Option<OptimizationView>, Error> {
    let actual =
        header(facts, key).ok_or_else(|| Error::unlocated("optimization header is absent"))?;
    super::validate_source(facts, report, key, actual)?;
    let unit = report.package.unit(key.file);
    if !matches!(
        unit.configured.ast.node(key.node).kind,
        Kind::Closure { .. }
    ) && !matches!(unit.configured.ast.node(key.node).kind, Kind::Item(index) if matches!(unit.configured.ast.items[index as usize].kind, ItemKind::Function { .. }))
    {
        return Err(Error::unlocated("optimization owner is no longer callable"));
    }
    let Some(marker) = marker(facts, key) else {
        return Ok(None);
    };
    let owner = actual
        .owners
        .get(marker.owner_slot)
        .filter(|row| row.effects.contains(&marker.effect))
        .ok_or_else(|| Error::unlocated("optimization configured owner changed"))?;
    let item = match unit.configured.ast.node(key.node).kind {
        Kind::Closure { .. } => None,
        Kind::Item(index) => Some(unit.configured.ast.items[index as usize]),
        _ => return Err(Error::unlocated("optimization callable changed")),
    };
    let retained = unit
        .retained_attributes(marker.effect..marker.effect + 1)
        .next()
        .ok_or_else(|| Error::unlocated("optimization source attribute disappeared"))?;
    if item != marker.item
        || retained.0 != marker.attribute
        || retained.0.name(retained.1, unit.source.text()) != Some("optimize")
    {
        return Err(Error::unlocated(
            "optimization source item or attribute changed",
        ));
    }
    Ok(Some(OptimizationView {
        source: SourceContractSite {
            owner: owner.key,
            declaration: key,
            span: marker.attribute.span,
        },
        preference: marker.preference,
    }))
}
pub(super) fn bind(
    report: &Report,
    facts: &mut Declarations,
    id: DefId,
    key: NodeKey,
) -> Result<(), Error> {
    if marker(facts, key).is_none() {
        return Ok(());
    }
    current(report, facts, key)?;
    let actual = &report.resolution.definitions[id.0 as usize];
    if actual.kind != DefKind::Function
        || actual.namespace != Namespace::Value
        || actual.node != Some(key)
        || report.resolution.foreign_owners.contains_key(&id)
    {
        return Err(Error::unlocated(
            "optimization requires its actual non-foreign function body",
        ));
    }
    let marker = ledger(facts, 0)?
        .local
        .declarations
        .get_mut(&key)
        .and_then(|row| row.promises.as_deref_mut())
        .and_then(|row| row.optimization.as_deref_mut())
        .ok_or_else(|| Error::unlocated("optimization binding disappeared"))?;
    if marker.definition.replace(id).is_some() {
        return Err(Error::unlocated("optimization function bound twice"));
    }
    Ok(())
}
pub(in crate::sema) fn check_inline(
    report: &Report,
    facts: &Declarations,
    key: NodeKey,
    hint: Hint,
) -> Result<(), Error> {
    if hint != Hint::Never
        && marker(facts, key).is_some_and(|row| row.preference == Optimization::Disabled)
    {
        current(report, facts, key)?;
        return Err(Error::at(
            key.file,
            0,
            "optimize(none) cannot be used with inline or inline(always)",
        ));
    }
    Ok(())
}
impl Declarations {
    /// Checked optimization intent never grants const, unsafe, borrow or ABI permission.
    /// # Errors
    /// Rejects changed source owners, callable syntax and exhausted query budgets.
    pub fn source_optimization(
        &self,
        report: &Report,
        id: DefId,
    ) -> Result<ContractKnowledge<OptimizationView>, Error> {
        self.require_lint_context(report)?;
        let actual = report
            .resolution
            .definitions
            .get(id.0 as usize)
            .filter(|row| row.kind == DefKind::Function)
            .ok_or_else(|| Error::unlocated("optimization query requires a function"))?;
        let Some(key) = actual.node else {
            return Ok(ContractKnowledge::Unknown);
        };
        let Some(local) = self
            .must_use_contracts
            .as_deref()
            .and_then(|row| row.headers.as_deref())
            .map(|row| &row.local)
        else {
            return Ok(ContractKnowledge::Unknown);
        };
        super::charge_source_contract_query(self, 3)?;
        if !local.comptime_complete || local.definitions.get(&id) != Some(&key) {
            return Ok(ContractKnowledge::Unknown);
        }
        let value = current(report, self, key)?;
        if marker(self, key).is_some_and(|marker| marker.definition != Some(id)) {
            return Err(Error::unlocated("optimization function binding changed"));
        }
        Ok(value.map_or(
            ContractKnowledge::KnownAbsent,
            ContractKnowledge::KnownPresent,
        ))
    }
    pub(crate) fn require_optimization_transport(
        &self,
        report: &Report,
    ) -> Result<(), &'static str> {
        if let Some(key) = self
            .must_use_contracts
            .as_deref()
            .and_then(|row| row.headers.as_deref())
            .and_then(|row| row.local.first_optimization)
        {
            current(report, self, key)
                .map_err(|_| "optimization lost its authenticated source before export")?
                .ok_or("optimization lost its source intent")?;
            return Err("optimization intent requires authenticated metadata transport");
        }
        Ok(())
    }
}
