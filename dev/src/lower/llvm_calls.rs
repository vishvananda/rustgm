//! Reached LLVM calls use validated typed shims, never an unresolved native symbol.
use super::{
    bit_kernels,
    context::{Context, Representation},
};
use crate::{
    lowir::{
        asm,
        builder::Builder,
        program::{SymbolId, Terminator},
    },
    sema::foreign::Function,
    support::llvm_symbols::{Intrinsic, Operation},
    types::{Kind, TypeId, callable::Convention, instance::Instance},
};
impl Context<'_> {
    pub(super) fn demand_llvm(
        &mut self,
        instance: Instance,
        function: Function,
        intrinsic: Intrinsic,
    ) -> Result<SymbolId, &'static str> {
        if let Some(&symbol) = self.intrinsic_symbols.get(&instance) {
            return Ok(symbol);
        }
        if intrinsic.operation == Operation::Unsupported {
            return Err("selected LLVM intrinsic operation is not implemented");
        }
        if intrinsic.operation == Operation::Pause {
            return self.demand_pause(instance, function);
        }
        let Some(Kind::Function {
            parameters,
            result,
            convention,
            ..
        }) = self.types.kind(function.signature)
        else {
            return Err("LLVM intrinsic has no selected function signature");
        };
        let parameters = self
            .types
            .list(parameters)
            .ok_or("missing LLVM intrinsic parameter types")?;
        if convention.variadic || parameters.len() != 1 {
            return Err("LLVM scalar unary intrinsic signature mismatch");
        }
        let input = self.types.storage_head(parameters[0])?;
        let result = self.types.storage_head(result)?;
        let width = |ty| match self.types.kind(ty) {
            Some(Kind::Primitive(primitive)) => primitive.integer().map(|(_, bits)| bits),
            _ => None,
        };
        if width(input) != Some(intrinsic.bits) || width(result) != Some(intrinsic.bits) {
            return Err("LLVM unary intrinsic width differs from its selected signature");
        }
        let Representation::Scalar(input) = self.representation(input)? else {
            return Err("LLVM unary intrinsic lacks a scalar input representation");
        };
        let Representation::Scalar(result) = self.representation(result)? else {
            return Err("LLVM unary intrinsic lacks a scalar result representation");
        };
        self.reserve_shim()?;
        let signature = self.signature(function.signature)?;
        let body = bit_kernels::build(intrinsic.operation, input, result, intrinsic.bits)?;
        let symbol = self.output.declare(
            self.internal_name(format!("rg.llvm.unary.t{}", instance.ty().0)),
            signature,
            false,
        )?;
        self.output.define(symbol, body)?;
        self.intrinsic_symbols.insert(instance, symbol);
        Ok(symbol)
    }
    /// The spin-wait instruction has no operands or architectural state clobbers.
    fn demand_pause(
        &mut self,
        instance: Instance,
        function: Function,
    ) -> Result<SymbolId, &'static str> {
        let hint = matches!(
            self.types.kind(function.signature),
            Some(Kind::Function {
                bound_regions: 0,
                parameters,
                result: TypeId::UNIT,
                convention: Convention::C | Convention::UNADJUSTED,
                ..
            }) if self.types.list(parameters) == Some(&[])
        );
        if !hint {
            return Err(
                "LLVM x86 pause requires a fixed foreign signature with no inputs or result",
            );
        }
        self.reserve_shim()?;
        let mut builder = Builder::default();
        let entry = builder.block()?;
        builder.enter(entry)?;
        let mut assembly = self.output.assembly()?;
        assembly.text("pause")?;
        let assembly = assembly.finish(asm::SIDE_EFFECT, 0, 0)?;
        builder.asm(assembly, &[])?;
        builder.terminate(Terminator::Return(None))?;
        let signature = self.signature(function.signature)?;
        let symbol = self.output.declare(
            self.internal_name(format!("rg.llvm.pause.t{}", instance.ty().0)),
            signature,
            false,
        )?;
        self.output.define(symbol, builder.finish()?)?;
        self.intrinsic_symbols.insert(instance, symbol);
        Ok(symbol)
    }
}
