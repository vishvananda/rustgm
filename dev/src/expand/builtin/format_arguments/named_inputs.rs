//! Names are a transient index; capture order is retained independently.
use super::Charge;
use crate::{
    lex::token::LexError,
    support::{hash::HashMap, retained::Growth},
    syntax::token_tree::error,
};
pub(super) struct Inputs<'text, 'pay> {
    names: HashMap<&'text str, usize>,
    pub(super) captured: Vec<&'text str>,
    explicit: usize,
    allow_capture: bool,
    charge: &'pay Charge<'pay>,
}
impl<'text, 'pay> Inputs<'text, 'pay> {
    pub(super) fn new(
        explicit: usize,
        names: &[Option<&'text str>],
        allow_capture: bool,
        charge: &'pay Charge<'pay>,
    ) -> Result<Self, LexError> {
        charge(1, size_of::<Self>())?;
        let mut result = Self {
            names: HashMap::default(),
            captured: Vec::new(),
            explicit,
            allow_capture,
            charge,
        };
        let mut seen_named = false;
        for (ordinal, name) in names.iter().enumerate() {
            charge(1, 0)?;
            if let Some(name) = name {
                seen_named = true;
                if result.insert(name, ordinal)?.is_some() {
                    return Err(error(0, "duplicate named formatting argument"));
                }
            } else if seen_named {
                return Err(error(
                    0,
                    "positional formatting argument follows a named argument",
                ));
            }
        }
        Ok(result)
    }
    fn insert(&mut self, name: &'text str, index: usize) -> Result<Option<usize>, LexError> {
        let growth = Growth::map::<&str, usize>(self.names.len(), self.names.capacity(), 1);
        (self.charge)(
            name.len().saturating_add(growth.work).saturating_add(1),
            growth.bytes,
        )?;
        self.names
            .try_reserve(1)
            .map_err(|_| error(0, "formatting name index allocation failed"))?;
        Ok(self.names.insert(name, index))
    }
    pub(super) fn resolve(&mut self, name: &'text str) -> Result<usize, LexError> {
        (self.charge)(name.len().saturating_add(1), 0)?;
        if let Some(&index) = self.names.get(name) {
            return Ok(index);
        }
        if !self.allow_capture {
            return Err(error(
                0,
                "implicit formatting capture requires a direct literal",
            ));
        }
        let growth = Growth::vector::<&str>(self.captured.len(), self.captured.capacity(), 1);
        (self.charge)(growth.work.saturating_add(1), growth.bytes)?;
        self.captured
            .try_reserve(1)
            .map_err(|_| error(0, "formatting capture allocation failed"))?;
        let index = self.explicit.saturating_add(self.captured.len());
        self.insert(name, index)?;
        self.captured.push(name);
        Ok(index)
    }
    pub(super) fn positional(&self, index: usize) -> Result<usize, LexError> {
        if index < self.explicit {
            Ok(index)
        } else {
            Err(error(0, "formatting argument index is out of range"))
        }
    }
}
