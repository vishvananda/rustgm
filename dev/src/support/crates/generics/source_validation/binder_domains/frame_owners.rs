//! Binder kinds bind exact header/body coordinates, never a matching arity alone.
use super::{BinderIndex, HeaderIndex, HeaderRef, Work, g, s};

impl BinderIndex<'_> {
    pub(super) fn complete(
        &self,
        headers: &HeaderIndex<'_>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        for header in &self.rows.traits {
            self.declaration(header.declaration, work)?;
        }
        for header in &self.rows.implementations {
            self.declaration(header.self_definition, work)?;
        }
        for header in &self.rows.functions {
            for (reference, kind) in [
                (header.original.binder, s::BinderKind::OriginalFunction),
                (header.callable.binder, s::BinderKind::PublicFunction),
            ] {
                work.charge(1)?;
                let frame = self
                    .rows
                    .binders
                    .get(reference.0 as usize)
                    .ok_or("source function omitted its actual signature binder")?;
                if frame.owner != header.declaration || frame.kind != kind {
                    return Err("source signature binder is foreign or has another header role");
                }
            }
            headers.scopes().complete_slots(header.scope, work)?;
        }
        for clause in &self.rows.clauses {
            work.charge(1)?;
            if let Some(reference) = clause.binder {
                let frame = self
                    .rows
                    .binders
                    .get(reference.0 as usize)
                    .ok_or("source quantified clause binder is unknown")?;
                if frame.kind != s::BinderKind::Clause || frame.owner != clause.owner {
                    return Err("source quantified clause has a foreign actual owner");
                }
            }
        }
        Ok(())
    }
    pub(super) fn frame(
        &mut self,
        headers: &HeaderIndex<'_>,
        ordinal: usize,
        frame: &s::Binder,
        profile: s::Profile,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(2)?;
        let reference =
            g::BinderRef(u32::try_from(ordinal).map_err(|_| "source binder ID overflow")?);
        let owner = headers.header(frame.owner, work)?;
        let parent = frame
            .parent
            .map(|parent| {
                if parent.0 as usize >= ordinal {
                    return Err("source binder parent is forward or cyclic");
                }
                self.rows
                    .binders
                    .get(parent.0 as usize)
                    .ok_or("source binder parent is unknown")
            })
            .transpose()?;
        match frame.kind {
            s::BinderKind::DeclarationHeader => {
                self.header_frame(headers, owner, reference, frame, work)?;
            }
            s::BinderKind::OriginalFunction | s::BinderKind::PublicFunction => {
                let Some(HeaderRef::Function(index)) = owner else {
                    return Err("source callable binder belongs to a non-function declaration");
                };
                let header = &self.rows.functions[index as usize];
                let expected = if frame.kind == s::BinderKind::OriginalFunction {
                    header.original.binder
                } else {
                    header.callable.binder
                };
                if parent.is_some() || expected != reference {
                    return Err(
                        "source callable binder lost its exact original/public header role",
                    );
                }
                if frame.kind == s::BinderKind::OriginalFunction
                    && frame.count != header.regions.total
                {
                    return Err("source original binder differs from its complete lifetime scheme");
                }
            }
            s::BinderKind::Clause => {
                if owner.is_none() || parent.is_none_or(|parent| parent.owner != frame.owner) {
                    return Err(
                        "source quantified input binder lost its exact owning header frame",
                    );
                }
            }
            s::BinderKind::CallableType => {
                let admitted = match owner {
                    Some(HeaderRef::Function(_)) => true,
                    Some(HeaderRef::Nominal(_)) => profile.body_nodes(),
                    Some(_) => profile.declaration_policies(),
                    None => false,
                };
                let valid_parent = match parent {
                    None => true,
                    Some(parent) => profile.declaration_policies() && parent.owner == frame.owner,
                };
                if !admitted
                    || !valid_parent
                    || (frame.count != 0 && !profile.declaration_policies())
                {
                    return Err("source shared call type requires an actual closed callable frame");
                }
            }
            s::BinderKind::DynamicType => {
                if !profile.declaration_policies()
                    || parent.is_some_and(|parent| parent.owner != frame.owner)
                {
                    return Err("source dynamic type requires its actual enclosing binder");
                }
            }
            s::BinderKind::CallSignature { body, value } => {
                self.call_frame(headers, reference, frame, body, value)?;
            }
        }
        Ok(())
    }
    fn header_frame(
        &mut self,
        headers: &HeaderIndex<'_>,
        owner: Option<HeaderRef>,
        reference: g::BinderRef,
        frame: &s::Binder,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let count = match owner {
            Some(HeaderRef::Nominal(_)) => 0,
            Some(HeaderRef::AssociatedType(_) | HeaderRef::AssociatedDefinition(_)) => {
                let scope = frame.owner;
                if matches!(owner, Some(HeaderRef::AssociatedDefinition(_))) {
                    let own = headers.scopes().declared_slots(scope, work)?;
                    if own.start == own.end {
                        return Err("plain associated definition uses its actual impl binder");
                    }
                }
                headers
                    .scopes()
                    .complete_slots(scope, work)?
                    .checked_sub(headers.scopes().complete_type_slots(scope, work)?)
                    .ok_or("source associated header has an invalid bound region count")?
            }
            Some(HeaderRef::Trait(index)) => self.rows.traits[index as usize].region_captures,
            Some(HeaderRef::Implementation(index)) => {
                self.rows.implementations[index as usize].region_captures
            }
            Some(HeaderRef::Constant(index))
                if !self.rows.constants()[index as usize].body.evaluated() =>
            {
                let parent = self.rows.declarations[frame.owner.0 as usize]
                    .definition
                    .parent
                    .ok_or("source associated constant binder lacks its actual owner")?;
                match headers.header(parent, work)? {
                    Some(HeaderRef::Trait(index)) => {
                        self.rows.traits[index as usize].region_captures
                    }
                    Some(HeaderRef::Implementation(index)) => {
                        self.rows.implementations[index as usize].region_captures
                    }
                    _ => return Err("source associated constant binder has a foreign owner"),
                }
            }
            _ => return Err("source declaration binder belongs to another header kind"),
        };
        if frame.parent.is_some()
            || count != frame.count
            || self.declarations[frame.owner.0 as usize]
                .replace(reference)
                .is_some()
        {
            return Err("source header binder has foreign captures or duplicate ownership");
        }
        Ok(())
    }
    fn call_frame(
        &self,
        headers: &HeaderIndex<'_>,
        reference: g::BinderRef,
        frame: &s::Binder,
        body: crate::support::crates::owner_catalog::BodyRef,
        value: u32,
    ) -> Result<(), &'static str> {
        let original = self
            .rows
            .originals
            .get(headers.original_index(body)?)
            .ok_or("source Call binder has an unknown original body")?;
        let input = &self.rows.body_inputs[headers.original_index(body)?];
        let call = original
            .values
            .get(value as usize)
            .ok_or("source Call binder has an unknown representative value")?;
        let (s::ValueKind::Call { signature, .. } | s::ValueKind::PointerCall { signature, .. }) =
            call.kind
        else {
            return Err("source Call binder representative is not a Call value");
        };
        let Some(s::Type::Function(signature)) = self.rows.types.get(signature.0 as usize) else {
            return Err("source Call binder representative has no function type");
        };
        if original.declaration != frame.owner
            || frame.parent != Some(input.frame)
            || signature.binder != reference
            || frame.count != 0
        {
            return Err(
                "source Call binder changed its original owner, opening or actual signature",
            );
        }
        Ok(())
    }
}
