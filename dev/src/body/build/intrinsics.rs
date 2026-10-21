//! Canonical callable identities become typed operations before lowering.
use super::{Builder, Error, NodeId, Rvalue, ValueId, ValueList};
use crate::{
    resolve::model::DefId,
    types::{Kind, Primitive, instance::Instance},
};
impl Builder<'_> {
    pub(super) fn intrinsic_call(
        &self,
        node: NodeId,
        callee: ValueId,
        arguments: ValueList,
    ) -> Result<Option<Rvalue>, Error> {
        let ty = self.body.values[callee.0 as usize].ty;
        let Some(Kind::FunctionItem {
            item, signature, ..
        }) = self.facts.types.kind(ty)
        else {
            return Ok(None);
        };
        let Some(intrinsic) = self.facts.intrinsics.identify(DefId(item.0)) else {
            return Ok(None);
        };
        if !intrinsic.is_volatile() {
            let instance = Instance::new(&self.facts.types, ty).map_err(|e| self.error(node, e))?;
            return Ok(Some(Rvalue::Intrinsic {
                intrinsic,
                instance,
                arguments,
            }));
        }
        let Some(Kind::Function { parameters, .. }) = self.facts.types.kind(signature) else {
            return Err(self.error(node, "intrinsic has no callable signature"));
        };
        let pointer = self
            .facts
            .types
            .list(parameters)
            .and_then(|parameters| parameters.first())
            .and_then(|&ty| self.facts.types.kind(ty));
        let Some(Kind::Pointer { pointee, .. }) = pointer else {
            return Err(self.error(node, "volatile intrinsic requires a typed raw pointer"));
        };
        match self.facts.types.kind(pointee) {
            Some(Kind::Primitive(primitive)) if primitive != Primitive::Str => {}
            Some(
                Kind::Pointer { .. }
                | Kind::Reference { .. }
                | Kind::Function { .. }
                | Kind::Parameter(_)
                | Kind::Projection(_)
                | Kind::Opaque(..),
            ) => {}
            _ => {
                return Err(self.error(
                    node,
                    "aggregate and zero-sized volatile operations are not implemented",
                ));
            }
        }
        let instance = Instance::new(&self.facts.types, ty).map_err(|e| self.error(node, e))?;
        Ok(Some(Rvalue::Intrinsic {
            intrinsic,
            instance,
            arguments,
        }))
    }
}
