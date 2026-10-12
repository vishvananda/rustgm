//! Complete nominal declarations after reserving their recursive identities.
use super::{Checker, DefId, DefKind, Error, ItemId, ItemKind, Kind, List, NodeKey, Type, regions};
use crate::{
    literal::Value,
    resolve::{
        model::{Access, Namespace},
        unit::FileId,
    },
    sema::{
        members::{Field, Name},
        representation,
    },
    syntax::{
        ast::{NodeId, Unary},
        item::Shape,
    },
    types::{
        Primitive,
        aggregate::{self, Repr},
    },
};
use std::collections::HashSet;

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
            let ty = self.intern(key, Type::Adt(ItemId(id)))?;
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
            let repr = representations.get(&key).copied().unwrap_or_default();
            let (kind, groups) = self.groups(key)?;
            let tag = repr.tag.unwrap_or(Primitive::Isize);
            let mut next = Some(0);
            let mut used = HashSet::new();
            let mut variants = Vec::with_capacity(groups.len());
            for (ordinal, group) in groups.iter().enumerate() {
                self.field_annotations(group, steps, limit)?;
                let fields = self.fields(group, kind == aggregate::Kind::Enum)?;
                let discr = if let Some(node) = group.discriminant {
                    discriminant(
                        self.report,
                        NodeKey {
                            file: key.file,
                            node,
                        },
                        tag,
                    )?
                } else {
                    next.ok_or_else(|| {
                        self.error(
                            group.key,
                            "implicit enum discriminant overflows its representation",
                        )
                    })?
                };
                if !used.insert(discr) {
                    return Err(self.error(group.key, "duplicate enum discriminant"));
                }
                next = successor(tag, discr);
                self.children.clear();
                self.children.extend(fields.iter().map(|field| field.ty));
                let list = self
                    .facts
                    .types
                    .intern_list(&self.children)
                    .map_err(|e| self.error(group.key, e))?;
                variants.push(aggregate::Variant {
                    fields: list,
                    discriminant: discr,
                });
                let ordinal = u32::try_from(ordinal)
                    .map_err(|_| self.error(group.key, "too many variants"))?;
                self.facts
                    .members
                    .insert(item, ordinal, group.key, group.shape, &fields)
                    .map_err(|e| self.error(group.key, e))?;
            }
            self.check_discriminant_forms(key, repr, &groups)?;
            self.facts
                .aggregates
                .insert(item, kind, repr, &variants)
                .map_err(|e| self.error(key, e))?;
        }
        // Every declared aggregate is checked, including unreachable declarations.
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
        self.constructors()
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

fn discriminant(
    report: &crate::resolve::expansion::Report,
    mut key: NodeKey,
    tag: Primitive,
) -> Result<u128, Error> {
    let error = |message| super::Declarations::error(report, key, message);
    let (signed, width) = tag
        .integer()
        .ok_or_else(|| error("invalid discriminant type"))?;
    let mut negative = false;
    loop {
        match super::Declarations::kind(report, key) {
            Kind::Group(node) => key.node = node,
            Kind::Unary {
                op: Unary::Negate,
                operand,
            } if !negative => {
                negative = true;
                key.node = operand;
            }
            _ => break,
        }
    }
    let error = |message| super::Declarations::error(report, key, message);
    let Kind::Literal(index) = super::Declarations::kind(report, key) else {
        return Err(error(
            "nonliteral discriminants require constant expression evaluation",
        ));
    };
    let Value::Integer(value) = report
        .package
        .unit(key.file)
        .literal(index)
        .map_err(|e| error(e.message))?
    else {
        return Err(error("enum discriminant must be an integer"));
    };
    if value.suffix.is_some_and(|s| s.name() != tag.name()) {
        return Err(error("discriminant literal has the wrong integer type"));
    }
    let maximum = if signed {
        (1_u128 << (width - 1)) - u128::from(!negative)
    } else {
        mask(width)
    };
    if (negative && !signed) || value.magnitude > maximum {
        return Err(error("enum discriminant overflows its representation"));
    }
    Ok(if negative {
        value.magnitude.wrapping_neg() & mask(width)
    } else {
        value.magnitude
    })
}
fn mask(width: u16) -> u128 {
    u128::MAX >> (128 - width)
}
fn successor(tag: Primitive, bits: u128) -> Option<u128> {
    let (signed, width) = tag.integer()?;
    let maximum = if signed {
        (1_u128 << (width - 1)) - 1
    } else {
        mask(width)
    };
    (bits != maximum).then(|| bits.wrapping_add(1) & mask(width))
}
