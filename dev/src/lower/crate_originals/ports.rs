//! Encode actual typed values and CFG actions; unsupported operations fail closed.
use super::{
    Action, Body, BodyOrigin, BodyRef, BodySite, DeclarationRef, Export, HashMap, LocalKind, Place,
    Rvalue, Terminator, g, lexical,
};
pub(super) struct Sites {
    source: BodyOrigin,
    map: HashMap<BodySite, u32>,
}
impl Sites {
    fn new(export: &mut Export<'_>, body: &Body) -> Result<Self, &'static str> {
        let mut sites = Self {
            source: body.source,
            map: HashMap::default(),
        };
        sites.get(export, body.root_site())?;
        Ok(sites)
    }
    pub(super) fn get(
        &mut self,
        export: &mut Export<'_>,
        site: BodySite,
    ) -> Result<u32, &'static str> {
        export.charge(1)?;
        self.source.proof_site(site)?;
        if let Some(&index) = self.map.get(&site) {
            return Ok(index);
        }
        export.reserve(4 * size_of::<(BodySite, u32)>())?;
        let index = Export::ordinal(self.map.len())?;
        self.map.insert(site, index);
        Ok(index)
    }
}
pub(super) fn emit(
    export: &mut Export<'_>,
    body: &Body,
    reference: BodyRef,
    declaration: DeclarationRef,
) -> Result<g::Original, &'static str> {
    validate(body)?;
    let mut sites = Sites::new(export, body)?;
    let mut raw = g::Original {
        body: reference,
        declaration,
        result: export.ty(body.result)?,
        sites: 0,
        locals: Vec::new(),
        parameter_bindings: Vec::new(),
        values: Vec::new(),
        statements: Vec::new(),
        blocks: Vec::new(),
        scopes: Vec::new(),
        exits: Vec::new(),
        temporaries: Vec::new(),
    };
    for local in &body.locals {
        export.reserve(4 * size_of::<g::Local>())?;
        let definition = local
            .definition
            .map(|d| export.declaration(d))
            .transpose()?;
        let source = sites.get(export, local.source)?;
        raw.locals.push(g::Local {
            scope: local.scope.0,
            storage_parent: local.storage_parent.map(|p| p.0),
            ty: export.ty(local.ty)?,
            kind: match local.kind {
                LocalKind::Parameter(i) => g::LocalKind::Parameter(i),
                LocalKind::Binding => g::LocalKind::Binding,
                LocalKind::Merge => g::LocalKind::Merge,
                LocalKind::Guard(_) => {
                    return Err("generic guard port is outside row version zero");
                }
            },
            definition,
            mutable: local.mutable,
            site: source,
        });
        if let Some(definition) = local.definition
            && let Some(&(input, local, _)) = export.formals.get(&definition)
        {
            let declaration = export.declaration(definition)?;
            export.reserve(4 * size_of::<g::ParameterBinding>())?;
            raw.parameter_bindings.push(g::ParameterBinding {
                declaration,
                input,
                local,
            });
        }
    }
    for value in &body.values {
        export.reserve(4 * size_of::<g::Value>())?;
        let kind = match value.kind {
            Rvalue::Unit => g::ValueKind::Unit,
            Rvalue::Read(Place::Local(local)) => g::ValueKind::Read(local.0),
            Rvalue::Constrain(value) => g::ValueKind::Constrain(value.0),
            _ => return Err("generic value operation is outside row version zero"),
        };
        raw.values.push(g::Value {
            ty: export.ty(value.ty)?,
            kind,
        });
    }
    for statement in &body.statements {
        export.reserve(4 * size_of::<g::Statement>())?;
        raw.statements.push(g::Statement {
            site: sites.get(export, statement.source)?,
            action: action(statement.action)?,
        });
    }
    for block in &body.blocks {
        export.reserve(4 * size_of::<g::Block>())?;
        raw.blocks.push(g::Block {
            statements: g::Range {
                start: block.start,
                end: block.end,
            },
            site: sites.get(export, block.source)?,
            terminator: match block.terminator {
                Terminator::Return(value) => g::Terminator::Return(value.0),
                Terminator::Jump(block) => g::Terminator::Jump(block.0),
                Terminator::Unreachable => g::Terminator::Unreachable,
                _ => return Err("generic CFG terminator is outside row version zero"),
            },
        });
    }
    lexical::emit(export, body, &mut raw, &mut sites)?;
    raw.sites = Export::ordinal(sites.map.len())?;
    Ok(raw)
}
fn action(action: Action) -> Result<g::Action, &'static str> {
    Ok(match action {
        Action::StorageLive(l) => g::Action::StorageLive(l.0),
        Action::StorageDead(l) => g::Action::StorageDead(l.0),
        Action::StorageEnd { first, last } => g::Action::StorageEnd {
            first: first.0,
            last: last.0,
        },
        Action::Evaluate(v) => g::Action::Evaluate(v.0),
        Action::Store {
            place: Place::Local(l),
            value,
        } => g::Action::Store {
            local: l.0,
            value: value.0,
        },
        Action::CheckInitialized(Place::Local(l)) => g::Action::CheckInitialized(l.0),
        Action::Mention(Place::Local(l)) => g::Action::Mention(l.0),
        _ => return Err("generic CFG statement is outside row version zero"),
    })
}

fn validate(body: &Body) -> Result<(), &'static str> {
    if !body.projections.is_empty()
        || !body.scalars.is_empty()
        || !body.arguments.is_empty()
        || !body.guards.is_empty()
        || !body.guard_places.is_empty()
        || body.closure_calls.is_some()
        || body.resolved_callables.is_some()
        || body.static_types.as_ref().is_some_and(|m| !m.is_empty())
        || body.mutable_statics.as_ref().is_some_and(|s| !s.is_empty())
        || body
            .owned
            .as_ref()
            .is_some_and(|o| !o.created.is_empty() || !o.returned.is_empty())
    {
        return Err(
            "generic original requires operation or ownership data outside row version zero",
        );
    }
    Ok(())
}
