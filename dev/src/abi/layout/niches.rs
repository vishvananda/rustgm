//! Canonical nullable carriers are physical facts, never reference permissions.
use super::zero_carriers::Answer;
use super::{Carrier, Layout, Layouts, Range, Shape, Variant, index};
use crate::support::hash::HashMap;
use crate::types::{
    Kind, Store, TypeId,
    aggregate::{Definitions, NullableEnum, Representation, SharedMutation},
};
#[cfg(test)]
#[path = "../../../tests/invariants/nullable_layout.rs"]
mod tests;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NicheId(u32);
#[derive(Clone, Copy, Debug)]
pub struct Niche {
    pub owner: TypeId,
    pub empty: u32,
    pub payload: u32,
    pub field: u32,
    pub carrier: Carrier,
    pub ty: TypeId,
    pub offset: u64,
}
pub(super) struct Budget {
    remaining: usize,
    pub exhausted: bool,
}
impl Budget {
    pub(super) const fn new(remaining: usize) -> Self {
        Self {
            remaining,
            exhausted: false,
        }
    }
    pub(super) fn charge(&mut self, steps: &mut usize, count: usize) -> Result<(), &'static str> {
        let Some(remaining) = self.remaining.checked_sub(count) else {
            self.exhausted = true;
            return Err("nullable layout work budget exceeded");
        };
        *steps = steps
            .checked_add(count)
            .ok_or("nullable layout work overflow")?;
        self.remaining = remaining;
        Ok(())
    }
    fn reserve(&mut self, bytes: usize) -> Result<(), &'static str> {
        if bytes > 64 * 1024 * 1024 {
            self.exhausted = true;
            return Err("nullable layout storage budget exceeded");
        }
        Ok(())
    }
}
#[derive(Default)]
pub(super) struct Niches {
    records: Vec<Niche>,
    carriers: HashMap<TypeId, Answer>,
}
impl Niches {
    fn retained(&self) -> usize {
        self.records
            .capacity()
            .saturating_mul(size_of::<Niche>())
            .saturating_add(
                self.carriers
                    .capacity()
                    .saturating_mul(4 * size_of::<(TypeId, Answer)>()),
            )
            .saturating_add(4 * size_of::<Self>())
    }
    fn remember(
        &mut self,
        ty: TypeId,
        answer: Answer,
        scratch: usize,
        budget: &mut Budget,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        budget.charge(steps, 1)?;
        if self.carriers.len() == self.carriers.capacity() {
            budget.charge(steps, self.carriers.len())?;
            let growth = self
                .carriers
                .capacity()
                .saturating_mul(2)
                .max(4)
                .saturating_mul(4 * size_of::<(TypeId, Answer)>());
            budget.reserve(
                self.retained()
                    .saturating_add(scratch)
                    .saturating_add(growth),
            )?;
            self.carriers.try_reserve(1).map_err(|_| {
                budget.exhausted = true;
                "nullable carrier cache allocation failed"
            })?;
        }
        self.carriers.insert(ty, answer);
        Ok(())
    }
    fn carrier(
        &mut self,
        layouts: &mut Layouts,
        types: &Store,
        definitions: &Definitions,
        root: TypeId,
        budget: &mut Budget,
    ) -> Result<Answer, &'static str> {
        let mut path = Vec::new();
        let mut ty = root;
        let answer = loop {
            budget.charge(&mut layouts.steps, 1)?;
            if let Some(answer) = self.carriers.get(&ty) {
                break *answer;
            }
            if path.len() == path.capacity() {
                let capacity = path.capacity().saturating_mul(2).max(4);
                budget.reserve(
                    self.retained()
                        .saturating_add(capacity.saturating_mul(4 * size_of::<TypeId>())),
                )?;
                budget.charge(&mut layouts.steps, path.len())?;
                path.try_reserve(1).map_err(|_| {
                    budget.exhausted = true;
                    "nullable carrier scratch allocation failed"
                })?;
            }
            path.push(ty);
            let layout = layouts
                .get(ty)
                .ok_or("nullable carrier lacks its completed layout")?;
            match types.kind(ty).ok_or("missing nullable carrier type")? {
                Kind::Pattern {
                    constraint: crate::types::Constraint::NonNull,
                    ..
                } if matches!(layout.shape, Shape::Scalar | Shape::WidePointer)
                    && matches!(layout.size, 8 | 16)
                    && layout.align == 8 =>
                {
                    break Answer::Carrier(Carrier::NonNullPointer { leaf: ty });
                }
                Kind::Pattern {
                    base,
                    constraint: crate::types::Constraint::ScalarRange(id),
                    ..
                } => {
                    break Carrier::unsigned(types, ty, base, id, layout.size)?
                        .map_or(Answer::WiderScalar, Answer::Carrier);
                }
                Kind::Reference { .. } | Kind::Function { .. }
                    if matches!(layout.shape, Shape::Scalar)
                        && layout.size == 8
                        && layout.align == 8 =>
                {
                    break Answer::Carrier(Carrier::NonNullPointer { leaf: ty });
                }
                Kind::Adt(item, _) => {
                    let definition = definitions
                        .get(item)
                        .ok_or("missing nullable wrapper declaration")?;
                    if definition.kind != crate::types::aggregate::Kind::Struct
                        || definition.repr.representation != Representation::Transparent
                        || definitions.shared_mutation(item) != Some(SharedMutation::Ordinary)
                    {
                        break Answer::Absent;
                    }
                    let Some(child) = layouts.transparent_carrier_child(types, *layout, budget)?
                    else {
                        break Answer::Absent;
                    };
                    ty = child;
                }
                _ => break Answer::Absent,
            }
        };
        let scratch = path.capacity().saturating_mul(size_of::<TypeId>());
        for ty in path {
            self.remember(ty, answer, scratch, budget, &mut layouts.steps)?;
        }
        Ok(answer)
    }
}
impl Layouts {
    /// Read the retained nullable encoding without interpreting an absent physical tag.
    /// # Errors
    /// Rejects a niche shape whose opaque descriptor was not published.
    pub fn niche(&self, layout: &Layout) -> Result<Option<&Niche>, &'static str> {
        let Shape::Niche { encoding, variants } = layout.shape else {
            return Ok(None);
        };
        let niche = self
            .niches
            .as_ref()
            .and_then(|n| n.records.get(encoding.0 as usize))
            .ok_or("missing retained nullable encoding")?;
        let actual = self
            .get(niche.owner)
            .ok_or("nullable encoding lacks its completed owner")?;
        if !matches!(actual.shape, Shape::Niche { encoding: id, variants: range } if id == encoding && range.start == variants.start && range.end == variants.end)
            || actual.size != layout.size
            || actual.align != layout.align
            || actual.inhabited != layout.inhabited
            || actual.contains_reference != layout.contains_reference
            || actual.contains_explicit_align != layout.contains_explicit_align
        {
            return Err("nullable encoding differs from its completed owner layout");
        }
        Ok(Some(niche))
    }
    // Absence is not a proof of ineligibility; this facade returns only positive witnesses.
    pub(crate) fn retained_carrier(&self, ty: TypeId) -> Option<Carrier> {
        self.niches.as_ref()?.carriers.get(&ty).copied()?.carrier()
    }
    pub(crate) fn retained_pointer_carrier(&self, ty: TypeId) -> Option<TypeId> {
        self.retained_carrier(ty)
            .filter(|carrier| carrier.is_pointer())
            .map(Carrier::leaf)
    }
    pub(crate) fn scalar_carrier(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
    ) -> Result<Option<Carrier>, &'static str> {
        let mut budget = Budget::new(1_000_000);
        self.nullable_carrier(types, definitions, ty, &mut budget)
            .map(Answer::carrier)
    }
    #[cfg(test)]
    pub(crate) fn pointer_carrier(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
    ) -> Result<Option<TypeId>, &'static str> {
        self.scalar_carrier(types, definitions, ty).map(|answer| {
            answer
                .filter(|carrier| carrier.is_pointer())
                .map(Carrier::leaf)
        })
    }
    fn nullable_carrier(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
        budget: &mut Budget,
    ) -> Result<Answer, &'static str> {
        budget.reserve(4 * size_of::<Niches>())?;
        let mut cache = self.niches.take().unwrap_or_default();
        let answer = cache.carrier(self, types, definitions, ty, budget);
        self.niches = Some(cache);
        answer
    }
    fn transparent_carrier_child(
        &mut self,
        types: &Store,
        layout: Layout,
        budget: &mut Budget,
    ) -> Result<Option<TypeId>, &'static str> {
        let [variant] = self.variants(&layout) else {
            return Err("transparent carrier lacks one physical variant");
        };
        let fields = types
            .list(variant.fields)
            .ok_or("missing transparent carrier fields")?;
        let offsets = self.offsets(variant);
        if fields.len() != offsets.len() {
            return Err("transparent carrier field offsets differ");
        }
        let count = fields.len();
        budget.charge(&mut self.steps, count)?;
        let [variant] = self.variants(&layout) else {
            return Err("transparent carrier layout changed");
        };
        let offsets = self.offsets(variant);
        let mut selected = None;
        for (&ty, &offset) in fields.iter().zip(offsets) {
            let child = self.get(ty).ok_or("missing transparent child layout")?;
            if child.size == 0 {
                if child.align != 1 {
                    return Ok(None);
                }
            } else if selected.replace(ty).is_some()
                || offset != 0
                || child.size != layout.size
                || child.align != layout.align
            {
                return Ok(None);
            }
        }
        Ok(selected)
    }
    pub(super) fn nullable_layout(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        owner: TypeId,
        shape: crate::types::aggregate::Shape<'_>,
        policy: NullableEnum,
        budget: &mut Budget,
    ) -> Result<Option<Layout>, &'static str> {
        let payload = shape
            .variant(policy.payload as usize)
            .ok_or("missing nullable payload layout")?;
        let [ty] = types
            .list(payload.fields)
            .ok_or("missing nullable payload fields")?
        else {
            return Err("nullable payload differs from canonical field count");
        };
        let carrier = match self.nullable_carrier(types, definitions, *ty, budget)? {
            Answer::Carrier(carrier) => carrier,
            Answer::Absent => return Ok(None),
            Answer::WiderScalar => {
                return Err("scalar pattern enum niches require their complete ABI profile");
            }
        };
        let layout = *self.get(*ty).ok_or("missing nullable payload storage")?;
        if (carrier.is_pointer() && (!matches!(layout.size, 8 | 16) || layout.align != 8))
            || (!carrier.is_pointer()
                && (layout.size != carrier.bytes() || u64::from(layout.align) != carrier.bytes()))
        {
            return Err("nullable payload differs from pointer storage");
        }
        let variants = Range {
            start: index(self.variants.len())?,
            end: index(self.variants.len() + 2)?,
        };
        for ordinal in 0..2 {
            let variant = shape.variant(ordinal).ok_or("missing nullable variant")?;
            let fields = types
                .list(variant.fields)
                .ok_or("missing nullable physical fields")?;
            let start = index(self.offsets.len())?;
            budget.charge(&mut self.steps, fields.len().saturating_add(1))?;
            self.offsets.extend(std::iter::repeat_n(0, fields.len()));
            self.variants.push(Variant {
                fields: variant.fields,
                offsets: Range {
                    start,
                    end: index(self.offsets.len())?,
                },
                discriminant: variant.discriminant,
            });
        }
        let table = self.niches.as_mut().ok_or("missing nullable table")?;
        let growth = if table.records.len() == table.records.capacity() {
            table
                .records
                .capacity()
                .saturating_mul(2)
                .max(4)
                .saturating_mul(4 * size_of::<Niche>())
        } else {
            0
        };
        budget.reserve(table.retained().saturating_add(growth))?;
        if growth != 0 {
            budget.charge(&mut self.steps, table.records.len())?;
        }
        table.records.try_reserve(1).map_err(|_| {
            budget.exhausted = true;
            "nullable encoding allocation failed"
        })?;
        let encoding = NicheId(index(table.records.len())?);
        table.records.push(Niche {
            owner,
            empty: policy.empty,
            payload: policy.payload,
            field: policy.field,
            carrier,
            ty: *ty,
            offset: 0,
        });
        Ok(Some(Layout {
            shape: Shape::Niche { variants, encoding },
            inhabited: true,
            ..layout
        }))
    }
    pub(super) fn nominal_layout(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
        budget: &mut Budget,
    ) -> Result<Layout, &'static str> {
        let Kind::Adt(item, _) = types.kind(ty).ok_or("missing nominal layout type")? else {
            return Err("nominal layout requires a nominal identity");
        };
        let shape = definitions.shape(types, ty)?;
        let policy = definitions.nullable_enum(item);
        if let Some(policy) = policy
            && let Some(layout) =
                self.nullable_layout(types, definitions, ty, shape, policy, budget)?
        {
            return Ok(layout);
        }
        if shape.definition.repr.representation == Representation::Simd {
            return self.simd_layout(types, definitions, ty, shape, budget);
        }
        self.aggregate(
            types,
            shape,
            definitions.packing_checked(),
            policy.and_then(|p| p.fallback_tag),
        )
    }
    pub(super) fn niche_mark(&self) -> usize {
        self.niches.as_ref().map_or(0, |n| n.records.len())
    }
    pub(super) fn rollback_niches(&mut self, mark: usize) {
        if let Some(niches) = self.niches.as_mut() {
            niches.records.truncate(mark);
        }
    }
}
