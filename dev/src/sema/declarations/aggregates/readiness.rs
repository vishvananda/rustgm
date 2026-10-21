//! Separate nominal identities, field templates and constant-value prerequisites.
use super::{
    Checker, DefId, DefKind, Error, Group, ItemId, NodeKey, Repr, State, Type, TypeId, Work,
    aggregate,
};
use std::collections::{HashMap, HashSet};

pub(super) struct Entry {
    pub item: ItemId,
    pub key: NodeKey,
    pub kind: aggregate::Kind,
    pub repr: Repr,
    pub groups: Vec<Group>,
    pub state: State,
    pub contract: State,
    constructors: Vec<DefId>,
}
#[derive(Default)]
pub(in super::super) struct Pending {
    pub(super) entries: Vec<Entry>,
    pub(super) index: HashMap<ItemId, usize>,
    constructors: HashMap<DefId, ItemId>,
    pub(super) contracts: HashMap<TypeId, State>,
    shapes: HashSet<TypeId>,
    value_templates: HashSet<ItemId>,
}
impl Checker<'_> {
    pub(super) fn reserve_fields(
        &mut self,
        item: ItemId,
        key: NodeKey,
        attributes: crate::sema::representation::Attributes,
    ) -> Result<(), Error> {
        if attributes.derives != crate::types::copy::Properties::default() {
            self.facts.derives.insert(item, attributes.derives);
        }
        if attributes.equality != crate::types::equality::Properties::default() {
            self.facts
                .equality_derives
                .insert(item, attributes.equality);
        }
        let (kind, groups) = self.groups(key)?;
        self.nominals
            .index
            .insert(item, self.nominals.entries.len());
        self.nominals.entries.push(Entry {
            item,
            key,
            kind,
            groups,
            repr: attributes.repr,
            state: State::Unseen,
            contract: State::Unseen,
            constructors: Vec::new(),
        });
        Ok(())
    }
    pub(super) fn index_constructors(&mut self) -> Result<(), Error> {
        let groups: HashMap<_, _> = self
            .nominals
            .entries
            .iter()
            .flat_map(|entry| entry.groups.iter().map(|group| (group.key, entry.item)))
            .collect();
        for (index, definition) in self.report.resolution.definitions.iter().enumerate() {
            if !matches!(
                definition.kind,
                DefKind::Constructor | DefKind::Variant | DefKind::Struct
            ) {
                continue;
            }
            let Some((key, owner)) = definition
                .node
                .and_then(|key| groups.get(&key).map(|&owner| (key, owner)))
            else {
                continue;
            };
            let id =
                DefId(u32::try_from(index).map_err(|_| self.error(key, "too many constructors"))?);
            self.nominals.constructors.insert(id, owner);
            self.nominals.entries[self.nominals.index[&owner]]
                .constructors
                .push(id);
        }
        Ok(())
    }
    pub(super) fn publish_constructors(&mut self, item: ItemId) -> Result<(), Error> {
        let entry = &self.nominals.entries[self.nominals.index[&item]];
        for &id in &entry.constructors {
            let key = self.report.resolution.definitions[id.0 as usize]
                .node
                .ok_or_else(|| self.error(entry.key, "missing constructor syntax"))?;
            let variant = self
                .facts
                .members
                .at_node(key)
                .ok_or_else(|| self.error(key, "missing completed constructor fields"))?;
            self.facts.members.constructors.insert(id, variant);
        }
        Ok(())
    }
    pub(in super::super) fn constant_constructor(&mut self, id: DefId) -> Result<(), Error> {
        let &owner = self.nominals.constructors.get(&id).ok_or_else(|| {
            Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "missing constructor owner",
            )
        })?;
        self.work.push(Work::Constructor(id));
        self.work.push(Work::ValueType(DefId(owner.0)));
        Ok(())
    }
    pub(in super::super) fn constant_value_type(&mut self, id: DefId) {
        let Some(ty) = self.facts.definitions[id.0 as usize] else {
            self.work.push(Work::ValueType(id));
            self.work.push(Work::Definition(id, false));
            return;
        };
        self.work.push(Work::ValueShape(ty));
    }
    pub(in super::super) fn constant_value_shape(&mut self, ty: TypeId) -> Result<(), Error> {
        if self.nominals.shapes.contains(&ty) {
            return Ok(());
        }
        if self.nominals.contracts.get(&ty) != Some(&State::Complete) {
            self.work.push(Work::ValueShape(ty));
            self.work.push(Work::ValueContract(ty, false));
            return Ok(());
        }
        if let Some(Type::Adt(item, _)) = self.facts.types.kind(ty) {
            let index = self.nominals.index[&item];
            if self.nominals.entries[index].state != State::Complete {
                self.work.push(Work::ValueShape(ty));
                self.work.push(Work::Fields(item, false));
                return Ok(());
            }
        }
        self.nominals.shapes.insert(ty);
        match self.facts.types.kind(ty) {
            Some(Type::Tuple(fields)) => {
                let fields = self.facts.types.list(fields).ok_or_else(|| {
                    Error::at(
                        crate::resolve::unit::FileId(0),
                        0,
                        "missing tuple value fields",
                    )
                })?;
                self.work
                    .extend(fields.iter().rev().map(|&ty| Work::ValueShape(ty)));
            }
            Some(Type::Array { element, .. } | Type::ArrayTerm { element, .. }) => {
                self.work.push(Work::ValueShape(element));
            }
            Some(Type::Adt(item, arguments)) => {
                let index = self.nominals.index[&item];
                let key = self.nominals.entries[index].key;
                for argument in self
                    .facts
                    .types
                    .arguments(arguments)
                    .ok_or_else(|| self.error(key, "missing nominal value arguments"))?
                {
                    if let crate::types::arguments::Argument::Type(ty) = argument {
                        self.work.push(Work::ValueShape(ty));
                    }
                }
                if self.nominals.value_templates.insert(item) {
                    // Readiness belongs to the declaration. Concrete field
                    // substitution happens after inference, keyed by the actual
                    // arguments; expanding each symbolic owner here is quadratic.
                    let definition = self
                        .facts
                        .aggregates
                        .get(item)
                        .ok_or_else(|| self.error(key, "missing nominal value template"))?;
                    for variant in self.facts.aggregates.variants(definition).iter().rev() {
                        let fields = self
                            .facts
                            .types
                            .list(variant.fields)
                            .ok_or_else(|| self.error(key, "missing nominal value fields"))?;
                        self.work
                            .extend(fields.iter().rev().map(|&ty| Work::ValueShape(ty)));
                    }
                    for group in 0..self.nominals.entries[index].groups.len() {
                        let group = self.nominals.entries[index].groups[group];
                        if let Some(node) = group.discriminant {
                            self.constant_dependencies(
                                NodeKey { node, ..group.key },
                                self.nominal_context(item),
                            )?;
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl Checker<'_> {
    pub(in super::super) fn register_library_nominal(
        &mut self,
        item: ItemId,
        key: NodeKey,
        kind: aggregate::Kind,
        repr: Repr,
        constructors: &[DefId],
    ) -> Result<(), Error> {
        if self
            .nominals
            .index
            .insert(item, self.nominals.entries.len())
            .is_some()
        {
            return Err(self.error(key, "nominal metadata is already registered"));
        }
        for &constructor in constructors {
            self.nominals.constructors.insert(constructor, item);
        }
        self.nominals.entries.push(Entry {
            item,
            key,
            kind,
            repr,
            groups: Vec::new(),
            state: State::Complete,
            contract: State::Complete,
            constructors: constructors.to_vec(),
        });
        self.facts.generics.nominal_contracts.insert(DefId(item.0));
        self.facts.generics.has_nominal_requirements = true;
        Ok(())
    }
}
