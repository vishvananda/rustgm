//! Full applied signatures share structure; each Call still supplies its own context.
use super::{
    super::Target,
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
    flags: Flags,
}
/// The safety, convention and variadic flags two signatures must share.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Flags {
    unsafe_: bool,
    convention: g::Convention,
    variadic: bool,
}
impl Flags {
    /// A called constructor is a safe, non-variadic Rust function.
    const CONSTRUCTOR: Self = Self {
        unsafe_: false,
        convention: g::Convention::Rust,
        variadic: false,
    };
    const fn of(signature: g::Signature) -> Self {
        Self {
            unsafe_: signature.unsafe_,
            convention: signature.convention,
            variadic: signature.variadic,
        }
    }
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
            if s::operation_contracts::function_signatures(rows, domains.headers, header, work)? {
                continue;
            }
            same_flags(Flags::of(header.original), Flags::of(header.callable))?;
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
                frame: Some(rows.body_inputs[domains.headers.original_index(original.body)?].frame),
                body: Some(original.body),
            };
            for (ordinal, request) in original.calls.iter().enumerate() {
                work.charge(1)?;
                let template = self.template(domains, request.target, caller, work)?;
                let Some(s::Type::Function(actual)) = rows.types.get(request.signature.0 as usize)
                else {
                    return Err("source Call has no actual function signature");
                };
                same_flags(template.flags, Flags::of(*actual))?;
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
                self.method_goal(domains, request, caller, opening, work)?;
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
        // A called constructor's inputs are its payload and its result is its
        // enum or struct, both in that nominal's own frame.
        let (parameters, result, flags) = match domains.target(raw.declaration.declaration, work)? {
            Target::Function(header) => (
                Parameters::Roots(header.callable.parameters.slice(&rows.roots)?),
                header.callable.result,
                Flags::of(header.callable),
            ),
            Target::Constructor {
                enumeration,
                payload,
            } => (
                Parameters::Payload(payload),
                enumeration.self_type,
                Flags::CONSTRUCTOR,
            ),
        };
        self.reserve(
            retained::new_map::<Key, Template>(1)
                .saturating_add(parameters.len().saturating_mul(4 * size_of::<NodeId>())),
            0,
            work,
        )?;
        let start = u32::try_from(self.signature_roots.len())
            .map_err(|_| "source applied signature-root index overflow")?;
        for index in 0..parameters.len() {
            let use_ = uses::key(
                domains,
                parameters.get(index)?,
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
            result,
            Rewrite::Application(application),
            key.caller,
            work,
        )?;
        let result = self.root(domains, use_, work)?;
        self.application_inputs(domains, application, key.caller, work)?;
        let template = Template {
            roots,
            result,
            flags,
        };
        self.templates.insert(key, template);
        Ok(template)
    }
}
/// A target's input recipes: a function's signature roots or a constructor's
/// payload.
#[derive(Clone, Copy)]
enum Parameters<'rows> {
    Roots(&'rows [g::TypeRef]),
    Payload(s::ConstructorPayload<'rows>),
}
impl Parameters<'_> {
    const fn len(self) -> usize {
        match self {
            Self::Roots(roots) => roots.len(),
            Self::Payload(fields) => fields.len(),
        }
    }
    fn get(self, index: usize) -> Result<g::TypeRef, &'static str> {
        match self {
            Self::Roots(roots) => roots.get(index).copied(),
            Self::Payload(fields) => fields.ty(index),
        }
        .ok_or("source applied signature input is outside its target")
    }
}
fn same_flags(expected: Flags, actual: Flags) -> Result<(), &'static str> {
    if expected != actual {
        return Err("source callable signature changed safety, convention or variadicness");
    }
    Ok(())
}
