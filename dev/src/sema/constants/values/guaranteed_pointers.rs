//! Guaranteed comparison uses allocation provenance without reading pointed storage.
use super::{Constant, Evaluator, Failure, LocationId, Origin, Reader, View};
use crate::{
    resolve::body_origin::BodyOrigin,
    sema::{compiler_items::guaranteed_compare::Selection, model::Scalar},
    types::instance::Instance,
};

enum Word {
    Integer(u64),
    Pointer {
        root: Option<LocationId>,
        size: u64,
        align: u32,
        offset: u64,
    },
}
impl Evaluator<'_> {
    pub(crate) fn checked_guaranteed_compare(
        &self,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<Selection>, Failure> {
        let Some(catalog) = self.facts().source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = catalog.guaranteed_compare(self.facts(), instance, origin)? else {
            return Ok(None);
        };
        catalog.require_guaranteed_report(self.queries.report, self.facts(), &selected)?;
        Ok(Some(selected))
    }
    fn guaranteed_word(
        &mut self,
        value: Constant,
        work: &mut usize,
        limit: usize,
    ) -> Result<Word, Failure> {
        *work = work.saturating_add(1);
        if *work > limit {
            return Err("guaranteed pointer query work exceeded".into());
        }
        let outcome = (|| match Reader::new(self.facts()).view(value)? {
            View::Address { bits } => Ok(Word::Integer(bits)),
            View::Function(_) => Ok(Word::Pointer {
                root: None,
                size: 0,
                align: 1,
                offset: 0,
            }),
            View::Reference(_) => {
                // This follows ancestry only: neither liveness nor pointee initialization is read.
                let (address, added) = self.word_address(value, limit.saturating_sub(*work))?;
                *work = work.saturating_add(added);
                let ty = Reader::new(self.facts()).location(address.root())?.ty;
                let (layout, added) = self.address_layout(ty, limit.saturating_sub(*work))?;
                *work = work.saturating_add(added);
                if *work > limit {
                    return Err("guaranteed pointer layout work exceeded".into());
                }
                Ok(Word::Pointer {
                    root: Some(address.root()),
                    size: layout.size,
                    align: layout.align,
                    offset: address.offset(),
                })
            }
            _ => Err("guaranteed pointer query requires its actual supported symbolic word".into()),
        })();
        // Existing layout adapters report successful work. Conservatively consume the
        // remaining admitted bound on failure; no failed query work is refunded.
        if outcome.is_err() {
            *work = limit;
        }
        outcome
    }
    pub(crate) fn guaranteed_pointer_compare(
        &mut self,
        left: Constant,
        right: Constant,
        work: &mut usize,
        limit: usize,
    ) -> Result<Constant, Failure> {
        let left = self.guaranteed_word(left, work, limit)?;
        let right = self.guaranteed_word(right, work, limit)?;
        let answer = match (left, right) {
            (Word::Integer(left), Word::Integer(right)) => u8::from(left == right),
            (
                Word::Integer(bits),
                Word::Pointer {
                    size,
                    align,
                    offset,
                    ..
                },
            )
            | (
                Word::Pointer {
                    size,
                    align,
                    offset,
                    ..
                },
                Word::Integer(bits),
            ) => {
                let offset = offset.wrapping_sub(bits);
                if offset <= size || !offset.is_multiple_of(u64::from(align)) {
                    0
                } else {
                    2
                }
            }
            (
                Word::Pointer {
                    root: left,
                    size: left_size,
                    align: left_align,
                    offset: left_offset,
                },
                Word::Pointer {
                    root: right,
                    size: right_size,
                    align: right_align,
                    offset: right_offset,
                },
            ) => {
                let align = u64::from(left_align.min(right_align));
                if align > 1 && left_offset % align != right_offset % align {
                    0
                } else if let (Some(left), Some(right)) = (left, right)
                    && let (Origin::Static(left), Origin::Static(right)) = (
                        Reader::new(self.facts()).location(left)?.origin,
                        Reader::new(self.facts()).location(right)?.origin,
                    )
                {
                    if left == right {
                        u8::from(left_offset == right_offset)
                    } else if left_offset < left_size && right_offset < right_size {
                        0
                    } else {
                        2
                    }
                } else {
                    2
                }
            }
        };
        Ok(Constant::scalar(Scalar::Integer(u128::from(answer))))
    }
}
