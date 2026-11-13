//! Assembly operands share ordinary inference, place checks and the current unsafe context.
use super::{Checker, Error, Kind, NodeId, TermId, Type, TypeId};
use crate::{
    sema::target_features::FeatureSet,
    syntax::assembly::{self, Direction, Register},
    types::{Primitive, aggregate::SimdShape},
};
const INCOMPATIBLE: &str = "assembly operand type is incompatible with register class";
impl Checker<'_> {
    pub(super) fn enter_assembly(
        &mut self,
        node: NodeId,
        args: crate::syntax::ast::List,
    ) -> Result<(), Error> {
        self.require_unsafe(node, "inline assembly requires an unsafe context")?;
        self.retain_assembly_context(node)?;
        if !self.state.in_function
            || self
                .state
                .environment_owner
                .is_some_and(|id| self.facts.is_const_function(id))
        {
            return Err(self.error(node, crate::support::initial_assembly::CONSTANT));
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
        let available = self.assembly_features(node)?;
        let mut explicit: Vec<(u8, bool, bool, bool, bool)> = Vec::with_capacity(count);
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
            if let (Some(input), Some(output)) = (input, output)
                && self.ty(input)? != self.ty(output)?
            {
                return Err(self.error(
                    arg,
                    "assembly tied input and output must have the same type",
                ));
            }
            if matches!(direction, Direction::Const | Direction::Sym) {
                return Err(self.error(arg,"assembly const and sym operands require their retained constant and relocation contract"));
            }
            let register =
                register.ok_or_else(|| self.error(arg, "assembly operand has no register"))?;
            let class = unit
                .assembly_register(register)
                .map_err(|e| self.error(arg, e.message))?;
            if matches!(class, Register::Xmm | Register::Explicit { xmm: true, .. })
                && !available.supports_xmm()
            {
                return Err(self.error(
                    arg,
                    "assembly register class requires the sse target feature",
                ));
            }
            if class == Register::Ymm && !available.supports_ymm() {
                return Err(self.error(
                    arg,
                    "assembly register class requires the avx target feature",
                ));
            }
            if matches!(class, Register::Zmm | Register::Kreg) && !available.supports_zmm() {
                return Err(self.error(
                    arg,
                    "assembly register class requires the avx512f target feature",
                ));
            }
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
                self.assembly_type(operand, class, available)?;
            }
            classes.push(class);
        }
        self.assembly_modifiers(args, &classes)
    }
    pub(super) fn assembly_features(&mut self, node: NodeId) -> Result<FeatureSet, Error> {
        let key = self.key(node);
        let baseline = FeatureSet::target_baseline(
            |growth| self.report.prepare_assembly(key, growth.work, growth.bytes),
            |message| self.error(node, message),
        )?;
        Ok(baseline.union(self.target_body_features(node)?))
    }
    fn assembly_modifiers(
        &self,
        args: crate::syntax::ast::List,
        classes: &[Register],
    ) -> Result<(), Error> {
        let unit = self.report.package.unit(self.state.file);
        for &template in unit.configured.ast.list(args) {
            let Kind::AssemblyTemplate { pieces, .. } = unit.configured.ast.node(template).kind
            else {
                continue;
            };
            for &piece in unit.configured.ast.list(pieces) {
                let Kind::AssemblyPlaceholder {
                    operand, modifier, ..
                } = unit.configured.ast.node(piece).kind
                else {
                    continue;
                };
                let class = classes
                    .get(operand as usize)
                    .ok_or_else(|| self.error(template, "assembly placeholder lost its operand"))?;
                if modifier != 0 {
                    let valid = match class {
                        Register::General | Register::Explicit { xmm: false, .. } => {
                            b"lxer".contains(&modifier)
                        }
                        Register::Abcd => b"lhxer".contains(&modifier),
                        Register::Byte | Register::Kreg => false,
                        Register::Xmm
                        | Register::Ymm
                        | Register::Zmm
                        | Register::Explicit { xmm: true, .. } => b"xyz".contains(&modifier),
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
        Ok(())
    }
    fn assembly_type(
        &self,
        node: NodeId,
        class: Register,
        available: FeatureSet,
    ) -> Result<(), Error> {
        let ty = self.ty(node)?;
        let (bytes, float, vector) = match self.facts.types.kind(ty) {
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
                (bytes, p.is_float(), false)
            }
            Some(Type::Function { .. }) => (8, false, false),
            Some(Type::Pointer { pointee, .. })
                if self.facts.types.is_sized(pointee) == Some(true) =>
            {
                (8, false, false)
            }
            Some(Type::Adt(..)) => {
                match class {
                    Register::Ymm => ymm_operand(&*self.facts, ty)
                        .map_err(|message| self.error(node, message))?,
                    Register::Zmm => zmm_operand(&*self.facts, ty)
                        .map_err(|message| self.error(node, message))?,
                    Register::Kreg => return Err(self.error(node, INCOMPATIBLE)),
                    _ => self.assembly_simd_type(node, ty)?,
                }
                (16, false, true)
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
            }
            | Register::Byte => !float && bytes == 1,
            Register::General | Register::Abcd | Register::Explicit { xmm: false, .. } => {
                bytes >= 2 && !vector
            }
            Register::Xmm
            | Register::Ymm
            | Register::Zmm
            | Register::Explicit { xmm: true, .. } => matches!(bytes, 4 | 8) || vector,
            Register::Kreg if float || vector => false,
            Register::Kreg if matches!(bytes, 4 | 8) && !available.supports_wide_masks() => {
                return Err(self.error(
                    node,
                    "assembly kreg operand of 32 or 64 bits requires the avx512bw target feature",
                ));
            }
            Register::Kreg => true,
        };
        if valid {
            Ok(())
        } else {
            Err(self.error(node, INCOMPATIBLE))
        }
    }
    fn assembly_simd_type(&self, node: NodeId, ty: TypeId) -> Result<(), Error> {
        simd_operand(&*self.facts, ty).map_err(|message| self.error(node, message))
    }
}
/// An `xmm_reg` vector: exactly 128 bits of supported numeric lanes. The
/// local checker and every imported body's re-proof share this one check.
pub(crate) fn simd_operand(
    facts: &crate::sema::model::Declarations,
    ty: TypeId,
) -> Result<(), &'static str> {
    if simd_bits(facts, ty)? == Some(128) {
        Ok(())
    } else {
        Err("assembly xmm vector requires exactly 128 bits of supported numeric lanes")
    }
}
/// A `ymm_reg` vector: 128 or 256 bits of supported numeric lanes. No
/// imported body carries one, since the initial assembly profile rejects it.
fn ymm_operand(facts: &crate::sema::model::Declarations, ty: TypeId) -> Result<(), &'static str> {
    if matches!(simd_bits(facts, ty)?, Some(128 | 256)) {
        Ok(())
    } else {
        Err("assembly ymm vector requires 128 or 256 bits of supported numeric lanes")
    }
}
/// A `zmm_reg` vector: 128, 256 or 512 bits of supported numeric lanes. No
/// imported body carries one, since the initial assembly profile rejects it.
fn zmm_operand(facts: &crate::sema::model::Declarations, ty: TypeId) -> Result<(), &'static str> {
    if matches!(simd_bits(facts, ty)?, Some(128 | 256 | 512)) {
        Ok(())
    } else {
        Err("assembly zmm vector requires 128, 256 or 512 bits of supported numeric lanes")
    }
}
/// The width of a completed genuine vector, or `None` for unsupported lanes.
/// `f16` lanes count, as rustc's `VecF16` types do.
fn simd_bits(
    facts: &crate::sema::model::Declarations,
    ty: TypeId,
) -> Result<Option<u64>, &'static str> {
    let shape = facts.aggregates.simd_shape(&facts.types, ty)?;
    let SimdShape::Ready { lane, count } = shape else {
        return Err("assembly SIMD operand requires its completed genuine representation");
    };
    let bits = match facts.types.kind(lane) {
        Some(Type::Primitive(p)) => match p {
            Primitive::F16 => Some(16),
            Primitive::F32 => Some(32),
            Primitive::F64 => Some(64),
            _ => p
                .integer()
                .map(|(_, bits)| bits)
                .filter(|bits| matches!(bits, 8 | 16 | 32 | 64)),
        },
        _ => None,
    };
    Ok(bits.and_then(|bits| count.checked_mul(u64::from(bits))))
}
