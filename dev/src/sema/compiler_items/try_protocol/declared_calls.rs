//! Protocol signatures borrow the ordinary canonical header and member tables.
use super::{Declaration, DefId, Error, ItemId, LangItem, Query, TypeId};
use crate::{
    resolve::model::DefKind,
    sema::compiler_items::headers,
    syntax::{ast::Kind as Syntax, item::ItemKind},
    types::{Kind, arguments::Argument, callable::Convention},
};

pub(super) struct Method {
    pub declaration: Declaration,
    pub input: TypeId,
    pub result: TypeId,
}
impl Query<'_> {
    pub(super) fn method(&self, role: LangItem) -> Result<Method, Error> {
        let declaration = self.claimed(role)?;
        let actual = self
            .report
            .resolution
            .definitions
            .get(declaration.definition.0 as usize)
            .ok_or_else(|| self.error("source Try method lost its actual definition"))?;
        if actual.kind != DefKind::Function
            || actual.node != Some(declaration.origin)
            || self.facts.associated.trait_owner(declaration.definition) != declaration.parent
        {
            return Err(self.error("source Try method changed its actual owner or declaration"));
        }
        self.current_header(declaration.definition, declaration.origin)?;
        let header = self
            .facts
            .declaration_header(declaration.definition)
            .map_err(|e| self.error(e))?
            .ok_or_else(|| self.error("source Try method has no completed function header"))?;
        let Some(Kind::Function {
            parameters,
            result,
            unsafe_: false,
            bound_regions: 0,
            convention: Convention::RUST,
        }) = self.facts.types.kind(header.signature)
        else {
            return Err(self.error("source Try method requires its actual safe Rust signature"));
        };
        let Some([input]) = self.facts.types.list(parameters) else {
            return Err(self.error("source Try method requires exactly one actual input"));
        };
        if header.declaration.source() != Some(declaration.origin)
            || header.universals != 0
            || !matches!(self.facts.definitions.get(declaration.definition.0 as usize)
                .copied().flatten().and_then(|ty| self.facts.types.kind(ty)),
                Some(Kind::FunctionItem {item,signature,..})
                    if item.0 == declaration.definition.0 && signature == header.signature)
        {
            return Err(self.error("source Try callable differs from its current declaration"));
        }
        Ok(Method {
            declaration,
            input: *input,
            result,
        })
    }
    fn trait_header(&self, definition: DefId) -> Result<headers::Trait<'_>, Error> {
        let actual = self
            .report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .ok_or_else(|| self.error("source Try protocol lost its actual trait parent"))?;
        let origin = actual
            .node
            .ok_or_else(|| self.error("imported Try protocol contracts are not transported"))?;
        self.current_header(definition, origin)?;
        let header = headers::Trait::source(
            self.report,
            self.facts,
            Declaration {
                definition,
                parent: None,
                origin,
            },
        )
        .map_err(|e| self.error(e))?;
        header
            .validate_self(&self.facts.types)
            .map_err(|e| self.error(e))?;
        if header.flags.unsafe_ || header.flags.auto_ || header.captures != 0 {
            return Err(
                self.error("source Try protocol requires its actual nonauto safe trait parent")
            );
        }
        Ok(header)
    }
    pub(super) fn trait_members(
        &self,
        trait_: DefId,
        output_method: Declaration,
        branch_method: Declaration,
    ) -> Result<(DefId, DefId, TypeId), Error> {
        let header = self.trait_header(trait_)?;
        crate::sema::header_attributes::charge_source_contract_query(
            self.facts,
            header.members.len(),
        )?;
        if !header.parameters.is_empty()
            || header.members.len() != 4
            || !header.members.contains(&output_method.definition)
            || !header.members.contains(&branch_method.definition)
        {
            return Err(self.error("source Try requires its actual two types and two methods"));
        }
        let mut output = None;
        let mut residual = None;
        for &member in header.members {
            if member == output_method.definition || member == branch_method.definition {
                continue;
            }
            let definition = &self.report.resolution.definitions[member.0 as usize];
            let origin = definition
                .node
                .ok_or_else(|| self.error("source Try type lost its source origin"))?;
            if definition.kind != DefKind::TypeAlias
                || self.facts.associated.trait_owner(member) != Some(trait_)
            {
                return Err(self.error("source Try associated type has a different actual owner"));
            }
            let unit = self.report.package.unit(origin.file);
            let Syntax::Item(index) = unit.configured.ast.node(origin.node).kind else {
                return Err(self.error("source Try type lost its actual declaration"));
            };
            let item = unit.configured.ast.items[index as usize];
            if !matches!(item.kind, ItemKind::TypeAlias { ty: None, .. })
                || !unit.configured.ast.list(item.generics).is_empty()
            {
                return Err(self.error("source Try requires nongeneric required associated types"));
            }
            let name = item.name.and_then(|span| unit.spelling(span).ok());
            let slot = match name {
                Some("Output") => &mut output,
                Some("Residual") => &mut residual,
                _ => {
                    return Err(
                        self.error("source Try lost its actual Output or Residual declaration")
                    );
                }
            };
            if slot.replace(member).is_some() {
                return Err(self.error("source Try has duplicate associated declarations"));
            }
        }
        Ok((
            output.ok_or_else(|| self.error("source Try has no actual Output"))?,
            residual.ok_or_else(|| self.error("source Try has no actual Residual"))?,
            header.self_type,
        ))
    }
    pub(super) fn projection(&self, definition: DefId) -> Result<TypeId, Error> {
        self.facts
            .associated
            .types
            .get(&definition)
            .map(|contract| contract.projection)
            .ok_or_else(|| {
                self.error("source Try requires its completed actual associated projection")
            })
    }
    pub(super) fn from_output(
        &self,
        method: &Method,
        output: DefId,
        subject: TypeId,
    ) -> Result<(), Error> {
        if method.input != self.projection(output)? || method.result != subject {
            return Err(
                self.error("source Try from_output changed its actual Output-to-Self signature")
            );
        }
        Ok(())
    }
    pub(super) fn residual_method(&self, method: &Method) -> Result<(), Error> {
        let parent = method
            .declaration
            .parent
            .ok_or_else(|| self.error("source from_residual has no actual trait parent"))?;
        let header = self.trait_header(parent)?;
        if header.members != [method.declaration.definition]
            || header.parameters.len() != 1
            || header.parameters[0].argument != Argument::Type(method.input)
            || method.result != header.self_type
        {
            return Err(self.error(
                "source from_residual requires its actual residual-argument-to-Self contract",
            ));
        }
        Ok(())
    }
    pub(super) fn residual_conversion(
        &self,
        method: &Method,
        from_residual: Declaration,
    ) -> Result<(), Error> {
        if method.declaration.parent.is_some()
            || !matches!(self.facts.types.kind(method.input),
            Some(Kind::Parameter(id)) if self.facts.types.parameter(id).is_some_and(|p| p.owner == ItemId(method.declaration.definition.0)))
        {
            return Err(self.error(
                "source into_try_type requires its actual module generic residual argument",
            ));
        }
        let Some(Kind::Projection(result)) = self.facts.types.kind(method.result) else {
            return Err(
                self.error("source into_try_type requires its actual residual TryType projection")
            );
        };
        let actual = self
            .facts
            .types
            .trait_application(result.trait_application)
            .ok_or_else(|| {
                self.error("source into_try_type lost its actual residual trait application")
            })?;
        let header = self.trait_header(DefId(actual.item.0))?;
        if result.subject != method.input
            || header.members != [DefId(result.item.0)]
            || header.parameters.len() != 1
            || from_residual.parent == Some(header.definition)
            || self.facts.associated.trait_owner(DefId(result.item.0)) != Some(header.definition)
        {
            return Err(
                self.error("source into_try_type projection has a different actual residual owner")
            );
        }
        Ok(())
    }
}
