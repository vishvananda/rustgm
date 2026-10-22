//! Typed const inference variables are distinct from rigid canonical parameters.
use super::{Context, Error, Primitive, Store, TypeId};
use crate::types::arguments::{ConstId, ConstKind, Constant};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConstTermId(pub u32);
struct Term {
    parent: ConstTermId,
    rank: u8,
    ty: TypeId,
    value: Option<ConstKind>,
}
#[derive(Default)]
pub(super) struct Constants {
    terms: Vec<Term>,
    known: HashMap<ConstKind, ConstTermId>,
}
impl Constants {
    pub(super) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.vector(&self.terms);
        total.map(&self.known);
        total
    }

    fn push(&mut self, ty: TypeId, value: Option<ConstKind>) -> Result<ConstTermId, Error> {
        let id = ConstTermId(u32::try_from(self.terms.len()).map_err(|_| Error::Overflow)?);
        self.terms.push(Term {
            parent: id,
            rank: 0,
            ty,
            value,
        });
        Ok(id)
    }
    pub(super) fn ty(&self, id: ConstTermId) -> Result<TypeId, Error> {
        self.terms
            .get(id.0 as usize)
            .map(|t| t.ty)
            .ok_or(Error::InvalidIdentity)
    }
    pub(super) fn value(&mut self, id: ConstTermId) -> Result<Option<ConstKind>, Error> {
        let id = self.root(id)?;
        Ok(self.terms[id.0 as usize].value)
    }
    pub(super) fn root(&mut self, mut id: ConstTermId) -> Result<ConstTermId, Error> {
        let mut root = id;
        loop {
            let parent = self
                .terms
                .get(root.0 as usize)
                .ok_or(Error::InvalidIdentity)?
                .parent;
            if parent == root {
                break;
            }
            root = parent;
        }
        while id != root {
            let term = &mut self.terms[id.0 as usize];
            let parent = term.parent;
            term.parent = root;
            id = parent;
        }
        Ok(root)
    }
    pub(super) fn equate(
        &mut self,
        a: ConstTermId,
        b: ConstTermId,
    ) -> Result<Option<(ConstTermId, ConstTermId)>, Error> {
        let mut a = self.root(a)?;
        let mut b = self.root(b)?;
        if a == b {
            return Ok(None);
        }
        let left = &self.terms[a.0 as usize];
        let right = &self.terms[b.0 as usize];
        if left.ty != right.ty
            || (left.value.is_some() && right.value.is_some() && left.value != right.value)
        {
            return Err(Error::Mismatch);
        }
        let value = left.value.or(right.value);
        if left.rank < right.rank {
            std::mem::swap(&mut a, &mut b);
        }
        if self.terms[a.0 as usize].rank == self.terms[b.0 as usize].rank {
            self.terms[a.0 as usize].rank = self.terms[a.0 as usize]
                .rank
                .checked_add(1)
                .ok_or(Error::Overflow)?;
        }
        self.terms[b.0 as usize].parent = a;
        self.terms[a.0 as usize].value = value;
        Ok(Some((a, b)))
    }
}
impl Context {
    pub(super) fn constants(&mut self) -> &mut Constants {
        &mut self
            .arguments
            .get_or_insert_with(Default::default)
            .constants
    }
    /// # Errors
    /// Rejects non-scalar const parameter types and exhausted inference resources.
    pub fn const_variable(&mut self, store: &Store, ty: TypeId) -> Result<ConstTermId, Error> {
        if !matches!(store.kind(ty), Some(crate::types::Kind::Primitive(p))
            if p.integer().is_some() || matches!(p, Primitive::Bool | Primitive::Char))
        {
            return Err(Error::InvalidIdentity);
        }
        self.step()?;
        self.constants().push(ty, None)
    }
    /// # Errors
    /// Rejects invalid canonical constants and exhausted inference resources.
    pub fn known_const(&mut self, store: &Store, id: ConstId) -> Result<ConstTermId, Error> {
        let value = store.const_kind(id).ok_or(Error::InvalidIdentity)?;
        let ty = store.const_type(id).ok_or(Error::InvalidIdentity)?;
        self.const_value(ty, value)
    }
    pub(super) fn const_value(
        &mut self,
        ty: TypeId,
        value: ConstKind,
    ) -> Result<ConstTermId, Error> {
        self.step()?;
        let constants = self.constants();
        if let Some(id) = constants.known.get(&value) {
            return Ok(*id);
        }
        let id = constants.push(ty, Some(value))?;
        constants.known.insert(value, id);
        Ok(id)
    }
    pub(super) fn const_length(&mut self, length: u64) -> Result<ConstTermId, Error> {
        let ty = TypeId::primitive(Primitive::Usize);
        self.const_value(
            ty,
            ConstKind::Value(Constant {
                ty,
                bits: u128::from(length),
            }),
        )
    }
    /// Obtain only a checked array head's usize length, independently of its element.
    /// # Errors
    /// Rejects nonarray heads, invalid const identities and exhausted work.
    pub fn array_length(
        &mut self,
        store: &Store,
        head: super::Constructor,
    ) -> Result<ConstTermId, Error> {
        let id = match head {
            super::Constructor::Array(length) => self.const_length(length)?,
            super::Constructor::ArrayTerm(length) => self.known_const(store, length)?,
            super::Constructor::ArrayInfer(length) => length,
            _ => return Err(Error::InvalidIdentity),
        };
        if self.constants().ty(id)? != TypeId::primitive(Primitive::Usize) {
            return Err(Error::InvalidIdentity);
        }
        Ok(id)
    }
    /// No integer fallback applies to const arguments, even for an integer type.
    /// # Errors
    /// Rejects unresolved/invalid variables and exhausted canonical storage.
    pub fn resolved_const(&mut self, store: &mut Store, id: ConstTermId) -> Result<ConstId, Error> {
        self.step()?;
        let constants = self.constants();
        let root = constants.root(id)?;
        let value = constants.terms[root.0 as usize]
            .value
            .ok_or(Error::Unresolved)?;
        store
            .intern_const(value)
            .map_err(|_| Error::InvalidIdentity)
    }
    /// # Errors
    /// Rejects incompatible types/values, invalid variables and exhausted work.
    pub fn equate_consts(&mut self, a: ConstTermId, b: ConstTermId) -> Result<(), Error> {
        self.step()?;
        if let Some((a, b)) = self.constants().equate(a, b)? {
            self.wake_resolution(super::dependencies::Dependency::Const(a))?;
            self.wake_resolution(super::dependencies::Dependency::Const(b))?;
        }
        Ok(())
    }
}
