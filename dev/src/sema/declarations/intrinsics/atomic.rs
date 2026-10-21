//! Canonical atomic methods are safe inherent callables with runtime-only bodies.
use super::{Checker, Error, ItemId, State, Type};
use crate::{
    resolve::model::BuiltinIntrinsic,
    sema::{associated::Schema, lifetimes::FunctionRegions, model::Function},
    support::{intrinsics::Intrinsic, library_types::LibraryType},
    syntax::ast::List,
    types::{GenericArgs, RegionId, TypeId, callable::Convention},
};
impl Checker<'_> {
    pub(super) fn atomic_signature(&mut self, declaration: BuiltinIntrinsic) -> Result<(), Error> {
        let BuiltinIntrinsic {
            origin,
            owner,
            kind,
            ..
        } = declaration;
        let owner = owner.ok_or_else(|| self.error(origin, "missing atomic method owner"))?;
        let subject = self.facts.definitions[owner.0 as usize]
            .ok_or_else(|| self.error(origin, "missing atomic nominal type"))?;
        let ordering = self
            .report
            .resolution
            .library_types
            .get(LibraryType::Ordering)
            .and_then(|nominal| self.facts.definitions[nominal.definition.0 as usize])
            .ok_or_else(|| self.error(origin, "missing canonical Ordering type"))?;
        let region = RegionId::bound(0, 0).map_err(|e| self.error(origin, e))?;
        let receiver = self.intern(
            origin,
            Type::Reference {
                mutable: false,
                region,
                referent: subject,
            },
        )?;
        let scalar = TypeId::primitive(crate::types::Primitive::U32);
        let inputs = if kind == Intrinsic::AtomicLoad {
            vec![receiver, ordering]
        } else {
            vec![receiver, scalar, ordering]
        };
        let parameters = self
            .facts
            .types
            .intern_list(&inputs)
            .map_err(|e| self.error(origin, e))?;
        let signature = self.intern(
            origin,
            Type::Function {
                bound_regions: 1,
                parameters,
                result: if kind == Intrinsic::AtomicStore {
                    TypeId::UNIT
                } else {
                    scalar
                },
                unsafe_: false,
                convention: Convention::RUST,
            },
        )?;
        self.atomic_method(declaration, owner, subject, receiver, signature)
    }
    fn atomic_method(
        &mut self,
        declaration: BuiltinIntrinsic,
        owner: crate::resolve::model::DefId,
        subject: TypeId,
        receiver: TypeId,
        signature: TypeId,
    ) -> Result<(), Error> {
        let BuiltinIntrinsic {
            origin, definition, ..
        } = declaration;
        self.facts
            .lifetimes
            .callables
            .insert_monomorphic(&self.facts.types, ItemId(definition.0), signature)
            .map_err(|e| self.error(origin, e))?;
        self.facts.lifetimes.functions.insert(
            definition,
            FunctionRegions {
                total: 1,
                captures: 0,
                early: Vec::new(),
                late: vec![0],
                has_declared_late: false,
                bounds: Vec::new(),
            },
        );
        self.facts
            .generics
            .parameters
            .insert(definition, Vec::new());
        self.facts.generics.parents.insert(definition, owner);
        self.facts
            .generics
            .complete_environment(&mut self.facts.types, definition)
            .map_err(|e| self.error(origin, e))?;
        self.facts
            .associated
            .declare_language_inherent(
                owner,
                Schema {
                    self_type: subject,
                    captures: 0,
                    bounds: Vec::new(),
                },
                definition,
                origin,
            )
            .map_err(|e| self.error(origin, e))?;
        let name = self.report.resolution.definitions[definition.0 as usize].name;
        self.facts
            .methods
            .inherent
            .entry((ItemId(owner.0), name))
            .or_default()
            .push(definition);
        self.facts
            .methods
            .receivers
            .insert(definition, (receiver, 1));
        let callable = self.intern(
            origin,
            Type::FunctionItem {
                item: ItemId(definition.0),
                arguments: GenericArgs::EMPTY,
                signature,
            },
        )?;
        self.facts.definitions[definition.0 as usize] = Some(callable);
        self.states[definition.0 as usize] = State::Complete;
        self.facts
            .function_indices
            .insert(definition, self.facts.functions.len());
        self.facts.functions.push(Function {
            definition,
            node: origin,
            signature,
            parameters: List { start: 0, end: 0 },
            body: None,
        });
        Ok(())
    }
}
