//! Pointer validity is a typed traversal context, never authority on a stored value.
use super::{Constant, Failure, Machine, Reader, TypeId, View};
use crate::sema::values::{Metadata, StorageLayout};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate::const_eval) struct ValueType {
    pub ty: TypeId,
    pub may_dangle: bool,
}
impl ValueType {
    pub(in crate::const_eval) const fn root(ty: TypeId) -> Self {
        Self {
            ty,
            may_dangle: false,
        }
    }
    pub(in crate::const_eval) fn field(self, reader: Reader<'_>, ty: TypeId, index: u64) -> Self {
        Self {
            ty,
            may_dangle: self.may_dangle || reader.field_may_dangle(self.ty, index),
        }
    }
}
impl Machine<'_, '_> {
    /// Size, metadata, alignment and nonnull validity remain required while dangling.
    pub(in crate::const_eval) fn reference_bits(
        &mut self,
        value: Constant,
        referent: TypeId,
    ) -> Result<StorageLayout, Failure> {
        let layout = self.reference_extent(value, referent)?;
        match Reader::new(self.evaluator.facts()).view(value)? {
            View::Address { bits } => {
                if bits == 0 {
                    return Err("constant reference is null".into());
                }
                if !bits.is_multiple_of(u64::from(layout.align)) {
                    return Err("constant reference is unaligned".into());
                }
            }
            View::Reference(_) => {
                let (crate::sema::values::WordTarget::Value(data), _) =
                    self.evaluator.pointer_words(value)?
                else {
                    return Err("constant reference has no data address".into());
                };
                let (address, work) = self
                    .evaluator
                    .word_address(data, self.limits.steps.saturating_sub(self.work.steps))?;
                self.charge(work)?;
                let ty = Reader::new(self.evaluator.facts())
                    .location(address.root())?
                    .ty;
                let (root, work) = self
                    .evaluator
                    .address_layout(ty, self.limits.steps.saturating_sub(self.work.steps))?;
                self.charge(work)?;
                if root.align < layout.align
                    || !address.offset().is_multiple_of(u64::from(layout.align))
                {
                    return Err("constant reference is unaligned".into());
                }
            }
            _ => return Err("constant reference has no pointer representation".into()),
        }
        Ok(layout)
    }
    fn reference_extent(
        &mut self,
        value: Constant,
        referent: TypeId,
    ) -> Result<StorageLayout, Failure> {
        match value.metadata() {
            None => self.footprint(referent),
            Some(Metadata::Object { concrete, target })
                if referent == concrete || referent == target =>
            {
                self.footprint(concrete)
            }
            Some(Metadata::Length(length)) => {
                let element = Reader::new(self.evaluator.facts()).sequence_element(referent)?;
                let (layout, work) = self
                    .evaluator
                    .address_layout(element, self.limits.steps.saturating_sub(self.work.steps))?;
                self.charge(work)?;
                let size = layout
                    .size
                    .checked_mul(length)
                    .filter(|&size| size <= u64::MAX / 2)
                    .ok_or("constant reference metadata exceeds the target object size")?;
                Ok(StorageLayout {
                    size,
                    align: layout.align,
                })
            }
            Some(Metadata::Object { .. }) => {
                Err("constant reference has mismatched object metadata".into())
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Entry {
    Value {
        value: Constant,
        validity: ValueType,
    },
    Provenance(Constant),
}
use crate::support::{
    hash::HashSet,
    retained::{self, Growth},
};
use crate::types::Kind;
impl Machine<'_, '_> {
    pub(in crate::const_eval) fn validate_result(
        &mut self,
        root: Constant,
        ty: TypeId,
    ) -> Result<(), Failure> {
        let view = Reader::new(self.evaluator.facts()).view(root)?;
        match view {
            View::Product { .. } | View::Repeat { .. } | View::Image { .. } => {}
            _ => {
                // A terminal value needs no graph queue or visited table. Keep
                // pointer and backing validation while avoiding artificial
                // temporary peaks on every scalar function/initializer return.
                self.charge(1)?;
                if matches!(view, View::Reference(_) | View::Address { .. }) {
                    self.result_reference(root, ValueType::root(ty))?;
                }
                return self.result_reference_storage(ty, root);
            }
        }
        let previous = self.slots;
        let result = (|| {
            let mut pending = Vec::new();
            let mut seen = HashSet::default();
            self.reserve_result_work(&mut pending, &mut seen, 1, previous)?;
            pending.push(Entry::Value {
                value: root,
                validity: ValueType::root(ty),
            });
            while let Some(entry) = pending.pop() {
                self.reserve_result_work(&mut pending, &mut seen, 0, previous)?;
                if !seen.insert(entry) {
                    continue;
                }
                match entry {
                    Entry::Provenance(value) => {
                        self.path(value)?;
                    }
                    Entry::Value { value, validity } => {
                        self.validate_result_value(
                            value,
                            validity,
                            &mut pending,
                            &mut seen,
                            previous,
                        )?;
                    }
                }
            }
            self.result_reference_storage(ty, root)
        })();
        self.slots = previous;
        result
    }

    fn result_reference_storage(&mut self, ty: TypeId, root: Constant) -> Result<(), Failure> {
        let (answer, work) = Reader::new(self.evaluator.facts()).shared_reference_work(ty, root)?;
        self.charge(work)?;
        match answer {
            crate::types::interior_mutability::Answer::Ordinary => Ok(()),
            crate::types::interior_mutability::Answer::Interior => Err(Failure::Fault(
                "constant reference cannot escape with interior mutable allocation backing",
            )),
            crate::types::interior_mutability::Answer::Unresolved => {
                Err("constant reference storage requires completed type and value facts".into())
            }
        }
    }

    fn reserve_result_work(
        &mut self,
        pending: &mut Vec<Entry>,
        seen: &mut HashSet<Entry>,
        additional: usize,
        base: usize,
    ) -> Result<(), Failure> {
        let growth = Growth::vector::<Entry>(pending.len(), pending.capacity(), additional);
        let table = Growth::map::<Entry, ()>(seen.len(), seen.capacity(), 1);
        self.charge(
            1usize
                .saturating_add(growth.work)
                .saturating_add(table.work),
        )?;
        self.result_work_slots(pending, seen, base)?;
        self.reserve_cells(
            growth
                .bytes
                .saturating_add(table.bytes)
                .div_ceil(size_of::<Constant>()),
        )?;
        pending
            .try_reserve(additional)
            .map_err(|_| "constant result work allocation failed")?;
        seen.try_reserve(1)
            .map_err(|_| "constant result memo allocation failed")?;
        self.result_work_slots(pending, seen, base)
    }
    fn result_work_slots(
        &mut self,
        pending: &Vec<Entry>,
        seen: &HashSet<Entry>,
        base: usize,
    ) -> Result<(), Failure> {
        let bytes = retained::vector(pending)
            .saturating_add(seen.capacity().saturating_mul(size_of::<Entry>()));
        self.slots = base
            .checked_add(bytes.div_ceil(size_of::<Constant>()))
            .ok_or("constant result work retention overflow")?;
        self.reserve_cells(0)
    }
    fn validate_result_value(
        &mut self,
        value: Constant,
        validity: ValueType,
        pending: &mut Vec<Entry>,
        seen: &mut HashSet<Entry>,
        base: usize,
    ) -> Result<(), Failure> {
        match Reader::new(self.evaluator.facts()).view(value)? {
            View::Reference(_) | View::Address { .. } => self.result_reference(value, validity)?,
            View::Image { .. } => {
                if let Some(image) = Reader::new(self.evaluator.facts()).array_image(value)? {
                    self.charge(image.bytes.len())?;
                    Reader::new(self.evaluator.facts()).require_publishable_array(value)?;
                }
                let length = Reader::new(self.evaluator.facts())
                    .image(value)?
                    .fragments
                    .len();
                self.charge(length)?;
                self.reserve_result_work(pending, seen, length, base)?;
                let image = Reader::new(self.evaluator.facts()).image(value)?;
                for fragment in image.complete_words()? {
                    if let crate::sema::values::WordTarget::Value(value) = fragment.target
                        && value.location().is_some()
                    {
                        pending.push(Entry::Provenance(value));
                    }
                }
            }
            View::Product { length, .. } => {
                self.charge(
                    usize::try_from(length).map_err(|_| "constant result exceeds host size")?,
                )?;
                self.reserve_result_work(
                    pending,
                    seen,
                    usize::try_from(length).map_err(|_| "constant result exceeds host size")?,
                    base,
                )?;
                let fields =
                    Reader::new(self.evaluator.facts()).field_recipe(validity.ty, value)?;
                if fields.len(&self.evaluator.facts().types)? != length {
                    return Err("constant result arity differs from its stored types".into());
                }
                for index in 0..length {
                    let reader = Reader::new(self.evaluator.facts());
                    let ty = fields
                        .get(&self.evaluator.facts().types, index)
                        .ok_or("missing constant result field type")?;
                    let child = validity.field(reader, ty, index);
                    pending.push(Entry::Value {
                        value: self.stored_field(value, index)?,
                        validity: child,
                    });
                }
            }
            View::Repeat {
                value: child,
                length,
            } if length != 0 => {
                let reader = Reader::new(self.evaluator.facts());
                let ty = reader
                    .field_types(validity.ty, value)?
                    .get(0)
                    .ok_or("missing constant result element type")?;
                let validity = validity.field(reader, ty, 0);
                self.reserve_result_work(pending, seen, 1, base)?;
                pending.push(Entry::Value {
                    value: child,
                    validity,
                });
            }
            _ => {}
        }
        Ok(())
    }
    fn result_reference(&mut self, value: Constant, validity: ValueType) -> Result<(), Failure> {
        let ty = Reader::new(self.evaluator.facts()).storage_type(validity.ty)?;
        let Some(Kind::Reference {
            referent, mutable, ..
        }) = self.evaluator.facts().types.kind(ty)
        else {
            if value.location().is_some() {
                self.path(value)?;
            }
            return Ok(());
        };
        if validity.may_dangle {
            self.reference_bits(value, referent)?;
            return Ok(());
        }
        if matches!(
            Reader::new(self.evaluator.facts()).view(value)?,
            View::Address { .. }
        ) {
            if self.reference_bits(value, referent)?.size != 0 {
                return Err("constant reference is dangling without allocation provenance".into());
            }
            return Ok(());
        }
        let path = self.path(value)?;
        // Ordinary source values already carry construction validity. A source
        // relaxation can introduce unchecked referents, so replay the normal
        // access proof after leaving that typed scope. Named static contents
        // remain owned by their initializer (or the external definition).
        if self
            .evaluator
            .facts()
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.language(crate::support::bindings::LangItem::MaybeDangling))
            .is_some()
            && !matches!(
                Reader::new(self.evaluator.facts())
                    .location(path.root)?
                    .origin,
                super::Origin::Static(_)
            )
            && !matches!(
                self.validate_word_reference(value, referent, mutable)?,
                super::Read::Value(_)
            )
        {
            return Err("constant result reference requires completed backing validity".into());
        }
        Ok(())
    }
}
