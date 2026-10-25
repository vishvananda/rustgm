//! Canonical summaries select actual ownership keys; graph traversal is memoized.
mod memo_owner;
mod owned_types;
mod regional_roots;
mod signatures;
#[cfg(test)]
#[path = "../../../../tests/invariants/anonymous_functions.rs"]
mod tests;
use super::{DeclarationRef, Declarations, DefId, Stage, g, s};
use crate::support::crates::owner_catalog::BodyRef;
use crate::{
    support::{hash::HashMap, retained},
    types::{Kind, RegionId, TypeId, parameters::ParameterId},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct FrameRef(u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Context {
    pub(super) owner: DeclarationRef,
    pub(super) frame: Option<FrameRef>,
    pub(super) body: Option<BodyRef>,
}
impl Context {
    pub(crate) const fn owner(self) -> DeclarationRef {
        self.owner
    }
    pub(crate) const fn body(self) -> Option<BodyRef> {
        self.body
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Formation {
    Recipe,
    SharedCall,
    Call { value: u32 },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    ty: TypeId,
    context: Option<Context>,
    formation: Formation,
}
struct Frame {
    binder: g::BinderRef,
    parent: Option<FrameRef>,
    count: u32,
}
#[derive(Default)]
pub(super) struct Graph {
    arena: Option<crate::types::arena::Identity>,
    pub(super) parameters: HashMap<ParameterId, s::SlotRef>,
    types: HashMap<Key, g::TypeRef>,
    regions: HashMap<(RegionId, Option<Context>), s::RegionRef>,
    variables: HashMap<(BodyRef, RegionId), u32>,
    frames: Vec<Frame>,
    ancestors: HashMap<(FrameRef, u16), FrameRef>,
    signatures: HashMap<signatures::Key, (g::Signature, Context)>,
    applications: HashMap<
        (crate::types::ItemId, crate::types::GenericArgs, Context),
        s::FunctionApplicationRef,
    >,
    /// Visits are retained separately from cache lookups: depth times distinct
    /// contextual roots may perform real work even when later reuse is O(1).
    contextual_nodes: usize,
    ancestor_steps: usize,
    roots: usize,
    hits: usize,
}
impl Graph {
    pub(super) const fn work(&self) -> (usize, usize, usize, usize) {
        (
            self.contextual_nodes,
            self.ancestor_steps,
            self.roots,
            self.hits,
        )
    }
}

impl Stage {
    pub(crate) fn parameter_slot(
        &mut self,
        parameter: ParameterId,
    ) -> Result<s::SlotRef, &'static str> {
        self.charge(1)?;
        self.graph
            .parameters
            .get(&parameter)
            .copied()
            .ok_or("source original has an unreserved parameter identity")
    }
    pub(crate) fn universals(&mut self, context: Context) -> Result<u32, &'static str> {
        self.charge(1)?;
        context
            .frame
            .and_then(|frame| self.graph.frames.get(frame.0 as usize))
            .map(|frame| frame.count)
            .ok_or("source original has no actual universal frame")
    }
    pub(crate) fn scoped_context(
        &mut self,
        context: Context,
        count: u32,
    ) -> Result<Context, &'static str> {
        Ok(Context {
            body: context.body,
            ..self.frame(context.owner, context.frame, s::BinderKind::Clause, count)?
        })
    }
    pub(crate) fn binder(&mut self, context: Context) -> Result<g::BinderRef, &'static str> {
        self.charge(1)?;
        context
            .frame
            .and_then(|frame| self.graph.frames.get(frame.0 as usize))
            .map(|frame| frame.binder)
            .ok_or("source context has no actual binder frame")
    }
    pub(super) fn frame(
        &mut self,
        owner: DeclarationRef,
        parent: Option<FrameRef>,
        kind: s::BinderKind,
        count: u32,
    ) -> Result<Context, &'static str> {
        self.charge(1)?;
        if self.rows.declarations.get(owner.0 as usize).is_none()
            || parent.is_some_and(|parent| self.graph.frames.get(parent.0 as usize).is_none())
        {
            return Err("source binder has an unknown owner or parent frame");
        }
        self.reserve(4 * (size_of::<Frame>() + size_of::<s::Binder>()))?;
        let frame = FrameRef(
            u32::try_from(self.graph.frames.len()).map_err(|_| "source frame table overflow")?,
        );
        let binder = g::BinderRef(
            u32::try_from(self.rows.binders.len()).map_err(|_| "source binder table overflow")?,
        );
        self.rows.binders.push(s::Binder {
            owner,
            parent: parent.map(|parent| self.graph.frames[parent.0 as usize].binder),
            kind,
            count,
        });
        self.graph.frames.push(Frame {
            binder,
            parent,
            count,
        });
        Ok(Context {
            owner,
            frame: Some(frame),
            body: None,
        })
    }

    fn type_key(
        &mut self,
        facts: &Declarations,
        ty: TypeId,
        context: Context,
    ) -> Result<Key, &'static str> {
        self.charge(1)?;
        self.graph.require_arena(&facts.types)?;
        let contextual = facts
            .types
            .needs_region_context(ty)
            .ok_or("source type root is unknown")?;
        let anonymous = facts
            .types
            .contains_anonymous_callables(ty)
            .ok_or("source type root has no actual callable summary")?;
        Ok(Key {
            ty,
            context: (contextual || anonymous).then_some(context),
            formation: Formation::Recipe,
        })
    }

    pub(crate) fn function_application(
        &mut self,
        facts: &Declarations,
        item: crate::types::ItemId,
        arguments: crate::types::GenericArgs,
        context: Context,
    ) -> Result<s::FunctionApplicationRef, &'static str> {
        self.charge(1)?;
        self.graph.require_arena(&facts.types)?;
        let key = (item, arguments, context);
        if let Some(&application) = self.graph.applications.get(&key) {
            return Ok(application);
        }
        let actual = facts
            .types
            .arguments(arguments)
            .ok_or("source function application arguments are unknown")?;
        for argument in actual {
            self.charge(1)?;
            if let crate::types::arguments::Argument::Type(ty) = argument
                && facts.types.contains_callables(ty) != Some(false)
            {
                return Err("nested callable source arguments require a wider transport profile");
            }
        }
        let declaration = self.address(DefId(item.0))?;
        let arguments = self.arguments(facts, arguments, context)?;
        self.reserve(
            4 * size_of::<s::FunctionApplication>()
                + retained::new_map::<
                    (crate::types::ItemId, crate::types::GenericArgs, Context),
                    s::FunctionApplicationRef,
                >(1),
        )?;
        let id = s::FunctionApplicationRef(
            u32::try_from(self.rows.function_applications.len())
                .map_err(|_| "source function application table overflow")?,
        );
        self.rows
            .function_applications
            .push(s::FunctionApplication {
                declaration,
                arguments,
            });
        self.graph.applications.insert(key, id);
        Ok(id)
    }
}
