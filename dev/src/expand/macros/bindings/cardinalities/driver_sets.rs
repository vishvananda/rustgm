//! Typed template driver sets are shared across nested and repeated queries.
use super::super::super::model::{Definition, Expression, Kind, List};
use super::{HashMap, LexError, NameId, VariableId, charge, error};
#[derive(Default)]
pub(super) struct DriverSets(HashMap<(u32, u32), Bits>);
pub(super) enum Bits {
    Inline(u64),
    Wide(Box<[u64]>),
}
impl Bits {
    fn new(variables: usize, work: &mut usize) -> Result<Self, LexError> {
        let words = variables.div_ceil(64);
        charge(work, words.max(1))?;
        Ok(if words <= 1 {
            Self::Inline(0)
        } else {
            Self::Wide(vec![0; words].into_boxed_slice())
        })
    }
    fn words(&self) -> &[u64] {
        match self {
            Self::Inline(value) => std::slice::from_ref(value),
            Self::Wide(values) => values,
        }
    }
    fn words_mut(&mut self) -> &mut [u64] {
        match self {
            Self::Inline(value) => std::slice::from_mut(value),
            Self::Wide(values) => values,
        }
    }
    fn insert(&mut self, variable: VariableId) -> Result<(), LexError> {
        let index = variable.0 as usize;
        let word = self
            .words_mut()
            .get_mut(index / 64)
            .ok_or_else(|| error(0, "template driver exceeds its actual variable table"))?;
        *word |= 1_u64 << (index % 64);
        Ok(())
    }
    fn include(&mut self, other: &Self, work: &mut usize) -> Result<(), LexError> {
        charge(work, other.words().len())?;
        if self.words().len() != other.words().len() {
            return Err(error(0, "template driver variable table changed"));
        }
        for (word, &value) in self.words_mut().iter_mut().zip(other.words()) {
            *word |= value;
        }
        Ok(())
    }
    pub(super) fn variables(&self) -> impl Iterator<Item = Result<VariableId, LexError>> + '_ {
        self.words().iter().enumerate().flat_map(|(word, &value)| {
            let mut bits = value;
            std::iter::from_fn(move || {
                if bits == 0 {
                    return None;
                }
                let bit = bits.trailing_zeros();
                bits &= bits - 1;
                Some(
                    word.checked_mul(64)
                        .and_then(|base| base.checked_add(bit as usize))
                        .and_then(|id| u32::try_from(id).ok())
                        .map(VariableId)
                        .ok_or_else(|| error(0, "template driver identity overflow")),
                )
            })
        })
    }
}
impl DriverSets {
    pub(super) fn get(
        &mut self,
        definition: &Definition,
        variables: &HashMap<NameId, VariableId>,
        list: List,
        work: &mut usize,
    ) -> Result<&Bits, LexError> {
        charge(work, 1)?;
        let key = (list.start, list.end);
        if !self.0.contains_key(&key) {
            let mut bits = Bits::new(variables.len(), work)?;
            for id in definition.list(list) {
                charge(work, 1)?;
                match definition.node(*id).kind {
                    Kind::Substitute { name, .. }
                    | Kind::Expression(Expression::Ignore(name) | Expression::Count { name, .. }) => {
                        if let Some(&variable) = variables.get(&name) {
                            bits.insert(variable)?;
                        }
                    }
                    Kind::Group { children, .. }
                    | Kind::Repeat { children, .. }
                    | Kind::Concat(children) => {
                        bits.include(self.get(definition, variables, children, work)?, work)?;
                    }
                    Kind::Atom(_)
                    | Kind::Capture(_)
                    | Kind::Crate
                    | Kind::Doc { .. }
                    | Kind::EscapedDollar(_)
                    | Kind::Expression(Expression::Index(_) | Expression::Length(_)) => {}
                }
            }
            self.0.insert(key, bits);
        }
        self.0
            .get(&key)
            .ok_or_else(|| error(0, "template driver set disappeared"))
    }
}
