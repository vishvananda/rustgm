//! Structural encoding of a closed type in a frozen source graph.
//! Each node writes a tag and counts, and declarations use their resolved owner.
//! Free lifetimes are erased. Lifetimes within a `for<..>` binder are numbered
//! in their order of first use.
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    support::hash::HashMap,
    types::{
        GenericArgs, ItemId, Kind, RegionId, TypeId,
        arguments::{Argument, ConstKind},
        callable::Abi,
        closure::ClosureId,
    },
};
use std::sync::{LazyLock, Mutex, PoisonError};

/// A spelling longer than this is refused instead of being walked further.
const LIMIT: usize = 1 << 16;

pub(super) struct Spelling<'a> {
    facts: &'a Declarations,
    bytes: Vec<u8>,
    /// The open binders, outermost first: each one's lifetimes in first-use order.
    binders: Vec<Vec<u16>>,
}
impl<'a> Spelling<'a> {
    /// Spell `ty`, which must be closed: no parameter, no unresolved alias.
    pub(super) fn of(facts: &'a Declarations, ty: TypeId) -> Result<Vec<u8>, &'static str> {
        let mut spelling = Self {
            facts,
            bytes: Vec::new(),
            binders: Vec::new(),
        };
        spelling.ty(ty)?;
        Ok(spelling.bytes)
    }
    fn tag(&mut self, tag: u8) -> Result<(), &'static str> {
        if self.bytes.len() >= LIMIT {
            return Err("type identity spelling exceeds its size budget");
        }
        self.bytes.push(tag);
        Ok(())
    }
    fn count(&mut self, count: usize) -> Result<(), &'static str> {
        let count = u32::try_from(count).map_err(|_| "type identity spelling count overflow")?;
        self.word(count);
        Ok(())
    }
    fn word(&mut self, word: u32) {
        self.bytes.extend_from_slice(&word.to_le_bytes());
    }
    fn text(&mut self, text: &str) {
        self.bytes.extend_from_slice(text.as_bytes());
        self.bytes.push(0);
    }
    /// The crate that owns every local declaration of this compilation.
    fn owner(&mut self) {
        let identity = self
            .facts
            .crates
            .as_deref()
            .and_then(crate::sema::crates::Linkage::identity);
        self.text(identity.map_or("", |identity| identity.name()));
        self.text(identity.map_or("", |identity| identity.disambiguator()));
    }
    /// A declaration of this crate. One that another crate's artifact supplied
    /// has a different number in its own crate, so it is refused.
    fn item(&mut self, item: ItemId) -> Result<(), &'static str> {
        let imported = self
            .facts
            .crates
            .as_deref()
            .and_then(crate::sema::crates::Linkage::origin_catalog)
            .map(|catalog| catalog.declaration_if_imported(DefId(item.0)))
            .transpose()?
            .flatten();
        if imported.is_some() {
            return Err("type identity of another crate's declaration is not implemented");
        }
        self.owner();
        self.word(item.0);
        Ok(())
    }
    fn closure(&mut self, identity: ClosureId) -> Result<(), &'static str> {
        let source = self
            .facts
            .closures
            .identity(identity)
            .ok_or("type identity of a closure without its source body is not implemented")?
            .source;
        self.owner();
        self.word(source.file.0);
        self.word(source.node.get());
        Ok(())
    }
    /// A lifetime of an open binder is its binder's distance and its order of
    /// first use there. Every other lifetime is free in this type and erased:
    /// a named or static one, and one bound outside every binder of the type,
    /// which is a lifetime parameter of the surrounding declaration.
    fn region(&mut self, region: RegionId) -> Result<(), &'static str> {
        let open = self.binders.len();
        let bound = region.bound_parts().and_then(|(depth, index)| {
            let binder = open.checked_sub(1 + usize::from(depth))?;
            Some((depth, index, self.binders.get_mut(binder)?))
        });
        let Some((depth, index, binder)) = bound else {
            return self.tag(b'_');
        };
        let order = if let Some(order) = binder.iter().position(|&seen| seen == index) {
            order
        } else {
            binder.push(index);
            binder.len() - 1
        };
        self.tag(b'^')?;
        self.word(u32::from(depth));
        self.count(order)
    }
    fn arguments(&mut self, arguments: GenericArgs) -> Result<(), &'static str> {
        let facts = self.facts;
        let arguments = facts
            .types
            .arguments(arguments)
            .ok_or("type identity has invalid generic arguments")?;
        self.count(arguments.len())?;
        for argument in arguments {
            match argument {
                Argument::Lifetime(region) => self.region(region)?,
                Argument::Type(ty) => self.ty(ty)?,
                Argument::Const(id) => {
                    let Some(ConstKind::Value(value)) = facts.types.const_kind(id) else {
                        return Err("type identity requires evaluated scalar const arguments");
                    };
                    self.tag(b'c')?;
                    self.ty(value.ty)?;
                    self.bytes.extend_from_slice(&value.bits.to_le_bytes());
                }
            }
        }
        Ok(())
    }
    fn list(&mut self, types: crate::types::ListId) -> Result<(), &'static str> {
        let facts = self.facts;
        let types = facts
            .types
            .list(types)
            .ok_or("type identity has an invalid type list")?;
        self.count(types.len())?;
        types.iter().try_for_each(|&ty| self.ty(ty))
    }
    fn array(&mut self, element: TypeId, length: u128) -> Result<(), &'static str> {
        self.tag(b'a')?;
        self.bytes.extend_from_slice(&length.to_le_bytes());
        self.ty(element)
    }
    /// An object type: its bounds and their inputs are inside one binder, its
    /// own lifetime is outside.
    fn object(
        &mut self,
        shape: crate::types::dynamic::ShapeId,
        arguments: GenericArgs,
        region: RegionId,
    ) -> Result<(), &'static str> {
        let facts = self.facts;
        let invalid = "type identity has an invalid object shape";
        let bounds = facts.types.dynamic_shape(shape).ok_or(invalid)?;
        self.tag(b'D')?;
        match bounds.principal {
            Some(principal) => {
                self.tag(1)?;
                self.item(principal)?;
            }
            None => self.tag(0)?,
        }
        self.word(bounds.principal_arguments);
        let bindings = facts.types.dynamic_bindings(shape).ok_or(invalid)?;
        self.count(bindings.len())?;
        for binding in bindings {
            self.item(binding.item)?;
            self.item(binding.trait_item)?;
            self.word(binding.arguments);
        }
        let autos = facts.types.dynamic_auto_traits(shape).ok_or(invalid)?;
        self.count(autos.len())?;
        self.bytes.extend(autos.iter().map(|&auto| auto as u8));
        self.binders.push(Vec::new());
        let inputs = self.arguments(arguments);
        self.binders.pop();
        inputs?;
        self.region(region)
    }
    /// A function pointer's inputs and output, inside its own binder.
    fn signature(
        &mut self,
        parameters: crate::types::ListId,
        result: TypeId,
    ) -> Result<(), &'static str> {
        self.binders.push(Vec::new());
        let signature = self.list(parameters).and_then(|()| self.ty(result));
        self.binders.pop();
        signature
    }
    fn ty(&mut self, ty: TypeId) -> Result<(), &'static str> {
        let kind = self
            .facts
            .types
            .kind(ty)
            .ok_or("type identity has an invalid type")?;
        match kind {
            Kind::Never => self.tag(b'!'),
            Kind::Primitive(primitive) => {
                self.tag(b'p')?;
                self.text(primitive.name());
                Ok(())
            }
            Kind::Adt(item, arguments) => {
                self.tag(b'A')?;
                self.item(item)?;
                self.arguments(arguments)
            }
            Kind::Foreign(item) => {
                self.tag(b'E')?;
                self.item(item)
            }
            Kind::Tuple(elements) => {
                self.tag(b'T')?;
                self.list(elements)
            }
            Kind::Array { element, length } => self.array(element, u128::from(length)),
            Kind::ArrayTerm { element, length } => {
                let Some(ConstKind::Value(length)) = self.facts.types.const_kind(length) else {
                    return Err("type identity requires evaluated array lengths");
                };
                self.array(element, length.bits)
            }
            Kind::Slice(element) => {
                self.tag(b's')?;
                self.ty(element)
            }
            Kind::Reference {
                mutable,
                region,
                referent,
            } => {
                self.tag(b'&')?;
                self.tag(u8::from(mutable))?;
                self.region(region)?;
                self.ty(referent)
            }
            Kind::Pointer { mutable, pointee } => {
                self.tag(b'*')?;
                self.tag(u8::from(mutable))?;
                self.ty(pointee)
            }
            Kind::Function {
                parameters,
                result,
                unsafe_,
                convention,
                ..
            } => {
                self.tag(b'F')?;
                self.tag(u8::from(unsafe_))?;
                self.tag(u8::from(convention.variadic))?;
                self.text(match convention.abi {
                    Abi::Rust => "Rust",
                    Abi::RustCall => "rust-call",
                    Abi::C => "C",
                    Abi::CUnwind => "C-unwind",
                    Abi::Unadjusted => "unadjusted",
                });
                self.signature(parameters, result)
            }
            Kind::FunctionItem {
                item, arguments, ..
            } => {
                self.tag(b'f')?;
                self.item(item)?;
                self.arguments(arguments)
            }
            Kind::Closure {
                identity,
                arguments,
                ..
            } => {
                self.tag(b'C')?;
                self.closure(identity)?;
                self.arguments(arguments)
            }
            Kind::Dynamic {
                shape,
                arguments,
                region,
                ..
            } => self.object(shape, arguments, region),
            Kind::Parameter(_) | Kind::Projection(_) => {
                Err("type identity requires a closed normalized type")
            }
            Kind::Opaque(..) | Kind::Future { .. } | Kind::Pattern { .. } => {
                Err("type identity of an opaque, future or pattern type is not implemented")
            }
        }
    }
}

/// Every identity that this process handed out, with the spelling it names.
static NAMED: LazyLock<Mutex<HashMap<u128, Box<[u8]>>>> = LazyLock::new(Mutex::default);

/// The identity that names `spelling`: its digest. A second spelling with the
/// digest of an earlier one is refused, so that no two types of one
/// compilation ever share an identity.
pub(super) fn identity(spelling: &[u8]) -> Result<u128, &'static str> {
    let identity = crate::support::digest::digest(spelling);
    let mut named = NAMED.lock().unwrap_or_else(PoisonError::into_inner);
    match named.get(&identity) {
        Some(known) if **known != *spelling => Err("two types share one identity digest"),
        Some(_) => Ok(identity),
        None => {
            named.insert(identity, spelling.into());
            Ok(identity)
        }
    }
}
