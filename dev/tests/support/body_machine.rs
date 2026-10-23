//! Test-only integer CFG executor. It checks dynamic storage state and never implements the frontend.
use rustgm::{
    body::ir::{Action, Body, LocalKind, Owner, Place, Program, Rvalue, Terminator, ValueId},
    lex::token::Punctuation as Op,
    resolve::model::DefId,
    sema::model::Scalar,
    syntax::ast::Unary,
    types::{Kind, Primitive, Store, TypeId},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Data {
    Bits(u128),
    Function(DefId),
    Unit,
}
impl Data {
    pub fn bits(self) -> Result<u128, String> {
        if let Self::Bits(bits) = self {
            Ok(bits)
        } else {
            Err(format!("expected integer, got {self:?}"))
        }
    }
}
pub struct Machine<'a> {
    pub program: &'a Program,
    pub types: &'a Store,
    pub fuel: usize,
}
fn declaration(owner: Owner) -> Option<DefId> {
    match owner {
        Owner::Function { definition, .. } | Owner::Initializer { definition, .. } => {
            Some(definition)
        }
        Owner::AnonymousConstant
        | Owner::AnonymousInstance(_)
        | Owner::InitializerInstance { .. }
        | Owner::Closure { .. }
        | Owner::ClosureAdapter { .. } => None,
    }
}
impl Machine<'_> {
    pub fn run(
        &mut self,
        definition: DefId,
        arguments: &[Data],
        depth: usize,
    ) -> Result<Data, String> {
        if depth > 128 {
            return Err("test recursion exhausted".into());
        }
        let program = self.program;
        let body = program
            .bodies
            .iter()
            .find(|body| declaration(body.owner) == Some(definition))
            .ok_or("missing test function")?;
        let mut frame = Frame {
            body,
            values: vec![None; body.values.len()],
            locals: vec![None; body.locals.len()],
            live: vec![false; body.locals.len()],
        };
        let mut block = 0;
        loop {
            let block_data = &body.blocks[block];
            for statement in body.statements(block_data) {
                self.fuel = self.fuel.checked_sub(1).ok_or("test execution exhausted")?;
                match statement.action {
                    Action::CheckInitialized(_) => {
                        return Err(
                            "closure initialization checks require the ordinary proof pipeline"
                                .into(),
                        );
                    }
                    Action::Mention(_) => {}
                    Action::StorageLive(local) => {
                        let local = local.0 as usize;
                        if frame.live[local] {
                            return Err(format!("local {local} is already live"));
                        }
                        frame.live[local] = true;
                        frame.locals[local] =
                            if let LocalKind::Parameter(index) = body.locals[local].kind {
                                Some(*arguments.get(index as usize).ok_or("missing argument")?)
                            } else {
                                None
                            };
                    }
                    Action::StorageDead(local) => frame.end_storage(local.0 as usize)?,
                    Action::StorageEnd { first, last } => {
                        for local in body.storage_range(first, last) {
                            self.fuel =
                                self.fuel.checked_sub(1).ok_or("test execution exhausted")?;
                            frame.end_storage(local?.0 as usize)?;
                        }
                    }
                    Action::Store {
                        place: Place::Local(local),
                        value,
                    } => {
                        let local = local.0 as usize;
                        if !frame.live[local] {
                            return Err("store outside storage lifetime".into());
                        }
                        frame.locals[local] = Some(frame.value(value)?);
                    }
                    Action::Store { .. } => {
                        return Err("test executor has no mutable globals".into());
                    }
                    Action::GuardStart(_) | Action::GuardUse(_) | Action::BindView { .. } => {
                        return Err("guard views require the native executor".into());
                    }
                    Action::Project(_) | Action::Select { .. } => {
                        return Err(
                            "scalar test executor does not implement aggregate places".into()
                        );
                    }
                    Action::Evaluate(value) => {
                        frame.values[value.0 as usize] = Some(self.evaluate(&frame, value, depth)?);
                    }
                }
            }
            self.fuel = self.fuel.checked_sub(1).ok_or("test execution exhausted")?;
            block = match block_data.terminator {
                Terminator::Jump(target) | Terminator::FalseEdge { real: target, .. } => {
                    target.0 as usize
                }
                Terminator::Branch { condition, yes, no } => {
                    if frame.value(condition)?.bits()? != 0 {
                        yes.0 as usize
                    } else {
                        no.0 as usize
                    }
                }
                Terminator::Return(value) => {
                    if frame.live.iter().any(|live| *live) {
                        return Err("return with live storage".into());
                    }
                    return frame.value(value);
                }
                Terminator::Unreachable => return Err("executed unreachable terminator".into()),
                Terminator::Panic => return Err("executed panic terminator".into()),
            };
        }
    }
    #[expect(
        clippy::too_many_lines,
        reason = "One typed value dispatch keeps scalar execution and its unsupported operations together"
    )]
    fn evaluate(
        &mut self,
        frame: &Frame<'_>,
        value: ValueId,
        depth: usize,
    ) -> Result<Data, String> {
        let value = &frame.body.values[value.0 as usize];
        let data = match value.kind {
            Rvalue::Borrow { .. }
            | Rvalue::Address(_)
            | Rvalue::Unsize { .. }
            | Rvalue::AssociatedConstant(_)
            | Rvalue::Text(_)
            | Rvalue::Aggregate { .. }
            | Rvalue::Union { .. }
            | Rvalue::Intrinsic { .. }
            | Rvalue::Length(_)
            | Rvalue::TextEquals { .. }
            | Rvalue::IsVariant { .. }
            | Rvalue::Validate(_)
            | Rvalue::Repeat { .. }
            | Rvalue::Read(Place::Temporary(_) | Place::Projection(_)) => {
                return Err(
                    "scalar test executor does not implement aggregates or associated values"
                        .into(),
                );
            }
            Rvalue::Unit => Data::Unit,
            Rvalue::Scalar(id) => match frame.body.scalars[id.0 as usize] {
                Scalar::Integer(bits) => Data::Bits(bits),
                Scalar::Boolean(value) => Data::Bits(u128::from(value)),
                Scalar::Character(value) => Data::Bits(u128::from(u32::from(value))),
                Scalar::Float(_) => return Err("test executor has no floating arithmetic".into()),
            },
            Rvalue::Function(definition) => Data::Function(definition),
            Rvalue::ConstArgument(id) => Data::Bits(
                self.types
                    .constant(id)
                    .ok_or("test executor requires an evaluated const argument")?
                    .bits,
            ),
            Rvalue::Read(Place::Local(local)) => {
                let local = local.0 as usize;
                if !frame.live[local] {
                    return Err("read outside storage lifetime".into());
                }
                frame.locals[local].ok_or("read of uninitialized local")?
            }
            Rvalue::Read(Place::Static(definition)) | Rvalue::Constant(definition) => {
                self.run(definition, &[], depth + 1)?
            }
            Rvalue::Reify(operand) | Rvalue::Constrain(operand) | Rvalue::Cast { operand, .. } => {
                frame.value(operand)?
            }
            Rvalue::Unary {
                operator, operand, ..
            } => {
                let bits = frame.value(operand)?.bits()?;
                Data::Bits(match operator {
                    Unary::Negate => bits.wrapping_neg(),
                    Unary::Not if value.ty == TypeId::primitive(Primitive::Bool) => {
                        u128::from(bits == 0)
                    }
                    Unary::Not => !bits,
                    _ => return Err("unexpected test unary operator".into()),
                })
            }
            Rvalue::Binary {
                operator,
                left,
                right,
                ..
            } => {
                let left_type = frame.body.values[left.0 as usize].ty;
                let signed = self
                    .types
                    .kind(left_type)
                    .and_then(|t| {
                        if let Kind::Primitive(p) = t {
                            p.integer()
                        } else {
                            None
                        }
                    })
                    .is_some_and(|(signed, _)| signed);
                let mask = mask(self.types, left_type);
                Data::Bits(binary(
                    operator,
                    frame.value(left)?.bits()?,
                    frame.value(right)?.bits()?,
                    signed,
                    mask,
                )?)
            }
            Rvalue::Invoke { .. } => {
                return Err("call operators are outside the PA10 scalar executor".into());
            }
            Rvalue::SourceAssociated(_) => {
                return Err(
                    "source-associated values require the checked constant provider".into(),
                );
            }
            Rvalue::InlineConstant(_) | Rvalue::InlineInstance(_) => {
                return Err("inline constants are outside the PA10 scalar executor".into());
            }
            Rvalue::Call {
                callee, arguments, ..
            } => {
                let Data::Function(definition) = frame.value(callee)? else {
                    return Err("call has no function identity".into());
                };
                let arguments: Result<Vec<_>, _> = frame
                    .body
                    .arguments(arguments)
                    .iter()
                    .map(|id| frame.value(*id))
                    .collect();
                self.run(definition, &arguments?, depth + 1)?
            }
        };
        Ok(if let Data::Bits(bits) = data {
            Data::Bits(bits & mask(self.types, value.ty))
        } else {
            data
        })
    }
}
struct Frame<'a> {
    body: &'a Body,
    values: Vec<Option<Data>>,
    locals: Vec<Option<Data>>,
    live: Vec<bool>,
}
impl Frame<'_> {
    fn end_storage(&mut self, local: usize) -> Result<(), String> {
        if !self.live[local] {
            return Err(format!("local {local} was not live"));
        }
        self.live[local] = false;
        self.locals[local] = None;
        Ok(())
    }

    fn value(&self, id: ValueId) -> Result<Data, String> {
        self.values[id.0 as usize].ok_or_else(|| format!("v{} was not evaluated", id.0))
    }
}
fn mask(types: &Store, ty: TypeId) -> u128 {
    let Some(Kind::Primitive(p)) = types.kind(ty) else {
        return u128::MAX;
    };
    let width = p.integer().map_or(128, |(_, width)| width);
    if width == 128 {
        u128::MAX
    } else {
        (1_u128 << width) - 1
    }
}
fn binary(op: Op, a: u128, b: u128, signed: bool, mask: u128) -> Result<u128, String> {
    let negative = |n: u128| signed && n & (mask ^ (mask >> 1)) != 0;
    let signed_value =
        |n: u128| i128::from_le_bytes((if negative(n) { n | !mask } else { n }).to_le_bytes());
    Ok(match op {
        Op::Plus => a.wrapping_add(b),
        Op::Minus => a.wrapping_sub(b),
        Op::Star => a.wrapping_mul(b),
        Op::Slash if !signed => a.checked_div(b).ok_or("division by zero")?,
        Op::Percent if !signed => a.checked_rem(b).ok_or("remainder by zero")?,
        Op::Slash | Op::Percent => {
            let n = if op == Op::Slash {
                signed_value(a).checked_div(signed_value(b))
            } else {
                signed_value(a).checked_rem(signed_value(b))
            };
            u128::from_le_bytes(n.ok_or("signed division failure")?.to_le_bytes())
        }
        Op::And => a & b,
        Op::Or => a | b,
        Op::Caret => a ^ b,
        Op::ShiftLeft => a
            .checked_shl(u32::try_from(b).map_err(|e| e.to_string())?)
            .ok_or("invalid shift")?,
        Op::ShiftRight if !signed => a
            .checked_shr(u32::try_from(b).map_err(|e| e.to_string())?)
            .ok_or("invalid shift")?,
        Op::ShiftRight => u128::from_le_bytes(
            signed_value(a)
                .checked_shr(u32::try_from(b).map_err(|e| e.to_string())?)
                .ok_or("invalid shift")?
                .to_le_bytes(),
        ),
        Op::EqualEqual => u128::from(a == b),
        Op::BangEqual => u128::from(a != b),
        Op::Less | Op::LessEqual | Op::Greater | Op::GreaterEqual => {
            let ordering = if signed {
                signed_value(a).cmp(&signed_value(b))
            } else {
                a.cmp(&b)
            };
            u128::from(match op {
                Op::Less => ordering.is_lt(),
                Op::LessEqual => ordering.is_le(),
                Op::Greater => ordering.is_gt(),
                _ => ordering.is_ge(),
            })
        }
        _ => return Err(format!("unexpected test binary operator {op:?}")),
    })
}
