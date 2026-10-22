//! Validate stored values against real field roots before importing any local IDs.
use super::{Storage, StorageOrigin, StoredKind, StoredValue, ValueRef};
use crate::support::crates::{
    Constant, CrateIdentity, Function, Lifetime, MAX_COMPONENTS, MAX_NAME_BYTES, Nominals, Scalar,
    Symbol, TypeNode,
};
use std::collections::HashSet;

impl Storage {
    pub(in crate::support::crates) fn validate(
        &self,
        owner: &CrateIdentity,
        functions: &[Function],
        constants: &[Constant],
        types: &Nominals,
    ) -> Result<(usize, usize), &'static str> {
        let mut names: HashSet<_> = functions
            .iter()
            .map(|f| f.definition().path())
            .chain(constants.iter().map(|c| c.definition().path()))
            .collect();
        let mut atoms = self.statics.len().saturating_add(self.values.len());
        let mut bytes = 0usize;
        let closed = closed_types(types);
        for entry in &self.statics {
            if entry.definition().owner() != owner || !names.insert(entry.definition().path()) {
                return Err("foreign or duplicate metadata storage definition");
            }
            if closed.get(usize::from(entry.ty().0)) != Some(&true) {
                return Err("metadata static requires a closed storage type");
            }
            if let StorageOrigin::Defined(root) = entry.origin()
                && self.value(root)?.ty != entry.ty()
            {
                return Err("metadata static initializer differs from its declared type");
            }
            atoms = atoms
                .saturating_add(entry.definition().path().len())
                .saturating_add(1);
            bytes = bytes.saturating_add(
                entry
                    .definition()
                    .path()
                    .iter()
                    .map(String::len)
                    .sum::<usize>(),
            );
            if let Symbol::Exact(name) = entry.symbol() {
                bytes = bytes.saturating_add(name.len());
            }
        }
        for (index, value) in self.values.iter().enumerate() {
            atoms = atoms.saturating_add(self.validate_value(index, value, types)?);
            if atoms > MAX_COMPONENTS || bytes > MAX_NAME_BYTES {
                return Err("metadata storage exceeds profile budget");
            }
        }
        self.reachable()?;
        Ok((atoms, bytes))
    }

    fn value(&self, id: ValueRef) -> Result<&StoredValue, &'static str> {
        self.values
            .get(id.0 as usize)
            .ok_or("unknown metadata stored value")
    }

    fn validate_value(
        &self,
        index: usize,
        value: &StoredValue,
        types: &Nominals,
    ) -> Result<usize, &'static str> {
        match &value.kind {
            StoredKind::Scalar(scalar) => {
                if types.types().node(value.ty) != Some(TypeNode::Scalar(scalar.ty())) {
                    return Err("metadata scalar storage differs from its declared type");
                }
                Ok(1)
            }
            StoredKind::Product(fields) => {
                let expected = match types.types().node(value.ty) {
                    Some(TypeNode::Record(record)) => {
                        types.records()[usize::from(record.0)].fields()
                    }
                    Some(TypeNode::Scalar(Scalar::Unit)) => &[],
                    _ => return Err("metadata product lacks canonical record storage"),
                };
                if fields.len() != expected.len() {
                    return Err("metadata product field count differs");
                }
                for (&child, field) in fields.iter().zip(expected) {
                    if child.0 as usize >= index || self.value(child)?.ty != field.ty() {
                        return Err("metadata product has a forward or differently typed field");
                    }
                }
                Ok(fields.len())
            }
            StoredKind::Reference { target, fields } => {
                let Some(TypeNode::Reference {
                    region: Lifetime::Static,
                    mutable: false,
                    referent,
                }) = types.types().node(value.ty)
                else {
                    return Err("metadata stored reference requires a static shared type");
                };
                let entry = self
                    .statics
                    .get(target.0 as usize)
                    .ok_or("unknown metadata static target")?;
                let mut selected = entry.ty();
                for &ordinal in fields {
                    let Some(TypeNode::Record(record)) = types.types().node(selected) else {
                        return Err("metadata static projection lacks record storage");
                    };
                    selected = types.records()[usize::from(record.0)]
                        .fields()
                        .get(ordinal as usize)
                        .ok_or("metadata static projection exceeds its record")?
                        .ty();
                }
                if selected != referent {
                    return Err("metadata static reference has a different referent");
                }
                Ok(fields.len().saturating_add(1))
            }
        }
    }

    fn reachable(&self) -> Result<(), &'static str> {
        enum Work {
            Static(usize),
            Value(ValueRef),
        }
        let mut work: Vec<_> = self
            .statics
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| entry.public().then_some(Work::Static(i)))
            .collect();
        let mut statics = vec![false; self.statics.len()];
        let mut values = vec![false; self.values.len()];
        while let Some(next) = work.pop() {
            match next {
                Work::Static(index) => {
                    if std::mem::replace(&mut statics[index], true) {
                        continue;
                    }
                    if let StorageOrigin::Defined(value) = self.statics[index].origin() {
                        work.push(Work::Value(value));
                    }
                }
                Work::Value(id) => {
                    if std::mem::replace(&mut values[id.0 as usize], true) {
                        continue;
                    }
                    match &self.value(id)?.kind {
                        StoredKind::Scalar(_) => {}
                        StoredKind::Product(fields) => {
                            work.extend(fields.iter().copied().map(Work::Value));
                        }
                        StoredKind::Reference { target, .. } => {
                            work.push(Work::Static(target.0 as usize));
                        }
                    }
                }
            }
        }
        if statics.contains(&false) || values.contains(&false) {
            return Err("metadata contains unreachable storage or values");
        }
        Ok(())
    }
}

fn closed_types(types: &Nominals) -> Vec<bool> {
    let mut closed = Vec::with_capacity(types.types().nodes().len());
    for node in types.types().nodes() {
        closed.push(match *node {
            TypeNode::Scalar(_) | TypeNode::Record(_) => true,
            TypeNode::Array { element, .. } => closed[usize::from(element.0)],
            TypeNode::Reference {
                region, referent, ..
            } => region == Lifetime::Static && closed[usize::from(referent.0)],
        });
    }
    closed
}
