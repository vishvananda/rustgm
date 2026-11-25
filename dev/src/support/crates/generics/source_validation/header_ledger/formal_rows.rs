//! Header formals remain genuine declarations even when a required body is absent.
use super::{DefinitionKind, HeaderRef, SourceRows, Work, g, s};

pub(super) fn validate(
    rows: &SourceRows,
    headers: &[Option<HeaderRef>],
    index: Option<&s::bodymap::Lookup>,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(rows.declarations.len().saturating_add(rows.formals.len()))?;
    let mut seen = vec![false; rows.declarations.len()];
    let mut covered = vec![false; rows.formals.len()];
    let mut destructured = vec![false; rows.functions.len()];
    for (fn_index, header) in rows.functions.iter().enumerate() {
        let formals = header.formals.slice(&rows.formals)?;
        if formals.len() != header.original.parameters.slice(&rows.roots)?.len()
            || formals.len() != header.callable.parameters.slice(&rows.roots)?.len()
        {
            return Err("source formal inventory differs from its actual callable/original arity");
        }
        for (ordinal, formal) in formals.iter().enumerate() {
            work.charge(1)?;
            if formal.ordinal as usize != ordinal
                || std::mem::replace(&mut covered[header.formals.start as usize + ordinal], true)
            {
                return Err("source formal ranges overlap or change their actual ordinal");
            }
            if let Some(declaration) = formal.declaration {
                let row = rows
                    .declarations
                    .get(declaration.0 as usize)
                    .ok_or("source formal declaration is unknown")?;
                if row.definition.kind != DefinitionKind::ValueParameter
                    || row.definition.parent != Some(header.declaration)
                    || std::mem::replace(&mut seen[declaration.0 as usize], true)
                    || !matches!(header.body, s::FunctionBody::Original(_)) && row.local.is_some()
                {
                    return Err(
                        "source formal has duplicate, foreign or fabricated body ownership",
                    );
                }
            } else {
                destructured[fn_index] = true;
            }
        }
        if let s::FunctionBody::Original(body) = header.body {
            bindings(
                rows,
                &rows.originals[s::bodymap::original(index, body)?],
                formals,
                work,
            )?;
        }
    }
    work.charge(rows.declarations.len().saturating_add(covered.len()))?;
    if covered.contains(&false) {
        return Err("source value parameter/formal rows have incomplete actual header coverage");
    }
    if let Some(closures) = rows.closures() {
        for header in &closures.headers {
            work.charge(1)?;
            let original = &rows.originals[s::bodymap::original(index, header.body)?];
            closure_bindings(rows, original, header, &mut seen, work)?;
        }
    }
    for (index, row) in rows.declarations.iter().enumerate() {
        if row.definition.kind != DefinitionKind::ValueParameter || seen[index] {
            continue;
        }
        if let Some(local) = row.local
            && let Some((_, closure)) = rows
                .closures()
                .and_then(|closures| closures.header_for_body(local.body))
            && row.definition.parent == Some(closure.owner)
        {
            continue;
        }
        let Some(HeaderRef::Function(fn_index)) = row
            .definition
            .parent
            .and_then(|parent| headers.get(parent.0 as usize).copied().flatten())
        else {
            return Err(
                "source value parameter/formal rows have incomplete actual header coverage",
            );
        };
        let header = &rows.functions[fn_index as usize];
        let body_matches = match header.body {
            s::FunctionBody::Original(body) => row.local.is_some_and(|slot| slot.body == body),
            _ => row.local.is_none(),
        };
        if !destructured[fn_index as usize] || !body_matches {
            return Err(
                "source value parameter/formal rows have incomplete actual header coverage",
            );
        }
    }
    Ok(())
}

fn bindings(
    rows: &SourceRows,
    original: &s::Original,
    formals: &[s::Formal],
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(
        formals
            .len()
            .saturating_mul(2)
            .saturating_add(original.locals.len()),
    )?;
    let mut inputs = vec![false; formals.len()];
    let mut bound = vec![false; formals.len()];
    let mut locals = vec![false; original.locals.len()];
    for local in &original.locals {
        work.charge(1)?;
        if let g::LocalKind::Parameter(ordinal) = local.kind {
            let input = inputs
                .get_mut(ordinal as usize)
                .ok_or("source parameter input has an unknown formal ordinal")?;
            if std::mem::replace(input, true) {
                return Err("source original repeats an actual parameter input");
            }
        }
    }
    for binding in &original.parameter_bindings {
        work.charge(1)?;
        let input = original
            .locals
            .get(binding.input as usize)
            .ok_or("source parameter binding input is unknown")?;
        let local = original
            .locals
            .get(binding.local as usize)
            .ok_or("source parameter binding local is unknown")?;
        let g::LocalKind::Parameter(ordinal) = input.kind else {
            return Err("source parameter binding input has another actual local kind");
        };
        let formal = formals
            .get(ordinal as usize)
            .ok_or("source parameter binding has an unknown formal")?;
        let declaration = rows
            .declarations
            .get(binding.declaration.0 as usize)
            .ok_or("source parameter binding declaration is unknown")?;
        if formal.declaration != Some(binding.declaration)
            || local.definition != Some(binding.declaration)
            || declaration.local.is_none_or(|slot| {
                slot.body != original.body
                    || slot.ordinal != binding.local
                    || slot.scope != local.scope
            })
            || std::mem::replace(&mut bound[ordinal as usize], true)
            || std::mem::replace(&mut locals[binding.local as usize], true)
        {
            return Err(
                "source parameter binding changed its exact declaration/input/local mapping",
            );
        }
    }
    work.charge(formals.len())?;
    if inputs.contains(&false)
        || formals
            .iter()
            .enumerate()
            .any(|(ordinal, formal)| formal.declaration.is_some() != bound[ordinal])
    {
        return Err("source original omitted its actual parameter input or named binding");
    }
    Ok(())
}

fn closure_bindings(
    rows: &SourceRows,
    original: &s::Original,
    header: &s::ClosureHeader,
    seen: &mut [bool],
    work: &mut Work,
) -> Result<(), &'static str> {
    let Some(s::Type::Function(signature)) = rows.types.get(header.body_signature().0 as usize)
    else {
        return Err("source closure header lacks its actual function signature");
    };
    let arity = signature.parameters.slice(&rows.roots)?.len();
    work.charge(
        arity
            .saturating_mul(2)
            .saturating_add(original.locals.len()),
    )?;
    let mut inputs = vec![false; arity];
    let mut bound = vec![false; arity];
    let mut locals = vec![false; original.locals.len()];
    for local in &original.locals {
        work.charge(1)?;
        if let g::LocalKind::Parameter(ordinal) = local.kind {
            let input = inputs
                .get_mut(ordinal as usize)
                .ok_or("source closure parameter input has an unknown formal ordinal")?;
            if std::mem::replace(input, true) {
                return Err("source closure repeats an actual parameter input");
            }
        }
    }
    for binding in &original.parameter_bindings {
        work.charge(1)?;
        let input = original
            .locals
            .get(binding.input as usize)
            .ok_or("source closure parameter binding input is unknown")?;
        let local = original
            .locals
            .get(binding.local as usize)
            .ok_or("source closure parameter binding local is unknown")?;
        let g::LocalKind::Parameter(ordinal) = input.kind else {
            return Err("source closure parameter binding input has another actual local kind");
        };
        if (ordinal as usize) >= arity {
            return Err("source closure parameter binding has an unknown formal");
        }
        let declaration = rows
            .declarations
            .get(binding.declaration.0 as usize)
            .ok_or("source closure parameter binding declaration is unknown")?;
        if declaration.definition.kind != DefinitionKind::ValueParameter
            || declaration.definition.parent != Some(header.owner)
            || local.definition != Some(binding.declaration)
            || declaration.local.is_none_or(|slot| {
                slot.body != original.body
                    || slot.ordinal != binding.local
                    || slot.scope != local.scope
            })
            || std::mem::replace(&mut seen[binding.declaration.0 as usize], true)
            || std::mem::replace(&mut bound[ordinal as usize], true)
            || std::mem::replace(&mut locals[binding.local as usize], true)
        {
            return Err(
                "source closure parameter binding changed its exact declaration/input/local mapping",
            );
        }
    }
    work.charge(arity)?;
    if inputs.contains(&false) {
        return Err("source closure omitted its actual parameter input");
    }
    Ok(())
}
