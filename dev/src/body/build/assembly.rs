//! A real asm effect retains inputs and ordered output stores without native substitution.
use super::{Builder, Error, Kind, List, NodeId, Rvalue, ValueId, ValueList, Work};
use crate::{
    body::assembly::{Instruction, Operand, Payload, Piece, Template},
    syntax::assembly::{self, Direction},
    types::TypeId,
};
impl Builder<'_> {
    pub(super) fn assembly_read_place(&mut self, node: NodeId) -> Result<(), Error> {
        let place = self.pop_place(node)?;
        self.scratch.places.push(place);
        if let Some(place) = place
            && self.current.is_some()
        {
            let value = self.evaluate(node, self.ty(node)?, Rvalue::Read(place))?;
            self.save(node, value)?;
        }
        Ok(())
    }

    pub(super) fn assembly_children(&mut self, args: List) {
        let unit = self.report.package.unit(self.source.file);
        for &arg in unit.configured.ast.list(args).iter().rev() {
            if let Kind::AssemblyOperand { input, output, .. } = unit.configured.ast.node(arg).kind
            {
                if input.is_some() && input == output {
                    if let Some(place) = input {
                        self.scratch.work.push(Work::AssemblyReadPlace(place));
                        self.scratch.work.push(Work::Place(place));
                    }
                } else {
                    if let Some(output) = output {
                        self.scratch.work.push(Work::Place(output));
                    }
                    if let Some(input) = input {
                        self.scratch.work.push(Work::Enter(input));
                    }
                }
            }
        }
    }
    pub(super) fn assembly_value(&mut self, node: NodeId, args: List) -> Result<ValueId, Error> {
        if self.body.const_context {
            return Err(self.error(
                node,
                "inline assembly is not permitted in constant contexts",
            ));
        }
        let operands = self.assembly_preflight(node, args)?;
        let collected = self.assembly_operands(node, args, operands)?;
        let start = self.index(node, self.body.arguments.len())?;
        self.body
            .arguments
            .extend(collected.iter().filter_map(|o| o.input));
        let arguments = ValueList {
            start,
            end: self.index(node, self.body.arguments.len())?,
        };
        let mut data = self.body.assembly.take().unwrap_or_default();
        let source = self
            .body
            .source
            .source()
            .ok_or_else(|| self.error(node, "assembly body has no actual source owner"))?;
        if data.source.is_some_and(|actual| actual != source) {
            return Err(self.error(node, "assembly payload belongs to another source body"));
        }
        data.source = Some(source);
        let instruction = self.index(node, data.instructions.len())?;
        let operand_start = self.index(node, data.operands.len())?;
        data.operands.extend(collected.iter().copied());
        let operand_range = List {
            start: operand_start,
            end: self.index(node, data.operands.len())?,
        };
        let (templates, options) = self.assembly_templates(node, args, &mut data)?;
        data.instructions.push(Instruction {
            source: crate::resolve::unit::NodeKey {
                node,
                ..self.source
            },
            templates,
            operands: operand_range,
            options,
        });
        self.body.assembly = Some(data);
        let effect = self.evaluate(
            node,
            TypeId::UNIT,
            Rvalue::Assembly {
                instruction,
                arguments,
            },
        )?;
        if options & assembly::NORETURN != 0 {
            self.terminate(node, super::Terminator::Unreachable);
            return Ok(effect);
        }
        for (operand, output) in collected.iter().enumerate() {
            if let Some(place) = output.output {
                let value = self.evaluate(
                    node,
                    output
                        .ty
                        .ok_or_else(|| self.error(node, "assembly output has no actual type"))?,
                    Rvalue::AssemblyOutput {
                        effect,
                        operand: u32::try_from(operand)
                            .map_err(|_| self.error(node, "too many assembly outputs"))?,
                    },
                )?;
                self.store(node, place, value)?;
            }
        }
        Ok(effect)
    }
    fn assembly_preflight(&self, node: NodeId, args: List) -> Result<usize, Error> {
        let unit = self.report.package.unit(self.source.file);
        let rows = unit.configured.ast.list(args);
        let mut templates = 0usize;
        let mut operands = 0usize;
        let mut pieces = 0usize;
        let mut bytes = 0usize;
        for &arg in rows {
            match unit.configured.ast.node(arg).kind {
                Kind::AssemblyTemplate {
                    value,
                    pieces: range,
                } => {
                    templates += 1;
                    pieces = pieces.saturating_add((range.end - range.start) as usize);
                    let text = unit
                        .assembly_string(value)
                        .map_err(|e| self.error(node, e.message))?;
                    bytes = bytes.saturating_add(text.len());
                }
                Kind::AssemblyOperand { .. } => operands += 1,
                _ => {}
            }
        }
        let empty = Payload::default();
        let data = self.body.assembly.as_deref().unwrap_or(&empty);
        let bound = data
            .growth_bound(1, templates, pieces, operands, bytes)
            .saturating_add(if self.body.assembly.is_none() {
                size_of::<Payload>()
            } else {
                0
            })
            .saturating_add(operands.saturating_mul(size_of::<Operand>() * 4 + 128));
        self.report.prepare_assembly(
            crate::resolve::unit::NodeKey {
                node,
                ..self.source
            },
            rows.len().saturating_add(bytes).saturating_add(pieces),
            bound,
        )?;
        Ok(operands)
    }
    fn assembly_operands(
        &mut self,
        node: NodeId,
        args: List,
        operands: usize,
    ) -> Result<Vec<Operand>, Error> {
        let unit = self.report.package.unit(self.source.file);
        let rows = unit.configured.ast.list(args);
        let mut collected = Vec::with_capacity(operands);
        for &arg in rows.iter().rev() {
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
            let out = if let Some(output) = output {
                Some(self.pop_place(output)?.ok_or_else(|| {
                    self.error(output, "assembly output place was not constructed")
                })?)
            } else {
                None
            };
            let input_value = input.map(|input| self.value(input)).transpose()?;
            let ty = if let Some(input) = input {
                Some(self.ty(input)?)
            } else if let Some(output) = output {
                Some(self.ty(output)?)
            } else {
                None
            };
            let register = register.ok_or_else(|| {
                self.error(
                    node,
                    "assembly constant and symbolic operands have no retained runtime contract",
                )
            })?;
            let register = unit
                .assembly_register(register)
                .map_err(|e| self.error(node, e.message))?;
            collected.push(Operand {
                register,
                input: input_value,
                output: out,
                ty,
                late: matches!(direction, Direction::LateOut | Direction::InLateOut),
            });
        }
        collected.reverse();
        Ok(collected)
    }
    fn assembly_templates(
        &self,
        node: NodeId,
        args: List,
        data: &mut Payload,
    ) -> Result<(List, u16), Error> {
        let unit = self.report.package.unit(self.source.file);
        let rows = unit.configured.ast.list(args);
        let template_start = self.index(node, data.templates.len())?;
        let mut options = 0;
        for &arg in rows {
            match unit.configured.ast.node(arg).kind {
                Kind::AssemblyOptions(bits) => options |= bits,
                Kind::AssemblyTemplate { value, pieces } => {
                    let text = unit
                        .assembly_string(value)
                        .map_err(|e| self.error(node, e.message))?;
                    let start = self.index(node, data.pieces.len())?;
                    for &piece in unit.configured.ast.list(pieces) {
                        let Kind::AssemblyPlaceholder {
                            start,
                            end,
                            operand,
                            modifier,
                        } = unit.configured.ast.node(piece).kind
                        else {
                            return Err(
                                self.error(node, "assembly placeholder changed its checked kind")
                            );
                        };
                        data.pieces.push(Piece {
                            start,
                            end,
                            operand,
                            modifier,
                        });
                    }
                    data.templates.push(Template {
                        text: text.to_owned(),
                        pieces: List {
                            start,
                            end: self.index(node, data.pieces.len())?,
                        },
                    });
                }
                _ => {}
            }
        }
        Ok((
            List {
                start: template_start,
                end: self.index(node, data.templates.len())?,
            },
            options,
        ))
    }
}
