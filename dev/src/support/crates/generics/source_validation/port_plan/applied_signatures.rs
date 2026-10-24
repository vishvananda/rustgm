//! Full applied signatures share structure; each Call still supplies its own context.
use super::{
    Context, DomainIndex, Work, g,
    matching::Opening,
    nodes::{Arena, NodeId, Rewrite},
    s, uses,
};
use crate::support::retained;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Key {
    application: s::FunctionApplicationRef,
    caller: Option<Context>,
}
#[derive(Clone, Copy)]
pub(super) struct Template {
    roots: g::Range,
    result: NodeId,
    signature: g::Signature,
}
impl Arena {
    pub(super) fn signatures(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = domains.headers.rows();
        for header in &rows.functions {
            work.charge(1)?;
            same_flags(header.original, header.callable)?;
            let original = header.original.parameters.slice(&rows.roots)?;
            let public = header.callable.parameters.slice(&rows.roots)?;
            if original.len() != public.len() {
                return Err("source public signature changed the complete original parameter list");
            }
            let expected = Context {
                owner: header.scope,
                frame: Some(header.original.binder),
                body: None,
            };
            let actual = Context {
                frame: Some(header.callable.binder),
                ..expected
            };
            for (&left, &right) in original.iter().zip(public).chain(std::iter::once((
                &header.original.result,
                &header.callable.result,
            ))) {
                let left = uses::key(
                    domains,
                    left,
                    Rewrite::Public(header.declaration),
                    Some(expected),
                    work,
                )?;
                let right = uses::key(domains, right, Rewrite::Original, Some(actual), work)?;
                let left = self.root(domains, left, work)?;
                let right = self.root(domains, right, work)?;
                self.same(
                    domains,
                    left,
                    right,
                    Opening::Header(header.declaration),
                    work,
                )?;
            }
        }
        self.call_signatures(domains, work)
    }
    fn call_signatures(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = domains.headers.rows();
        for original in &rows.originals {
            let caller = Context {
                owner: original.declaration,
                frame: Some(rows.body_inputs[original.body.0 as usize].frame),
                body: Some(original.body),
            };
            for (ordinal, request) in original.calls.iter().enumerate() {
                work.charge(1)?;
                let template = self.template(domains, request.target, caller, work)?;
                let Some(s::Type::Function(actual)) = rows.types.get(request.signature.0 as usize)
                else {
                    return Err("source Call has no actual function signature");
                };
                same_flags(template.signature, *actual)?;
                if rows.binders[actual.binder.0 as usize].count != 0 {
                    return Err("source opened Call retained an uninstantiated outer binder");
                }
                let expected = template.roots.slice(&self.signature_roots)?;
                let actuals = actual.parameters.slice(&rows.roots)?;
                if expected.len() != actuals.len() {
                    return Err("source opened signature changed its complete actual arity");
                }
                let opening = Opening::Call {
                    body: original.body,
                    request: u32::try_from(ordinal)
                        .map_err(|_| "source Call request ordinal overflow")?,
                };
                let context = Context {
                    frame: Some(actual.binder),
                    ..caller
                };
                for (index, &root) in actuals.iter().enumerate() {
                    let expected = self.signature_roots[template.roots.start as usize + index];
                    let key = uses::key(domains, root, Rewrite::Original, Some(context), work)?;
                    let actual = self.root(domains, key, work)?;
                    self.same(domains, expected, actual, opening, work)?;
                }
                let key = uses::key(
                    domains,
                    actual.result,
                    Rewrite::Original,
                    Some(context),
                    work,
                )?;
                let actual = self.root(domains, key, work)?;
                self.same(domains, template.result, actual, opening, work)?;
            }
        }
        Ok(())
    }
    fn template(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        application: s::FunctionApplicationRef,
        caller: Context,
        work: &mut Work,
    ) -> Result<Template, &'static str> {
        work.charge(1)?;
        let key = Key {
            application,
            caller: if domains.types.closed_application(application, work)? {
                None
            } else {
                Some(caller)
            },
        };
        if let Some(&template) = self.templates.get(&key) {
            self.hits = self.hits.saturating_add(1);
            return Ok(template);
        }
        let rows = domains.headers.rows();
        let raw = rows
            .function_applications
            .get(application.0 as usize)
            .ok_or("source applied signature has no actual application")?;
        let header = domains.function(raw.declaration.declaration, work)?;
        let parameters = header.callable.parameters.slice(&rows.roots)?;
        self.reserve(
            retained::new_map::<Key, Template>(1)
                .saturating_add(parameters.len().saturating_mul(4 * size_of::<NodeId>())),
            0,
            work,
        )?;
        let start = u32::try_from(self.signature_roots.len())
            .map_err(|_| "source applied signature-root index overflow")?;
        for &root in parameters {
            let use_ = uses::key(
                domains,
                root,
                Rewrite::Application(application),
                key.caller,
                work,
            )?;
            let node = self.root(domains, use_, work)?;
            self.signature_roots.push(node);
        }
        let roots = g::Range {
            start,
            end: u32::try_from(self.signature_roots.len())
                .map_err(|_| "source applied signature-root index overflow")?,
        };
        let use_ = uses::key(
            domains,
            header.callable.result,
            Rewrite::Application(application),
            key.caller,
            work,
        )?;
        let result = self.root(domains, use_, work)?;
        self.application_inputs(domains, application, key.caller, work)?;
        let template = Template {
            roots,
            result,
            signature: header.callable,
        };
        self.templates.insert(key, template);
        Ok(template)
    }
}
fn same_flags(expected: g::Signature, actual: g::Signature) -> Result<(), &'static str> {
    if expected.unsafe_ != actual.unsafe_
        || expected.convention != actual.convention
        || expected.variadic != actual.variadic
    {
        return Err("source callable signature changed safety, convention or variadicness");
    }
    Ok(())
}
