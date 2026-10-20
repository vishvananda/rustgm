//! Complete structural equality is shared by ordinary and quantified input checks.
mod shapes;
use crate::types::{
    GenericArgs, Kind, ListId, RegionId, Store, TypeId, arguments::Argument, region_graph::Error,
};
pub(crate) use shapes::Shapes;

pub(crate) trait Compare<'t> {
    fn types(&self) -> &'t Store;
    fn charge(&mut self, count: usize) -> Result<(), Error>;
    fn child(&mut self, a: TypeId, b: TypeId, depth: u16) -> Result<(), Error>;
    fn regions(&mut self, a: RegionId, b: RegionId, depth: u16) -> Result<bool, Error>;
    fn arguments(&mut self, a: GenericArgs, b: GenericArgs, depth: u16) -> Result<bool, Error> {
        self.compare_arguments(a, b, depth)
    }
    fn compare_arguments(
        &mut self,
        a: GenericArgs,
        b: GenericArgs,
        depth: u16,
    ) -> Result<bool, Error> {
        let a = self.types().arguments(a).ok_or(Error::Identity)?;
        let b = self.types().arguments(b).ok_or(Error::Identity)?;
        if a.len() != b.len() {
            return Ok(false);
        }
        for (a, b) in a.zip(b) {
            self.charge(1)?;
            match (a, b) {
                (Argument::Type(a), Argument::Type(b)) => self.child(a, b, depth)?,
                (Argument::Lifetime(a), Argument::Lifetime(b)) => {
                    if !self.regions(a, b, depth)? {
                        return Ok(false);
                    }
                }
                (Argument::Const(a), Argument::Const(b)) if a == b => {}
                _ => return Ok(false),
            }
        }
        Ok(true)
    }
    fn list(&mut self, a: ListId, b: ListId, depth: u16) -> Result<bool, Error> {
        self.compare_list(a, b, depth)
    }
    fn compare_list(&mut self, a: ListId, b: ListId, depth: u16) -> Result<bool, Error> {
        let a = self.types().list(a).ok_or(Error::Identity)?;
        let b = self.types().list(b).ok_or(Error::Identity)?;
        if a.len() != b.len() {
            return Ok(false);
        }
        for (&a, &b) in a.iter().zip(b) {
            self.child(a, b, depth)?;
        }
        Ok(true)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive structural dispatch keeps conditional identity distinct from coercion"
    )]
    fn compare(&mut self, a: TypeId, b: TypeId, depth: u16) -> Result<bool, Error> {
        let a = self.types().kind(a).ok_or(Error::Identity)?;
        let b = self.types().kind(b).ok_or(Error::Identity)?;
        match (a, b) {
            (Kind::Never, Kind::Never) => {}
            (Kind::Primitive(a), Kind::Primitive(b)) if a == b => {}
            (Kind::Parameter(a), Kind::Parameter(b)) if a == b => {}
            (
                Kind::Reference {
                    region: a,
                    mutable: x,
                    referent: left,
                },
                Kind::Reference {
                    region: b,
                    mutable: y,
                    referent: right,
                },
            ) if x == y => {
                if !self.regions(a, b, depth)? {
                    return Ok(false);
                }
                self.child(left, right, depth)?;
            }
            (
                Kind::Pointer {
                    mutable: x,
                    pointee: a,
                },
                Kind::Pointer {
                    mutable: y,
                    pointee: b,
                },
            ) if x == y => self.child(a, b, depth)?,
            (
                Kind::Array {
                    length: x,
                    element: a,
                },
                Kind::Array {
                    length: y,
                    element: b,
                },
            ) if x == y => self.child(a, b, depth)?,
            (
                Kind::ArrayTerm {
                    length: x,
                    element: a,
                },
                Kind::ArrayTerm {
                    length: y,
                    element: b,
                },
            ) if x == y => self.child(a, b, depth)?,
            (Kind::Slice(a), Kind::Slice(b)) => self.child(a, b, depth)?,
            (Kind::Tuple(a), Kind::Tuple(b)) => return self.list(a, b, depth),
            (Kind::Adt(x, a), Kind::Adt(y, b)) if x == y => return self.arguments(a, b, depth),
            (Kind::Opaque(x, a), Kind::Opaque(y, b)) if x == y => {
                return self.arguments(a, b, depth);
            }
            (
                Kind::Function {
                    bound_regions: x,
                    parameters: a,
                    result: ar,
                    unsafe_: au,
                },
                Kind::Function {
                    bound_regions: y,
                    parameters: b,
                    result: br,
                    unsafe_: bu,
                },
            ) if x == y && au == bu => {
                let depth = depth.checked_add(1).ok_or(Error::Budget)?;
                self.child(ar, br, depth)?;
                return self.list(a, b, depth);
            }
            (
                Kind::FunctionItem {
                    item: x,
                    arguments: a,
                    signature: ar,
                },
                Kind::FunctionItem {
                    item: y,
                    arguments: b,
                    signature: br,
                },
            ) if x == y => {
                self.child(ar, br, depth)?;
                return self.arguments(a, b, depth);
            }
            (Kind::Projection(a), Kind::Projection(b)) if a.item == b.item => {
                let at = self
                    .types()
                    .trait_application(a.trait_application)
                    .ok_or(Error::Identity)?;
                let bt = self
                    .types()
                    .trait_application(b.trait_application)
                    .ok_or(Error::Identity)?;
                if at.item != bt.item {
                    return Ok(false);
                }
                self.child(a.subject, b.subject, depth)?;
                return Ok(self.arguments(at.arguments, bt.arguments, depth)?
                    && self.arguments(a.arguments, b.arguments, depth)?);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
