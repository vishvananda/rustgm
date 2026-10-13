//! The CFG text adapter observes typed records; the executable path never parses it.
use super::ir::{
    Action, Body, LocalKind, Owner, Place, Program, ProjectionKind, Rvalue, Terminator,
};
use crate::{
    sema::{model::InitializerKind, type_view},
    types::Store,
};
use std::io::{self, Write};

/// # Errors
/// Propagates output errors and inconsistent type identities.
pub fn write(program: &Program, types: &Store, out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "body-crate 1")?;
    type_view::types(types, out)?;
    type_view::regions(&program.region_equalities, out)?;
    type_view::body_regions(&program.body_regions, out)?;
    type_view::coercions(&program.coercions, out)?;
    type_view::outlives(&program.outlives, out)?;
    for (index, body) in program.bodies.iter().enumerate() {
        write!(out, "body {index} ")?;
        match body.owner {
            Owner::Function {
                definition,
                signature,
            } => write!(
                out,
                "function def={} signature={}",
                definition.0, signature.0
            )?,
            Owner::Initializer { definition, kind } => {
                let kind = match kind {
                    InitializerKind::Const => "const",
                    InitializerKind::Static { mutable: false } => "static",
                    InitializerKind::Static { mutable: true } => "static-mut",
                };
                write!(out, "{kind} def={}", definition.0)?;
            }
        }
        writeln!(
            out,
            " result={} source={}:{}",
            body.result.0, body.source.file.0, body.source.node.0
        )?;
        contents(body, out)?;
        writeln!(out, "end-body")?;
    }
    write!(out, "eof ")?;
    count(program, out)
}
/// # Errors
/// Propagates output errors.
pub fn count(program: &Program, out: &mut impl Write) -> io::Result<()> {
    let mut locals = 0;
    let mut values = 0;
    let mut statements = 0;
    let mut blocks = 0;
    for body in &program.bodies {
        locals += body.locals.len();
        values += body.values.len();
        statements += body.statements.len();
        blocks += body.blocks.len();
    }
    writeln!(
        out,
        "bodies={} locals={locals} values={values} statements={statements} blocks={blocks}",
        program.bodies.len()
    )
}
fn contents(body: &Body, out: &mut impl Write) -> io::Result<()> {
    for (id, local) in body.locals.iter().enumerate() {
        write!(out, "  local l{id} type={} ", local.ty.0)?;
        match local.kind {
            LocalKind::Parameter(index) => write!(out, "parameter={index}")?,
            LocalKind::Binding => write!(out, "binding")?,
            LocalKind::Merge => write!(out, "merge")?,
            LocalKind::Guard(id) => write!(out, "guard-view={}", id.0)?,
        }
        if let Some(definition) = local.definition {
            write!(out, " def={}", definition.0)?;
        }
        write!(out, " mutable={} storage-parent=", local.mutable)?;
        if let Some(parent) = local.storage_parent {
            write!(out, "l{}", parent.0)?;
        } else {
            write!(out, "-")?;
        }
        writeln!(out, " source={}", local.source.0)?;
    }
    for (id, projection) in body.projections.iter().enumerate() {
        write!(out, "  projection p{id} type={} base=", projection.ty.0)?;
        place(projection.base, out)?;
        match projection.kind {
            ProjectionKind::Dereference { region, mutable } => {
                write!(out, " dereference region={} mutable={mutable}", region.0)?;
            }
            ProjectionKind::Field(field) => write!(out, " field={field:?}")?,
            ProjectionKind::Index { index, length } => {
                write!(out, " index=v{} length={length}", index.0)?;
            }
            ProjectionKind::SliceIndex { index } => write!(out, " slice-index=v{}", index.0)?,
            ProjectionKind::FromEnd { offset, minimum } => {
                write!(out, " from-end={offset} minimum={minimum}")?;
            }
            ProjectionKind::Subslice { prefix, suffix } => {
                write!(out, " subslice={prefix}:{suffix}")?;
            }
            ProjectionKind::ConstantIndex(index) => write!(out, " constant-index={index}")?,
            ProjectionKind::Subarray { start, end } => write!(out, " subarray={start}..{end}")?,
        }
        writeln!(out)?;
    }
    for (id, guard) in body.guards.iter().enumerate() {
        writeln!(
            out,
            "  guard g{id} tested={}:{} views={}:{}",
            guard.tested_start, guard.tested_end, guard.views_start, guard.views_end
        )?;
    }
    for (id, protected) in body.guard_places.iter().enumerate() {
        write!(out, "  guard-place {id} shallow=")?;
        place(*protected, out)?;
        writeln!(out)?;
    }
    blocks(body, out)
}
fn blocks(body: &Body, out: &mut impl Write) -> io::Result<()> {
    for (id, block) in body.blocks.iter().enumerate() {
        writeln!(out, "  block b{id}:")?;
        for statement in body.statements(block) {
            write!(out, "    ")?;
            match statement.action {
                Action::GuardStart(id) => write!(out, "guard-start g{}", id.0)?,
                Action::GuardUse(id) => write!(out, "guard-use g{}", id.0)?,
                Action::BindView {
                    view,
                    place: source,
                } => {
                    write!(out, "bind-view l{} ", view.0)?;
                    place(source, out)?;
                }
                Action::Select { projection, anchor } => {
                    write!(out, "select p{} anchor=", projection.0)?;
                    place(anchor, out)?;
                }
                Action::Project(id) => write!(out, "project p{}", id.0)?,
                Action::StorageLive(local) => write!(out, "live l{}", local.0)?,
                Action::StorageDead(local) => write!(out, "dead l{}", local.0)?,
                Action::StorageEnd { first, last } => {
                    write!(out, "end-storage first=l{} last=l{}", first.0, last.0)?;
                }
                Action::Store {
                    place: target,
                    value,
                } => {
                    write!(out, "store ")?;
                    place(target, out)?;
                    write!(out, " v{}", value.0)?;
                }
                Action::Evaluate(id) => {
                    let value = &body.values[id.0 as usize];
                    write!(out, "v{} type={} = ", id.0, value.ty.0)?;
                    rvalue(body, value.kind, out)?;
                }
            }
            writeln!(out, " source={}", statement.source.0)?;
        }
        write!(out, "    ")?;
        match block.terminator {
            Terminator::Jump(target) => write!(out, "jump b{}", target.0)?,
            Terminator::FalseEdge { real, imaginary } => write!(
                out,
                "false-edge real=b{} imaginary=b{}",
                real.0, imaginary.0
            )?,
            Terminator::Branch { condition, yes, no } => {
                write!(out, "branch v{} b{} b{}", condition.0, yes.0, no.0)?;
            }
            Terminator::Return(value) => write!(out, "return v{}", value.0)?,
            Terminator::Unreachable => write!(out, "unreachable")?,
        }
        writeln!(out, " source={}", block.source.0)?;
    }
    Ok(())
}
fn place(value: Place, out: &mut impl Write) -> io::Result<()> {
    match value {
        Place::Local(id) => write!(out, "l{}", id.0),
        Place::Static(id) => write!(out, "static:{}", id.0),
        Place::Temporary(id) => write!(out, "temporary:v{}", id.0),
        Place::Projection(id) => write!(out, "p{}", id.0),
    }
}
fn rvalue(body: &Body, value: Rvalue, out: &mut impl Write) -> io::Result<()> {
    match value {
        Rvalue::Aggregate { variant, fields } => {
            write!(out, "aggregate")?;
            if let Some(variant) = variant {
                write!(out, " variant={}", variant.0)?;
            }
            write!(out, " [")?;
            for (index, value) in body.arguments(fields).iter().enumerate() {
                if index != 0 {
                    write!(out, ",")?;
                }
                write!(out, "v{}", value.0)?;
            }
            write!(out, "]")
        }
        Rvalue::Repeat { value } => write!(out, "repeat v{}", value.0),
        Rvalue::Unit => write!(out, "unit"),
        Rvalue::Scalar(id) => type_view::scalar(body.scalars[id.0 as usize], out),
        Rvalue::Length(value) => {
            write!(out, "length ")?;
            place(value, out)
        }
        Rvalue::TextEquals { place: value, text } => {
            write!(out, "text-equals t{} ", text.0)?;
            place(value, out)
        }
        Rvalue::Text(id) => write!(out, "text t{}", id.0),
        Rvalue::Borrow { place: value, kind } => {
            write!(
                out,
                "{} ",
                if kind == super::ir::BorrowKind::TwoPhase {
                    "borrow-two-phase"
                } else {
                    "borrow"
                }
            )?;
            place(value, out)
        }
        Rvalue::Read(value) => {
            write!(out, "read ")?;
            place(value, out)
        }
        Rvalue::IsVariant {
            place: value,
            variant,
        } => {
            write!(out, "is-variant {} ", variant.0)?;
            place(value, out)
        }
        Rvalue::Validate(value) => {
            write!(out, "validate ")?;
            place(value, out)
        }
        Rvalue::Function(id) => write!(out, "function def={}", id.0),
        Rvalue::Constant(id) => write!(out, "constant def={}", id.0),
        Rvalue::Reify(value) => write!(out, "reify v{}", value.0),
        Rvalue::Constrain(value) => write!(out, "constrain v{}", value.0),
        Rvalue::Unary {
            operator,
            operand,
            literal,
        } => write!(
            out,
            "unary {} v{} literal={literal}",
            operator.name(),
            operand.0
        ),
        Rvalue::Binary {
            operator,
            left,
            right,
        } => write!(out, "binary {operator:?} v{} v{}", left.0, right.0),
        Rvalue::Cast { kind, operand } => write!(out, "cast {kind:?} v{}", operand.0),
        Rvalue::Call {
            signature,
            callee,
            arguments,
        } => {
            write!(out, "call signature={} v{} [", signature.0, callee.0)?;
            for (index, value) in body.arguments(arguments).iter().enumerate() {
                if index != 0 {
                    write!(out, ",")?;
                }
                write!(out, "v{}", value.0)?;
            }
            write!(out, "]")
        }
    }
}
