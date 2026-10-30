//! Each distinct CFG Call owns a complete request, even at a shared source site.
use super::{Body, Context, Declarations, ItemId, NodeKey, Sites, Source, g, ir, s};
use crate::{sema::model::Operation, types::Kind as Type};

pub(in super::super) fn emit(
    source: &mut Source,
    facts: &mut Declarations,
    body: &Body,
    context: Context,
    raw: &mut s::Original,
    sites: &mut Sites,
) -> Result<(), &'static str> {
    let owner = body
        .source
        .source()
        .ok_or("source call export has no actual semantic body")?;
    source
        .stage
        .reserve(body.values.len().saturating_mul(size_of::<bool>()))?;
    let mut seen = vec![false; body.values.len()];
    let mut ports = Ports {
        source,
        facts,
        body,
        context,
        raw,
        sites,
        owner,
    };
    for statement in &body.statements {
        ports.source.stage.charge(1)?;
        let ir::Action::Evaluate(value) = statement.action else {
            continue;
        };
        let actual = body
            .values
            .get(value.0 as usize)
            .ok_or("source evaluated value is unknown")?;
        let ir::Rvalue::Call {
            signature, callee, ..
        } = actual.kind
        else {
            continue;
        };
        if std::mem::replace(&mut seen[value.0 as usize], true) {
            return Err("source call value has duplicate evaluation ownership");
        }
        ports.one(statement, value, signature, callee)?;
    }
    for (index, value) in body.values.iter().enumerate() {
        ports.source.stage.charge(1)?;
        if matches!(value.kind, ir::Rvalue::Call { .. }) && !seen[index] {
            return Err("source Call has no exact evaluated request coverage");
        }
    }
    Ok(())
}

struct Ports<'a> {
    source: &'a mut Source,
    facts: &'a mut Declarations,
    body: &'a Body,
    context: Context,
    raw: &'a mut s::Original,
    sites: &'a mut Sites,
    owner: NodeKey,
}
impl Ports<'_> {
    fn one(
        &mut self,
        statement: &ir::Statement,
        value: ir::ValueId,
        signature: crate::types::TypeId,
        callee: ir::ValueId,
    ) -> Result<(), &'static str> {
        let key = self
            .body
            .source
            .proof_site(statement.source)?
            .source()
            .ok_or("source Call has no actual source operation")?;
        self.source.stage.charge(2)?;
        let Some(Operation::Call {
            signature: retained,
            source_use: Some(source_use),
        }) = self.facts.node(key).and_then(|facts| facts.operation)
        else {
            return Err(
                "source Call has no exact completed owner/effect publication; ForPlan transport is not implemented",
            );
        };
        if signature != retained {
            return Err("source Call changed its completed typed signature");
        }
        self.source.stage.charge(1)?;
        let effect = source_use.effect_for(self.facts, self.owner)?;
        self.source.stage.charge(32)?;
        let safety = source_use.call_safety(self.facts, self.owner, key.node, signature)?;
        if effect != self.raw.context.request() {
            return Err("source Call effect differs from its actual original defining context");
        }
        let callable = self
            .body
            .values
            .get(callee.0 as usize)
            .ok_or("source Call has an unknown callee value")?;
        let Some(Type::FunctionItem {
            item,
            arguments,
            signature: declared,
        }) = self.facts.types.kind(callable.ty)
        else {
            return Err("source Call requires its actual direct function-item application");
        };
        let callable_type = callable.ty;
        self.method_target(key, item, callable_type)?;
        let target =
            self.source
                .stage
                .function_application(self.facts, item, arguments, self.context)?;
        let method = self
            .source
            .stage
            .method_request(self.facts, callable_type, self.context)?;
        let s::ValueKind::Call {
            signature: wire_signature,
            ..
        } = self
            .raw
            .values
            .get(value.0 as usize)
            .ok_or("source Call value row is absent")?
            .kind
        else {
            return Err("source emitted Call value changed kind");
        };
        let opening = self.opening(key, item, declared)?;
        self.source.stage.reserve(4 * size_of::<s::CallRequest>())?;
        self.raw.calls.push(s::CallRequest {
            value: value.0,
            site: self.sites.body(&mut self.source.stage, statement.source)?,
            target,
            method,
            signature: wire_signature,
            opening,
            effect,
            safety,
        });
        Ok(())
    }
    fn method_target(
        &mut self,
        key: NodeKey,
        item: ItemId,
        callable: crate::types::TypeId,
    ) -> Result<(), &'static str> {
        let Some(method) = self.facts.methods.calls.get(&key).copied() else {
            return Ok(());
        };
        self.source.stage.charge(1)?;
        if !matches!(self.facts.types.kind(method.callable),Some(Type::FunctionItem{item,..}) if item.0==method.method.0)
        {
            return Err("source method Call changed its actual semantic declaration");
        }
        if method.method.0 == item.0 && method.callable == callable {
            return Ok(());
        }
        if !self.source.stage.conversion_roles() {
            return Err(
                "source method Call changed its actual selected declaration or application",
            );
        }
        let selected = self.source.stage.paying(|pay| {
            crate::sema::dispatch::selected_callable(self.facts, key, method.callable, pay)
        })?;
        if selected != Some(callable) {
            return Err(
                "source method Call changed its actual selected declaration or application",
            );
        }
        Ok(())
    }
    fn opening(
        &mut self,
        key: NodeKey,
        item: ItemId,
        declared: crate::types::TypeId,
    ) -> Result<Option<s::Opening>, &'static str> {
        let Some(Type::Function { bound_regions, .. }) = self.facts.types.kind(declared) else {
            return Err("source callee signature is not a function");
        };
        let (binder, count) = self.source.stage.callable_binder(item)?;
        if count != bound_regions {
            return Err("source application changed its actual late binder");
        }
        self.source.stage.charge(1)?;
        let regions = self
            .source
            .openings
            .get(&(self.owner, key))
            .map_or(&[][..], Vec::as_slice);
        if regions.len() != bound_regions as usize {
            return Err("source Call has incomplete or foreign late-opening allocations");
        }
        let start = u32::try_from(self.raw.openings.len())
            .map_err(|_| "source late-opening table overflow")?;
        self.source
            .stage
            .reserve(regions.len().saturating_mul(4 * size_of::<s::RegionRef>()))?;
        for &region in regions {
            self.source.stage.charge(1)?;
            self.raw
                .openings
                .push(self.source.stage.region(region, self.context)?);
        }
        let end = u32::try_from(self.raw.openings.len())
            .map_err(|_| "source late-opening table overflow")?;
        Ok((bound_regions != 0).then_some(s::Opening {
            binder,
            regions: g::Range { start, end },
        }))
    }
}
