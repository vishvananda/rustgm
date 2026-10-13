//! Complete nominal declarations after reserving their recursive identities.
use super::{Checker, DefId, DefKind, Error, ItemId, ItemKind, Kind, List, NodeKey, Type, regions};
use crate::{
    resolve::{
        model::{Access, Namespace},
        unit::FileId,
    },
    sema::{
        members::{Field, Name},
        representation,
    },
    syntax::{ast::NodeId, item::Shape},
    types::{
        Primitive, TypeId,
        aggregate::{self, Repr},
    },
};

struct Group {
    key: NodeKey,
    shape: Shape,
    fields: List,
    discriminant: Option<NodeId>,
}
impl Checker<'_> {
    pub(super) fn reserve_nominals(&mut self) -> Result<(), Error> {
        for index in 0..self.report.resolution.definitions.len() {
            let definition = &self.report.resolution.definitions[index];
            if !matches!(definition.kind, DefKind::Struct | DefKind::Enum) {
                continue;
            }
            let key = definition
                .node
                .ok_or_else(|| Error::at(FileId(0), 0, "nominal declaration has no syntax"))?;
            let item = self.item(key)?;
            if !matches!(item.kind, ItemKind::Struct { .. } | ItemKind::Enum { .. }) {
                return Err(self.error(
                    key,
                    "union semantics require the unsafe language assignment",
                ));
            }
            if item.generics.start != item.generics.end
                || item.where_clause.start != item.where_clause.end
            {
                return Err(self.error(
                    key,
                    "parameterized aggregates require lifetime or generic assignments",
                ));
            }
            let id = u32::try_from(index)
                .map_err(|_| self.error(key, "too many nominal declarations"))?;
            let ty = self.intern(key, Type::Adt(ItemId(id), crate::types::RegionArgs::EMPTY))?;
            // Resolving an annotation needs the identity, not a completed field
            // graph. Layout later rejects by-value cycles but permits pointers.
            self.complete(DefId(id), key, ty)?;
        }
        Ok(())
    }
    pub(super) fn complete_nominals(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        let representations = representation::analyze(self.report)?;
        for index in 0..self.report.resolution.definitions.len() {
            let definition = &self.report.resolution.definitions[index];
            if !matches!(definition.kind, DefKind::Struct | DefKind::Enum) {
                continue;
            }
            let key = definition
                .node
                .ok_or_else(|| Error::at(FileId(0), 0, "missing nominal syntax"))?;
            let item = ItemId(
                u32::try_from(index)
                    .map_err(|_| self.error(key, "too many nominal declarations"))?,
            );
            let attributes = representations.get(&key).copied().unwrap_or_default();
            if attributes.derives != crate::types::copy::Properties::default() {
                self.facts.derives.insert(item, attributes.derives);
            }
            if attributes.equality != crate::types::equality::Properties::default() {
                self.facts
                    .equality_derives
                    .insert(item, attributes.equality);
            }
            let repr = attributes.repr;
            let (kind, groups) = self.groups(key)?;
            let tag = repr.tag.unwrap_or(Primitive::Isize);
            let mut previous = None;
            let mut variants = Vec::with_capacity(groups.len());
            for (ordinal, group) in groups.iter().enumerate() {
                self.field_annotations(group, steps, limit)?;
                let fields = self.fields(group, kind == aggregate::Kind::Enum)?;
                self.children.clear();
                self.children.extend(fields.iter().map(|field| field.ty));
                let list = self
                    .facts
                    .types
                    .intern_list(&self.children)
                    .map_err(|e| self.error(group.key, e))?;
                variants.push(aggregate::Variant {
                    fields: list,
                    discriminant: 0,
                });
                let ordinal = u32::try_from(ordinal)
                    .map_err(|_| self.error(group.key, "too many variants"))?;
                let variant = self
                    .facts
                    .members
                    .insert(item, ordinal, group.key, group.shape, &fields)
                    .map_err(|e| self.error(group.key, e))?;
                if kind == aggregate::Kind::Enum {
                    self.discriminants
                        .push(super::super::constants::Discriminant {
                            variant,
                            value: group.discriminant.map(|node| NodeKey { node, ..group.key }),
                            tag,
                            previous,
                        });
                    previous = Some(variant);
                }
            }
            self.check_discriminant_forms(key, repr, &groups)?;
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
        }
        self.copy_obligations()?;
        self.equality_obligations()?;
        self.constructors()
    }
    pub(super) fn complete_layouts(&mut self) -> Result<(), Error> {
        // Values are final before any layout is cached. Every declaration is checked.
        for definition in &self.facts.aggregates {
            let ty = self.facts.definitions[definition.item.0 as usize]
                .ok_or_else(|| Error::at(FileId(0), 0, "missing aggregate type"))?;
            self.facts
                .layouts
                .demand(&self.facts.types, &self.facts.aggregates, ty)
                .map_err(|error| {
                    let key = self.report.resolution.definitions[definition.item.0 as usize].node;
                    key.map_or_else(
                        || Error::at(FileId(0), 0, error.message),
                        |key| self.error(key, error.message),
                    )
                })?;
        }
        Ok(())
    }
    fn copy_obligations(&mut self) -> Result<(), Error> {
        for definition in &self.facts.aggregates {
            let key = self.report.resolution.definitions[definition.item.0 as usize]
                .node
                .ok_or_else(|| Error::at(FileId(0), 0, "missing derive syntax"))?;
            let ty = self.facts.definitions[definition.item.0 as usize]
                .ok_or_else(|| self.error(key, "missing derive type"))?;
            let properties = self
                .facts
                .copies
                .demand(
                    &self.facts.types,
                    &self.facts.aggregates,
                    &self.facts.derives,
                    ty,
                )
                .map_err(|e| self.error(key, e))?;
            let selected = self
                .facts
                .derives
                .get(&definition.item)
                .copied()
                .unwrap_or_default();
            if selected.copy && !properties.copy {
                return Err(self.error(key, "derived Copy requires every field to implement Copy"));
            }
            if selected.clone_ && !properties.clone_ {
                if definition.repr.packed.is_some() {
                    return Err(
                        self.error(key, "derived Clone on a packed struct requires Copy fields")
                    );
                }
                return Err(
                    self.error(key, "derived Clone requires every field to implement Clone")
                );
            }
        }
        Ok(())
    }
    fn equality_obligations(&mut self) -> Result<(), Error> {
        for definition in &self.facts.aggregates {
            let Some(&selected) = self.facts.equality_derives.get(&definition.item) else {
                continue;
            };
            let key = self.report.resolution.definitions[definition.item.0 as usize]
                .node
                .ok_or_else(|| Error::at(FileId(0), 0, "missing equality derive syntax"))?;
            let ty = self.facts.definitions[definition.item.0 as usize]
                .ok_or_else(|| self.error(key, "missing equality derive type"))?;
            let properties = self
                .facts
                .equality
                .demand(
                    &self.facts.types,
                    &self.facts.aggregates,
                    &self.facts.equality_derives,
                    ty,
                )
                .map_err(|e| self.error(key, e))?;
            if selected.partial_eq && !properties.partial_eq {
                return Err(self.error(
                    key,
                    "derived PartialEq requires every field to implement PartialEq",
                ));
            }
            if selected.eq && !properties.eq {
                return Err(self.error(key, "derived Eq requires every field to implement Eq"));
            }
            if selected.partial_eq && definition.repr.packed.is_some() {
                for variant in self.facts.aggregates.variants(definition) {
                    for &field in self
                        .facts
                        .types
                        .list(variant.fields)
                        .ok_or_else(|| self.error(key, "missing packed equality fields"))?
                    {
                        let copy = self
                            .facts
                            .copies
                            .demand(
                                &self.facts.types,
                                &self.facts.aggregates,
                                &self.facts.derives,
                                field,
                            )
                            .map_err(|e| self.error(key, e))?;
                        if !copy.copy {
                            return Err(self.error(
                                key,
                                "derived PartialEq on a packed struct requires Copy fields",
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn groups(&self, key: NodeKey) -> Result<(aggregate::Kind, Vec<Group>), Error> {
        match self.item(key)?.kind {
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
                    if !unit.configured.active[node.0 as usize] {
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
    fn unattributed(&self, mut key: NodeKey) -> NodeKey {
        while let Kind::Attributed { value, .. } = self.kind(key) {
            key.node = value;
        }
        key
    }
    fn field_annotations(
        &mut self,
        group: &Group,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        let unit = self.report.package.unit(group.key.file);
        for node in unit.configured.ast.list(group.fields).iter().copied() {
            if !unit.configured.active[node.0 as usize] {
                continue;
            }
            let key = self.unattributed(NodeKey { node, ..group.key });
            let Kind::FieldDefinition { ty, .. } = self.kind(key) else {
                return Err(self.error(key, "invalid field definition"));
            };
            self.node_work(key.file, ty, regions::Context::ALIAS);
        }
        self.drain(steps, limit)
    }
    fn fields(&mut self, group: &Group, enum_: bool) -> Result<Vec<Field>, Error> {
        let unit = self.report.package.unit(group.key.file);
        let mut fields = Vec::new();
        for node in unit.configured.ast.list(group.fields).iter().copied() {
            if !unit.configured.active[node.0 as usize] {
                continue;
            }
            let key = self.unattributed(NodeKey { node, ..group.key });
            let Kind::FieldDefinition { name, ty, .. } = self.kind(key) else {
                return Err(self.error(key, "invalid field definition"));
            };
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
            let ty = self.ty(NodeKey { node: ty, ..key })?;
            if self.facts.types.is_well_formed(ty) != Some(true)
                || self.facts.types.is_sized(ty) != Some(true)
            {
                return Err(self.error(
                    key,
                    "aggregate fields must be well-formed and sized in this profile",
                ));
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
                node: key,
                name,
                ty,
                access,
            });
        }
        Ok(fields)
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
            if !matches!(
                definition.kind,
                DefKind::Constructor | DefKind::Variant | DefKind::Struct
            ) {
                continue;
            }
            let Some(key) = definition.node else {
                continue;
            };
            let Some(variant_id) = self.facts.members.at_node(key) else {
                continue;
            };
            let variant = self.facts.members.variants[variant_id.0 as usize];
            let id =
                DefId(u32::try_from(index).map_err(|_| self.error(key, "too many constructors"))?);
            self.facts.members.constructors.insert(id, variant_id);
            if definition.namespace != Namespace::Value {
                continue;
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
                        bound_regions: 0,
                        parameters: fields,
                        result: owner,
                        unsafe_: false,
                    },
                )?;
                self.intern(
                    key,
                    Type::FunctionItem {
                        item: ItemId(id.0),
                        signature,
                    },
                )?
            } else {
                owner
            };
            self.facts.definitions[index] = Some(ty);
        }
        Ok(())
    }
}
