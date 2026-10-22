//! Canonical symbolic words are independent of each storage cell and encoding.
use super::{Constant, Failure, Machine};
use crate::sema::values::{WordAddress, WordTarget};
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum WordKey {
    Address(WordAddress),
    Function(crate::types::instance::Instance),
    Vtable {
        concrete: crate::types::TypeId,
        target: crate::types::TypeId,
    },
}
struct Word {
    target: WordTarget,
    key: WordKey,
}
use std::collections::{BTreeMap, HashMap};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct WordId(u32);
#[derive(Clone, Copy)]
pub(super) struct Fragment {
    pub offset: u64,
    pub source_offset: u8,
    pub length: u8,
    pub word: WordId,
}
pub(super) type Fragments = BTreeMap<u64, Fragment>;
#[derive(Default)]
pub(super) struct Words {
    entries: Vec<Word>,
    index: HashMap<WordKey, WordId>,
    shapes: Option<Box<crate::types::substitute::Bound>>,
    shape_slots: usize,
}
impl Words {
    pub(super) fn retained_slots(&self) -> usize {
        self.entries
            .capacity()
            .saturating_mul(size_of::<Word>())
            .saturating_add(
                self.index
                    .capacity()
                    .saturating_mul(4 * size_of::<WordKey>()),
            )
            .saturating_add(4 * size_of::<Self>())
            .div_ceil(size_of::<Constant>())
            .saturating_add(self.shape_slots)
    }
}
pub(super) fn fragment_slots(fragments: &Fragments) -> usize {
    // Covers sparse leaf/internal tree capacity, the empty retained root and headers.
    fragments
        .len()
        .saturating_add(1)
        .saturating_mul(512)
        .div_ceil(size_of::<Constant>())
}
impl Machine<'_, '_> {
    pub(super) fn intern_word(&mut self, target: WordTarget) -> Result<WordId, Failure> {
        self.charge(1)?;
        let key = match target {
            WordTarget::Value(value) => match super::Reader::new(self.evaluator.facts())
                .view(value)?
            {
                crate::sema::values::View::Function(instance) => WordKey::Function(instance),
                crate::sema::values::View::Reference(_) => {
                    let (address, work) = self
                        .evaluator
                        .word_address(value, self.limits.steps.saturating_sub(self.work.steps))?;
                    self.charge(work)?;
                    WordKey::Address(address)
                }
                _ => return Err("symbolic data word has no pointer or function provenance".into()),
            },
            WordTarget::Vtable { concrete, target } => {
                if !matches!(
                    self.evaluator.facts().types.kind(target),
                    Some(crate::types::Kind::Dynamic { .. })
                ) {
                    return Err("symbolic vtable word lacks a dynamic target".into());
                }
                self.footprint(concrete)?;
                WordKey::Vtable {
                    concrete: self.erased_word_type(concrete)?,
                    target: self.erased_word_type(target)?,
                }
            }
        };
        if let Some(word) = self
            .memory
            .bytes
            .words
            .as_ref()
            .and_then(|arena| arena.index.get(&key))
        {
            return Ok(*word);
        }
        let length = self
            .memory
            .bytes
            .words
            .as_ref()
            .map_or(0, |arena| arena.entries.len());
        let id =
            WordId(u32::try_from(length).map_err(|_| "constant symbolic word identity exhausted")?);
        // Conservatively covers retained and resizing vector/hash capacities before mutation.
        self.reserve_cells(
            length
                .saturating_add(4)
                .saturating_mul(1024)
                .div_ceil(size_of::<Constant>()),
        )?;
        let arena = self
            .memory
            .bytes
            .words
            .get_or_insert_with(|| Box::new(Words::default()));
        arena
            .entries
            .try_reserve(1)
            .map_err(|_| "constant symbolic word allocation failed")?;
        arena
            .index
            .try_reserve(1)
            .map_err(|_| "constant symbolic word index allocation failed")?;
        arena.entries.push(Word { target, key });
        arena.index.insert(key, id);
        Ok(id)
    }
    pub(super) fn word_target(&self, id: WordId) -> Result<WordTarget, Failure> {
        self.memory
            .bytes
            .words
            .as_ref()
            .and_then(|arena| arena.entries.get(id.0 as usize))
            .map(|word| word.target)
            .ok_or("missing canonical symbolic word".into())
    }
    pub(super) fn word_address(&self, id: WordId) -> Result<WordAddress, Failure> {
        match self
            .memory
            .bytes
            .words
            .as_ref()
            .and_then(|arena| arena.entries.get(id.0 as usize))
            .map(|word| word.key)
        {
            Some(WordKey::Address(address)) => Ok(address),
            _ => Err("symbolic word is not a data address".into()),
        }
    }
    pub(super) fn erased_word_type(
        &mut self,
        ty: crate::types::TypeId,
    ) -> Result<crate::types::TypeId, Failure> {
        self.charge(1)?;
        let words = self.memory.bytes.words.as_ref();
        let headers = if words.is_none() {
            4 * size_of::<Words>()
        } else {
            0
        } + if words.is_none_or(|words| words.shapes.is_none()) {
            size_of::<crate::types::substitute::Bound>()
        } else {
            0
        };
        self.reserve_cells(headers.div_ceil(size_of::<Constant>()))?;
        let words = self
            .memory
            .bytes
            .words
            .get_or_insert_with(|| Box::new(Words::default()));
        let mut fold = words
            .shapes
            .take()
            .unwrap_or_else(|| Box::new(crate::types::substitute::Bound::shape()));
        let old = words.shape_slots;
        let retained = self.memory.retained_cells().saturating_add(self.slots);
        let bytes = self
            .limits
            .slots
            .saturating_sub(retained)
            .saturating_add(old)
            .saturating_mul(size_of::<Constant>());
        let before = fold.steps;
        let result = self.evaluator.word_shape_key(
            &mut fold,
            ty,
            self.limits.steps.saturating_sub(self.work.steps),
            bytes,
        );
        let charged = fold.steps.saturating_sub(before);
        let slots = fold.shape_retained_bytes()?.div_ceil(size_of::<Constant>());
        let words = self
            .memory
            .bytes
            .words
            .as_mut()
            .ok_or("missing vtable word shape arena")?;
        words.shape_slots = slots;
        words.shapes = Some(fold);
        self.charge(charged)?;
        self.reserve_cells(0)?;
        result
    }
}
