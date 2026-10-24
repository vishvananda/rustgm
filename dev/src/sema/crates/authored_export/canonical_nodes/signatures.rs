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

impl Stage {
    pub(in super::super) fn signature(
        &mut self,
        facts: &Declarations,
        ty: TypeId,
        parent: Context,
        public: bool,
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
        )
    }

    pub(crate) fn call_signature(
        &mut self,
        facts: &Declarations,
        ty: TypeId,
        context: Context,
        value: u32,
    ) -> Result<g::TypeRef, &'static str> {
        self.charge(2)?;
        let closed = facts.types.needs_region_context(ty) == Some(false)
            && facts.types.contains_parameters(ty) == Some(false);
        let key = super::Key {
            ty,
            context: (!closed).then_some(context),
        };
        self.graph.roots = self.graph.roots.saturating_add(1);
        if let Some(&root) = self.graph.types.get(&key) {
            self.graph.hits = self.graph.hits.saturating_add(1);
            return Ok(root);
        }
        let body = context
            .body
            .ok_or("source call signature has no actual original owner")?;
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
        let (signature, _) = self.signature_for(facts, ty, parent, purpose)?;
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
    ) -> Result<(g::Signature, Context), &'static str> {
        self.charge(1)?;
        let key = Key {
            ty,
            context: parent,
            purpose,
        };
        if let Some(&completed) = self.graph.signatures.get(&key) {
            return Ok(completed);
        }
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
        let kind = match purpose {
            Purpose::Public => s::BinderKind::PublicFunction,
            Purpose::Original => s::BinderKind::OriginalFunction,
            Purpose::SharedCall => s::BinderKind::CallableType,
            Purpose::Call { body, value } => s::BinderKind::CallSignature { body, value },
        };
        let context = Context {
            body: parent.body,
            ..self.frame(parent.owner, parent.frame, kind, bound_regions)?
        };
        let parameter_types = facts
            .types
            .list(parameters)
            .ok_or("source callable input list is unknown")?;
        self.reserve(
            parameter_types
                .len()
                .saturating_mul(4 * size_of::<g::TypeRef>()),
        )?;
        let start =
            u32::try_from(self.rows.roots.len()).map_err(|_| "source callable roots overflow")?;
        for &parameter in parameter_types {
            self.charge(1)?;
            let ty = self.ty(facts, parameter, context)?;
            self.rows.roots.push(ty);
        }
        let end =
            u32::try_from(self.rows.roots.len()).map_err(|_| "source callable roots overflow")?;
        let result = self.ty(facts, result, context)?;
        let signature = g::Signature {
            binder: self.graph.frames
                [context.frame.ok_or("source callable frame was lost")?.0 as usize]
                .binder,
            parameters: g::Range { start, end },
            result,
            unsafe_,
            convention: match convention.abi {
                crate::types::callable::Abi::Rust => g::Convention::Rust,
                crate::types::callable::Abi::RustCall => g::Convention::RustCall,
                crate::types::callable::Abi::C => g::Convention::C,
            },
            variadic: convention.variadic,
        };
        self.reserve(retained::new_map::<Key, (g::Signature, Context)>(1))?;
        self.graph.signatures.insert(key, (signature, context));
        Ok((signature, context))
    }
}
