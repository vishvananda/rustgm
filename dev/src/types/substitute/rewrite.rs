//! One typed rebuild dispatch shared by substitution and immutable canonical lookup.
use super::{Action, Bound, Kind, ListId, RegionId, Store, TypeId, nested};
use crate::types::{GenericArgs, arguments::ConstId, traits::ApplicationId};

pub(super) trait Folder {
    type Error: From<&'static str>;
    fn child(&mut self, ty: TypeId, depth: u16) -> Result<TypeId, Self::Error>;
    fn region(&mut self, region: RegionId, depth: u16) -> Result<RegionId, Self::Error>;
    fn list(&mut self, list: ListId, depth: u16) -> Result<ListId, Self::Error>;
    fn arguments(&mut self, args: GenericArgs, depth: u16) -> Result<GenericArgs, Self::Error>;
    fn application(&mut self, id: ApplicationId, depth: u16) -> Result<ApplicationId, Self::Error>;
    fn constant(&mut self, id: ConstId) -> Result<ConstId, Self::Error>;
    fn bound_regions(&self, count: u32) -> u32;
}

pub(super) fn rebuild<F: Folder>(folder: &mut F, kind: Kind, depth: u16) -> Result<Kind, F::Error> {
    Ok(match kind {
        Kind::Closure {
            identity,
            arguments,
            signature,
            environment,
        } => Kind::Closure {
            identity,
            arguments: folder.arguments(arguments, depth)?,
            signature: folder.child(signature, depth)?,
            environment: folder.child(environment, depth)?,
        },
        Kind::Never | Kind::Foreign(_) | Kind::Primitive(_) | Kind::Parameter(_) => kind,
        Kind::Adt(item, args) => Kind::Adt(item, folder.arguments(args, depth)?),
        Kind::Opaque(id, args) => Kind::Opaque(id, folder.arguments(args, depth)?),
        Kind::Dynamic {
            shape,
            arguments,
            bound_regions,
            region,
        } => Kind::Dynamic {
            shape,
            arguments: folder.arguments(arguments, nested(depth).map_err(F::Error::from)?)?,
            bound_regions: folder.bound_regions(bound_regions),
            region: folder.region(region, depth)?,
        },
        Kind::Projection(application) => {
            let trait_application = folder.application(application.trait_application, depth)?;
            Kind::Projection(crate::types::projection::Application {
                subject: folder.child(application.subject, depth)?,
                trait_application,
                arguments: folder.arguments(application.arguments, depth)?,
                ..application
            })
        }
        Kind::Tuple(list) => Kind::Tuple(folder.list(list, depth)?),
        Kind::Array { element, length } => Kind::Array {
            element: folder.child(element, depth)?,
            length,
        },
        Kind::ArrayTerm { element, length } => Kind::ArrayTerm {
            element: folder.child(element, depth)?,
            length: folder.constant(length)?,
        },
        Kind::Pattern { base, constraint } => Kind::Pattern {
            base: folder.child(base, depth)?,
            constraint,
        },
        Kind::Slice(element) => Kind::Slice(folder.child(element, depth)?),
        Kind::Reference {
            mutable,
            region,
            referent,
        } => Kind::Reference {
            mutable,
            region: folder.region(region, depth)?,
            referent: folder.child(referent, depth)?,
        },
        Kind::Pointer { mutable, pointee } => Kind::Pointer {
            mutable,
            pointee: folder.child(pointee, depth)?,
        },
        Kind::Function {
            bound_regions,
            parameters,
            result,
            unsafe_,
            convention,
        } => {
            let depth = nested(depth).map_err(F::Error::from)?;
            Kind::Function {
                bound_regions: folder.bound_regions(bound_regions),
                parameters: folder.list(parameters, depth)?,
                result: folder.child(result, depth)?,
                unsafe_,
                convention,
            }
        }
        Kind::FunctionItem {
            item,
            signature,
            arguments,
        } => Kind::FunctionItem {
            item,
            arguments: folder.arguments(arguments, depth)?,
            signature: folder.child(signature, depth)?,
        },
    })
}

pub(super) struct Mutable<'a> {
    pub bound: &'a mut Bound,
    pub store: &'a mut Store,
}
impl Folder for Mutable<'_> {
    type Error = &'static str;
    fn child(&mut self, ty: TypeId, depth: u16) -> Result<TypeId, Self::Error> {
        self.bound.child(self.store, ty, depth)
    }
    fn region(&mut self, region: RegionId, depth: u16) -> Result<RegionId, Self::Error> {
        self.bound.region(region, depth)
    }
    fn list(&mut self, list: ListId, depth: u16) -> Result<ListId, Self::Error> {
        self.bound.list(self.store, list, depth)
    }
    fn arguments(&mut self, args: GenericArgs, depth: u16) -> Result<GenericArgs, Self::Error> {
        self.bound.arguments(self.store, args, depth)
    }
    fn application(&mut self, id: ApplicationId, depth: u16) -> Result<ApplicationId, Self::Error> {
        let original = self
            .store
            .trait_application(id)
            .ok_or("invalid substituted projection trait")?;
        let arguments = self
            .bound
            .arguments(self.store, original.arguments, depth)?;
        self.store
            .intern_trait_application(crate::types::traits::Application {
                arguments,
                ..original
            })
    }
    fn constant(&mut self, id: ConstId) -> Result<ConstId, Self::Error> {
        self.bound.constant(self.store, id)
    }
    fn bound_regions(&self, count: u32) -> u32 {
        if matches!(self.bound.action, Action::Shape { .. }) {
            0
        } else {
            count
        }
    }
}
