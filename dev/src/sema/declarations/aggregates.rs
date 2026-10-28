//! Complete nominal declarations after reserving their recursive identities.
mod derives;
mod formation;
mod headers;
mod nullable_members;
mod outlives;
mod readiness;
mod sizedness;
mod unions;
mod validation;
use super::{
    Checker, DefId, DefKind, Error, ItemId, ItemKind, Kind, List, NodeKey, State, Type, Work,
    regions,
};
use crate::{
    resolve::{
        model::{Access, ItemContext, Namespace},
        unit::FileId,
    },
    sema::members::{Field, Name, VariantId},
    syntax::{ast::NodeId, item::Shape},
    types::{
        Primitive, TypeId,
        aggregate::{self, Repr},
    },
};
pub(super) use readiness::Pending;

#[derive(Clone, Copy)]
struct Group {
    key: NodeKey,
    shape: Shape,
    fields: List,
    discriminant: Option<NodeId>,
}
struct DeclaredFields {
    fields: Vec<Field>,
    declared: Option<crate::types::ListId>,
    contexts: Option<Vec<ItemContext>>,
}
impl Checker<'_> {
    fn nominal_defaults(&mut self) -> Result<(), Error> {
        for index in 0..self.nominals.entries.len() {
            let item = self.nominals.entries[index].item;
            let key = self.nominals.entries[index].key;
            let mut proof = self.declaration_proof(DefId(item.0), None);
            self.parameter_defaults(DefId(item.0), key, &mut proof)?;
            self.retain_trait_proof(DefId(item.0), key, &mut proof);
        }
        Ok(())
    }
    pub(super) fn complete_nominals(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        for index in 0..self.nominals.entries.len() {
            let item = self.nominals.entries[index].item;
            self.work.push(Work::Fields(item, false));
            self.drain(steps, limit)?;
        }
        self.validate_storage()?;
        self.complete_nominal_sizes()?;
        self.complete_size_rechecks(steps, limit)?;
        self.validate_fields()?;
        self.nominal_defaults()?;
        self.lifetime_variances()?;
        self.nominal_outlives()?;
        self.copy_obligations()?;
        self.equality_obligations()?;
        self.constructors()
    }
    pub(super) fn nominal_fields(&mut self, item: ItemId, finish: bool) -> Result<(), Error> {
        if self.external_record(item) {
            return Ok(());
        }
        let index = self.nominals.index[&item];
        let entry = &self.nominals.entries[index];
        let (key, kind, repr, state) = (entry.key, entry.kind, entry.repr, entry.state);
        if state == State::Complete {
            return Ok(());
        }
        if !finish {
            if state == State::Active {
                return Err(self.error(key, "cycle in nominal field dependencies"));
            }
            self.nominals.entries[index].state = State::Active;
            self.facts
                .generics
                .size_rules
                .get_or_insert_with(Default::default)
                .active
                .insert(item);
            self.work.push(Work::Fields(item, true));
            for group in (0..self.nominals.entries[index].groups.len()).rev() {
                let group = self.nominals.entries[index].groups[group];
                self.field_annotations(&group, self.nominal_context(item))?;
            }
            self.work.push(Work::NominalContract(item, false));
            return Ok(());
        }
        let kind =
            if crate::sema::compiler_items::phantom::declaration(self.report, &self.facts, item)? {
                aggregate::Kind::PhantomData
            } else {
                kind
            };
        let tag = repr.tag.unwrap_or(Primitive::Isize);
        let mut previous = None;
        let mut variants = Vec::with_capacity(self.nominals.entries[index].groups.len());
        for ordinal in 0..self.nominals.entries[index].groups.len() {
            let group = self.nominals.entries[index].groups[ordinal];
            let fields = self.fields(&group, kind == aggregate::Kind::Enum)?;
            self.children.clear();
            self.children
                .extend(fields.fields.iter().map(|field| field.ty));
            let list = self
                .facts
                .types
                .intern_list(&self.children)
                .map_err(|e| self.error(group.key, e))?;
            self.facts
                .aggregates
                .index_field_projections(&self.facts.types, list)
                .map_err(|e| self.error(group.key, e))?;
            variants.push(aggregate::Variant {
                fields: list,
                discriminant: 0,
            });
            let ordinal =
                u32::try_from(ordinal).map_err(|_| self.error(group.key, "too many variants"))?;
            if let Some(declared) = fields.declared {
                self.facts.unreduced.retain_fields(item, ordinal, declared);
            }
            let variant = self.publish_fields(item, ordinal, &group, &fields)?;
            if kind == aggregate::Kind::Enum {
                self.discriminants
                    .push(super::super::constants::Discriminant {
                        variant,
                        value: group.discriminant.map(|node| NodeKey { node, ..group.key }),
                        tag,
                        previous,
                    })
                    .map_err(|e| self.error(group.key, e))?;
                previous = Some(variant);
            }
        }
        let groups = &self.nominals.entries[index].groups;
        self.check_discriminant_forms(key, repr, groups)?;
        if kind == aggregate::Kind::Enum
            && variants.iter().all(|v| {
                self.facts
                    .types
                    .list(v.fields)
                    .is_some_and(<[TypeId]>::is_empty)
            })
            && groups
                .iter()
                .all(|g| g.discriminant.is_none() || g.shape == Shape::Unit)
        {
            self.facts.enum_casts.insert(item);
        }
        self.facts
            .aggregates
            .insert(item, kind, repr, &variants)
            .map_err(|e| self.error(key, e))?;
        self.register_value_rules(item, key, kind)?;
        self.source_nullable_members(item)?;
        self.nominals.entries[index].state = State::Complete;
        if let Some(pending) = &mut self.facts.generics.size_rules {
            pending.active.remove(&item);
        }
        self.publish_constructors(item)
    }
    fn register_value_rules(
        &mut self,
        item: ItemId,
        key: NodeKey,
        kind: aggregate::Kind,
    ) -> Result<(), Error> {
        match kind {
            aggregate::Kind::Enum => self
                .facts
                .aggregates
                .mark_discriminants_pending(item)
                .map_err(|e| self.error(key, e))?,
            aggregate::Kind::Union => self.register_union(item, key)?,
            aggregate::Kind::Struct | aggregate::Kind::PhantomData => {}
        }
        Ok(())
    }
    pub(super) fn complete_layouts(&mut self) -> Result<(), Error> {
        // Concrete discriminants are final. Symbolic declarations still prove
        // storage, field requirements and representation without a fake layout.
        let items: Vec<_> = self.facts.aggregates.iter().map(|d| d.item).collect();
        for &item in &items {
            let definition = self
                .facts
                .aggregates
                .get(item)
                .ok_or_else(|| Error::at(FileId(0), 0, "missing representation definition"))?;
            self.facts
                .aggregates
                .validate_simd_declaration(&self.facts.types, definition)
                .map_err(|message| self.nominal_layout_error(item, message))?;
            crate::abi::layout::validate_repr(
                definition,
                self.facts.aggregates.variants(definition).len(),
            )
            .map_err(|message| self.nominal_layout_error(item, message))?;
        }
        self.facts
            .aggregates
            .validate_packing(&self.facts.types)
            .map_err(|(item, message)| self.nominal_layout_error(item, message))?;
        let mut templates = crate::abi::templates::Templates::default();
        for item in items {
            let ty = self.facts.definitions[item.0 as usize]
                .ok_or_else(|| Error::at(FileId(0), 0, "missing aggregate type"))?;
            let transparent =
                self.facts.aggregates.get(item).is_some_and(|d| {
                    d.repr.representation == aggregate::Representation::Transparent
                });
            if self.facts.generics.storage.is_none()
                && !transparent
                && !self.facts.associated.has_associated_types()
                && self.facts.types.is_sized(ty) == Some(true)
            {
                self.facts
                    .layouts
                    .demand(&self.facts.types, &self.facts.aggregates, ty)
                    .map_err(|error| self.nominal_layout_error(item, error.message))?;
                continue;
            }
            templates
                .declaration(
                    &mut self.facts.types,
                    &mut self.facts.aggregates,
                    &mut self.facts.layouts,
                    ty,
                )
                .map_err(|error| self.nominal_layout_error(item, error.message))?;
        }
        Ok(())
    }
    fn nominal_layout_error(&self, item: ItemId, message: &'static str) -> Error {
        self.report.resolution.definitions[item.0 as usize]
            .node
            .map_or_else(
                || Error::at(FileId(0), 0, message),
                |key| self.error(key, message),
            )
    }
    fn groups(&self, key: NodeKey) -> Result<(aggregate::Kind, Vec<Group>), Error> {
        match self.item(key)?.kind {
            ItemKind::Union { fields } => Ok((
                aggregate::Kind::Union,
                vec![Group {
                    key,
                    shape: Shape::Record,
                    fields,
                    discriminant: None,
                }],
            )),
            ItemKind::Struct { shape, fields } => Ok((
                aggregate::Kind::Struct,
                vec![Group {
                    key,
                    shape,
                    fields,
                    discriminant: None,
                }],
            )),
            ItemKind::Enum { variants } => {
                let unit = self.report.package.unit(key.file);
                let mut groups = Vec::new();
                for node in unit.configured.ast.list(variants).iter().copied() {
                    if !unit.configured.active[node.get() as usize] {
                        continue;
                    }
                    let key = self.unattributed(NodeKey { node, ..key });
                    let Kind::Variant(index) = self.kind(key) else {
                        return Err(self.error(key, "invalid enum variant syntax"));
                    };
                    let variant = unit.configured.ast.variants[index as usize];
                    groups.push(Group {
                        key,
                        shape: variant.shape,
                        fields: variant.fields,
                        discriminant: variant.discriminant,
                    });
                }
                Ok((aggregate::Kind::Enum, groups))
            }
            _ => Err(self.error(key, "expected aggregate declaration")),
        }
    }
    pub(super) fn unattributed(&self, mut key: NodeKey) -> NodeKey {
        while let Kind::Attributed { value, .. } = self.kind(key) {
            key.node = value;
        }
        key
    }
    fn field_annotations(&mut self, group: &Group, context: regions::Context) -> Result<(), Error> {
        let unit = self.report.package.unit(group.key.file);
        for node in unit.configured.ast.list(group.fields).iter().copied() {
            if !unit.configured.active[node.get() as usize] {
                continue;
            }
            let key = self.unattributed(NodeKey { node, ..group.key });
            let Kind::FieldDefinition { ty, .. } = self.kind(key) else {
                return Err(self.error(key, "invalid field definition"));
            };
            self.node_work(key.file, ty, context);
        }
        Ok(())
    }
    fn fields(&mut self, group: &Group, enum_: bool) -> Result<DeclaredFields, Error> {
        let unit = self.report.package.unit(group.key.file);
        let mut fields: Vec<Field> = Vec::new();
        let mut declared: Option<Vec<TypeId>> = None;
        let mut contexts = None;
        for node in unit.configured.ast.list(group.fields).iter().copied() {
            if !unit.configured.active[node.get() as usize] {
                continue;
            }
            let key = self.unattributed(NodeKey { node, ..group.key });
            let Kind::FieldDefinition { name, ty, .. } = self.kind(key) else {
                return Err(self.error(key, "invalid field definition"));
            };
            let context = name.map_or(Ok(ItemContext::ROOT), |span| {
                self.report.package.item_context(key.file, span.start)
            })?;
            if context != ItemContext::ROOT && contexts.is_none() {
                contexts = Some(vec![ItemContext::ROOT; fields.len()]);
            }
            if let Some(contexts) = &mut contexts {
                contexts.push(context);
            }
            let name = if let Some(span) = name {
                let spelling = unit
                    .spelling(span)
                    .map_err(|e| self.error(key, e.message))?;
                Name::Named(
                    self.facts
                        .members
                        .names
                        .intern(spelling)
                        .map_err(|e| self.error(key, e))?,
                )
            } else {
                Name::Index(
                    u32::try_from(fields.len()).map_err(|_| self.error(key, "too many fields"))?,
                )
            };
            let annotation = NodeKey { node: ty, ..key };
            let ty = self.ty(annotation)?;
            let original = self.facts.unreduced.node(annotation).unwrap_or(ty);
            if original != ty && declared.is_none() {
                declared = Some(fields.iter().map(|field| field.ty).collect());
            }
            if let Some(declared) = &mut declared {
                declared.push(original);
            }
            let access = if enum_ {
                Access::Public
            } else {
                *self
                    .report
                    .resolution
                    .field_access
                    .get(&key)
                    .ok_or_else(|| self.error(key, "missing resolved field access"))?
            };
            fields.push(Field {
                origin: crate::sema::members::Origin::Source(key),
                name,
                ty,
                access,
            });
        }
        let declared = declared
            .map(|fields| self.facts.types.intern_list(&fields))
            .transpose()
            .map_err(|e| self.error(group.key, e))?;
        Ok(DeclaredFields {
            fields,
            declared,
            contexts,
        })
    }
    fn publish_fields(
        &mut self,
        owner: ItemId,
        ordinal: u32,
        group: &Group,
        fields: &DeclaredFields,
    ) -> Result<VariantId, Error> {
        let members = &mut self.facts.members;
        let variant = if let Some(contexts) = &fields.contexts {
            members.insert_contextual(
                owner,
                ordinal,
                group.key,
                group.shape,
                &fields.fields,
                contexts,
            )
        } else {
            members.insert(owner, ordinal, group.key, group.shape, &fields.fields)
        };
        variant.map_err(|error| self.error(group.key, error))
    }
    fn check_discriminant_forms(
        &self,
        key: NodeKey,
        repr: Repr,
        groups: &[Group],
    ) -> Result<(), Error> {
        if repr.tag.is_none() && groups.iter().any(|g| g.discriminant.is_some()) {
            // Empty tuple/record variants are non-unit too: fieldlessness alone
            // does not permit an explicitly numbered default-representation enum.
            if groups.iter().any(|g| g.shape != Shape::Unit) {
                return Err(self.error(
                    key,
                    "explicit discriminants on data variants require an integer representation",
                ));
            }
        }
        Ok(())
    }
    fn constructors(&mut self) -> Result<(), Error> {
        for (index, definition) in self.report.resolution.definitions.iter().enumerate() {
            if matches!(
                definition.kind,
                DefKind::Constructor | DefKind::Variant | DefKind::Struct
            ) {
                let id = DefId(
                    u32::try_from(index)
                        .map_err(|_| Error::at(FileId(0), 0, "too many constructors"))?,
                );
                self.constructor(id)?;
            }
        }
        Ok(())
    }
    pub(super) fn constructor(&mut self, id: DefId) -> Result<(), Error> {
        let definition = &self.report.resolution.definitions[id.0 as usize];
        let Some(key) = definition.node else {
            return Ok(());
        };
        let Some(variant_id) = self.facts.members.at_node(key) else {
            return Ok(());
        };
        let variant = self.facts.members.variants[variant_id.0 as usize];
        self.facts.members.constructors.insert(
            id,
            crate::sema::members::Constructor {
                variant: variant_id,
                kind: crate::sema::members::ConstructorKind::Variant,
            },
        );
        if definition.namespace != Namespace::Value
            || self.facts.definitions[id.0 as usize].is_some()
        {
            return Ok(());
        }
        let owner = self.facts.definitions[variant.owner.0 as usize]
            .ok_or_else(|| self.error(key, "missing constructor result"))?;
        let ty = if variant.shape == Shape::Tuple {
            let def = self
                .facts
                .aggregates
                .get(variant.owner)
                .ok_or_else(|| self.error(key, "missing constructor definition"))?;
            let fields = self.facts.aggregates.variants(def)[variant.ordinal as usize].fields;
            let signature = self.intern(
                key,
                Type::Function {
                    bound_regions: u32::try_from(
                        self.facts
                            .lifetimes
                            .parameters
                            .get(&DefId(variant.owner.0))
                            .map_or(0, Vec::len),
                    )
                    .map_err(|_| self.error(key, "too many constructor lifetimes"))?,
                    parameters: fields,
                    result: owner,
                    unsafe_: false,
                    convention: crate::types::callable::Convention::RUST,
                },
            )?;
            self.constructor_template(key, ItemId(id.0), variant.owner, signature)?;
            self.intern(
                key,
                Type::FunctionItem {
                    arguments: crate::types::GenericArgs::EMPTY,
                    item: ItemId(id.0),
                    signature,
                },
            )?
        } else {
            owner
        };
        self.facts.definitions[id.0 as usize] = Some(ty);
        Ok(())
    }

    pub(super) fn constructor_template(
        &mut self,
        key: NodeKey,
        item: ItemId,
        owner: ItemId,
        signature: TypeId,
    ) -> Result<(), Error> {
        super::comparisons::LibraryContracts {
            report: self.report,
            facts: &mut self.facts,
        }
        .constructor_template(key, item, owner, signature)
    }
}
impl super::comparisons::LibraryContracts<'_> {
    pub(super) fn constructor_template(
        &mut self,
        key: NodeKey,
        item: ItemId,
        owner: ItemId,
        signature: TypeId,
    ) -> Result<(), Error> {
        let Some(Type::Function { bound_regions, .. }) = self.facts.types.kind(signature) else {
            return Err(self.error(key, "missing constructor signature"));
        };
        let generics = self
            .facts
            .generics
            .parameter_ids(&self.facts.types, DefId(owner.0))
            .map_err(|e| self.error(key, e))?;
        if bound_regions == 0 && generics.is_empty() {
            return Ok(());
        }
        let mapping = (0..bound_regions)
            .map(|index| {
                crate::types::RegionId::bound(
                    1,
                    u16::try_from(index).map_err(|_| "too many constructor parameters")?,
                )
            })
            .collect::<Result<Vec<_>, &'static str>>()
            .map_err(|e| self.error(key, e))?;
        let template = crate::types::substitute::Bound::new(mapping)
            .signature(&mut self.facts.types, signature, 0)
            .map_err(|e| self.error(key, e))?;
        self.facts
            .lifetimes
            .callables
            .insert_constructor(
                &self.facts.types,
                item,
                owner,
                template,
                bound_regions,
                &generics,
            )
            .map_err(|e| self.error(key, e))
    }
}
