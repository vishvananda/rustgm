//! Typed operation edges retain exact caller parameter ownership.
use super::{Function, Original, Rows, Type, ValueKind, Work, graph::Graph};
fn site(original: &Original, site: u32) -> Result<(), &'static str> {
    if site >= original.sites {
        return Err("generic operation site belongs outside its body");
    }
    Ok(())
}
pub(super) fn validate(
    rows: &Rows,
    graph: &Graph,
    header: &Function,
    original: &Original,
    applications: &mut [bool],
    work: &mut Work,
) -> Result<(), &'static str> {
    let mut next_arguments = 0;
    for (index, value) in original.values.iter().enumerate() {
        work.operations = work.operations.saturating_add(1);
        work.charge(1)?;
        graph.root(header.declaration, value.ty, work)?;
        match value.kind {
            ValueKind::Unit
                if rows.types.get(value.ty.0 as usize)
                    != Some(&Type::Scalar(crate::support::crates::Scalar::Unit)) =>
            {
                return Err("generic unit operation has a different actual type");
            }
            ValueKind::Read(local)
                if original.locals.get(local as usize).map(|local| local.ty) != Some(value.ty) =>
            {
                return Err("generic read type differs from its actual local");
            }
            ValueKind::Constrain(input) if input as usize >= index => {
                return Err("generic value graph has a forward or cyclic edge");
            }
            ValueKind::Function(application) => {
                let calls = rows
                    .calls
                    .as_ref()
                    .ok_or("generic function value requires row version one")?;
                if value.ty != calls.item_type(application)? {
                    return Err("generic function value changed its full application type");
                }
                super::calls::target(rows, header.declaration, application)?;
                applications[application.0 as usize] = true;
            }
            ValueKind::Call {
                signature,
                callee,
                arguments,
                site: source,
            } => {
                site(original, source)?;
                let calls = rows
                    .calls
                    .as_ref()
                    .ok_or("generic call requires row version one")?;
                let callee = original
                    .values
                    .get(callee as usize)
                    .filter(|_| (callee as usize) < index)
                    .ok_or("generic call has a forward or unknown callee")?;
                let Some(Type::FunctionItem(application)) = rows.types.get(callee.ty.0 as usize)
                else {
                    return Err("generic direct call requires a complete function-item type");
                };
                if signature != calls.signature_type(*application)?
                    || arguments.start != next_arguments
                {
                    return Err("generic call changed its exact signature or argument order");
                }
                let target = super::calls::target(rows, header.declaration, *application)?;
                let input_count = Graph::signature(rows, target.callable)?
                    .parameters
                    .slice(&rows.roots)?
                    .len();
                let arguments = arguments.slice(&original.arguments)?;
                if arguments.len() != input_count {
                    return Err("generic call arguments are incomplete");
                }
                for &argument in arguments {
                    work.charge(1)?;
                    if argument as usize >= index {
                        return Err("generic call argument has a forward value reference");
                    }
                }
                next_arguments = u32::try_from(next_arguments as usize + arguments.len())
                    .map_err(|_| "generic call argument index overflow")?;
            }
            _ => {}
        }
    }
    if next_arguments as usize != original.arguments.len() {
        return Err("generic value argument table has unowned entries");
    }
    Ok(())
}
