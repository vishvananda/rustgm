//! Reserve genuine definitions in canonical owner/order, including unused formals.
use super::{
    BodyRef, Checked, DeclarationRef, DefId, DefKind, Export, GenericRequest, ItemId, TypeId, g,
};
use crate::{
    support::crates::ExternalKind,
    types::{Kind, parameters::ParameterKind},
};
impl Export<'_> {
    fn push_declaration(
        &mut self,
        id: DefId,
        row: g::Declaration,
    ) -> Result<DeclarationRef, &'static str> {
        self.reserve(
            4 * (size_of::<g::Declaration>()
                + size_of::<u32>()
                + size_of::<(DefId, DeclarationRef)>()),
        )?;
        let reference = DeclarationRef(Self::ordinal(self.rows.declarations.len())?);
        if self.declarations.insert(id, reference).is_some() {
            return Err("generic export repeats an actual declaration");
        }
        self.rows.declarations.push(row);
        self.rows.declaration_sites.push(1);
        Ok(reference)
    }
    fn name(&mut self, id: DefId, kind: DefKind) -> Result<String, &'static str> {
        let actual = self
            .report
            .resolution
            .definitions
            .get(id.0 as usize)
            .ok_or("generic declaration is unknown")?;
        if actual.kind != kind || actual.node.is_none() {
            return Err("generic declaration changed its actual source kind");
        }
        let name = self.report.package.names.get(actual.name);
        self.reserve(name.len())?;
        Ok(name.to_owned())
    }
}
pub(super) fn reserve(
    export: &mut Export<'_>,
    checked: &Checked<'_>,
    requests: &[GenericRequest],
) -> Result<(), &'static str> {
    for request in requests {
        export.charge(1)?;
        if request.external.kind() != ExternalKind::Function {
            return Err("generic request is not its actual function identity");
        }
        export.push_declaration(
            request.definition,
            g::Declaration::Function {
                identity: request.external.clone(),
                public: true,
            },
        )?;
    }
    for request in requests {
        let owner = export.declaration(request.definition)?;
        for (ordinal, parameter) in export
            .facts
            .generics
            .parameters_in_scope(request.definition)
            .enumerate()
        {
            export.charge(1)?;
            let crate::types::arguments::Argument::Type(ty) = parameter.argument else {
                return Err("generic row version zero cannot export lifetime or const formals");
            };
            let Some(Kind::Parameter(id)) = export.facts.types.kind(ty) else {
                return Err("generic formal lost its rigid identity");
            };
            let formal = export
                .facts
                .types
                .parameter(id)
                .ok_or("generic formal is unknown")?;
            let ParameterKind::Type { sized } = formal.kind else {
                return Err("generic formal changed its actual type kind");
            };
            if formal.owner != ItemId(request.definition.0)
                || formal.index as usize != ordinal
                || parameter.default.is_some()
            {
                return Err(
                    "generic row version zero cannot export inherited or defaulted formals",
                );
            }
            let name = export.name(parameter.binding, DefKind::Generic)?;
            let reference = export.push_declaration(
                parameter.binding,
                g::Declaration::TypeParameter {
                    owner,
                    ordinal: Export::ordinal(ordinal)?,
                    name,
                    sized,
                },
            )?;
            export.reserve(4 * (size_of::<g::Type>() + size_of::<(TypeId, g::TypeRef)>()))?;
            let root = g::TypeRef(Export::ordinal(export.rows.types.len())?);
            export.rows.types.push(g::Type::Parameter(reference));
            export.types.insert(ty, root);
        }
    }
    for (body, request) in requests.iter().enumerate() {
        let template = checked
            .get(ItemId(request.definition.0))
            .ok_or("missing checked generic original")?;
        super::formal_links::record(export, &template.body)?;
        for (ordinal, local) in template.body.locals.iter().enumerate() {
            export.charge(1)?;
            let Some(definition) = local.definition else {
                continue;
            };
            let actual = export
                .report
                .resolution
                .definitions
                .get(definition.0 as usize)
                .ok_or("unknown generic body declaration")?;
            let body = BodyRef(Export::ordinal(body)?);
            let ordinal = Export::ordinal(ordinal)?;
            let row = match actual.kind {
                DefKind::Parameter => g::Declaration::ValueParameter {
                    body,
                    ordinal: export
                        .formals
                        .get(&definition)
                        .ok_or("named parameter lacks its actual initializing input")?
                        .2,
                    name: export.name(definition, DefKind::Parameter)?,
                },
                DefKind::Local => g::Declaration::Local {
                    body,
                    ordinal,
                    name: export.name(definition, DefKind::Local)?,
                },
                _ => return Err("generic local changed its actual defining declaration kind"),
            };
            export.push_declaration(definition, row)?;
        }
    }
    Ok(())
}
