//! Assembly operands share ordinary inference, place checks and the current unsafe context.
use super::{Checker, Error, Kind, NodeId, TermId, Type, TypeId};
use crate::{
    syntax::assembly::{self, Direction, Register},
    types::Primitive,
};
impl Checker<'_> {
    pub(super) fn enter_assembly(
        &mut self,
        node: NodeId,
        args: crate::syntax::ast::List,
    ) -> Result<(), Error> {
        self.require_unsafe(node, "inline assembly requires an unsafe context")?;
        if !self.state.in_function
            || self
                .state
                .environment_owner
                .is_some_and(|id| self.facts.is_const_function(id))
        {
            return Err(self.error(
                node,
                "inline assembly is not permitted in constant contexts",
            ));
        }
        let unit = self.report.package.unit(self.state.file);
        for &arg in unit.configured.ast.list(args) {
            if let Kind::AssemblyOperand { input, output, .. } = unit.configured.ast.node(arg).kind
            {
                if let Some(out) = output {
                    self.mark_union_write(out)?;
                    self.mark_mutable_language_place(out)?;
                }
                if let Some(input) = input {
                    self.state.children.push(input);
                }
                if output != input {
                    self.state.children.extend(output);
                }
            }
        }
        Ok(())
    }
    pub(super) fn assembly_result(
        &mut self,
        node: NodeId,
        args: crate::syntax::ast::List,
    ) -> Result<TermId, Error> {
        self.state.checks.push(node);
        let ast = &self.report.package.unit(self.state.file).configured.ast;
        let diverges=ast.list(args).iter().any(|&arg|matches!(ast.node(arg).kind,Kind::AssemblyOptions(bits)if bits&assembly::NORETURN!=0));
        self.known(
            node,
            if diverges {
                TypeId::NEVER
            } else {
                TypeId::UNIT
            },
        )
    }
    pub(super) fn validate_assembly(
        &mut self,
        node: NodeId,
        args: crate::syntax::ast::List,
    ) -> Result<(), Error> {
        let unit = self.report.package.unit(self.state.file);
        let count = unit.configured.ast.list(args).len();
        self.report
            .prepare_assembly(self.key(node), count, count.saturating_mul(256))?;
        let mut explicit = Vec::with_capacity(count);
        let mut classes = Vec::with_capacity(count);
        let unit = self.report.package.unit(self.state.file);
        for &arg in unit.configured.ast.list(args) {
            let Kind::AssemblyOperand {
                register,
                input,
                output,
                direction,
                ..
            } = unit.configured.ast.node(arg).kind
            else {
                continue;
            };
            if let Some(out) = output {
                self.assignment_target(node, out)?;
            }
            if let (Some(input), Some(output)) = (input, output) {
                if self.ty(input)? != self.ty(output)? {
                    return Err(self.error(
                        arg,
                        "assembly tied input and output must have the same type",
                    ));
                }
            }
            if matches!(direction, Direction::Const | Direction::Sym) {
                return Err(self.error(arg,"assembly const and sym operands require their retained constant and relocation contract"));
            }
            let register =
                register.ok_or_else(|| self.error(arg, "assembly operand has no register"))?;
            let Kind::AssemblyRegister {
                explicit: is_explicit,
                value,
            } = unit.configured.ast.node(register).kind
            else {
                return Err(self.error(arg, "assembly register changed its typed form"));
            };
            let spelling = if is_explicit {
                let Kind::Literal(index) = unit.configured.ast.node(value).kind else {
                    return Err(self.error(arg, "explicit assembly register is not a literal"));
                };
                let crate::literal::Value::String(bytes) = unit
                    .literal(index)
                    .map_err(|e| self.error(arg, e.message))?
                else {
                    return Err(self.error(arg, "assembly register must be a string"));
                };
                std::str::from_utf8(&bytes)
                    .map_err(|_| self.error(arg, "assembly register string is invalid"))?
                    .to_owned()
            } else {
                let span = unit.configured.ast.node(value).span;
                unit.source
                    .text()
                    .get(span.start as usize..span.end as usize)
                    .ok_or_else(|| self.error(arg, "assembly register span is invalid"))?
                    .to_owned()
            };
            let class =
                assembly::register(&spelling, is_explicit).map_err(|e| self.error(arg, e))?;
            if let Register::Explicit { base, xmm, .. } = class {
                let late = matches!(direction, Direction::LateOut | Direction::InLateOut);
                let reads = input.is_some();
                let writes = output.is_some()
                    || matches!(
                        direction,
                        Direction::Out
                            | Direction::LateOut
                            | Direction::InOut
                            | Direction::InLateOut
                    );
                if explicit.iter().any(|&(b, x, r, w, l)| {
                    b == base
                        && x == xmm
                        && ((reads && r)
                            || (writes && w)
                            || (reads && w && !l)
                            || (writes && r && !late))
                }) {
                    return Err(
                        self.error(arg, "assembly operands overlap the same explicit register")
                    );
                }
                explicit.push((base, xmm, reads, writes, late));
            }
            for operand in [input, output].into_iter().flatten() {
                self.assembly_type(operand, class)?;
            }
            classes.push(class);
        }
        for &template in unit.configured.ast.list(args) {
            if let Kind::AssemblyTemplate { pieces, .. } = unit.configured.ast.node(template).kind {
                for &piece in unit.configured.ast.list(pieces) {
                    if let Kind::AssemblyPlaceholder {
                        operand, modifier, ..
                    } = unit.configured.ast.node(piece).kind
                    {
                        let class = classes.get(operand as usize).ok_or_else(|| {
                            self.error(template, "assembly placeholder lost its operand")
                        })?;
                        if modifier != 0 {
                            let valid = match class {
                                Register::General | Register::Explicit { xmm: false, .. } => {
                                    b"lxer".contains(&modifier)
                                }
                                Register::Abcd => b"lhxer".contains(&modifier),
                                Register::Byte => false,
                                Register::Xmm | Register::Explicit { xmm: true, .. } => {
                                    modifier == b'x'
                                }
                            };
                            if !valid {
                                return Err(self.error(
                                    template,
                                    "assembly register modifier is not supported by its class",
                                ));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn assembly_type(&self, node: NodeId, class: Register) -> Result<(), Error> {
        let ty = self.ty(node)?;
        let (bytes, float) = match self.facts.types.kind(ty) {
            Some(Type::Primitive(p)) => {
                let bytes = match p {
                    Primitive::U8 | Primitive::I8 => 1,
                    Primitive::U16 | Primitive::I16 => 2,
                    Primitive::U32 | Primitive::I32 | Primitive::F32 => 4,
                    Primitive::U64
                    | Primitive::I64
                    | Primitive::Usize
                    | Primitive::Isize
                    | Primitive::F64 => 8,
                    _ => {
                        return Err(self.error(
                            node,
                            "type is not supported as an assembly register operand",
                        ));
                    }
                };
                (bytes, p.is_float())
            }
            Some(Type::Function { .. }) => (8, false),
            Some(Type::Pointer { pointee, .. })
                if self.facts.types.is_sized(pointee) == Some(true) =>
            {
                (8, false)
            }
            _ => return Err(self.error(
                node,
                "assembly SIMD and unresolved operands require their genuine type representation",
            )),
        };
        let valid = match class {
            Register::Explicit {
                xmm: false,
                bytes: 1,
                ..
            } => !float && bytes == 1,
            Register::General | Register::Abcd | Register::Explicit { xmm: false, .. } => {
                bytes >= 2
            }
            Register::Byte => !float && bytes == 1,
            Register::Xmm | Register::Explicit { xmm: true, .. } => matches!(bytes, 4 | 8),
        };
        if valid {
            Ok(())
        } else {
            Err(self.error(
                node,
                "assembly operand type is incompatible with register class",
            ))
        }
    }
}
