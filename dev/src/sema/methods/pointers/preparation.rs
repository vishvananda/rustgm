//! Pointer offset has an unsized-capable impl owner and a method-local Sized duty.
use super::Offset;
use crate::{
    resolve::{
        expansion::Report,
        model::{BuiltinIntrinsic, DefId},
    },
    sema::{
        generics::{Parameter, Predicate},
        model::{Declarations, Function},
    },
    support::intrinsics::Intrinsic,
    syntax::ast::List,
    types::{
        GenericArgs, ItemId, Kind, Primitive, TypeId,
        arguments::Argument,
        callable::Convention,
        parameters::{
            Parameter as Formal, ParameterKind,
            predicates::{Bound, Predicate as Requirement},
        },
    },
};
pub(in crate::sema) fn prepare(
    report: &Report,
    facts: &mut Declarations,
    definition: DefId,
) -> Result<Offset, &'static str> {
    if let Some(method) = facts.methods.pointer_method(definition) {
        return Ok(method);
    }
    let kind = report
        .resolution
        .intrinsics
        .identify(definition)
        .filter(|kind| kind.is_pointer_offset())
        .ok_or("unregistered pointer method")?;
    let declaration = *report
        .resolution
        .intrinsics
        .get(kind)
        .ok_or("missing pointer method metadata")?;
    Preparation { facts, declaration }.run()
}
struct Preparation<'a> {
    facts: &'a mut Declarations,
    declaration: BuiltinIntrinsic,
}
impl Preparation<'_> {
    fn run(&mut self) -> Result<Offset, &'static str> {
        let d = self.declaration;
        let owner = d.owner.ok_or("pointer method lacks impl owner")?;
        let parameter = self.facts.types.intern_parameter(Formal {
            owner: ItemId(owner.0),
            index: 0,
            kind: ParameterKind::Type { sized: false },
        })?;
        let element = self.facts.types.intern(Kind::Parameter(parameter))?;
        let receiver = self.facts.types.intern(Kind::Pointer {
            mutable: d.kind == Intrinsic::MutPointerAdd,
            pointee: element,
        })?;
        let parameters = self
            .facts
            .types
            .intern_list(&[receiver, TypeId::primitive(Primitive::Usize)])?;
        let signature = self.facts.types.intern(Kind::Function {
            parameters,
            result: receiver,
            unsafe_: true,
            convention: Convention::RUST,
            bound_regions: 0,
        })?;
        self.environment(owner, element, receiver)?;
        self.facts.lifetimes.callables.insert_scoped(
            &self.facts.types,
            &[ItemId(owner.0), ItemId(d.definition.0)],
            signature,
            0,
            &[],
            &[parameter],
        )?;
        self.facts.definitions[d.definition.0 as usize] =
            Some(self.facts.types.intern(Kind::FunctionItem {
                item: ItemId(d.definition.0),
                signature,
                arguments: GenericArgs::EMPTY,
            })?);
        self.facts
            .methods
            .receivers
            .insert(d.definition, (receiver, 0));
        let method = Offset {
            definition: d.definition,
            parameter,
            receiver,
        };
        self.facts.methods.install_pointer_method(method)?;
        self.facts
            .function_indices
            .insert(d.definition, self.facts.functions.len());
        self.facts.functions.push(Function {
            definition: d.definition,
            node: d.origin,
            signature,
            parameters: List { start: 0, end: 0 },
            body: None,
        });
        self.facts.constant_functions.insert(d.definition);
        self.facts.typed_function_bodies.insert(d.definition);
        Ok(method)
    }
    fn environment(
        &mut self,
        owner: DefId,
        element: TypeId,
        receiver: TypeId,
    ) -> Result<(), &'static str> {
        let d = self.declaration;
        let parameter = d
            .parameter
            .ok_or("pointer method lacks inherited parameter")?;
        self.facts.definitions[owner.0 as usize] = Some(receiver);
        self.facts.definitions[parameter.0 as usize] = Some(element);
        self.facts.generics.parameters.insert(
            owner,
            vec![Parameter {
                binding: parameter,
                anonymous: false,
                argument: Argument::Type(element),
                default: None,
            }],
        );
        self.facts
            .generics
            .parameters
            .insert(d.definition, Vec::new());
        self.facts.generics.parents.insert(d.definition, owner);
        self.facts.generics.predicates.insert(
            d.definition,
            vec![Predicate {
                source: d.origin,
                value: Requirement {
                    ty: element,
                    bound: Bound::Sized,
                }
                .into(),
            }],
        );
        for owner in [owner, d.definition] {
            self.facts
                .generics
                .complete_environment(&mut self.facts.types, owner)?;
        }
        Ok(())
    }
}
