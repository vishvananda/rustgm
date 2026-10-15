//! Shared partial-type inference for bodies and isolated method-prefix queries.
mod applications;
mod requirements;
use super::{declarations, model::Declarations};
use crate::{
    resolve::{expansion::Report, model::Target, package::Error, unit::NodeKey},
    syntax::ast::{Kind, TypeFlavor},
    types::{
        TypeId,
        infer::{Constructor, Context, Domain, TermId},
        lifetime_proof::ScopeId,
    },
};
pub(super) use applications::Fields;

pub(super) trait Host {
    fn report(&self) -> &Report;
    fn facts(&self) -> &Declarations;
    fn inference(&mut self) -> (&mut Declarations, &mut Context);
    fn cached(&self, key: NodeKey) -> Option<TermId>;
    fn save(&mut self, key: NodeKey, term: TermId);
    fn nominal(&mut self, term: TermId, fields: Fields);
    fn requirement(&mut self, key: NodeKey, predicate: crate::types::infer::predicates::Predicate);
    fn callable(&mut self, key: NodeKey);
    fn child(&mut self, key: NodeKey);
    fn scope(&mut self, key: NodeKey, parent: ScopeId, count: u32) -> Result<ScopeId, Error>;
    fn deferred(
        &mut self,
        key: NodeKey,
        scope: ScopeId,
        expansion: super::alias_requirements::Expansion,
        supplied: &[super::generics::AnnotationArgument],
        fields: &Fields,
    ) -> Result<(), Error>;
    fn normalization(&mut self, key: NodeKey, scope: ScopeId, term: TermId) -> Result<(), Error>;
}

#[derive(Default)]
pub(super) struct Scratch {
    work: Vec<(NodeKey, bool, ScopeId)>,
    children: Vec<TermId>,
}
impl Scratch {
    pub(super) fn infer(&mut self, host: &mut impl Host, root: NodeKey) -> Result<TermId, Error> {
        self.work.push((root, false, ScopeId::ROOT));
        while let Some((key, finish, scope)) = self.work.pop() {
            if host.cached(key).is_some() {
                continue;
            }
            let term = if let Some(ty) = host.facts().node(key).and_then(|f| f.ty) {
                known(host, key, ty)?
            } else if finish {
                self.finish(host, key, scope)?
            } else {
                self.work.push((key, true, scope));
                let scope = match host.facts().partial_annotations.get(&key) {
                    Some(super::model::PartialAnnotation::Function { bound_regions }) => {
                        host.scope(key, scope, *bound_regions)?
                    }
                    _ => scope,
                };
                self.dependencies(host, key, scope)?;
                continue;
            };
            host.save(key, term);
            if key != root {
                host.child(key);
            }
        }
        term(host, root)
    }

    fn dependencies(
        &mut self,
        host: &impl Host,
        key: NodeKey,
        scope: ScopeId,
    ) -> Result<(), Error> {
        let unit = host.report().package.unit(key.file);
        let mut push = |node| self.work.push((NodeKey { node, ..key }, false, scope));
        match Declarations::kind(host.report(), key) {
            Kind::Group(value) | Kind::Attributed { value, .. } => push(value),
            Kind::Type { flavor, elements } => {
                let elements = unit.configured.ast.list(elements);
                if matches!(flavor, TypeFlavor::Ref | TypeFlavor::RefMut) {
                    push(*elements.last().ok_or_else(|| {
                        error(host, key, "missing reference annotation referent")
                    })?);
                } else {
                    let count = elements
                        .len()
                        .saturating_sub(usize::from(flavor == TypeFlavor::Array));
                    for &node in elements[..count].iter().rev() {
                        push(node);
                    }
                }
            }
            Kind::FunctionType(index) => {
                let function = unit.configured.ast.function_types[index as usize];
                if let Some(result) = function.result {
                    push(result);
                }
                for &parameter in unit.configured.ast.list(function.parameters).iter().rev() {
                    if unit.configured.active[parameter.0 as usize] {
                        push(declarations::parameter_type(
                            host.report(),
                            NodeKey {
                                node: parameter,
                                ..key
                            },
                        )?);
                    }
                }
            }
            Kind::Path { .. } | Kind::Name(_) | Kind::Applied { .. } => {
                if let Some(use_) = host.facts().generics.annotation_uses.get(&key) {
                    for argument in use_.arguments.iter().rev() {
                        if let super::generics::AnnotationArgument::Type(key) = argument {
                            self.work.push((*key, false, scope));
                        }
                    }
                }
            }
            Kind::InferType | Kind::NeverType => {}
            _ => return Err(error(host, key, "unsupported local type annotation")),
        }
        Ok(())
    }

    fn finish(
        &mut self,
        host: &mut impl Host,
        key: NodeKey,
        scope: ScopeId,
    ) -> Result<TermId, Error> {
        let constructor = match Declarations::kind(host.report(), key) {
            Kind::InferType => {
                return host
                    .inference()
                    .1
                    .variable(Domain::Any)
                    .map_err(|e| error(host, key, e));
            }
            Kind::NeverType => return known(host, key, TypeId::NEVER),
            Kind::Group(node) | Kind::Attributed { value: node, .. } => {
                return term(host, NodeKey { node, ..key });
            }
            Kind::Path { .. } | Kind::Name(_) | Kind::Applied { .. } => {
                if host.facts().generics.annotation_uses.contains_key(&key) {
                    return applications::infer(host, key, scope);
                }
                let Some(Target::Definition(id)) = host.facts().node(key).and_then(|f| f.reference)
                else {
                    return Err(error(host, key, "missing annotation resolution"));
                };
                let ty = host.facts().definitions[id.0 as usize]
                    .ok_or_else(|| error(host, key, "missing resolved annotation type"))?;
                return known(host, key, ty);
            }
            Kind::Type { flavor, elements } => {
                self.children.clear();
                let children = host
                    .report()
                    .package
                    .unit(key.file)
                    .configured
                    .ast
                    .list(elements);
                let children = if matches!(flavor, TypeFlavor::Ref | TypeFlavor::RefMut) {
                    children
                        .last()
                        .map(std::slice::from_ref)
                        .ok_or_else(|| error(host, key, "missing inferred reference referent"))?
                } else {
                    children
                };
                let count = children
                    .len()
                    .saturating_sub(usize::from(flavor == TypeFlavor::Array));
                for &node in &children[..count] {
                    self.children.push(term(host, NodeKey { node, ..key })?);
                }
                match flavor {
                    TypeFlavor::Path => {
                        return self
                            .children
                            .first()
                            .copied()
                            .ok_or_else(|| error(host, key, "missing type path"));
                    }
                    TypeFlavor::Tuple => Constructor::Tuple,
                    TypeFlavor::Slice => Constructor::Slice,
                    TypeFlavor::Array => Constructor::Array(declarations::array_length(
                        host.report(),
                        NodeKey {
                            node: *children
                                .get(1)
                                .ok_or_else(|| error(host, key, "missing array length"))?,
                            ..key
                        },
                    )?),
                    TypeFlavor::PointerConst | TypeFlavor::PointerMut => Constructor::Pointer {
                        mutable: flavor == TypeFlavor::PointerMut,
                    },
                    TypeFlavor::Ref | TypeFlavor::RefMut => {
                        let Some(super::model::PartialAnnotation::Reference(region)) =
                            host.facts().partial_annotations.get(&key)
                        else {
                            return Err(error(host, key, "missing elaborated annotation region"));
                        };
                        Constructor::Reference {
                            mutable: flavor == TypeFlavor::RefMut,
                            region: *region,
                        }
                    }
                }
            }
            Kind::FunctionType(index) => self.function(host, key, index)?,
            _ => return Err(error(host, key, "invalid local annotation work")),
        };
        host.inference()
            .1
            .construct(constructor, &self.children)
            .map_err(|e| error(host, key, e))
    }

    fn function(
        &mut self,
        host: &mut impl Host,
        key: NodeKey,
        index: u32,
    ) -> Result<Constructor, Error> {
        self.children.clear();
        let unit = host.report().package.unit(key.file);
        let function = unit.configured.ast.function_types[index as usize];
        for &parameter in unit.configured.ast.list(function.parameters) {
            if unit.configured.active[parameter.0 as usize] {
                let node = declarations::parameter_type(
                    host.report(),
                    NodeKey {
                        node: parameter,
                        ..key
                    },
                )?;
                self.children.push(term(host, NodeKey { node, ..key })?);
            }
        }
        let parameters = u32::try_from(self.children.len())
            .map_err(|_| error(host, key, "too many function parameters"))?;
        let result = if let Some(node) = function.result {
            term(host, NodeKey { node, ..key })?
        } else {
            known(host, key, TypeId::UNIT)?
        };
        self.children.push(result);
        let Some(super::model::PartialAnnotation::Function { bound_regions }) =
            host.facts().partial_annotations.get(&key)
        else {
            return Err(error(host, key, "missing elaborated callable binder"));
        };
        let bound_regions = *bound_regions;
        host.callable(key);
        Ok(Constructor::Function {
            bound_regions,
            parameters,
            unsafe_: function.unsafe_,
        })
    }
}
fn error(host: &impl Host, key: NodeKey, message: impl std::fmt::Display) -> Error {
    Declarations::error(host.report(), key, message.to_string())
}
fn term(host: &impl Host, key: NodeKey) -> Result<TermId, Error> {
    host.cached(key)
        .ok_or_else(|| error(host, key, "missing annotation constraint"))
}
fn known(host: &mut impl Host, key: NodeKey, ty: TypeId) -> Result<TermId, Error> {
    let (facts, infer) = host.inference();
    infer
        .known(&facts.types, ty)
        .map_err(|e| error(host, key, e))
}
