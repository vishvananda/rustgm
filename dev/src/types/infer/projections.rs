//! Inferred projection inputs remain graph edges, never injective result constructors.
use super::{
    Constructor, Context, Error, ItemId, Kind, Store, TermId, TypeId, Value,
    arguments::{Argument, ArgumentsId},
};
use crate::types::{arguments::Argument as Canonical, projection, traits};

/// A projection's complete inferred input, independent of its unknown result.
pub struct Application {
    pub item: ItemId,
    pub predicate: super::predicates::Predicate,
    pub arguments: Vec<Argument>,
}

/// The combined argument layout retains the trait/family boundary. Self is child
/// zero, followed by the layout's type edges. Identities belong to one context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Projection {
    pub(super) item: ItemId,
    pub(super) arguments: ArgumentsId,
    pub(super) trait_arguments: u32,
}
impl Projection {
    #[must_use]
    pub const fn item(self) -> ItemId {
        self.item
    }
}

impl Constructor {
    pub(super) fn application(self) -> Option<(ArgumentsId, usize)> {
        match self {
            Self::AdtArgs(_, args) => Some((args, 0)),
            Self::FunctionItemArgs(_, args) => Some((args, 1)),
            Self::Projection(value) => Some((value.arguments, 1)),
            _ => None,
        }
    }
    pub(super) fn with_arguments(self, arguments: ArgumentsId) -> Result<Self, Error> {
        match self {
            Self::AdtArgs(item, _) => Ok(Self::AdtArgs(item, arguments)),
            Self::FunctionItemArgs(item, _) => Ok(Self::FunctionItemArgs(item, arguments)),
            Self::Projection(value) => Ok(Self::Projection(Projection { arguments, ..value })),
            _ => Err(Error::InvalidIdentity),
        }
    }
}

impl Context {
    /// Inspect an associated operation's identity without resolving or materializing
    /// its input graph. Other outer shapes and unresolved variables return None.
    /// This metadata query supplies no evidence that the projection can be formed.
    /// # Errors
    /// Rejects malformed identities and exhausted inference work.
    pub fn projection_item(
        &mut self,
        store: &Store,
        term: TermId,
    ) -> Result<Option<ItemId>, Error> {
        self.step()?;
        let term = self.root(term)?;
        Ok(match self.terms[term.0 as usize].value {
            Value::Known(ty) => match store.kind(ty).ok_or(Error::InvalidIdentity)? {
                Kind::Projection(application) => Some(application.item),
                _ => None,
            },
            Value::Construct(Constructor::Projection(value)) => Some(value.item),
            _ => None,
        })
    }

    /// Read Self and mixed trait/family arguments without resolving their holes.
    /// The schema supplies the declaring trait; this is not selection evidence.
    /// # Errors
    /// Rejects a non-projection, malformed argument boundaries and exhausted work.
    pub fn projection_application(
        &mut self,
        store: &Store,
        term: TermId,
    ) -> Result<Application, Error> {
        let term = self.root(term)?;
        let Constructor::Projection(value) = self.materialize(store, term)? else {
            return Err(Error::InvalidIdentity);
        };
        let trait_item = store
            .projection_schema(value.item)
            .ok_or(Error::InvalidIdentity)?
            .trait_item;
        let subject = self.child(store, term, 0)?;
        let mut arguments = self.application_arguments(term, value.arguments, 1)?;
        if value.trait_arguments as usize > arguments.len() {
            return Err(Error::InvalidIdentity);
        }
        let family = arguments.split_off(value.trait_arguments as usize);
        Ok(Application {
            item: value.item,
            predicate: super::predicates::Predicate::trait_application(
                subject, trait_item, arguments,
            ),
            arguments: family,
        })
    }
    /// Preserve all inputs while their type/const variables remain unresolved.
    /// This constructs a type term, not evidence that the associated result exists.
    /// # Errors
    /// Rejects missing schemas, invalid inputs and exhausted inference resources.
    pub fn projected(
        &mut self,
        store: &Store,
        item: ItemId,
        subject: TermId,
        trait_arguments: &[Argument],
        family_arguments: &[Argument],
    ) -> Result<TermId, Error> {
        store
            .projection_schema(item)
            .ok_or(Error::InvalidIdentity)?;
        self.account_work(trait_arguments.len().saturating_add(family_arguments.len()))?;
        let trait_count = u32::try_from(trait_arguments.len()).map_err(|_| Error::Overflow)?;
        let mut arguments = trait_arguments.to_vec();
        arguments.extend_from_slice(family_arguments);
        let mut children = vec![subject];
        let arguments = self.argument_layout(&arguments, &mut children)?;
        self.construct(
            Constructor::Projection(Projection {
                item,
                arguments,
                trait_arguments: trait_count,
            }),
            &children,
        )
    }

    pub(super) fn rebuild_application(
        &mut self,
        kind: Constructor,
        prefix: &[TermId],
        arguments: &[Argument],
    ) -> Result<TermId, Error> {
        let mut children = prefix.to_vec();
        let arguments = self.argument_layout(arguments, &mut children)?;
        self.construct(kind.with_arguments(arguments)?, &children)
    }

    pub(super) fn materialize_projection(
        &mut self,
        store: &Store,
        value: projection::Application,
        children: &mut Vec<TermId>,
    ) -> Result<Constructor, Error> {
        children.push(self.known(store, value.subject)?);
        let trait_ = store
            .trait_application(value.trait_application)
            .ok_or(Error::InvalidIdentity)?;
        let trait_args = store
            .arguments(trait_.arguments)
            .ok_or(Error::InvalidIdentity)?;
        let family_args = store
            .arguments(value.arguments)
            .ok_or(Error::InvalidIdentity)?;
        let trait_arguments = u32::try_from(trait_args.len()).map_err(|_| Error::Overflow)?;
        self.account_work(trait_args.len().saturating_add(family_args.len()))?;
        let mut arguments = Vec::new();
        for argument in trait_args.chain(family_args) {
            arguments.push(match argument {
                Canonical::Lifetime(region) => Argument::Lifetime(region),
                Canonical::Type(ty) => Argument::Type(self.known(store, ty)?),
                Canonical::Const(id) => Argument::Const(self.known_const(store, id)?),
            });
        }
        Ok(Constructor::Projection(Projection {
            item: value.item,
            arguments: self.argument_layout(&arguments, children)?,
            trait_arguments,
        }))
    }

    pub(super) fn canonical_projection(
        &mut self,
        store: &mut Store,
        value: Projection,
        children: &[TypeId],
    ) -> Result<Kind, Error> {
        let (subject, types) = children.split_first().ok_or(Error::InvalidIdentity)?;
        let arguments = self.canonical_arguments(store, value.arguments, types)?;
        let arguments = store.arguments(arguments).ok_or(Error::InvalidIdentity)?;
        self.account_work(arguments.len())?;
        let arguments = arguments.collect::<Vec<_>>();
        let (trait_args, family_args) = arguments
            .split_at_checked(value.trait_arguments as usize)
            .ok_or(Error::InvalidIdentity)?;
        let item = store
            .projection_schema(value.item)
            .ok_or(Error::InvalidIdentity)?
            .trait_item;
        let arguments = store
            .intern_arguments(trait_args)
            .map_err(|_| Error::InvalidIdentity)?;
        let trait_application = store
            .intern_trait_application(traits::Application { item, arguments })
            .map_err(|_| Error::InvalidIdentity)?;
        let arguments = store
            .intern_arguments(family_args)
            .map_err(|_| Error::InvalidIdentity)?;
        Ok(Kind::Projection(projection::Application {
            item: value.item,
            subject: *subject,
            trait_application,
            arguments,
        }))
    }
}
