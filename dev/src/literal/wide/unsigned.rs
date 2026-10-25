//! Bounded nonnegative limbs used only by demanded wide literal conversion.
use std::cmp::Ordering;

const MAX_WORDS: usize = 2048;
#[derive(Clone, Default)]
pub(super) struct Unsigned {
    words: Vec<u32>,
}
impl Unsigned {
    #[cfg(test)]
    pub fn reserved_words(&self) -> usize {
        self.words.capacity()
    }
    pub fn one() -> Self {
        Self { words: vec![1] }
    }
    pub fn is_zero(&self) -> bool {
        self.words.is_empty()
    }
    pub fn bit_len(&self) -> usize {
        self.words.last().map_or(0, |top| {
            32 * (self.words.len() - 1) + (32 - top.leading_zeros()) as usize
        })
    }
    fn trim(&mut self) {
        while self.words.last() == Some(&0) {
            self.words.pop();
        }
    }
    pub fn multiply_add(&mut self, factor: u32, addend: u32) -> Result<(), &'static str> {
        let mut carry = u64::from(addend);
        for word in &mut self.words {
            let product = u64::from(*word) * u64::from(factor) + carry;
            let bytes = product.to_le_bytes();
            *word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            carry = product >> 32;
        }
        if carry != 0 {
            if self.words.len() == MAX_WORDS {
                return Err("wide literal scratch bound exceeded");
            }
            self.words
                .push(u32::try_from(carry).map_err(|_| "wide literal carry overflow")?);
        }
        Ok(())
    }
    pub fn power_ten(&mut self, mut count: i64) -> Result<(), &'static str> {
        while count >= 9 {
            self.multiply_add(1_000_000_000, 0)?;
            count -= 9;
        }
        let mut factor = 1;
        while count > 0 {
            factor *= 10;
            count -= 1;
        }
        self.multiply_add(factor, 0)
    }
    pub fn shift_left(&mut self, bits: i64) -> Result<(), &'static str> {
        if self.is_zero() || bits == 0 {
            return Ok(());
        }
        let bits = usize::try_from(bits).map_err(|_| "negative wide literal shift")?;
        let whole = bits / 32;
        if self.words.len().saturating_add(whole).saturating_add(1) > MAX_WORDS {
            return Err("wide literal scratch bound exceeded");
        }
        let rest = bits % 32;
        if rest != 0 {
            let mut carry = 0;
            for word in &mut self.words {
                let next = *word >> (32 - rest);
                *word = (*word << rest) | carry;
                carry = next;
            }
            if carry != 0 {
                self.words.push(carry);
            }
        }
        self.words.resize(self.words.len() + whole, 0);
        self.words.rotate_right(whole);
        Ok(())
    }
    pub fn shift_right_one(&mut self) {
        for index in 0..self.words.len() {
            self.words[index] = (self.words[index] >> 1)
                | self
                    .words
                    .get(index + 1)
                    .copied()
                    .unwrap_or(0)
                    .wrapping_shl(31);
        }
        self.trim();
    }
    pub fn compare(&self, other: &Self) -> Ordering {
        self.words
            .len()
            .cmp(&other.words.len())
            .then_with(|| self.words.iter().rev().cmp(other.words.iter().rev()))
    }
    pub fn subtract(&mut self, other: &Self) {
        let mut borrow = 0_u64;
        for (index, word) in self.words.iter_mut().enumerate() {
            let sub = u64::from(other.words.get(index).copied().unwrap_or(0)) + borrow;
            borrow = u64::from(u64::from(*word) < sub);
            let bytes = (u64::from(*word) + (borrow << 32) - sub).to_le_bytes();
            *word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        }
        self.trim();
    }
}
