//! Validate complete declaration schemas before frontend allocation or remapping.
use super::{BinderKind, Bound, CopyIdentity, Declaration, Function, Rows, Work, graph::Graph};
use crate::support::crates::{
    self, CrateIdentity, ExternalKind,
    owner_catalog::{BodyCatalog, BodyKind, BodyOwner, DeclarationBody},
};

pub(super) fn declarations(
    owner: &CrateIdentity,
    rows: &Rows,
    work: &mut Work,
) -> Result<usize, &'static str> {
    if rows.functions.is_empty()
        || rows.declarations.len() > BodyCatalog::MAX_DECLARATIONS
        || rows.declarations.len() != rows.declaration_sites.len()
        || rows.functions.len() > crates::MAX_FUNCTIONS
        || rows.functions.len() != rows.originals.len()
        || rows
            .types
            .len()
            .saturating_add(rows.binders.len())
            .saturating_add(rows.roots.len())
            .saturating_add(rows.parameters.len())
            .saturating_add(rows.clauses.len())
            > crates::MAX_COMPONENTS
    {
        return Err("generic declaration tables exceed profile budget");
    }
    let mut names = owner
        .name()
        .len()
        .saturating_add(owner.disambiguator().len());
    let mut previous_function = None;
    let mut previous_parameter = None;
    let mut previous_local = None;
    let mut phase = 0;
    let mut sites = 0_usize;
    for (index, declaration) in rows.declarations.iter().enumerate() {
        work.charge(1)?;
        let count = rows.declaration_sites[index];
        sites = sites.saturating_add(count as usize);
        if count == 0 || sites > crates::MAX_COMPONENTS {
            return Err("generic declaration sites exceed profile budget");
        }
        match declaration {
            Declaration::Function { identity, .. } => {
                if phase != 0
                    || identity.owner() != owner
                    || identity.kind() != ExternalKind::Function
                    || previous_function.is_some_and(|previous| previous >= identity)
                {
                    return Err("generic function declarations are foreign or noncanonical");
                }
                previous_function = Some(identity);
                for name in identity.path() {
                    work.charge(name.len().saturating_add(1))?;
                    names = names.saturating_add(name.len());
                }
            }
            Declaration::TypeParameter {
                owner,
                ordinal,
                name,
                ..
            } => {
                if phase > 1
                    || owner.0 as usize >= index
                    || !matches!(
                        rows.declarations.get(owner.0 as usize),
                        Some(Declaration::Function { .. })
                    )
                    || previous_parameter.is_some_and(|previous| previous >= (owner.0, *ordinal))
                {
                    return Err("generic parameter changed its owning declaration or order");
                }
                phase = 1;
                previous_parameter = Some((owner.0, *ordinal));
                work.charge(name.len())?;
                super::super::identity::validate_component(name)?;
                names = names.saturating_add(name.len());
            }
            Declaration::ValueParameter {
                body,
                ordinal,
                name,
            }
            | Declaration::Local {
                body,
                ordinal,
                name,
            } => {
                if body.0 as usize >= rows.originals.len()
                    || previous_local.is_some_and(|previous| previous >= (body.0, *ordinal))
                {
                    return Err("generic local declaration changed its actual body or order");
                }
                phase = 2;
                previous_local = Some((body.0, *ordinal));
                work.charge(name.len())?;
                super::super::identity::validate_component(name)?;
                names = names.saturating_add(name.len());
            }
        }
        if names > crates::MAX_NAME_BYTES {
            return Err("generic declaration names exceed profile budget");
        }
    }
    binders(rows, work)?;
    Ok(names)
}
pub(super) fn functions(
    owner: &CrateIdentity,
    rows: &Rows,
    graph: &Graph,
    work: &mut Work,
) -> Result<BodyCatalog, &'static str> {
    let mut counts = vec![0; rows.declarations.len()];
    let mut headers = vec![false; rows.declarations.len()];
    for declaration in &rows.declarations {
        work.charge(1)?;
        if let Declaration::TypeParameter { owner, ordinal, .. } = declaration {
            let count = &mut counts[owner.0 as usize];
            if *ordinal != *count {
                return Err("generic parameter order has a missing slot");
            }
            *count += 1;
        }
    }
    let mut previous = None;
    let mut next_parameters = 0;
    let mut next_clauses = 0;
    let mut bodies = Vec::with_capacity(rows.functions.len());
    for (index, function) in rows.functions.iter().enumerate() {
        work.charge(1)?;
        let declaration = function.declaration;
        if previous.is_some_and(|previous| previous >= declaration.0)
            || !matches!(
                rows.declarations.get(declaration.0 as usize),
                Some(Declaration::Function { .. })
            )
            || function.body.0 as usize != index
        {
            return Err("generic headers are duplicate, foreign or reordered");
        }
        if function.parameters.start != next_parameters || function.clauses.start != next_clauses {
            return Err("generic header inputs are not one complete ordered table");
        }
        next_parameters = function.parameters.end;
        next_clauses = function.clauses.end;
        previous = Some(declaration.0);
        headers[declaration.0 as usize] = true;
        header(rows, graph, function, counts[declaration.0 as usize], work)?;
        bodies.push(BodyOwner {
            declaration,
            parent: None,
            kind: BodyKind::Function,
            ordinal: 0,
        });
    }
    if next_parameters as usize != rows.parameters.len()
        || next_clauses as usize != rows.clauses.len()
    {
        return Err("generic parameter or clause table has unowned rows");
    }
    for (index, declaration) in rows.declarations.iter().enumerate() {
        work.charge(1)?;
        if matches!(declaration, Declaration::Function { .. }) && !headers[index] {
            return Err("generic declaration has no complete original header");
        }
    }
    let kinds = rows
        .declarations
        .iter()
        .map(|declaration| {
            if matches!(declaration, Declaration::Function { .. }) {
                DeclarationBody::Function
            } else {
                DeclarationBody::Other
            }
        })
        .collect::<Vec<_>>();
    BodyCatalog::new(owner.clone(), &kinds, bodies)
}
fn header(
    rows: &Rows,
    graph: &Graph,
    function: &Function,
    count: u32,
    work: &mut Work,
) -> Result<(), &'static str> {
    let declaration = function.declaration;
    let parameters = function.parameters.slice(&rows.parameters)?;
    if parameters.len() != count as usize {
        return Err("generic header omitted a declared parameter");
    }
    if parameters.is_empty() {
        return Err("generic row version zero requires an instantiated type parameter");
    }
    for (index, &parameter) in parameters.iter().enumerate() {
        work.charge(1)?;
        if !matches!(rows.declarations.get(parameter.0 as usize),
            Some(Declaration::TypeParameter {owner,ordinal,..}) if *owner == declaration && *ordinal as usize == index)
        {
            return Err("generic header parameter belongs to another declaration or slot");
        }
    }
    let original = Graph::signature(rows, function.original)?;
    let callable = Graph::signature(rows, function.callable)?;
    for (reference, kind) in [
        (function.original_frame, BinderKind::OriginalFunction),
        (function.callable_frame, BinderKind::PublicFunction),
    ] {
        let binder = rows
            .binders
            .get(reference.0 as usize)
            .ok_or("generic header binder is unknown")?;
        if binder.owner != declaration || binder.kind != kind || binder.parent.is_some() {
            return Err("generic header changed its original or callable binder identity");
        }
        // This first executable family has no body/early/late region arguments.
        // Nonzero binders remain an explicit required later family.
        if binder.count != 0 {
            return Err("generic lifetime transport is not implemented for this operation family");
        }
    }
    if original.binder != function.original_frame
        || callable.binder != function.callable_frame
        || original.parameters.slice(&rows.roots)? != callable.parameters.slice(&rows.roots)?
        || original.result != callable.result
        || original.unsafe_ != callable.unsafe_
        || original.convention != callable.convention
        || original.variadic != callable.variadic
    {
        return Err("generic public signature differs from its original relocation");
    }
    graph.root(declaration, function.original, work)?;
    graph.root(declaration, function.callable, work)?;
    if original.convention != super::Convention::Rust || original.variadic {
        return Err("generic foreign calling conventions are outside this operation family");
    }
    let clauses = function.clauses.slice(&rows.clauses)?;
    if function.declared_clauses as usize != clauses.len() {
        return Err("generic implied inputs require their checked trait consequence contracts");
    }
    for clause in clauses {
        work.charge(1)?;
        if clause.owner != declaration
            || clause.site >= rows.declaration_sites[declaration.0 as usize]
        {
            return Err("generic clause belongs to a foreign declaration or site");
        }
        graph.root(declaration, clause.subject, work)?;
        if let Some(binder) = clause.quantifier {
            let binder = rows
                .binders
                .get(binder.0 as usize)
                .ok_or("generic clause binder is unknown")?;
            if binder.count != 0 {
                return Err("quantified lifetime transport is outside generic row version zero");
            }
            if binder.owner != declaration
                || binder.kind != BinderKind::Clause
                || binder.parent != Some(function.original_frame)
            {
                return Err("generic clause changed its complete quantified frame");
            }
        }
        if let Bound::Copy(CopyIdentity::Source { .. }) = clause.bound {
            return Err("source-defined Copy transport requires its typed trait contract");
        }
    }
    Ok(())
}

fn binders(rows: &Rows, work: &mut Work) -> Result<(), &'static str> {
    for (index, binder) in rows.binders.iter().enumerate() {
        work.charge(1)?;
        if !matches!(
            rows.declarations.get(binder.owner.0 as usize),
            Some(Declaration::Function { .. })
        ) || binder.count > 65_536
        {
            return Err("generic binder has an invalid declaration or slot count");
        }
        if let Some(parent) = binder.parent
            && (parent.0 as usize >= index || rows.binders[parent.0 as usize].owner != binder.owner)
        {
            return Err("generic binder changed its actual parent or declaration");
        }
    }
    Ok(())
}
