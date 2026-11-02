//! Desugaring consumes actual tagged methods and variants, independently of names.
use crate::{
    resolve::{
        bindings::Declaration,
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
        unit::NodeKey,
    },
    sema::{header_attributes, iteration::ForPlan, members::VariantId, model::Declarations},
    support::bindings::LangItem,
    syntax::{
        ast::Kind as Syntax,
        item::{ItemKind, Shape},
    },
    types::{ItemId, Kind, arguments::Argument, callable::Convention},
};

pub(in crate::sema) struct Protocol {
    pub into_iter: DefId,
    pub next: DefId,
    pub option: ItemId,
    pub some: VariantId,
    pub none: VariantId,
}
struct Query<'a> {
    report: &'a Report,
    facts: &'a Declarations,
    site: NodeKey,
}
impl Query<'_> {
    fn error(&self, message: &'static str) -> Error {
        Declarations::error(self.report, self.site, message)
    }
    fn claimed(&self, role: LangItem) -> Result<Declaration, Error> {
        header_attributes::charge_source_contract_query(self.facts, 2)?;
        let catalog =
            self.facts.source_bindings.as_deref().ok_or_else(|| {
                self.error("source iteration requires its actual provider catalog")
            })?;
        catalog
            .require_provider_report(self.report)
            .map_err(|e| self.error(e))?;
        let declaration = catalog.language(role).ok_or_else(|| {
            self.error("source iteration requires its actual tagged declarations")
        })?;
        if self
            .report
            .resolution
            .source_bindings
            .as_deref()
            .and_then(|r| r.language(role))
            != Some(declaration)
        {
            return Err(self.error("source iteration changed its reserved declaration"));
        }
        Ok(declaration)
    }
    fn method(&self, role: LangItem) -> Result<DefId, Error> {
        let declaration = self.claimed(role)?;
        let image = header_attributes::completed_source_image(
            self.facts,
            self.report,
            declaration.definition,
        )?;
        if !image.current(
            self.report.package.unit(declaration.origin.file),
            declaration.origin.node,
        ) {
            return Err(self.error("source iteration changed its actual declaration image"));
        }
        let parent = declaration
            .parent
            .ok_or_else(|| self.error("source iteration method requires a real trait parent"))?;
        if self
            .report
            .resolution
            .definitions
            .get(parent.0 as usize)
            .is_none_or(|d| d.kind != DefKind::Trait)
            || self.facts.associated.trait_owner(declaration.definition) != Some(parent)
        {
            return Err(self.error("source iteration method changed its actual trait owner"));
        }
        let unit = self.report.package.unit(declaration.origin.file);
        if !matches!(unit.configured.ast.node(declaration.origin.node).kind,
            Syntax::Item(i) if matches!(unit.configured.ast.items[i as usize].kind, ItemKind::Function { body: None, .. }))
        {
            return Err(
                self.error("source iteration role requires an actual required trait method")
            );
        }
        let header = self
            .facts
            .declaration_header(declaration.definition)
            .map_err(|e| self.error(e))?
            .ok_or_else(|| {
                self.error("source iteration method lacks its actual completed header")
            })?;
        if !matches!(self.facts.types.kind(header.signature),
            Some(Kind::Function { parameters, unsafe_: false, convention: Convention::RUST, .. })
                if self.facts.types.list(parameters).is_some_and(|p| p.len() == 1))
        {
            return Err(
                self.error("source iteration requires an actual safe single-input Rust method")
            );
        }
        Ok(declaration.definition)
    }
    fn variant(&self, role: LangItem, owner: ItemId, payload: bool) -> Result<VariantId, Error> {
        let declaration = self.claimed(role)?;
        header_attributes::require_protocol_variant(self.facts, self.report, declaration, role)?;
        let id = self
            .facts
            .members
            .constructor(declaration.definition)
            .ok_or_else(|| self.error("source iteration lost its actual Option variant"))?;
        let variant = self
            .facts
            .members
            .variants
            .get(id.0 as usize)
            .ok_or_else(|| self.error("source iteration variant disappeared"))?;
        if variant.owner != owner
            || variant.origin.source() != Some(declaration.origin)
            || self
                .facts
                .members
                .fields(id)
                .is_none_or(|fields| fields.len() != usize::from(payload))
            || variant.shape != if payload { Shape::Tuple } else { Shape::Unit }
        {
            return Err(self.error("source iteration variant changed its actual owner or fields"));
        }
        if self
            .facts
            .members
            .variants(owner)
            .get(variant.ordinal as usize)
            != Some(&id)
        {
            return Err(self.error("source iteration changed its current variant ordinal"));
        }
        if payload {
            let parameters = self
                .facts
                .generics
                .parameters
                .get(&DefId(owner.0))
                .ok_or_else(|| self.error("source iteration Option lost its actual formal"))?;
            let [parameter] = parameters.as_slice() else {
                return Err(self.error("source iteration Option requires its actual single formal"));
            };
            if self
                .facts
                .members
                .fields(id)
                .and_then(|f| f.first())
                .map(|f| Argument::Type(f.ty))
                != Some(parameter.argument)
            {
                return Err(self.error("source iteration Option changed its actual payload type"));
            }
        }
        Ok(id)
    }
}
pub(in crate::sema) fn source(
    report: &Report,
    facts: &Declarations,
    site: NodeKey,
) -> Result<Protocol, Error> {
    let query = Query {
        report,
        facts,
        site,
    };
    let into_iter = query.method(LangItem::IntoIter)?;
    let next = query.method(LangItem::IteratorNext)?;
    let declaration = query.claimed(LangItem::Option)?;
    header_attributes::role_source_image(facts, report, declaration)?;
    let option = ItemId(declaration.definition.0);
    let some = query.variant(LangItem::OptionSome, option, true)?;
    let none = query.variant(LangItem::OptionNone, option, false)?;
    if some == none || facts.members.variants(option).len() != 2 {
        return Err(query.error("source iteration requires two distinct actual Option variants"));
    }
    Ok(Protocol {
        into_iter,
        next,
        option,
        some,
        none,
    })
}
pub(in crate::sema) fn check_plan(
    report: &Report,
    facts: &Declarations,
    site: NodeKey,
    plan: ForPlan,
) -> Result<(), Error> {
    let protocol = source(report, facts, site)?;
    let error = |message| Declarations::error(report, site, message);
    if plan.some != protocol.some || plan.none != protocol.none {
        return Err(error("iteration plan changed its actual Option variants"));
    }
    for (call, definition, result) in [
        (plan.into_iter, protocol.into_iter, plan.iterator),
        (plan.next, protocol.next, plan.option),
    ] {
        let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(call.callable) else {
            return Err(error("source iteration lost its actual callable"));
        };
        if item.0 != definition.0
            || !matches!(facts.types.kind(call.signature), Some(Kind::Function {
            result: actual, bound_regions: 0, unsafe_: false, convention: Convention::RUST, ..
        }) if actual == result)
        {
            return Err(error(
                "source iteration changed its actual callable or opened result",
            ));
        }
    }
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(plan.next.signature) else {
        return Err(error("source iteration lost its next signature"));
    };
    let Some([receiver]) = facts.types.list(parameters) else {
        return Err(error("source iteration changed its next arity"));
    };
    if !matches!(facts.types.kind(*receiver),Some(Kind::Reference {referent,mutable:true,..}) if referent==plan.iterator)
    {
        return Err(error(
            "source iteration changed its mutable iterator receiver",
        ));
    }
    if !matches!(facts.types.kind(plan.option),Some(Kind::Adt(owner,args))
        if owner==protocol.option && facts.types.arguments(args).is_some_and(|mut a|
            a.next()==Some(Argument::Type(plan.item)) && a.next().is_none()))
    {
        return Err(error("source iteration changed its actual Option Item"));
    }
    Ok(())
}
