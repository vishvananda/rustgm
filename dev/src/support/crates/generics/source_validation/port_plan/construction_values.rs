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
        (base, variant): (NodeId, Option<DeclarationRef>),
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
        let ty = if let Some(variant) = variant {
            // A downcast field: its variant belongs to exactly this enum and
            // inherits the enum's visibility. Which variant the place holds is
            // re-proved over the consumer's own CFG, never taken from here.
            let header = variant_header(domains, declaration.declaration, variant, work)?;
            header
                .fields
                .slice(&rows.enums().fields)?
                .get(ordinal as usize)
                .ok_or("source variant field ordinal is outside its actual variant")?
                .ty
        } else {
            let field = row
                .fields
                .slice(&rows.fields)?
                .get(ordinal as usize)
                .ok_or("source nominal field ordinal is outside its actual owner")?;
            require_access(rows, context.owner, field.visibility, work)?;
            field.ty
        };
        self.root(
            domains,
            uses::key(
                domains,
                ty,
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
        (ty, variant): (g::TypeRef, Option<DeclarationRef>),
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
        let rows = self.domains.headers.rows();
        let expected = match variant {
            Some(variant) => {
                variant_header(self.domains, declaration.declaration, variant, work)?.fields
            }
            None if rows.declarations[declaration.declaration.0 as usize]
                .definition
                .kind
                == s::DefinitionKind::Enum =>
            {
                return Err("source enum aggregate omitted its actual variant");
            }
            None => rows.nominals[index as usize].fields,
        };
        let values = fields.slice(&original.arguments)?;
        if values.len() != (expected.end - expected.start) as usize {
            return Err("source aggregate changed its complete actual field count");
        }
        let context = self.nominal_context(original)?;
        for (ordinal, &value) in values.iter().enumerate() {
            let expected = self.arena.nominal_field(
                self.domains,
                context,
                (base, variant),
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
        // A variant owner selects a downcast field of its enum base.
        let variant = matches!(
            self.domains.headers.header(owner, work)?,
            Some(s::HeaderRef::Variant(_))
        )
        .then_some(owner);
        let nominal = match variant {
            Some(_) => self.domains.headers.rows().declarations[owner.0 as usize]
                .definition
                .parent
                .ok_or("source variant field lost its actual enum")?,
            None => owner,
        };
        if !matches!(self.arena.nodes.get(base.0 as usize),Some(Node::Nominal {declaration,..}) if declaration.artifact.0 == 0 && declaration.declaration == nominal)
        {
            return Err("source field projection changed its actual nominal owner");
        }
        let context = self.nominal_context(original)?;
        let expected =
            self.arena
                .nominal_field(self.domains, context, (base, variant), ordinal, work)?;
        let actual = self.node(original, result, work)?;
        if actual != expected {
            return Err("source projected field type differs after actual nominal substitution");
        }
        Ok(())
    }
    /// A constructor test reads an enum place of exactly the variant's enum
    /// and yields `bool`.
    pub(super) fn variant_test(
        &mut self,
        original: &s::Original,
        (ty, place): (g::TypeRef, s::Place),
        variant: DeclarationRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let result = self.node(original, ty, work)?;
        let base = self.node(
            original,
            super::ports::Ports::place_type(original, place, work)?,
            work,
        )?;
        let Some(Node::Nominal { declaration, .. }) =
            self.arena.nodes.get(base.0 as usize).copied()
        else {
            return Err("source constructor test requires an actual enum place");
        };
        if self.arena.nodes.get(result.0 as usize)
            != Some(&Node::Scalar(crate::support::crates::Scalar::Bool))
            || declaration.artifact.0 != 0
        {
            return Err("source constructor test changed its actual bool result or enum owner");
        }
        variant_header(self.domains, declaration.declaration, variant, work)?;
        Ok(())
    }
    /// A fieldless enum converts to an integer result through its header's
    /// discriminants; data enums have no `as` conversion.
    pub(super) fn enum_cast(
        &mut self,
        original: &s::Original,
        ty: g::TypeRef,
        operand: u32,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let result = self.node(original, ty, work)?;
        let input = original
            .values
            .get(operand as usize)
            .ok_or("source enum cast lost its actual operand")?;
        let base = self.node(original, input.ty, work)?;
        let Some(Node::Nominal { declaration, .. }) =
            self.arena.nodes.get(base.0 as usize).copied()
        else {
            return Err("source enum cast requires an actual enum operand");
        };
        let rows = self.domains.headers.rows();
        let header = rows
            .enums()
            .headers
            .binary_search_by_key(&declaration.declaration.0, |row| row.declaration.0)
            .map(|index| rows.enums().headers[index])
            .map_err(|_| "source enum cast operand has no validated enum header")?;
        let variants = header.variants.slice(&rows.enums().variants)?;
        work.charge(variants.len())?;
        if declaration.artifact.0 != 0
            || variants
                .iter()
                .any(|variant| variant.shape != s::VariantShape::Unit)
            || !matches!(
                self.arena.nodes.get(result.0 as usize),
                Some(Node::Scalar(scalar)) if integer(*scalar)
            )
        {
            return Err("source enum cast requires a fieldless enum and an integer result");
        }
        Ok(())
    }
    fn nominal_context(&self, original: &s::Original) -> Result<Context, &'static str> {
        let input = self
            .domains
            .headers
            .rows()
            .body_inputs
            .get(self.domains.headers.original_index(original.body)?)
            .ok_or("source nominal operation lost its actual original frame")?;
        Ok(Context {
            owner: original.declaration,
            frame: Some(input.frame),
            body: Some(original.body),
        })
    }
}
/// The validated variant row that belongs to exactly `owner`.
fn variant_header(
    domains: &DomainIndex<'_, '_>,
    owner: DeclarationRef,
    variant: DeclarationRef,
    work: &mut Work,
) -> Result<s::VariantHeader, &'static str> {
    let rows = domains.headers.rows();
    let Some(s::HeaderRef::Variant(index)) = domains.headers.header(variant, work)? else {
        return Err("source variant operand does not name a validated enum variant");
    };
    if rows.declarations[variant.0 as usize].definition.parent != Some(owner) {
        return Err("source variant belongs to another actual enum");
    }
    Ok(rows.enums().variants[index as usize])
}
const fn integer(scalar: crate::support::crates::Scalar) -> bool {
    use crate::support::crates::Scalar;
    !matches!(
        scalar,
        Scalar::Unit | Scalar::Bool | Scalar::Char | Scalar::F32 | Scalar::F64
    )
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
