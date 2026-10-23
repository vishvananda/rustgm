//! Selected CFG operations reuse the canonical constant arithmetic and value arena.
use super::{Constant, Failure, Frame, Machine, ValueId};
use crate::{
    body::ir::{Rvalue, ValueList},
    sema::{
        model::Scalar,
        values::{Metadata, Overflow, Reader, View},
    },
    types::{Kind, Primitive, TypeId, instance::Instance},
};

pub(super) fn boolean(value: Constant) -> Result<bool, Failure> {
    if let Scalar::Boolean(value) = value.as_scalar()? {
        Ok(value)
    } else {
        Err("constant condition has no boolean value".into())
    }
}
pub(super) fn integer(value: Constant) -> Result<u128, Failure> {
    if let Scalar::Integer(value) = value.as_scalar()? {
        Ok(value)
    } else {
        Err("constant index has no integer value".into())
    }
}
impl Machine<'_, '_> {
    fn arguments(&mut self, frame: &Frame, list: ValueList) -> Result<Vec<Constant>, Failure> {
        let body = self.bodies[frame.body].clone();
        let arguments = body
            .arguments
            .get(list.start as usize..list.end as usize)
            .ok_or("invalid constant argument range")?;
        self.charge(arguments.len())?;
        arguments.iter().map(|&id| Self::value(frame, id)).collect()
    }
    #[expect(
        clippy::too_many_lines,
        reason = "One exhaustive typed rvalue dispatch preserves the checked CFG operation and its execution semantics together"
    )]
    pub(super) fn evaluate(
        &mut self,
        frame: &mut Frame,
        id: ValueId,
    ) -> Result<Option<(usize, Vec<Constant>, ValueId)>, Failure> {
        let body = self.bodies[frame.body].clone();
        let value = body
            .values
            .get(id.0 as usize)
            .ok_or("missing constant rvalue")?;
        let ty = value.ty;
        let result = match value.kind {
            Rvalue::InlineInstance(_) => Self::value(frame, id)?,
            Rvalue::Intrinsic {
                intrinsic,
                instance,
                arguments,
            } => {
                let inputs = self.arguments(frame, arguments)?;
                let Some(value) = self.intrinsic(intrinsic, instance, &inputs)? else {
                    return Ok(None);
                };
                value
            }
            Rvalue::Scalar(id) => Constant::scalar(
                *body
                    .scalars
                    .get(id.0 as usize)
                    .ok_or("missing constant scalar")?,
            ),
            Rvalue::Unit => self.evaluator.fields(&[], None)?,
            Rvalue::Union { field, value } => {
                self.make_union(ty, field, Self::value(frame, value)?)?
            }
            Rvalue::Aggregate { fields, variant } => {
                let arguments = self.arguments(frame, fields)?;
                self.evaluator.fields(&arguments, variant)?
            }
            Rvalue::Repeat { value, .. } => {
                let Some(Kind::Array { length, .. }) = self.evaluator.facts().types.kind(ty) else {
                    return Err("constant repeat lacks concrete array length".into());
                };
                self.evaluator.repeat(Self::value(frame, value)?, length)?
            }
            Rvalue::Constrain(value) => self.evaluator.constrain(
                Self::value(frame, value)?,
                body.values[value.0 as usize].ty,
                ty,
            )?,
            Rvalue::Reify(value) => self
                .evaluator
                .reify(body.values[value.0 as usize].ty, Self::value(frame, value)?)?,
            Rvalue::ConstArgument(id) => Constant::argument(&self.evaluator.facts().types, id)?
                .ok_or("constant value retains an unresolved const parameter")?,
            Rvalue::Function(_) => Constant::function(
                body.callable_target(Instance::new(&self.evaluator.facts().types, ty)?),
            ),
            Rvalue::Constant(definition) => {
                if let Some(value) = Reader::new(self.evaluator.facts()).definition(definition) {
                    value
                } else if let Some(&index) = self.bodies.definitions.get(&definition) {
                    return Ok(Some((index, Vec::new(), id)));
                } else {
                    self.waiting = Some(super::Requirement::InitializerBody(definition));
                    return Ok(None);
                }
            }
            Rvalue::InlineConstant(source) => {
                if let Some(value) = Reader::new(self.evaluator.facts()).inline(source) {
                    value
                } else if let Some(&index) = self.bodies.expressions.get(&source) {
                    return Ok(Some((index, Vec::new(), id)));
                } else {
                    self.waiting = Some(super::Requirement::InlineBody { source, result: ty });
                    return Ok(None);
                }
            }
            Rvalue::SourceAssociated(source) => {
                let enclosing =
                    body.owner
                        .source_caller(self.evaluator.facts(), body.source, body.instance)?;
                source.validate_use(self.evaluator.facts(), body.source, enclosing, ty)?;
                let (selected, _) = self.evaluator.checked_source(source)?;
                let index = self
                    .bodies
                    .sources
                    .as_deref()
                    .and_then(|map| map.get(&source))
                    .map(|&(index, _)| index);
                if index.is_none() {
                    self.waiting = Some(super::Requirement::SourceAssociatedBody(source));
                    return Ok(None);
                }
                if let Some(value) = Reader::new(self.evaluator.facts()).associated(selected) {
                    value
                } else {
                    return Ok(Some((
                        index.ok_or("missing proved source initializer body")?,
                        Vec::new(),
                        id,
                    )));
                }
            }
            Rvalue::AssociatedConstant(instance) => {
                if let Some(value) = Reader::new(self.evaluator.facts()).associated(instance) {
                    value
                } else if let Some(&index) = self
                    .bodies
                    .associated
                    .as_deref()
                    .and_then(|map| map.get(&(instance, ty)))
                {
                    return Ok(Some((index, Vec::new(), id)));
                } else {
                    self.waiting = Some(super::Requirement::AssociatedBody {
                        instance,
                        result: ty,
                    });
                    return Ok(None);
                }
            }
            Rvalue::Text(text) => {
                if matches!(self.evaluator.facts().types.kind(ty), Some(Kind::Reference { referent, .. }) if referent == TypeId::primitive(Primitive::Str))
                {
                    Constant::string(text)
                } else {
                    Constant::bytes(text)
                }
            }
            Rvalue::Read(place) => {
                let Some(address) = self.place(frame, place)? else {
                    return Ok(None);
                };
                let Some(value) = self.read(address)? else {
                    return Ok(None);
                };
                value
            }
            Rvalue::Address(place) => {
                let Some(place) = self.address_place(frame, place)? else {
                    return Ok(None);
                };
                if let Some(union) = place.union {
                    self.path(place.address)?;
                    let (value, work) = self.evaluator.union_address(
                        place.address,
                        union.owner,
                        union.field,
                        self.limits.steps.saturating_sub(self.work.steps),
                    )?;
                    self.charge(work)?;
                    value
                } else {
                    place.address
                }
            }
            Rvalue::Borrow { place, .. } => {
                let Some(place) = self.address_place(frame, place)? else {
                    return Ok(None);
                };
                let shared = matches!(
                    self.evaluator.facts().types.kind(ty),
                    Some(Kind::Reference { mutable: false, .. })
                );
                let value = if place.union.is_some() {
                    let Some(address) = self.union_reference(place, !shared)? else {
                        return Ok(None);
                    };
                    address
                } else if place.raw.is_some() {
                    let address = if shared {
                        self.shared_raw_reference(place)?
                    } else {
                        self.mutable_raw_reference(place)?
                    };
                    let Some(address) = address else {
                        return Ok(None);
                    };
                    address
                } else if shared {
                    place.address.restrict_shared()
                } else {
                    place.address
                };
                self.evaluator.adjust_reference(value, place.ty, ty)?
            }
            Rvalue::Unary {
                operator,
                operand,
                literal,
            } => self.evaluator.unary(
                operator,
                body.values[operand.0 as usize].ty,
                Self::value(frame, operand)?,
                literal,
                Overflow::Checked,
            )?,
            Rvalue::Binary {
                operator,
                left,
                right,
            } => self.evaluator.binary(
                operator,
                body.values[left.0 as usize].ty,
                Self::value(frame, left)?,
                Self::value(frame, right)?,
                Overflow::Checked,
            )?,
            Rvalue::Cast { kind, operand, .. } => {
                let Some(value) = self.cast(
                    kind,
                    body.values[operand.0 as usize].ty,
                    ty,
                    Self::value(frame, operand)?,
                )?
                else {
                    return Ok(None);
                };
                value
            }
            Rvalue::Call {
                callee, arguments, ..
            } => {
                let target = Self::value(frame, callee)?
                    .as_function()
                    .ok_or("constant call lacks a selected function item")?;
                let Some(Kind::FunctionItem { item, .. }) =
                    self.evaluator.facts().types.kind(target.ty())
                else {
                    return Err("constant callable lost its item identity".into());
                };
                let definition = crate::resolve::model::DefId(item.0);
                let arguments = self.arguments(frame, arguments)?;
                if let Some(variant) = self.evaluator.facts().members.constructor(definition) {
                    self.evaluator.fields(&arguments, Some(variant))?
                } else {
                    if !self.evaluator.facts().is_const_function(definition)
                        || !matches!(
                            self.evaluator
                                .facts()
                                .types
                                .kind(body.values[callee.0 as usize].ty),
                            Some(Kind::FunctionItem { .. })
                        )
                    {
                        return Err(
                            "non-const calls are not allowed during constant execution".into()
                        );
                    }
                    if let Some(catalog) = self.evaluator.facts().source_bindings.as_deref() {
                        catalog.check_const_call(definition)?;
                    }
                    if let Some(call) = super::reached_layout::validate(
                        self.evaluator.facts(),
                        &body,
                        frame,
                        frame
                            .next
                            .checked_sub(1)
                            .ok_or("source layout call has no current statement")?,
                        target,
                    )? {
                        let Some(bits) = self.bodies.layout_result(call.instance()) else {
                            self.waiting = Some(super::Requirement::FunctionBody(target));
                            return Ok(None);
                        };
                        self.work.calls += 1;
                        Constant::scalar(Scalar::Integer(u128::from(bits)))
                    } else {
                        let Some(index) = self
                            .bodies
                            .functions
                            .get(&target)
                            .or_else(|| self.bodies.definitions.get(&definition))
                            .copied()
                        else {
                            self.waiting = Some(super::Requirement::FunctionBody(target));
                            return Ok(None);
                        };
                        return Ok(Some((index, arguments, id)));
                    }
                }
            }
            Rvalue::Invoke { .. } => {
                return Err("call operators are not allowed during constant execution".into());
            }
            Rvalue::Unsize { value, conversion } => {
                let value = Self::value(frame, value)?;
                match conversion {
                    crate::types::unsizing::Conversion::Slice(array) => {
                        let Some(Kind::Array { length, .. }) =
                            self.evaluator.facts().types.kind(array)
                        else {
                            return Err("constant unsizing lacks concrete array length".into());
                        };
                        value.unsize(length)?
                    }
                    crate::types::unsizing::Conversion::Object { concrete, target } => {
                        value.with_metadata(Metadata::Object { concrete, target })?
                    }
                    crate::types::unsizing::Conversion::Upcast { target, .. } => {
                        value.upcast(target)?
                    }
                }
            }
            Rvalue::Validate(place) => {
                let Some(address) = self.place(frame, place)? else {
                    return Ok(None);
                };
                let Some(_) = self.read(address)? else {
                    return Ok(None);
                };
                self.evaluator.fields(&[], None)?
            }
            Rvalue::IsVariant { place, variant } => {
                let Some(address) = self.place(frame, place)? else {
                    return Ok(None);
                };
                let Some(value) = self.read(address)? else {
                    return Ok(None);
                };
                let actual = match Reader::new(self.evaluator.facts()).view(value)? {
                    View::Product { variant, .. } => variant,
                    _ => None,
                };
                Constant::scalar(Scalar::Boolean(actual == Some(variant)))
            }
            Rvalue::Length(place) => {
                let Some(address) = self.place(frame, place)? else {
                    return Ok(None);
                };
                let length = self.sequence_length(address)?;
                Constant::scalar(Scalar::Integer(u128::from(length)))
            }
            Rvalue::TextEquals { .. } => {
                return Err("constant text pattern execution is not implemented".into());
            }
        };
        frame.values[id.0 as usize] = Some(result);
        frame.temporaries.remove(&id);
        Ok(None)
    }
}
