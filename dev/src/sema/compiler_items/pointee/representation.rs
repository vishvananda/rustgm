//! One representation plan serves both scoped projection consumers.
use super::{CheckedMember, Declarations, Selection, header_attributes};
use crate::{
    abi::metadata::{self, Metadata},
    types::{ItemId, Kind, Primitive, TypeId, arguments::Argument, projection},
};

pub(in crate::sema) enum Step {
    Ready(TypeId),
    Fields(ItemId),
    Sized(TypeId),
}
pub(in crate::sema) struct Plan {
    member: CheckedMember,
    tail: TypeId,
}
impl Plan {
    pub(in crate::sema) fn new(
        facts: &mut Declarations,
        selected: &Selection,
        application: projection::Application,
    ) -> Result<Self, String> {
        let member = selected
            .member(facts, application)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            member,
            tail: application.subject,
        })
    }
    pub(in crate::sema) fn advance(&mut self, facts: &mut Declarations) -> Result<Step, String> {
        let application = self
            .member
            .current_facts(facts)
            .map_err(|error| error.to_string())?;
        loop {
            header_attributes::charge_source_retention(facts, 1, 0)
                .map_err(|error| error.to_string())?;
            if let Some(Kind::Adt(item, _)) = facts.types.kind(self.tail) {
                if facts.aggregates.get(item).is_none() {
                    return Ok(Step::Fields(item));
                }
                // Reuse canonical immediate-field substitution. Its own owner
                // retains and bounds failed work as well as completed instances.
                facts.aggregates.instantiate(&mut facts.types, self.tail)?;
            }
            match metadata::declared_step(&facts.types, &facts.aggregates, self.tail)? {
                metadata::Step::Tail(tail) => {
                    if tail == self.tail {
                        return Err("recursive source metadata tail".into());
                    }
                    self.tail = tail;
                }
                metadata::Step::Complete(Metadata::Thin) => {
                    return self.ready(facts, Kind::Tuple(crate::types::ListId::EMPTY));
                }
                metadata::Step::Complete(Metadata::Length) => {
                    return self.ready(facts, Kind::Primitive(Primitive::Usize));
                }
                metadata::Step::Complete(Metadata::VTable(tail)) => {
                    let nominal = self
                        .member
                        .contract
                        .plan
                        .nominal
                        .ok_or("source dynamic metadata requires actual DynMetadata")?;
                    let growth = facts.types.trait_arguments_growth(1);
                    header_attributes::charge_source_retention(
                        facts,
                        1 + growth.work,
                        growth.bytes,
                    )
                    .map_err(|error| error.to_string())?;
                    let arguments = facts.types.intern_arguments(&[Argument::Type(tail)])?;
                    return self.ready(facts, Kind::Adt(ItemId(nominal.definition.0), arguments));
                }
                metadata::Step::Complete(Metadata::Unknown(tail))
                    if tail != application.subject =>
                {
                    // The caller's existing normalizer handles this genuine
                    // nested projection and its input, equality and region proof.
                    return self.ready(
                        facts,
                        Kind::Projection(projection::Application {
                            subject: tail,
                            ..application
                        }),
                    );
                }
                metadata::Step::Complete(Metadata::Unknown(tail)) => return Ok(Step::Sized(tail)),
            }
        }
    }
    fn ready(&self, facts: &mut Declarations, kind: Kind) -> Result<Step, String> {
        header_attributes::charge_source_retention(facts, 1, size_of::<Kind>())
            .map_err(|error| error.to_string())?;
        Ok(Step::Ready(facts.types.intern(kind)?))
    }
}
