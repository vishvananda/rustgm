//! Request/value/site coverage is exact; closed signature sharing grants no proof.
use super::{Context, DomainIndex, HeaderRef, Types, Work, g, s};

impl DomainIndex<'_, '_> {
    pub(in super::super) fn calls(
        &mut self,
        context: Context,
        original: &s::Original,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let scratch = original
            .values
            .len()
            .saturating_mul(2 * size_of::<Option<u32>>());
        self.scratch(scratch, work)?;
        work.charge(original.values.len().saturating_mul(2))?;
        let mut evaluated = vec![None; original.values.len()];
        let mut requested = vec![None; original.values.len()];
        for statement in &original.statements {
            Self::site(original, statement.site, work)?;
            if let s::Action::Evaluate(value) = statement.action {
                let actual = original
                    .values
                    .get(value as usize)
                    .ok_or("source evaluated value is unknown")?;
                if matches!(actual.kind, s::ValueKind::Call { .. })
                    && evaluated[value as usize].replace(statement.site).is_some()
                {
                    return Err("source Call has duplicate actual evaluation ownership");
                }
            }
        }
        for (index, request) in original.calls.iter().enumerate() {
            Self::site(original, request.site, work)?;
            let value = original
                .values
                .get(request.value as usize)
                .ok_or("source request value is unknown")?;
            let s::ValueKind::Call {
                signature,
                callee,
                arguments,
            } = value.kind
            else {
                return Err("source request does not belong to a Call value");
            };
            if requested[request.value as usize]
                .replace(u32::try_from(index).map_err(|_| "source Call request index overflow")?)
                .is_some()
                || evaluated[request.value as usize] != Some(request.site)
                || signature != request.signature
                || request.effect != original.context.request()
            {
                return Err(
                    "source request changed its exact value/site/signature/effect ownership",
                );
            }
            self.call(context, original, request, callee, arguments, work)?;
        }
        for (index, value) in original.values.iter().enumerate() {
            work.charge(1)?;
            if matches!(value.kind, s::ValueKind::Call { .. }) && requested[index].is_none() {
                return Err("source Call has no complete retained request");
            }
        }
        Ok(())
    }
    fn call(
        &mut self,
        context: Context,
        original: &s::Original,
        request: &s::CallRequest,
        callee: u32,
        arguments: g::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = self.headers.rows();
        let callee = original
            .values
            .get(callee as usize)
            .ok_or("source Call callee is unknown")?;
        if callee.kind != s::ValueKind::Function(request.target)
            || rows.types.get(callee.ty.0 as usize) != Some(&s::Type::FunctionItem(request.target))
        {
            return Err("source direct Call lost its actual function/application operand");
        }
        self.function_use(context, request.target, work)?;
        let application = &rows.function_applications[request.target.0 as usize];
        let header = self.function(application.declaration.declaration, work)?;
        let Some(s::Type::Function(signature)) = rows.types.get(request.signature.0 as usize)
        else {
            return Err("source Call request signature has a different type kind");
        };
        let frame = &rows.binders[signature.binder.0 as usize];
        match frame.kind {
            s::BinderKind::CallSignature { body, .. } if body == original.body => {}
            s::BinderKind::CallableType if frame.parent.is_none() => {}
            _ => return Err("source Call signature belongs to another actual original context"),
        }
        if signature.unsafe_ != header.callable.unsafe_ {
            return Err("source Call changed its actual callable safety");
        }
        match (signature.unsafe_, request.safety) {
            (false, s::CallSafety::Safe) => {}
            (true, s::CallSafety::ExplicitUnsafeBlock | s::CallSafety::UnsafeFunction) => {
                let caller = self.function(original.declaration, work)?;
                if header.parent.is_some() || caller.parent.is_some() {
                    return Err(
                        "source unsafe associated Calls require their wider actual profile",
                    );
                }
                if request.safety == s::CallSafety::UnsafeFunction && !caller.callable.unsafe_ {
                    return Err(
                        "source unsafe-function Call changed its actual caller declaration",
                    );
                }
            }
            (true, s::CallSafety::Safe) => {
                return Err("source unsafe Call omitted its actual lexical context");
            }
            (false, _) => return Err("source safe Call cannot claim an unsafe obligation"),
        }
        if signature.parameters.slice(&rows.roots)?.len()
            != arguments.slice(&original.arguments)?.len()
        {
            return Err("source Call argument count differs from its actual request signature");
        }
        self.opening(context, original, request, header, work)?;
        self.method(context, request, header, application, work)
    }
    fn opening(
        &self,
        context: Context,
        original: &s::Original,
        request: &s::CallRequest,
        target: &s::FunctionHeader,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = self.headers.rows();
        let count = rows.binders[target.callable.binder.0 as usize].count;
        match request.opening {
            None if count == 0 => Ok(()),
            Some(opening) if count > 0 && opening.binder == target.callable.binder => {
                let opened = opening.regions.slice(&original.openings)?;
                if opened.len() != count as usize {
                    return Err("source Call opening omitted actual late-bound formals");
                }
                for &region in opened {
                    Types::region(self.headers, &self.binders, context, region, work)?;
                    if !matches!(rows.regions[region.0 as usize], s::Region::Variable { body, .. } if body == original.body)
                    {
                        return Err(
                            "source Call opening did not retain its actual fresh original allocation",
                        );
                    }
                }
                Ok(())
            }
            _ => Err("source Call opening changed its exact target binder identity"),
        }
    }
    fn method(
        &mut self,
        context: Context,
        request: &s::CallRequest,
        target: &s::FunctionHeader,
        application: &s::FunctionApplication,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = self.headers.rows();
        match (target.parent, request.method) {
            (Some(parent), Some(method)) => {
                let parent_header = self.headers.header(parent, work)?;
                if let Some(HeaderRef::Implementation(index)) = parent_header {
                    let implementation = &rows.implementations[index as usize];
                    if implementation.trait_application != Some(method) {
                        return Err(
                            "source selected method lost its exact implementation trait request",
                        );
                    }
                    self.trait_use(context, method, work)?;
                    return Ok(());
                }
                let Some(HeaderRef::Trait(index)) = parent_header else {
                    return Err(
                        "source selected impl/inherent requests require complete wider transport",
                    );
                };
                self.trait_use(context, method, work)?;
                let method = &rows.trait_applications[method.0 as usize];
                let trait_ = &rows.traits[index as usize];
                let Some(s::Type::Parameter(slot)) = rows.types.get(trait_.self_type.0 as usize)
                else {
                    return Err("source method has no actual trait Self slot");
                };
                let position =
                    self.headers
                        .scopes()
                        .type_argument_position(target.scope, *slot, work)?;
                let arguments = application.arguments.slice(&rows.arguments)?;
                if method.declaration
                    != (s::DeclarationAddress {
                        artifact: s::ArtifactRef(0),
                        declaration: parent,
                    })
                    || arguments.get(position as usize) != Some(&s::Argument::Type(method.subject))
                {
                    return Err(
                        "source method request lost its exact declared Self argument mapping",
                    );
                }
                Ok(())
            }
            (None, None) => Ok(()),
            _ => Err("source Call omitted or fabricated its actual associated method request"),
        }
    }
}
