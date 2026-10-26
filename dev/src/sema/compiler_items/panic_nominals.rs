//! Checked panic nominal declarations supply data, never intrinsic/handler permission.
use crate::{
    resolve::{bindings::Declaration, expansion::Report, package::Error},
    sema::model::Declarations,
    support::bindings::LangItem,
    types::{ItemId, Kind, Primitive, RegionId, TypeId, arguments::Argument},
};

pub(in crate::sema) fn complete(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    for role in [LangItem::PanicLocation, LangItem::PanicInfo] {
        let Some(claimed) = facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.language(role))
        else {
            continue;
        };
        let error = |message| Declarations::error(report, claimed.origin, message);
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| error("source panic nominal catalog disappeared"))?;
        catalog.require_provider_report(report).map_err(error)?;
        super::formatting::require_lifetime_nominal(report, facts, claimed).map_err(error)?;
        let ast = &report.package.unit(claimed.origin.file).configured.ast;
        let crate::syntax::ast::Kind::Item(index) = ast.node(claimed.origin.node).kind else {
            return Err(error("source panic nominal lost its actual item"));
        };
        if !matches!(
            ast.items[index as usize].kind,
            crate::syntax::item::ItemKind::Struct {
                shape: crate::syntax::item::Shape::Record,
                ..
            }
        ) {
            return Err(error(
                "source panic nominal requires its actual record declaration",
            ));
        }

        let fields = fields(facts, claimed).map_err(error)?;
        if role == LangItem::PanicLocation {
            location(report, facts, fields).map_err(error)?;
        } else {
            info(report, facts, fields).map_err(error)?;
        }
        super::roles::complete_nominal(facts, role, claimed)?;
    }
    Ok(())
}
fn fields(facts: &Declarations, declaration: Declaration) -> Result<&[TypeId], &'static str> {
    let item = ItemId(declaration.definition.0);
    let ty = facts
        .definitions
        .get(item.0 as usize)
        .copied()
        .flatten()
        .ok_or("source panic nominal has no canonical declaration type")?;
    let lifetime = RegionId::bound(0, 0)?;
    if argument(facts, ty, item)? != Argument::Lifetime(lifetime) {
        return Err("source panic nominal changed its actual lifetime arguments");
    }
    let definition = facts
        .aggregates
        .get(item)
        .ok_or("source panic nominal fields are not complete")?;
    let [variant] = facts.aggregates.variants(definition) else {
        return Err("source panic nominal requires one actual field group");
    };
    let fields = facts
        .types
        .list(variant.fields)
        .ok_or("source panic nominal has missing field types")?;
    if fields.len() != 4 {
        return Err("source panic nominal requires its actual four fields");
    }
    Ok(fields)
}
fn location(report: &Report, facts: &Declarations, fields: &[TypeId]) -> Result<(), &'static str> {
    let nominal = report
        .resolution
        .source_bindings
        .as_deref()
        .and_then(crate::resolve::bindings::Reservations::nonnull_declaration)
        .ok_or("source Location requires the actual NonNull declaration")?;
    if argument(facts, fields[0], ItemId(nominal.definition.0))?
        != Argument::Type(TypeId::primitive(Primitive::Str))
        || fields[1] != TypeId::primitive(Primitive::U32)
        || fields[2] != TypeId::primitive(Primitive::U32)
    {
        return Err(
            "source Location requires its actual NonNull<str>, line and column field types",
        );
    }
    let phantom = facts
        .source_bindings
        .as_deref()
        .ok_or("source Location catalog disappeared")?
        .validated_language(LangItem::PhantomData)?
        .ok_or("source Location requires completed PhantomData")?;
    let Argument::Type(target) = argument(facts, fields[3], ItemId(phantom.definition.0))? else {
        return Err("source Location marker has no actual type argument");
    };
    if !matches!(facts.types.kind(target), Some(Kind::Reference {referent, region,mutable:false}) if referent == TypeId::primitive(Primitive::Str) && region == RegionId::bound(0,0)?)
    {
        return Err("source Location requires its actual shared filename lifetime marker");
    }
    Ok(())
}
fn info(report: &Report, facts: &Declarations, fields: &[TypeId]) -> Result<(), &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source PanicInfo catalog disappeared")?;
    let arguments = super::formatting::declaration(report, facts, LangItem::FormatArguments)?
        .ok_or("source PanicInfo requires actual format Arguments")?;
    let location = catalog
        .validated_language(LangItem::PanicLocation)?
        .ok_or("source PanicInfo requires completed actual Location")?;
    let Some(Kind::Reference {
        referent: message,
        region: message_region,
        mutable: false,
    }) = facts.types.kind(fields[0])
    else {
        return Err("source PanicInfo message requires a shared reference");
    };
    let Some(Kind::Reference {
        referent: site,
        region: site_region,
        mutable: false,
    }) = facts.types.kind(fields[1])
    else {
        return Err("source PanicInfo location requires a shared reference");
    };
    let lifetime = RegionId::bound(0, 0)?;
    if message_region != lifetime
        || site_region != RegionId::STATIC
        || argument(facts, message, ItemId(arguments.definition.0))? != Argument::Lifetime(lifetime)
        || argument(facts, site, ItemId(location.definition.0))?
            != Argument::Lifetime(RegionId::STATIC)
        || fields[2] != TypeId::primitive(Primitive::Bool)
        || fields[3] != TypeId::primitive(Primitive::Bool)
    {
        return Err("source PanicInfo changed its actual message/location lifetimes or flags");
    }
    Ok(())
}
fn argument(facts: &Declarations, ty: TypeId, owner: ItemId) -> Result<Argument, &'static str> {
    let Some(Kind::Adt(item, args)) = facts.types.kind(ty) else {
        return Err("source panic field requires its actual nominal identity");
    };
    if item != owner {
        return Err("source panic field belongs to another actual nominal");
    }
    let mut args = facts
        .types
        .arguments(args)
        .ok_or("source panic field arguments are missing")?;
    let first = args
        .next()
        .ok_or("source panic field requires one actual argument")?;
    if args.next().is_some() {
        return Err("source panic field has extra actual arguments");
    }
    Ok(first)
}
