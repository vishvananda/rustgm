//! Source nominal field types are substituted through the same structural node arena.
use super::{
    Context, DomainIndex, Work, g,
    nodes::{Arena, Node, NodeId, Region, Rewrite},
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
    /// A version-thirteen nominal field's own region parameter selects the
    /// use's region argument; every other region keeps its ordinary rewrite.
    pub(super) fn rewritten_region(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        reference: s::RegionRef,
        rewrite: Rewrite,
        work: &mut Work,
    ) -> Result<Region, &'static str> {
        work.charge(1)?;
        let rows = domains.headers.rows();
        if let Rewrite::Nominal { owner, arguments } = rewrite
            && let Some(&s::Region::Parameter { slot, depth }) =
                rows.regions.get(reference.0 as usize)
            && (depth == 0 || domains.declaration_policies)
        {
            let formal = rows
                .slots
                .get(slot.0 as usize)
                .ok_or("source nominal region formal is unknown")?;
            if formal.owner != owner || formal.kind != s::ParameterKind::Region {
                return Err("source nominal region argument names a foreign formal");
            }
            if !self.nominal_arguments.contains_key(&arguments) {
                self.index_nominal_arguments(arguments, work)?;
            }
            return self
                .nominal_regions
                .get(&arguments)
                .and_then(|regions| regions.get(formal.index as usize))
                .copied()
                .ok_or("source nominal omitted its complete actual region argument");
        }
        if let Rewrite::Application(application) = rewrite
            && let Some(&s::Region::Parameter { slot, depth }) =
                rows.regions.get(reference.0 as usize)
            && (depth == 0 || domains.declaration_policies)
        {
            return applied_region(domains, self, application, slot, work);
        }
        uses::region(domains, self, reference, rewrite, work)
    }
    // Each canonical argument spine is indexed once. Repeated field-template
    // parameters then select their actual argument in constant work. Leading
    // region arguments are indexed apart from the type arguments they precede.
    fn index_nominal_arguments(
        &mut self,
        arguments: NodeId,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let mut cursor = arguments;
        let (mut count, mut regions) = (0_usize, 0_usize);
        loop {
            work.charge(1)?;
            match self.nodes.get(cursor.0 as usize) {
                Some(Node::NominalArgument { next, .. }) => {
                    count = count.saturating_add(1);
                    cursor = *next;
                }
                Some(Node::NominalRegion { next, .. }) if count == 0 => {
                    regions = regions.saturating_add(1);
                    cursor = *next;
                }
                Some(Node::NominalEnd) => break,
                _ => return Err("source nominal actual argument spine is malformed"),
            }
        }
        self.reserve(
            retained::new_map::<NodeId, Vec<NodeId>>(1)
                .saturating_add(retained::new_map::<NodeId, Vec<Region>>(1))
                .saturating_add(count.saturating_mul(size_of::<NodeId>()))
                .saturating_add(regions.saturating_mul(size_of::<Region>())),
            0,
            work,
        )?;
        let mut indexed = Vec::with_capacity(count);
        let mut leading = Vec::with_capacity(regions);
        cursor = arguments;
        for _ in 0..count.saturating_add(regions) {
            work.charge(1)?;
            match self.nodes.get(cursor.0 as usize) {
                Some(Node::NominalArgument { ty, next }) => {
                    indexed.push(*ty);
                    cursor = *next;
                }
                Some(Node::NominalRegion { region, next }) => {
                    leading.push(*region);
                    cursor = *next;
                }
                _ => return Err("source nominal actual argument spine changed during indexing"),
            }
        }
        self.nominal_argument_bytes = self
            .nominal_argument_bytes
            .saturating_add(retained::vector(&indexed))
            .saturating_add(retained::vector(&leading));
        self.nominal_arguments.insert(arguments, indexed);
        self.nominal_regions.insert(arguments, leading);
        Ok(())
    }
    fn nominal_field(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        context: Context,
        (base, variant): (NodeId, Option<DeclarationRef>),
        ordinal: u32,
        union: bool,
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
        let is_union = row.layout.form == s::NominalForm::Union;
        if is_union != union {
            return Err(
                "source union field operations require their explicit active-field contract",
            );
        }
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
/// A called lifetime-generic enum constructor's payload and result name the
/// enum's region slots; its application supplies them as leading arguments.
fn applied_region(
    domains: &DomainIndex<'_, '_>,
    arena: &Arena,
    application: s::FunctionApplicationRef,
    slot: s::SlotRef,
    work: &mut Work,
) -> Result<Region, &'static str> {
    work.charge(2)?;
    let rows = domains.headers.rows();
    let application = rows
        .function_applications
        .get(application.0 as usize)
        .ok_or("source applied region has no actual application")?;
    let target = domains.target(application.declaration.declaration, work)?;
    let formal = rows
        .slots
        .get(slot.0 as usize)
        .ok_or("source applied region formal is unknown")?;
    if formal.owner != target.scope() || formal.kind != s::ParameterKind::Region {
        return Err("source signature early-region transport is unsupported");
    }
    let Some(&s::Argument::Region(region)) = application
        .arguments
        .slice(&rows.arguments)?
        .get(formal.index as usize)
    else {
        return Err("source constructor application omitted its actual region argument");
    };
    uses::region(domains, arena, region, Rewrite::Original, work)
}
impl Builder<'_, '_, '_> {
    pub(super) fn aggregate_value(
        &mut self,
        original: &s::Original,
        (ty, variant): (g::TypeRef, Option<DeclarationRef>),
        fields: g::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        if variant.is_none() && self.domains.declaration_policies {
            match self.domains.headers.rows().types.get(ty.0 as usize) {
                Some(&s::Type::Tuple(elements)) => {
                    return self.tuple_value(original, elements, fields, work);
                }
                Some(&s::Type::Array { element, length }) => {
                    return self.array_value(original, element, length, fields, work);
                }
                Some(&s::Type::Closure(reference)) => {
                    return self.closure_value(original, reference, fields, work);
                }
                _ => {}
            }
        }
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
        if rows.nominals[index as usize].layout.form == s::NominalForm::Union {
            return Err("source union values require their explicit active-field contract");
        }
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
                false,
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
    fn tuple_value(
        &mut self,
        original: &s::Original,
        elements: g::Range,
        fields: g::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let elements = elements.slice(&self.domains.headers.rows().roots)?;
        let fields = fields.slice(&original.arguments)?;
        if elements.len() != fields.len() {
            return Err("source tuple value changed its complete actual field count");
        }
        for (&element, &value) in elements.iter().zip(fields) {
            let expected = self.node(original, element, work)?;
            let actual = self.node(original, original.values[value as usize].ty, work)?;
            if actual != expected {
                return Err("source tuple field type differs from its ordered result element");
            }
        }
        Ok(())
    }
    fn closure_value(
        &mut self,
        original: &s::Original,
        reference: s::ClosureTypeRef,
        fields: g::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = self.domains.headers.rows();
        let closure = rows
            .closures()
            .and_then(|closures| closures.types.get(reference.0 as usize))
            .ok_or("source closure value has no closure type row")?;
        match rows.types.get(closure.environment.0 as usize) {
            Some(&s::Type::Scalar(crate::support::crates::Scalar::Unit)) => {
                if fields.start != fields.end {
                    return Err("non-capturing source closure value carried capture fields");
                }
                Ok(())
            }
            Some(&s::Type::Tuple(elements)) => self.tuple_value(original, elements, fields, work),
            _ => Err("source closure value has an invalid capture environment type"),
        }
    }
    fn array_value(
        &mut self,
        original: &s::Original,
        element: g::TypeRef,
        length: s::ArrayLengthRef,
        fields: g::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = self.domains.headers.rows();
        let fields = fields.slice(&original.arguments)?;
        let Some(&s::ConstTerm::Value(count)) = rows.array_lengths().get(length.0 as usize) else {
            return Err("source array value requires a closed array length");
        };
        if rows.types.get(count.ty.0 as usize)
            != Some(&s::Type::Scalar(crate::support::crates::Scalar::Usize))
            || u128::try_from(fields.len()).ok() != Some(count.bits)
        {
            return Err("source array value changed its complete actual element count");
        }
        let expected = self.node(original, element, work)?;
        for &value in fields {
            let actual = self.node(original, original.values[value as usize].ty, work)?;
            if actual != expected {
                return Err("source array element type differs from its result element type");
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
        let expected = self.arena.nominal_field(
            self.domains,
            context,
            (base, variant),
            ordinal,
            false,
            work,
        )?;
        let actual = self.node(original, result, work)?;
        if actual != expected {
            return Err("source projected field type differs after actual nominal substitution");
        }
        Ok(())
    }
    pub(super) fn union_projection(
        &mut self,
        original: &s::Original,
        base: g::TypeRef,
        result: g::TypeRef,
        owner: DeclarationRef,
        ordinal: u32,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let base = self.node(original, base, work)?;
        if !matches!(
            self.arena.nodes.get(base.0 as usize),
            Some(Node::Nominal { declaration, .. })
                if declaration.artifact.0 == 0 && declaration.declaration == owner
        ) {
            return Err("source union field projection changed its actual union owner");
        }
        let context = self.nominal_context(original)?;
        let expected =
            self.arena
                .nominal_field(self.domains, context, (base, None), ordinal, true, work)?;
        let actual = self.node(original, result, work)?;
        if actual != expected {
            return Err("source union field type differs after actual nominal substitution");
        }
        Ok(())
    }
    pub(super) fn union_value(
        &mut self,
        original: &s::Original,
        ty: g::TypeRef,
        owner: DeclarationRef,
        ordinal: u32,
        operand: u32,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        self.union_projection(
            original,
            ty,
            original.values[operand as usize].ty,
            owner,
            ordinal,
            work,
        )
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
            super::ports::Ports::place_type(self.domains.headers, original, place, work)?,
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
