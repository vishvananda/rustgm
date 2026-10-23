//! Owned Ord helpers borrow for selected comparisons, then move one result and drop the rest.
use super::{statement, value};
use crate::{
    body::{
        ir::{
            Action, Block, BlockId, Body, BorrowKind, Local, LocalId, LocalKind, Owner, Place,
            Rvalue, Terminator, ValueId, ValueList,
        },
        regions::{Contract, Kind as Debt, Obligation},
    },
    resolve::{model::DefId, unit::NodeKey},
    sema::model::Declarations,
    support::comparisons::Method,
    types::{
        ItemId, Kind, RegionId, TypeId, arguments::Argument, callable::Convention,
        substitute::Bound,
    },
};
struct Helper {
    body: Body,
    subject: TypeId,
    next_region: u32,
}
impl Helper {
    fn new(
        facts: &Declarations,
        definition: DefId,
        method: Method,
        signature: TypeId,
        source: NodeKey,
    ) -> Result<Self, &'static str> {
        let Some(Kind::Function {
            parameters,
            result,
            bound_regions: 0,
            unsafe_: false,
            convention: Convention::RUST,
        }) = facts.types.kind(signature)
        else {
            return Err("owned helper has incorrect callable contract");
        };
        let parameters = facts
            .types
            .list(parameters)
            .ok_or("missing helper parameters")?;
        let count = if method == Method::Clamp { 3 } else { 2 };
        if !method.owned()
            || facts.associated.comparison_method(definition) != Some(method)
            || parameters.len() != count
            || parameters.iter().any(|&ty| ty != result)
        {
            return Err("owned helper parameters differ from canonical member");
        }
        let mut body = Body::new(
            Owner::Function {
                definition,
                signature,
                execution: None,
            },
            source,
            result,
        );
        body.storage = Some(crate::body::storage::Facts::new(
            crate::body::scopes::Facts::synthetic(source.node),
        ));
        for (index, &ty) in parameters.iter().enumerate() {
            let index = u32::try_from(index).map_err(|_| "too many helper parameters")?;
            body.locals.push(Local {
                scope: crate::body::scopes::Id(0),
                storage_parent: index.checked_sub(1).map(LocalId),
                ty,
                kind: LocalKind::Parameter(index),
                definition: None,
                mutable: false,
                source: source.node.into(),
            });
            statement(&mut body, Action::StorageLive(LocalId(index)));
        }
        Ok(Self {
            body,
            subject: result,
            next_region: 1,
        })
    }
    fn call(
        &mut self,
        facts: &mut Declarations,
        contract: &mut Contract,
        definition: DefId,
        member: Method,
        left: u32,
        right: u32,
    ) -> Result<ValueId, &'static str> {
        let (callable, signature) = callable(facts, definition, member, self.subject)?;
        let mut regions = Vec::new();
        for _ in 0..2 {
            let region = RegionId(self.next_region);
            self.next_region = self
                .next_region
                .checked_add(1)
                .ok_or("too many helper borrow regions")?;
            contract.obligations.push(Obligation {
                source: self.body.source.into(),
                kind: Debt::Variable {
                    id: region,
                    needs_provenance: true,
                },
            });
            contract.obligations.push(Obligation {
                source: self.body.source.into(),
                kind: Debt::LocalBorrow(region),
            });
            regions.push(region);
        }
        let opened = Bound::new(regions).signature(&mut facts.types, signature, 0)?;
        let Some(Kind::Function {
            parameters,
            result,
            bound_regions: 0,
            unsafe_: false,
            convention: Convention::RUST,
        }) = facts.types.kind(opened)
        else {
            return Err("helper comparison is not a safe Rust call");
        };
        let parameters = facts
            .types
            .list(parameters)
            .ok_or("missing helper comparison parameters")?
            .to_vec();
        if parameters.len() != 2 {
            return Err("helper comparison has incorrect arity");
        }
        let start =
            u32::try_from(self.body.arguments.len()).map_err(|_| "too many helper arguments")?;
        for (&ty, local) in parameters.iter().zip([left, right]) {
            if !matches!(facts.types.kind(ty),Some(Kind::Reference{referent,mutable:false,..}) if referent==self.subject)
            {
                return Err("helper comparison requires independent shared operands");
            }
            let borrow = value(
                &mut self.body,
                ty,
                Rvalue::Borrow {
                    place: Place::Local(LocalId(local)),
                    kind: BorrowKind::Immediate,
                },
            )?;
            self.body.arguments.push(borrow);
        }
        let callee = value(&mut self.body, callable, Rvalue::Function(definition))?;
        value(
            &mut self.body,
            result,
            Rvalue::Call {
                signature: opened,
                callee,
                arguments: ValueList {
                    start,
                    end: start + 2,
                },
            },
        )
    }
    fn branch(
        &mut self,
        start: u32,
        condition: ValueId,
        yes: u32,
        no: u32,
    ) -> Result<u32, &'static str> {
        let end =
            u32::try_from(self.body.statements.len()).map_err(|_| "too many helper statements")?;
        self.body.blocks.push(Block {
            start,
            end,
            source: self.body.root_site(),
            terminator: Terminator::Branch {
                condition,
                yes: BlockId(yes),
                no: BlockId(no),
            },
        });
        Ok(end)
    }
    fn finish(&mut self, start: u32, local: u32) -> Result<u32, &'static str> {
        let result = value(
            &mut self.body,
            self.subject,
            Rvalue::Read(Place::Local(LocalId(local))),
        )?;
        crate::body::build::finish_factory_block(&mut self.body, start, result)?;
        u32::try_from(self.body.statements.len()).map_err(|_| "too many helper statements")
    }
}
pub(super) fn build(
    facts: &mut Declarations,
    definition: DefId,
    default: crate::sema::associated::LibraryDefault,
    signature: TypeId,
    source: NodeKey,
    contract: &mut Contract,
) -> Result<Body, &'static str> {
    let crate::sema::associated::LibraryDefault::OrdHelper {
        method,
        cmp,
        partial,
    } = default
    else {
        return Err("provided helper has different default kind");
    };
    let mut helper = Helper::new(facts, definition, method, signature, source)?;
    if method == Method::Clamp {
        helper.clamp(facts, contract, partial)?;
    } else {
        helper.extreme(facts, contract, method, cmp)?;
    }
    Ok(helper.body)
}
impl Helper {
    fn extreme(
        &mut self,
        facts: &mut Declarations,
        contract: &mut Contract,
        method: Method,
        cmp: DefId,
    ) -> Result<(), &'static str> {
        let comparison = self.call(facts, contract, cmp, Method::Cmp, 0, 1)?;
        let ordering = self.body.values[comparison.0 as usize].ty;
        let Some(Kind::Adt(owner, _)) = facts.types.kind(ordering) else {
            return Err("helper cmp result has no Ordering schema");
        };
        let [_, _, greater] = facts.members.variants(owner) else {
            return Err("helper cmp has invalid Ordering variants");
        };
        let greater = *greater;
        let local =
            LocalId(u32::try_from(self.body.locals.len()).map_err(|_| "too many helper locals")?);
        self.body.locals.push(Local {
            scope: crate::body::scopes::Id(0),
            storage_parent: local.0.checked_sub(1).map(LocalId),
            ty: ordering,
            kind: LocalKind::Binding,
            definition: None,
            mutable: false,
            source: self.body.root_site(),
        });
        statement(&mut self.body, Action::StorageLive(local));
        statement(
            &mut self.body,
            Action::Store {
                place: Place::Local(local),
                value: comparison,
            },
        );
        let greater = value(
            &mut self.body,
            TypeId::primitive(crate::types::Primitive::Bool),
            Rvalue::IsVariant {
                place: Place::Local(local),
                variant: greater,
            },
        )?;
        let start = self.branch(0, greater, 1, 2)?;
        let selected = u32::from(method == Method::Min);
        let start = self.finish(start, selected)?;
        self.finish(start, 1 - selected)?;
        Ok(())
    }
    fn clamp(
        &mut self,
        facts: &mut Declarations,
        contract: &mut Contract,
        [le, lt, gt]: [DefId; 3],
    ) -> Result<(), &'static str> {
        let valid = self.call(facts, contract, le, Method::Le, 1, 2)?;
        let start = self.branch(0, valid, 1, 6)?;
        let low = self.call(facts, contract, lt, Method::Lt, 0, 1)?;
        let start = self.branch(start, low, 2, 3)?;
        let start = self.finish(start, 1)?;
        let high = self.call(facts, contract, gt, Method::Gt, 0, 2)?;
        let start = self.branch(start, high, 4, 5)?;
        let start = self.finish(start, 2)?;
        let end = self.finish(start, 0)?;
        self.body.blocks.push(Block {
            start: end,
            end,
            source: self.body.root_site(),
            terminator: Terminator::Panic,
        });
        Ok(())
    }
}

fn callable(
    facts: &mut Declarations,
    definition: DefId,
    member: Method,
    subject: TypeId,
) -> Result<(TypeId, TypeId), &'static str> {
    if facts.associated.comparison_method(definition) != Some(member) {
        return Err("helper comparison has different canonical member");
    }
    let Some(Kind::FunctionItem {
        item, signature, ..
    }) = facts
        .definitions
        .get(definition.0 as usize)
        .copied()
        .flatten()
        .and_then(|ty| facts.types.kind(ty))
    else {
        return Err("helper comparison has no callable declaration");
    };
    if item != ItemId(definition.0) {
        return Err("helper comparison owner differs");
    }
    let arguments = vec![Argument::Type(subject); if member.owner().has_rhs() { 2 } else { 1 }];
    let parameters = facts
        .lifetimes
        .callables
        .generic_parameters(item)
        .ok_or("helper comparison has no scoped parameters")?;
    if parameters.len() != arguments.len() {
        return Err("helper comparison generic arity differs");
    }
    let bindings = parameters
        .iter()
        .copied()
        .zip(arguments.iter().copied())
        .collect::<Vec<_>>();
    let mut substitute = Bound::parameters(&facts.types, &bindings)?;
    let signature = substitute.apply(&mut facts.types, signature)?;
    let arguments = facts.types.intern_arguments(&arguments)?;
    let callable = facts.types.intern(Kind::FunctionItem {
        item,
        signature,
        arguments,
    })?;
    if !matches!(
        facts.types.kind(signature),
        Some(Kind::Function {
            bound_regions: 2,
            unsafe_: false,
            convention: Convention::RUST,
            ..
        })
    ) {
        return Err("helper comparison requires its two-slot safe Rust contract");
    }
    Ok((callable, signature))
}
