//! Genuine source named structs retain canonical fields and declaration premises.
use super::{Declarations, DefId, DefKind, Report, Stage, g, s};
use crate::{
    resolve::model::Access,
    sema::members::Name,
    support::{bindings::LangItem, crates::RecordRepr, hash::HashMap, retained},
    syntax::item::Shape,
    types::{ItemId, aggregate, outlives::Pair},
};

pub(super) struct NominalShape {
    repr: RecordRepr,
    layout: s::NominalLayout,
    packed_simd: bool,
    enumeration: bool,
    exhaustiveness: s::Exhaustiveness,
}

impl Stage {
    /// Version thirteen: a tuple or unit struct's constructor travels as its
    /// struct's single variant declaration. Every consumer re-derives its
    /// shape, payload and visibility from the struct's own rows.
    pub(super) fn struct_constructor(
        &mut self,
        report: &Report,
        facts: &Declarations,
        constructor: DefId,
    ) -> Result<(), &'static str> {
        self.charge(2)?;
        let variant = facts
            .members
            .constructor(constructor)
            .ok_or("source struct constructor lacks its actual field group")?;
        let declaration = facts.members.variants[variant.0 as usize];
        let owner = report
            .resolution
            .definitions
            .get(declaration.owner.0 as usize)
            .ok_or("source struct constructor lost its actual struct")?;
        if owner.kind != DefKind::Struct || facts.members.variants(declaration.owner) != [variant] {
            return Err("source struct constructor changed its actual struct owner");
        }
        // `struct T();` has the unit form's empty field rows.
        if declaration.shape == Shape::Tuple && declaration.start == declaration.end {
            return Err("source empty tuple struct constructors require their wider profile");
        }
        if !self.declaration_policies() && non_exhaustive(report, facts, declaration.owner)? {
            return Err("source non_exhaustive struct constructors require their wider profile");
        }
        Ok(())
    }
    pub(super) fn nominals(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<(), &'static str> {
        self.reserve(
            report
                .package
                .modules
                .len()
                .saturating_mul(size_of::<Option<super::DeclarationRef>>()),
        )?;
        let lifetime_bounds = self.nominal_lifetime_index(facts)?;
        let mut modules = vec![None; report.package.modules.len()];
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            self.charge(1)?;
            if definition.kind == DefKind::Module {
                let owner =
                    DefId(u32::try_from(index).map_err(|_| "source nominal module overflow")?);
                let module = crate::resolve::impl_restrictions::module(
                    &report.package,
                    &report.resolution,
                    owner,
                )?;
                modules[module.0 as usize] = Some(self.declaration(owner)?);
            }
        }
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            self.charge(1)?;
            if definition.kind != DefKind::Struct
                && !(definition.kind == DefKind::Enum && self.body_nodes())
            {
                continue;
            }
            let owner = DefId(u32::try_from(index).map_err(|_| "source nominal owner overflow")?);
            let bounds = lifetime_bounds
                .get(&ItemId(owner.0))
                .map_or(&[][..], Vec::as_slice);
            self.nominal(report, facts, owner, &modules, bounds)?;
        }
        Ok(())
    }
    fn nominal_lifetime_index(
        &mut self,
        facts: &Declarations,
    ) -> Result<HashMap<ItemId, Vec<Pair>>, &'static str> {
        if !self.declaration_policies() {
            return Ok(HashMap::default());
        }
        let bounds = &facts.lifetimes.nominal_bounds;
        self.charge(bounds.len())?;
        self.reserve(
            retained::new_map::<ItemId, Vec<Pair>>(bounds.len())
                .saturating_add(size_of_val(bounds.as_slice()).saturating_mul(4)),
        )?;
        let mut by_owner = HashMap::default();
        for &(owner, pair) in bounds {
            by_owner.entry(owner).or_insert_with(Vec::new).push(pair);
        }
        Ok(by_owner)
    }

    fn nominal(
        &mut self,
        report: &Report,
        facts: &Declarations,
        owner: DefId,
        modules: &[Option<super::DeclarationRef>],
        explicit_bounds: &[Pair],
    ) -> Result<(), &'static str> {
        let item = ItemId(owner.0);
        let NominalShape {
            repr,
            layout,
            packed_simd,
            enumeration,
            exhaustiveness,
        } = self.nominal_shape(report, facts, owner)?;
        let policies = self.declaration_policies();
        let fields = if enumeration {
            &[]
        } else {
            let [variant] = facts.members.variants(item) else {
                return Err("source struct lacks its actual field group");
            };
            facts
                .members
                .fields(*variant)
                .ok_or("source struct field group is unknown")?
        };
        let declaration = self.declaration(owner)?;
        let contract = if enumeration {
            self.enum_contract(report, facts, owner, declaration)?
        } else {
            let view = facts
                .publication_header_contracts(report, owner)
                .map_err(|_| "source nominal advisory contract is incomplete")?;
            let contract = self
                .header_contract(declaration, view)
                .map_err(|_| "source nominal advisory contract is unsupported")?;
            if contract.inline.is_some() {
                return Err("source nominal cannot carry a callable inline policy");
            }
            contract
        };
        let context = self.nominal_frame(declaration)?;
        if !explicit_bounds.is_empty() {
            let bounds = self.region_bounds(explicit_bounds, context)?;
            self.reserve(4 * size_of::<s::NominalRegionBounds>())?;
            self.cold()?
                .nominal_region_bounds
                .push(s::NominalRegionBounds {
                    declaration,
                    bounds,
                });
        }
        let self_ty = facts
            .definitions
            .get(owner.0 as usize)
            .copied()
            .flatten()
            .ok_or("source nominal lacks its canonical type cell")?;
        let self_type = self.ty(facts, self_ty, context)?;
        let start = u32::try_from(self.rows.fields.len())
            .map_err(|_| "source nominal field count overflow")?;
        self.nominal_fields(report, facts, fields, declaration, context, modules)?;
        let end = u32::try_from(self.rows.fields.len())
            .map_err(|_| "source nominal field count overflow")?;
        let (inputs, declared_inputs) = self.nominal_inputs(facts, owner, context)?;
        let fundamental = self.declaration_policies()
            && facts.authored_definition_policy(owner)? & s::TraitFlags::FUNDAMENTAL != 0;
        let incoherent_inherent =
            self.declaration_policies() && facts.authored_incoherent_target(owner)?;
        self.reserve(4 * size_of::<s::NominalHeader>())?;
        self.rows.nominals.push(s::NominalHeader {
            contract,
            declaration,
            scope: declaration,
            self_type,
            fields: g::Range { start, end },
            inputs,
            declared_inputs,
            repr,
            layout,
            fundamental,
            incoherent_inherent,
            packed_simd,
            exhaustiveness,
            simd_lane_limit: if policies {
                facts.aggregates.simd_lane_limit(item)
            } else {
                None
            },
        });
        if enumeration {
            self.enum_header(facts, owner, declaration, context)?;
        }
        Ok(())
    }
    /// These source facts are available before any metadata IDs are assigned.
    pub(super) fn nominal_shape(
        &mut self,
        report: &Report,
        facts: &Declarations,
        owner: DefId,
    ) -> Result<NominalShape, &'static str> {
        let item = ItemId(owner.0);
        let shape = facts
            .aggregates
            .get(item)
            .ok_or("source nominal lacks its completed shape")?;
        let union = shape.kind == aggregate::Kind::Union;
        let enumeration = shape.kind == aggregate::Kind::Enum;
        let policies = self.declaration_policies();
        if union && !policies {
            return Err("source union representation requires its wider nominal profile");
        }
        let non_exhaustive = !union && non_exhaustive(report, facts, item)?;
        if non_exhaustive && !policies {
            return Err("source non-exhaustive nominal requires the declaration policy profile");
        }
        let (repr, packed_simd, layout) =
            record_repr(shape.repr, shape.kind, self.body_nodes(), policies)?;
        if !policies {
            concrete_simd(facts, owner, repr)?;
        }
        if !(shape.kind == aggregate::Kind::Struct
            || union
            || enumeration
            || (shape.kind == aggregate::Kind::PhantomData && self.body_nodes()))
        {
            return Err("source nominal shape requires its wider complete contract");
        }
        self.nominal_storage(facts, item, union)?;
        Ok(NominalShape {
            repr,
            layout,
            packed_simd,
            enumeration,
            exhaustiveness: if non_exhaustive {
                s::Exhaustiveness::Open
            } else {
                s::Exhaustiveness::Closed
            },
        })
    }
    fn nominal_storage(
        &mut self,
        facts: &Declarations,
        item: ItemId,
        union: bool,
    ) -> Result<(), &'static str> {
        let mut proved = |role| -> Result<bool, &'static str> {
            if !self.declaration_policies() {
                return Ok(false);
            }
            self.charge(1)?;
            let catalog = facts
                .source_bindings
                .as_deref()
                .ok_or("source storage wrapper lost its role catalog")?;
            Ok(catalog
                .validated_language(role)?
                .is_some_and(|d| d.definition.0 == item.0))
        };
        let admitted = match facts.aggregates.destruction(item) {
            Some(aggregate::FieldDestruction::Fields) => !union,
            Some(aggregate::FieldDestruction::Suppressed) => union,
            Some(aggregate::FieldDestruction::Never) => proved(LangItem::ManuallyDrop)?,
            _ => false,
        } && match facts.aggregates.permits_dangling_pointers(item) {
            Some(false) => true,
            Some(true) => proved(LangItem::MaybeDangling)?,
            None => false,
        } && match facts.aggregates.shared_mutation(item) {
            Some(aggregate::SharedMutation::Ordinary) => true,
            Some(aggregate::SharedMutation::Interior) => {
                proved(LangItem::UnsafeCell)? || proved(LangItem::UnsafePinned)?
            }
            None => false,
        };
        if !admitted {
            return Err("source nominal destruction/shape requires its wider complete contract");
        }
        Ok(())
    }
    fn nominal_inputs(
        &mut self,
        facts: &Declarations,
        owner: DefId,
        context: super::Context,
    ) -> Result<(g::Range, u32), &'static str> {
        let start =
            u32::try_from(self.rows.clauses.len()).map_err(|_| "source nominal clause overflow")?;
        for input in
            crate::sema::declaration_inputs::own(&facts.generics, facts.crates.as_deref(), owner)?
        {
            self.clause(facts, input, context, None)?;
        }
        let end =
            u32::try_from(self.rows.clauses.len()).map_err(|_| "source nominal clause overflow")?;
        let declared = u32::try_from(
            crate::sema::declaration_inputs::declared(
                &facts.generics,
                facts.crates.as_deref(),
                owner,
            )?
            .count(),
        )
        .map_err(|_| "source nominal declared input count overflow")?;
        Ok((g::Range { start, end }, declared))
    }
    fn nominal_fields(
        &mut self,
        report: &Report,
        facts: &Declarations,
        fields: &[crate::sema::members::Field],
        declaration: super::DeclarationRef,
        context: super::Context,
        modules: &[Option<super::DeclarationRef>],
    ) -> Result<(), &'static str> {
        for (ordinal, field) in fields.iter().enumerate() {
            self.charge(1)?;
            // Version thirteen: a tuple struct's fields are named by their
            // ordinals, which no record field can spell.
            let positional;
            let name = match field.name {
                Name::Named(name) => facts.members.names.get(name),
                Name::Index(index) if self.body_nodes() && index as usize == ordinal => {
                    positional = index.to_string();
                    &positional
                }
                Name::Index(_) => {
                    return Err("source nominal tuple fields require their wider profile");
                }
            };
            let source = field
                .origin
                .source()
                .ok_or("source nominal field has no original source identity")?;
            let unit = report.package.unit(source.file);
            if let Some(index) = unit
                .owner_indices
                .get(source.node.get() as usize)
                .copied()
                .flatten()
                && !unit.configured.owners[index.get() as usize - 1]
                    .attributes
                    .is_empty()
            {
                return Err(
                    "source field annotations require their wider complete policy contract",
                );
            }
            let visibility = match field.access {
                Access::Public => s::ModuleVisibility::Public,
                Access::Within(module) => s::ModuleVisibility::Within(
                    modules
                        .get(module.0 as usize)
                        .copied()
                        .flatten()
                        .ok_or("source field visibility lacks its actual module")?,
                ),
                Access::ExternalPrivate => {
                    return Err("source nominal field visibility requires its wider contract");
                }
            };
            let ty = self.ty(facts, field.ty, context)?;
            let site = self.site(declaration, source)?;
            self.reserve(4 * size_of::<s::NominalField>() + name.len())?;
            self.rows.fields.push(s::NominalField {
                owner: declaration,
                ordinal: u32::try_from(ordinal).map_err(|_| "source nominal ordinal overflow")?,
                name: name.to_owned(),
                ty,
                visibility,
                site,
            });
        }
        Ok(())
    }
}

fn non_exhaustive(
    report: &Report,
    facts: &Declarations,
    owner: ItemId,
) -> Result<bool, &'static str> {
    let node = report.resolution.definitions[owner.0 as usize]
        .node
        .ok_or("source nominal lost its actual declaration")?;
    Ok(facts
        .non_exhaustive_contract(report, node)
        .map_err(|_| "source nominal non_exhaustive contract is incomplete")?
        .is_some())
}
/// Stricter than rustc for now: a generic SIMD record waits for its wider
/// contract, so no consumer sees an imported generic SIMD definition.
fn concrete_simd(facts: &Declarations, owner: DefId, repr: RecordRepr) -> Result<(), &'static str> {
    let generic = facts
        .generics
        .parameters
        .get(&owner)
        .is_some_and(|parameters| !parameters.is_empty());
    if repr == RecordRepr::Simd && generic {
        return Err("source generic SIMD record requires its wider complete contract");
    }
    Ok(())
}
/// The record representation a nominal header carries; an enum's integer
/// tag travels in its variant table. Version thirteen also carries a
/// transparent struct and a SIMD struct with no modifier.
fn record_repr(
    repr: aggregate::Repr,
    kind: aggregate::Kind,
    body_nodes: bool,
    policies: bool,
) -> Result<(RecordRepr, bool, s::NominalLayout), &'static str> {
    let enumeration = kind == aggregate::Kind::Enum;
    let union = kind == aggregate::Kind::Union;
    let packed_simd =
        repr.packed == Some(1) && repr.representation == aggregate::Representation::Simd;
    let mut repr = repr;
    let layout = if policies && !enumeration {
        let modifier = match (repr.align.take(), repr.packed.take()) {
            (Some(width), None) => s::LayoutModifier::Aligned(s::LayoutUnit::new(width)?),
            (None, Some(_)) if packed_simd => s::LayoutModifier::Natural,
            (None, Some(width)) => s::LayoutModifier::Packed(s::LayoutUnit::new(width)?),
            (None, None) => s::LayoutModifier::Natural,
            _ => return Err("source nominal has conflicting alignment and packing"),
        };
        let form = match (union, std::mem::take(&mut repr.arguments)) {
            (false, aggregate::ArgumentPassing::Ordinary) => s::NominalForm::Ordinary,
            (false, aggregate::ArgumentPassing::Indirect) => s::NominalForm::IndirectArguments,
            (true, aggregate::ArgumentPassing::Ordinary) => s::NominalForm::Union,
            (true, aggregate::ArgumentPassing::Indirect) => {
                return Err("source union cannot pass indirectly as a struct");
            }
        };
        s::NominalLayout { modifier, form }
    } else {
        s::NominalLayout::default()
    };
    let record = match repr {
        aggregate::Repr {
            representation: aggregate::Representation::Rust,
            packed: None,
            align: None,
            tag: None,
            arguments: aggregate::ArgumentPassing::Ordinary,
        } => RecordRepr::Rust,
        aggregate::Repr {
            representation: aggregate::Representation::C,
            packed: None,
            align: None,
            tag: None,
            arguments: aggregate::ArgumentPassing::Ordinary,
        } if !enumeration => RecordRepr::C,
        aggregate::Repr {
            representation: aggregate::Representation::Transparent,
            packed: None,
            align: None,
            tag: None,
            arguments: aggregate::ArgumentPassing::Ordinary,
        } if !enumeration && body_nodes => RecordRepr::Transparent,
        aggregate::Repr {
            representation: aggregate::Representation::Simd,
            packed: None,
            align: None,
            tag: None,
            arguments: aggregate::ArgumentPassing::Ordinary,
        } if !enumeration && body_nodes => RecordRepr::Simd,
        // An enum's integer tag travels in its variant table.
        aggregate::Repr {
            representation: aggregate::Representation::Rust,
            packed: None,
            align: None,
            tag: Some(_),
            arguments: aggregate::ArgumentPassing::Ordinary,
        } if enumeration => RecordRepr::Rust,
        _ => return Err("source nominal representation requires its wider complete contract"),
    };
    layout.validate(record)?;
    Ok((record, packed_simd, layout))
}
