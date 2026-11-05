//! Source nominal field types are substituted through the same structural node arena.
use super::{
    Context, DomainIndex, Work, g,
    nodes::{Arena, Node, NodeId, Rewrite},
    requests::Builder,
    s, uses,
};
use crate::support::{crates::owner_catalog::DeclarationRef, retained};
impl Arena {
    pub(super) fn nominal_argument(
        &mut self,
        arguments: NodeId,
        position: u32,
        work: &mut Work,
    ) -> Result<NodeId, &'static str> {
        work.charge(1)?;
        if !self.nominal_arguments.contains_key(&arguments) {
            self.index_nominal_arguments(arguments, work)?;
        }
        self.nominal_arguments
            .get(&arguments)
            .and_then(|arguments| arguments.get(position as usize))
            .copied()
            .ok_or("source nominal omitted its complete actual argument")
    }
    // Each canonical argument spine is indexed once. Repeated field-template
    // parameters then select their actual argument in constant work.
    fn index_nominal_arguments(
        &mut self,
        arguments: NodeId,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let mut cursor = arguments;
        let mut count = 0_usize;
        loop {
            work.charge(1)?;
            match self.nodes.get(cursor.0 as usize) {
                Some(Node::NominalArgument { next, .. }) => {
                    count = count.saturating_add(1);
                    cursor = *next;
                }
                Some(Node::NominalEnd) => break,
                _ => return Err("source nominal actual argument spine is malformed"),
            }
        }
        self.reserve(
            retained::new_map::<NodeId, Vec<NodeId>>(1)
                .saturating_add(count.saturating_mul(size_of::<NodeId>())),
            0,
            work,
        )?;
        let mut indexed = Vec::with_capacity(count);
        cursor = arguments;
        for _ in 0..count {
            work.charge(1)?;
            let Some(Node::NominalArgument { ty, next }) = self.nodes.get(cursor.0 as usize) else {
                return Err("source nominal actual argument spine changed during indexing");
            };
            indexed.push(*ty);
            cursor = *next;
        }
        self.nominal_argument_bytes = self
            .nominal_argument_bytes
            .saturating_add(retained::vector(&indexed));
        self.nominal_arguments.insert(arguments, indexed);
        Ok(())
    }
    fn nominal_field(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        context: Context,
        base: NodeId,
        ordinal: u32,
        work: &mut Work,
    ) -> Result<NodeId, &'static str> {
        work.charge(1)?;
        let Some(Node::Nominal {
            declaration,
            arguments,
        }) = self.nodes.get(base.0 as usize).copied()
        else {
            return Err("source field base is not an actual nominal");
        };
        if declaration.artifact.0 != 0 {
            return Err("source field owner is foreign to its actual artifact");
        }
        let rows = domains.headers.rows();
        let Some(s::HeaderRef::Nominal(index)) =
            domains.headers.header(declaration.declaration, work)?
        else {
            return Err("source field base lacks its actual nominal header");
        };
        let row = &rows.nominals[index as usize];
        let field = row
            .fields
            .slice(&rows.fields)?
            .get(ordinal as usize)
            .ok_or("source nominal field ordinal is outside its actual owner")?;
        require_access(rows, context.owner, field.visibility, work)?;
        self.root(
            domains,
            uses::key(
                domains,
                field.ty,
                Rewrite::Nominal {
                    owner: row.scope,
                    arguments,
                },
                Some(context),
                work,
            )?,
            work,
        )
    }
}
impl Builder<'_, '_, '_> {
    pub(super) fn nominal_value(
        &mut self,
        original: &s::Original,
        ty: g::TypeRef,
        fields: g::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let base = self.node(original, ty, work)?;
        let Some(Node::Nominal { declaration, .. }) =
            self.arena.nodes.get(base.0 as usize).copied()
        else {
            return Err("source aggregate value requires its actual nominal type");
        };
        let Some(s::HeaderRef::Nominal(index)) =
            self.domains.headers.header(declaration.declaration, work)?
        else {
            return Err("source aggregate has no actual nominal header");
        };
        let expected = self.domains.headers.rows().nominals[index as usize].fields;
        let values = fields.slice(&original.arguments)?;
        if values.len() != (expected.end - expected.start) as usize {
            return Err("source aggregate changed its complete actual field count");
        }
        let context = self.nominal_context(original)?;
        for (ordinal, &value) in values.iter().enumerate() {
            let expected = self.arena.nominal_field(
                self.domains,
                context,
                base,
                u32::try_from(ordinal).map_err(|_| "source aggregate field ordinal overflow")?,
                work,
            )?;
            let actual = self.node(original, original.values[value as usize].ty, work)?;
            if actual != expected {
                return Err(
                    "source aggregate field type differs after actual nominal substitution",
                );
            }
        }
        Ok(())
    }
    pub(super) fn nominal_projection(
        &mut self,
        original: &s::Original,
        base: g::TypeRef,
        result: g::TypeRef,
        owner: DeclarationRef,
        ordinal: u32,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let base = self.node(original, base, work)?;
        if !matches!(self.arena.nodes.get(base.0 as usize),Some(Node::Nominal {declaration,..}) if declaration.artifact.0 == 0 && declaration.declaration == owner)
        {
            return Err("source field projection changed its actual nominal owner");
        }
        let context = self.nominal_context(original)?;
        let expected = self
            .arena
            .nominal_field(self.domains, context, base, ordinal, work)?;
        let actual = self.node(original, result, work)?;
        if actual != expected {
            return Err("source projected field type differs after actual nominal substitution");
        }
        Ok(())
    }
    fn nominal_context(&self, original: &s::Original) -> Result<Context, &'static str> {
        let input = self
            .domains
            .headers
            .rows()
            .body_inputs
            .get(original.body.0 as usize)
            .ok_or("source nominal operation lost its actual original frame")?;
        Ok(Context {
            owner: original.declaration,
            frame: Some(input.frame),
            body: Some(original.body),
        })
    }
}
fn require_access(
    rows: &s::SourceRows,
    mut owner: DeclarationRef,
    visibility: s::ModuleVisibility,
    work: &mut Work,
) -> Result<(), &'static str> {
    let s::ModuleVisibility::Within(module) = visibility else {
        return Ok(());
    };
    loop {
        work.charge(1)?;
        if owner == module {
            return Ok(());
        }
        owner = rows
            .declarations
            .get(owner.0 as usize)
            .and_then(|row| row.definition.parent)
            .ok_or("source original cannot access this actual private field")?;
    }
}
