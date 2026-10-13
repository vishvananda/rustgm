//! Ordered scalar domains and an interval index shared by pattern checks and lowering.
use super::Interval;
use crate::{literal::value::FloatBits, sema::model::Scalar, types::Primitive};
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
pub struct Domain {
    primitive: Primitive,
    maximum: u128,
    sign: u128,
}
impl Domain {
    #[must_use]
    pub fn new(primitive: Primitive) -> Option<Self> {
        let (maximum, sign) = if let Some((signed, width)) = primitive.integer() {
            let maximum = u128::MAX >> (128 - width);
            (maximum, if signed { 1_u128 << (width - 1) } else { 0 })
        } else {
            match primitive {
                Primitive::Bool => (1, 0),
                Primitive::Char => (0x10_f7ff, 0),
                // Numeric ranks collapse signed zero; one final class represents all NaNs.
                Primitive::F32 => (2 * 0x7f80_0000 + 1, 1 << 31),
                Primitive::F64 => (2 * 0x7ff0_0000_0000_0000 + 1, 1 << 63),
                _ => return None,
            }
        };
        Some(Self {
            primitive,
            maximum,
            sign,
        })
    }
    #[must_use]
    pub fn full(self) -> Interval {
        Interval {
            lower: 0,
            upper: self.maximum,
        }
    }
    /// Source range bounds exclude the unordered NaN coverage class.
    #[must_use]
    pub fn ordered(self) -> Interval {
        Interval {
            lower: 0,
            upper: self.maximum
                - u128::from(matches!(self.primitive, Primitive::F32 | Primitive::F64)),
        }
    }
    /// # Errors
    /// Rejects values outside this domain and invalid negations.
    pub fn rank(self, value: Scalar, negative: bool) -> Result<u128, &'static str> {
        match (self.primitive, value) {
            (Primitive::Bool, Scalar::Boolean(value)) if !negative => Ok(u128::from(value)),
            (Primitive::Char, Scalar::Character(value)) if !negative => {
                let value = u128::from(u32::from(value));
                Ok(if value >= 0xe000 {
                    value - 0x800
                } else {
                    value
                })
            }
            (_, Scalar::Integer(bits)) if self.primitive.integer().is_some() => {
                if bits > self.maximum || negative && self.sign == 0 {
                    return Err("invalid integer pattern value");
                }
                let bits = if negative {
                    0_u128.wrapping_sub(bits) & self.maximum
                } else {
                    bits
                };
                Ok(bits ^ self.sign)
            }
            (Primitive::F32, Scalar::Float(FloatBits::F32(bits))) => {
                self.float_rank(u128::from(bits), negative)
            }
            (Primitive::F64, Scalar::Float(FloatBits::F64(bits))) => {
                self.float_rank(u128::from(bits), negative)
            }
            _ => Err("pattern value disagrees with its scalar domain"),
        }
    }
    fn float_rank(self, bits: u128, negative: bool) -> Result<u128, &'static str> {
        let infinity = self.maximum / 2;
        let magnitude = bits & !self.sign;
        if magnitude > infinity {
            return Err("NaN is not a valid pattern value");
        }
        Ok(if (bits & self.sign != 0) ^ negative {
            infinity - magnitude
        } else {
            infinity + magnitude
        })
    }
    /// The unordered coverage class maps to a representative NaN, not a source endpoint.
    /// Both zero encodings map to the same rank, whose representative is positive zero.
    ///
    /// # Errors
    /// Rejects a rank outside this domain.
    pub fn value(self, rank: u128) -> Result<Scalar, &'static str> {
        if rank > self.maximum {
            return Err("scalar pattern rank is out of bounds");
        }
        Ok(match self.primitive {
            Primitive::Bool => Scalar::Boolean(rank != 0),
            Primitive::Char => {
                let code = u32::try_from(if rank >= 0xd800 { rank + 0x800 } else { rank })
                    .map_err(|_| "character pattern rank overflow")?;
                Scalar::Character(char::from_u32(code).ok_or("invalid character pattern rank")?)
            }
            Primitive::F32 | Primitive::F64 => return self.float_value(rank),
            _ => Scalar::Integer(rank ^ self.sign),
        })
    }
    fn float_value(self, rank: u128) -> Result<Scalar, &'static str> {
        let infinity = self.maximum / 2;
        let bits = if rank < infinity {
            (infinity - rank) | self.sign
        } else {
            rank - infinity
        };
        Ok(Scalar::Float(match self.primitive {
            Primitive::F32 => FloatBits::F32(if rank == self.maximum {
                f32::NAN.to_bits()
            } else {
                u32::try_from(bits).map_err(|_| "f32 pattern bits overflow")?
            }),
            Primitive::F64 => FloatBits::F64(if rank == self.maximum {
                f64::NAN.to_bits()
            } else {
                u64::try_from(bits).map_err(|_| "f64 pattern bits overflow")?
            }),
            _ => return Err("missing floating pattern domain"),
        }))
    }
    /// # Errors
    /// Rejects a rank outside this scalar domain.
    pub fn witness(self, rank: u128) -> Result<String, &'static str> {
        Ok(match self.value(rank)? {
            Scalar::Boolean(value) => value.to_string(),
            Scalar::Character(value) => format!("{value:?}"),
            Scalar::Integer(bits) if self.sign != 0 => {
                let extended = if bits & self.sign != 0 {
                    bits | !self.maximum
                } else {
                    bits
                };
                i128::from_le_bytes(extended.to_le_bytes()).to_string()
            }
            Scalar::Integer(bits) => bits.to_string(),
            Scalar::Float(FloatBits::F32(bits)) => format!("{:?}", f32::from_bits(bits)),
            Scalar::Float(FloatBits::F64(bits)) => format!("{:?}", f64::from_bits(bits)),
        })
    }
}

/// Disjoint maximal covered intervals. Ordered lookup is part of the algorithm,
/// not a rendering requirement; each insertion/removal costs logarithmic time.
#[derive(Default)]
pub struct Coverage {
    covered: BTreeMap<u128, u128>,
    work: u64,
}
impl Coverage {
    pub const LIMIT: u64 = 1_000_000;
    /// # Errors
    /// Rejects excessive pattern-analysis work.
    pub fn charge(&mut self) -> Result<(), &'static str> {
        self.work += 1;
        if self.work > Self::LIMIT {
            return Err("pattern coverage work budget exceeded");
        }
        Ok(())
    }
    #[must_use]
    pub fn work(&self) -> u64 {
        self.work
    }
    /// # Errors
    /// Rejects excessive pattern-analysis work.
    pub fn useful(&mut self, interval: Interval) -> Result<bool, &'static str> {
        self.charge()?;
        if interval.lower > interval.upper {
            return Err("empty scalar pattern interval");
        }
        Ok(self
            .covered
            .range(..=interval.lower)
            .next_back()
            .is_none_or(|(_, upper)| *upper < interval.upper))
    }
    /// # Errors
    /// Rejects excessive interval-index work.
    pub fn insert(&mut self, mut interval: Interval) -> Result<(), &'static str> {
        self.charge()?;
        if interval.lower > interval.upper {
            return Err("empty scalar pattern interval");
        }
        if let Some((&lower, &upper)) = self.covered.range(..=interval.lower).next_back()
            && upper.saturating_add(1) >= interval.lower
        {
            interval.lower = lower;
            interval.upper = interval.upper.max(upper);
            self.covered.remove(&lower);
        }
        while let Some((&lower, &upper)) = self.covered.range(interval.lower..).next() {
            if lower > interval.upper.saturating_add(1) {
                break;
            }
            self.charge()?;
            interval.upper = interval.upper.max(upper);
            self.covered.remove(&lower);
        }
        self.covered.insert(interval.lower, interval.upper);
        Ok(())
    }
    #[must_use]
    pub fn missing(&self, domain: Domain) -> Option<Interval> {
        let mut lower = 0;
        for (&start, &end) in &self.covered {
            if lower < start {
                return Some(Interval {
                    lower,
                    upper: (start - 1).min(domain.maximum),
                });
            }
            if end >= domain.maximum {
                return None;
            }
            lower = end + 1;
        }
        Some(Interval {
            lower,
            upper: domain.maximum,
        })
    }
}
