//! Inferred argument layouts keep type edges in the ordinary occurs-check graph.
use super::constants::{ConstTermId, Constants};
use super::{Constructor, Context, Error, ItemId, RegionId, Store, TermId, TypeId, Value};
use crate::{
    support::intern_slice::Slices,
    types::{GenericArgs, Kind, arguments::Argument as Canonical},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Argument {
    Lifetime(RegionId),
    Type(TermId),
    Const(ConstTermId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ArgumentsId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Slot {
    Lifetime(RegionId),
    Type,
    Const(ConstTermId),
}
#[derive(Default)]
pub(super) struct Storage {
    pub(super) constants: Constants,
    pub(super) objects: Option<Box<super::objects::Storage>>,
    slots: Slices<Slot>,
}
impl Context {
    pub(super) fn argument_regions(
        &mut self,
        id: ArgumentsId,
        mut replace: impl FnMut(RegionId) -> Result<RegionId, Error>,
    ) -> Result<ArgumentsId, Error> {
        self.account_work(self.slots(id)?.len())?;
        let slots = self
            .slots(id)?
            .iter()
            .map(|slot| match slot {
                Slot::Lifetime(region) => replace(*region).map(Slot::Lifetime),
                other => Ok(*other),
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.arguments
            .get_or_insert_with(Default::default)
            .slots
            .intern(&slots)
            .map(ArgumentsId)
            .map_err(|_| Error::Overflow)
    }
    pub(super) fn slots(&self, id: ArgumentsId) -> Result<&[Slot], Error> {
        self.arguments
            .as_ref()
            .and_then(|s| s.slots.get(id.0))
            .ok_or(Error::InvalidIdentity)
    }
    pub(super) fn argument_arity(&self, id: ArgumentsId) -> Result<usize, Error> {
        Ok(self
            .slots(id)?
            .iter()
            .filter(|s| matches!(s, Slot::Type))
            .count())
    }
    pub(super) fn argument_layout(
        &mut self,
        arguments: &[Argument],
        children: &mut Vec<TermId>,
    ) -> Result<ArgumentsId, Error> {
        self.account_work(arguments.len())?;
        let mut slots = Vec::with_capacity(arguments.len());
        for argument in arguments {
            slots.push(match *argument {
                Argument::Lifetime(r) => Slot::Lifetime(r),
                Argument::Type(term) => {
                    self.root(term)?;
                    children.push(term);
                    Slot::Type
                }
                Argument::Const(id) => {
                    self.arguments
                        .as_ref()
                        .ok_or(Error::InvalidIdentity)?
                        .constants
                        .ty(id)?;
                    Slot::Const(id)
                }
            });
        }
        self.arguments
            .get_or_insert_with(Default::default)
            .slots
            .intern(&slots)
            .map(ArgumentsId)
            .map_err(|_| Error::Overflow)
    }
    /// Retain all arguments, including parameters erased from the signature.
    /// Signature is child zero; type arguments are additional graph edges.
    /// # Errors
    /// Rejects invalid arguments/terms and exhausted inference resources.
    pub fn function_item(
        &mut self,
        item: ItemId,
        signature: TermId,
        arguments: &[Argument],
    ) -> Result<TermId, Error> {
        let mut children = vec![signature];
        let args = self.argument_layout(arguments, &mut children)?;
        self.construct(Constructor::FunctionItemArgs(item, args), &children)
    }
    /// Keep inferred capture types in the same occurs-check graph as the signature.
    /// # Errors
    /// Rejects invalid arguments/terms and exhausted inference resources.
    pub fn closure(
        &mut self,
        identity: crate::types::closure::ClosureId,
        signature: TermId,
        environment: TermId,
        arguments: &[Argument],
    ) -> Result<TermId, Error> {
        let mut children = vec![signature, environment];
        let args = self.argument_layout(arguments, &mut children)?;
        self.construct(Constructor::Closure(identity, args), &children)
    }
    /// # Errors
    /// Rejects invalid arguments/terms and exhausted inference resources.
    pub fn nominal(&mut self, item: ItemId, arguments: &[Argument]) -> Result<TermId, Error> {
        let mut children = Vec::new();
        let args = self.argument_layout(arguments, &mut children)?;
        self.construct(Constructor::AdtArgs(item, args), &children)
    }
    /// Read mixed nominal/callable arguments without resolving inference holes.
    /// # Errors
    /// Rejects a non-item term, malformed argument layout or exhausted resources.
    pub fn item_arguments(&mut self, store: &Store, term: TermId) -> Result<Vec<Argument>, Error> {
        let term = self.root(term)?;
        let (args, offset) = match self.materialize(store, term)? {
            Constructor::AdtArgs(_, args) | Constructor::Opaque(_, args) => (args, 0),
            Constructor::FunctionItemArgs(_, args) => (args, 1),
            Constructor::Closure(_, args) => (args, 2),
            _ => return Err(Error::InvalidIdentity),
        };
        self.application_arguments(term, args, offset)
    }
    pub(super) fn application_arguments(
        &mut self,
        term: TermId,
        args: ArgumentsId,
        offset: usize,
    ) -> Result<Vec<Argument>, Error> {
        let count = self.slots(args)?.len();
        self.account_work(count)?;
        let entry = self.terms[term.0 as usize];
        let mut types = self.edges[entry.start as usize + offset..entry.end as usize].iter();
        let arguments = self
            .slots(args)?
            .iter()
            .map(|slot| {
                Ok(match *slot {
                    Slot::Lifetime(region) => Argument::Lifetime(region),
                    Slot::Type => Argument::Type(*types.next().ok_or(Error::InvalidIdentity)?),
                    Slot::Const(id) => Argument::Const(id),
                })
            })
            .collect::<Result<_, _>>()?;
        if types.next().is_some() {
            return Err(Error::InvalidIdentity);
        }
        Ok(arguments)
    }
    pub(super) fn materialize_arguments(
        &mut self,
        store: &Store,
        args: GenericArgs,
        children: &mut Vec<TermId>,
    ) -> Result<ArgumentsId, Error> {
        let args = store.arguments(args).ok_or(Error::InvalidIdentity)?;
        self.account_work(args.len())?;
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(match arg {
                Canonical::Lifetime(r) => Argument::Lifetime(r),
                Canonical::Type(ty) => Argument::Type(self.known(store, ty)?),
                Canonical::Const(id) => Argument::Const(self.known_const(store, id)?),
            });
        }
        self.argument_layout(&values, children)
    }
    pub(super) fn equate_arguments(
        &mut self,
        a: ArgumentsId,
        b: ArgumentsId,
        regions: bool,
    ) -> Result<(), Error> {
        let count = self.slots(a)?.len();
        if count != self.slots(b)?.len() {
            return Err(Error::Mismatch);
        }
        for index in 0..count {
            self.step()?;
            match (self.slots(a)?[index], self.slots(b)?[index]) {
                (Slot::Lifetime(a), Slot::Lifetime(b)) => {
                    if regions && a != b {
                        self.region_equalities.push((a, b));
                    }
                }
                (Slot::Type, Slot::Type) => {}
                (Slot::Const(a), Slot::Const(b)) => self.equate_consts(a, b)?,
                _ => return Err(Error::Mismatch),
            }
        }
        Ok(())
    }
    pub(super) fn canonical_arguments(
        &mut self,
        store: &mut Store,
        id: ArgumentsId,
        children: &[TypeId],
    ) -> Result<GenericArgs, Error> {
        let count = self.slots(id)?.len();
        self.account_work(count)?;
        let mut values = Vec::with_capacity(count);
        let mut types = children.iter();
        for index in 0..count {
            values.push(match self.slots(id)?[index] {
                Slot::Lifetime(r) => Canonical::Lifetime(r),
                Slot::Type => Canonical::Type(*types.next().ok_or(Error::InvalidIdentity)?),
                Slot::Const(id) => Canonical::Const(self.resolved_const(store, id)?),
            });
        }
        if types.next().is_some() {
            return Err(Error::InvalidIdentity);
        }
        store
            .intern_arguments(&values)
            .map_err(|_| Error::InvalidIdentity)
    }
    /// The canonical identity of a determined subtree whose every node is
    /// already interned. This never interns, never steps and inspects at most
    /// a small bounded subtree; `None` covers every other case.
    /// # Errors
    /// Rejects invalid inference identities.
    pub fn find_identity(&mut self, store: &Store, term: TermId) -> Result<Option<TypeId>, Error> {
        const LIMIT: usize = 64;
        let mut work = vec![(self.root(term)?, false)];
        let mut results: Vec<(TermId, TypeId)> = Vec::new();
        let mut children = Vec::new();
        let mut visited = 0_usize;
        while let Some((id, finish)) = work.pop() {
            let id = self.root(id)?;
            if results.iter().any(|(done, _)| *done == id) {
                continue;
            }
            visited += 1;
            if visited > LIMIT {
                return Ok(None);
            }
            let term = self.terms[id.0 as usize];
            let kind = match term.value {
                Value::Variable(_) => return Ok(None),
                Value::Known(ty) => {
                    results.push((id, ty));
                    continue;
                }
                Value::Construct(kind) => kind,
            };
            if let Some(ty) = self.settled_type(id) {
                results.push((id, ty));
                continue;
            }
            if !finish {
                work.push((id, true));
                for child in self.edges[term.start as usize..term.end as usize]
                    .iter()
                    .rev()
                {
                    work.push((*child, false));
                }
                continue;
            }
            children.clear();
            for offset in term.start..term.end {
                let child = self.root(self.edges[offset as usize])?;
                let Some(&(_, ty)) = results.iter().find(|(done, _)| *done == child) else {
                    return Ok(None);
                };
                children.push(ty);
            }
            let Some(ty) = self.find_canonical(store, kind, &children)? else {
                return Ok(None);
            };
            results.push((id, ty));
        }
        let root = self.root(term)?;
        Ok(results
            .iter()
            .find(|(done, _)| *done == root)
            .map(|(_, ty)| *ty))
    }
    /// Lookup-only counterpart of [`Self::canonical`] for closed constructors.
    fn find_canonical(
        &self,
        store: &Store,
        kind: Constructor,
        children: &[TypeId],
    ) -> Result<Option<TypeId>, Error> {
        let first = || children.first().copied().ok_or(Error::InvalidIdentity);
        let kind = match kind {
            Constructor::Primitive(p) => Kind::Primitive(p),
            Constructor::Parameter(id) => Kind::Parameter(id),
            Constructor::Adt(item, args) => Kind::Adt(item, args),
            Constructor::Never => Kind::Never,
            Constructor::Tuple => {
                let Some(list) = store.find_list(children) else {
                    return Ok(None);
                };
                Kind::Tuple(list)
            }
            Constructor::Array(length) => Kind::Array {
                element: first()?,
                length,
            },
            Constructor::ArrayTerm(length) => Kind::ArrayTerm {
                element: first()?,
                length,
            },
            Constructor::Slice => Kind::Slice(first()?),
            Constructor::Reference { mutable, region } => Kind::Reference {
                mutable,
                region,
                referent: first()?,
            },
            Constructor::Pointer { mutable } => Kind::Pointer {
                mutable,
                pointee: first()?,
            },
            Constructor::Function {
                bound_regions,
                parameters,
                unsafe_,
                convention,
            } => {
                let parameters = parameters as usize;
                let Some(list) =
                    store.find_list(children.get(..parameters).ok_or(Error::InvalidIdentity)?)
                else {
                    return Ok(None);
                };
                Kind::Function {
                    bound_regions,
                    parameters: list,
                    result: *children.get(parameters).ok_or(Error::InvalidIdentity)?,
                    unsafe_,
                    convention,
                }
            }
            Constructor::FunctionItem(item, arguments) => Kind::FunctionItem {
                item,
                arguments,
                signature: first()?,
            },
            Constructor::AdtArgs(item, id) => {
                let mut values = Vec::with_capacity(self.slots(id)?.len());
                let mut types = children.iter();
                for slot in self.slots(id)? {
                    values.push(match *slot {
                        Slot::Lifetime(r) => Canonical::Lifetime(r),
                        Slot::Type => Canonical::Type(*types.next().ok_or(Error::InvalidIdentity)?),
                        Slot::Const(_) => return Ok(None),
                    });
                }
                if types.next().is_some() {
                    return Err(Error::InvalidIdentity);
                }
                let arguments = if values.iter().all(|v| matches!(v, Canonical::Lifetime(_))) {
                    let regions: Vec<_> = values
                        .iter()
                        .filter_map(|v| match v {
                            Canonical::Lifetime(region) => Some(*region),
                            _ => None,
                        })
                        .collect();
                    store.find_regions(&regions)
                } else {
                    store.find_mixed_arguments(&values)
                };
                let Some(arguments) = arguments else {
                    return Ok(None);
                };
                Kind::Adt(item, arguments)
            }
            _ => return Ok(None),
        };
        Ok(store.find_kind(kind))
    }
    pub(super) fn canonical(
        &mut self,
        store: &mut Store,
        kind: Constructor,
        children: &[TypeId],
    ) -> Result<TypeId, Error> {
        let kind = match kind {
            Constructor::Projection(value) => self.canonical_projection(store, value, children)?,
            Constructor::ArrayInfer(id) => Kind::ArrayTerm {
                element: *children.first().ok_or(Error::InvalidIdentity)?,
                length: self.resolved_const(store, id)?,
            },
            Constructor::Opaque(id, args) => {
                Kind::Opaque(id, self.canonical_arguments(store, args, children)?)
            }
            Constructor::Dynamic {
                shape,
                arguments,
                lifetimes,
            } => {
                let lifetimes = self.object_lifetimes(lifetimes)?;
                Kind::Dynamic {
                    shape,
                    arguments: self.canonical_arguments(store, arguments, children)?,
                    bound_regions: lifetimes.bound_regions,
                    region: lifetimes.region,
                }
            }
            Constructor::AdtArgs(item, args) => {
                Kind::Adt(item, self.canonical_arguments(store, args, children)?)
            }
            Constructor::FunctionItemArgs(item, args) => {
                let (signature, arguments) =
                    children.split_first().ok_or(Error::InvalidIdentity)?;
                Kind::FunctionItem {
                    item,
                    signature: *signature,
                    arguments: self.canonical_arguments(store, args, arguments)?,
                }
            }
            Constructor::Closure(identity, args) => {
                if store.closure_declaration(identity)
                    == Some(crate::types::closure::Declaration::Pending)
                {
                    return Err(Error::Unresolved);
                }
                let [signature, environment, arguments @ ..] = children else {
                    return Err(Error::InvalidIdentity);
                };
                Kind::Closure {
                    identity,
                    signature: *signature,
                    environment: *environment,
                    arguments: self.canonical_arguments(store, args, arguments)?,
                }
            }
            _ => return super::canonical(store, kind, children),
        };
        store.intern(kind).map_err(|_| Error::InvalidIdentity)
    }
}

impl Storage {
    pub(super) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = self.slots.allocation();
        total.include(self.constants.allocation());
        if let Some(objects) = &self.objects {
            total.boxed(objects.as_ref());
            total.include(objects.allocation());
        }
        total
    }
}
