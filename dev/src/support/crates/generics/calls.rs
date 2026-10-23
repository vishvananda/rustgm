//! Application recipes retain declaration identity without importing call evidence.
use super::{ApplicationRef, Declaration, Rows, Type, TypeRef, Work, graph::Graph};
use crate::support::{crates, crates::owner_catalog::DeclarationRef, hash::HashSet};

impl super::Calls {
    /// Complete typed references, not a body or proof handle.
    /// # Errors
    /// Rejects unknown applications and compact index overflow.
    pub fn item_type(&self, application: ApplicationRef) -> Result<TypeRef, &'static str> {
        if application.0 as usize >= self.applications.len() {
            return Err("generic application is unknown");
        }
        self.header_types
            .checked_add(
                application
                    .0
                    .checked_mul(2)
                    .ok_or("generic application type index overflow")?,
            )
            .map(TypeRef)
            .ok_or("generic application type index overflow")
    }
    /// # Errors
    /// Rejects the same foreign coordinates as `item_type`.
    pub fn signature_type(&self, application: ApplicationRef) -> Result<TypeRef, &'static str> {
        self.item_type(application)?
            .0
            .checked_add(1)
            .map(TypeRef)
            .ok_or("generic application signature index overflow")
    }
}

pub(super) fn validate(
    rows: &Rows,
    graph: &Graph,
    work: &mut Work,
) -> Result<Vec<Option<DeclarationRef>>, &'static str> {
    let Some(calls) = &rows.calls else {
        return Ok(Vec::new());
    };
    if calls.applications.is_empty()
        || calls.applications.len() > crates::MAX_COMPONENTS
        || calls.arguments.len() > crates::MAX_COMPONENTS
    {
        return Err("generic call tables are empty or exceed profile budget");
    }
    let mut keys = HashSet::with_capacity_and_hasher(
        calls.applications.len(),
        crate::support::hash::BuildFxHasher::default(),
    );
    let mut owners = Vec::with_capacity(calls.applications.len());
    let mut next = 0;
    for application in &calls.applications {
        work.charge(1)?;
        let target = rows
            .functions
            .get(application.target.0 as usize)
            .filter(|h| h.declaration == application.target)
            .ok_or("generic application target is not an actual function header")?;
        let signature = Graph::signature(rows, target.callable)?;
        if signature.unsafe_ {
            return Err("generic direct calls require a safe target");
        }
        for &root in signature
            .parameters
            .slice(&rows.roots)?
            .iter()
            .chain(std::iter::once(&signature.result))
        {
            work.charge(1)?;
            leaf(rows, root)?;
        }
        for clause in target.clauses.slice(&rows.clauses)? {
            work.charge(1)?;
            leaf(rows, clause.subject)?;
        }
        let arguments = application.arguments.slice(&calls.arguments)?;
        if application.arguments.start != next
            || arguments.len() != target.parameters.slice(&rows.parameters)?.len()
        {
            return Err("generic application arguments are incomplete, duplicate or reordered");
        }
        next = application.arguments.end;
        let mut owner = None;
        for &argument in arguments {
            work.charge(1)?;
            if argument.0 >= calls.header_types {
                return Err("generic application argument escapes its header prefix");
            }
            if let Some(actual) = graph.owner(argument)? {
                if owner.is_some_and(|previous| previous != actual) {
                    return Err("generic application captures unrelated function parameters");
                }
                owner = Some(actual);
            }
        }
        work.charge(arguments.len())?; // Hash/equality visits on success and duplicate failure.
        if !keys.insert((application.target, arguments)) {
            return Err("generic application arguments are duplicate");
        }
        owners.push(owner);
    }
    if next as usize != calls.arguments.len() {
        return Err("generic call arguments have unowned table entries");
    }
    Ok(owners)
}
fn leaf(rows: &Rows, root: TypeRef) -> Result<(), &'static str> {
    match rows.types.get(root.0 as usize) {
        Some(Type::Scalar(_) | Type::Parameter(_)) => Ok(()),
        _ => Err("generic direct-call target requires scalar or parameter ports and bounds"),
    }
}

pub(super) fn target(
    rows: &Rows,
    caller: DeclarationRef,
    application: ApplicationRef,
) -> Result<&super::Function, &'static str> {
    let calls = rows
        .calls
        .as_ref()
        .ok_or("generic call operation requires row version one")?;
    let selected = calls
        .applications
        .get(application.0 as usize)
        .ok_or("generic call selected an unknown application")?;
    let target = &rows.functions[selected.target.0 as usize];
    let Some(Declaration::Function {
        identity: target_name,
        public,
    }) = rows.declarations.get(selected.target.0 as usize)
    else {
        return Err("generic call target changed declaration kind");
    };
    let Some(Declaration::Function {
        identity: caller_name,
        ..
    }) = rows.declarations.get(caller.0 as usize)
    else {
        return Err("generic caller changed declaration kind");
    };
    if !public
        && target_name.path().split_last().map(|(_, p)| p)
            != caller_name.path().split_last().map(|(_, p)| p)
    {
        return Err("generic private calls require the same defining module");
    }
    Ok(target)
}
