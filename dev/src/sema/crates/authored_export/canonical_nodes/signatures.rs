//! Declaration and public callable frames remain distinct even for equal type IDs.
use super::{BodyRef, Context, Declarations, Kind, Stage, TypeId, g, retained, s};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Key {
    ty: TypeId,
    context: Context,
    purpose: Purpose,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Purpose {
    Original,
    Public,
    SharedCall,
    Call { body: BodyRef, value: u32 },
}

#[derive(Clone, Copy)]
pub(super) struct Parts {
    pub bound_regions: u32,
    pub parameters: crate::types::ListId,
    pub result: TypeId,
    unsafe_: bool,
    convention: crate::types::callable::Convention,
}
impl Parts {
    pub(super) fn actual(facts: &Declarations, ty: TypeId) -> Result<Self, &'static str> {
        let Some(Kind::Function {
            bound_regions,
            parameters,
            result,
            unsafe_,
            convention,
        }) = facts.types.kind(ty)
        else {
            return Err("source callable header has no actual function signature");
        };
        Ok(Self {
            bound_regions,
            parameters,
            result,
            unsafe_,
            convention,
        })
    }
    pub(super) fn ordinary(self) -> bool {
        self.convention.abi == crate::types::callable::Abi::Rust && !self.convention.variadic
    }
}

#[derive(Clone, Copy)]
pub(super) enum Roots {
    Demand,
    Completed,
}

impl Stage {
    pub(in super::super) fn signature(
        &mut self,
        facts: &Declarations,
        ty: TypeId,
        parent: Context,
        public: bool,
        regions: Option<u32>,
    ) -> Result<(g::Signature, Context), &'static str> {
        self.signature_for(
            facts,
            ty,
            parent,
            if public {
                Purpose::Public
            } else {
                Purpose::Original
            },
            regions,
        )
    }

    pub(in super::super) fn written_signature(
        &mut self,
        facts: &Declarations,
        ty: TypeId,
        context: Context,
    ) -> Result<g::Signature, &'static str> {
        self.signature_roots(facts, Parts::actual(facts, ty)?, context, Roots::Demand)
    }

    pub(crate) fn call_signature(
        &mut self,
        facts: &Declarations,
        ty: TypeId,
        context: Context,
        value: u32,
    ) -> Result<g::TypeRef, &'static str> {
        self.charge(2)?;
        self.graph.require_arena(&facts.types)?;
        let parts = Parts::actual(facts, ty)?;
        let body = context
            .body
            .ok_or("source call signature has no actual original owner")?;
        let local = super::Key {
            ty,
            context: Some(context),
            formation: super::Formation::Call { value },
        };
        let shared = super::Key {
            ty,
            context: None,
            formation: super::Formation::SharedCall,
        };
        self.graph.roots = self.graph.roots.saturating_add(1);
        // Only this producer writes SharedCall, and only without anonymous
        // signature fields. Formation separates real recipes and value sites.
        for key in [local, shared] {
            if let Some(&root) = self.graph.types.get(&key) {
                self.graph.hits = self.graph.hits.saturating_add(1);
                return Ok(root);
            }
        }
        let nested = self.anonymous_signature_field(facts, parts)?.is_some();
        // A constant can reuse a function's shared signature, but its own
        // signatures are bound to the actual Call in the initializer.
        let initializer = self.rows.body_owners[body.0 as usize].kind
            == crate::support::crates::owner_catalog::BodyKind::Constant;
        let closed = !initializer
            && facts.types.needs_region_context(ty) == Some(false)
            && facts.types.contains_parameters(ty) == Some(false)
            // Named item handles belong to the actual caller body, even when
            // their callable signature has no anonymous binder or region.
            && facts.types.contains_function_items(ty) == Some(false)
            && !nested;
        let key = if closed { shared } else { local };
        let (parent, purpose) = if closed {
            (
                Context {
                    frame: None,
                    body: None,
                    ..context
                },
                Purpose::SharedCall,
            )
        } else {
            (context, Purpose::Call { body, value })
        };
        let (signature, _) = self.signature_for(facts, ty, parent, purpose, None)?;
        self.reserve(4 * size_of::<s::Type>() + retained::new_map::<super::Key, g::TypeRef>(1))?;
        let id = g::TypeRef(
            u32::try_from(self.rows.types.len()).map_err(|_| "source call type table overflow")?,
        );
        self.rows.types.push(s::Type::Function(signature));
        self.graph.types.insert(key, id);
        if !closed {
            self.graph.contextual_nodes = self.graph.contextual_nodes.saturating_add(1);
        }
        Ok(id)
    }

    fn signature_for(
        &mut self,
        facts: &Declarations,
        ty: TypeId,
        parent: Context,
        purpose: Purpose,
        regions: Option<u32>,
    ) -> Result<(g::Signature, Context), &'static str> {
        self.charge(1)?;
        self.graph.require_arena(&facts.types)?;
        let key = Key {
            ty,
            context: parent,
            purpose,
        };
        if let Some(&completed) = self.graph.signatures.get(&key) {
            return Ok(completed);
        }
        let parts = Parts::actual(facts, ty)?;
        let count = match regions {
            Some(total) if total >= parts.bound_regions => total,
            Some(_) => {
                return Err("source original signature differs from its actual lifetime scheme");
            }
            None => parts.bound_regions,
        };
        let kind = match purpose {
            Purpose::Public => s::BinderKind::PublicFunction,
            Purpose::Original => s::BinderKind::OriginalFunction,
            Purpose::SharedCall => s::BinderKind::CallableType,
            Purpose::Call { body, value } => s::BinderKind::CallSignature { body, value },
        };
        let context = Context {
            body: parent.body,
            ..self.frame(parent.owner, parent.frame, kind, count)?
        };
        let signature = self.signature_roots(facts, parts, context, Roots::Demand)?;
        self.reserve(retained::new_map::<Key, (g::Signature, Context)>(1))?;
        self.graph.signatures.insert(key, (signature, context));
        Ok((signature, context))
    }

    pub(super) fn signature_roots(
        &mut self,
        facts: &Declarations,
        parts: Parts,
        context: Context,
        mode: Roots,
    ) -> Result<g::Signature, &'static str> {
        let parameter_types = facts
            .types
            .list(parts.parameters)
            .ok_or("source callable input list is unknown")?;
        // A nested type recipe may append its own root range while being emitted.
        // Reserve the signature's exact parameter range before demanding children.
        self.reserve(
            parameter_types
                .len()
                .saturating_mul(4 * size_of::<g::TypeRef>()),
        )?;
        let start = self.rows.roots.len();
        let end = start
            .checked_add(parameter_types.len())
            .ok_or("source callable roots overflow")?;
        let bounds = g::Range {
            start: u32::try_from(start).map_err(|_| "source callable roots overflow")?,
            end: u32::try_from(end).map_err(|_| "source callable roots overflow")?,
        };
        self.rows.roots.resize(end, g::TypeRef(0));
        for (index, &parameter) in parameter_types.iter().enumerate() {
            self.charge(1)?;
            let ty = self.signature_root(facts, parameter, context, mode)?;
            self.rows.roots[start + index] = ty;
        }
        let result = self.signature_root(facts, parts.result, context, mode)?;
        Ok(g::Signature {
            binder: self.graph.frames
                [context.frame.ok_or("source callable frame was lost")?.0 as usize]
                .binder,
            parameters: bounds,
            result,
            unsafe_: parts.unsafe_,
            convention: match parts.convention.abi {
                crate::types::callable::Abi::Rust => g::Convention::Rust,
                crate::types::callable::Abi::RustCall => g::Convention::RustCall,
                crate::types::callable::Abi::C => g::Convention::C,
                crate::types::callable::Abi::CUnwind => {
                    return Err("C-unwind source metadata requires its versioned unwind ABI");
                }
                crate::types::callable::Abi::Unadjusted if self.declaration_policies() => {
                    g::Convention::Unadjusted
                }
                crate::types::callable::Abi::Unadjusted => {
                    return Err("unadjusted ABI source metadata is not implemented");
                }
            },
            variadic: parts.convention.variadic,
        })
    }

    fn signature_root(
        &mut self,
        facts: &Declarations,
        ty: TypeId,
        context: Context,
        mode: Roots,
    ) -> Result<g::TypeRef, &'static str> {
        match mode {
            Roots::Demand => self.ty(facts, ty, context),
            Roots::Completed => {
                let key = self.type_key(facts, ty, context)?;
                self.graph
                    .types
                    .get(&key)
                    .copied()
                    .ok_or("source callable child was not completed before its parent")
            }
        }
    }
}
