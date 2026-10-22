//! Exact outer constructors, before const arguments or field graphs are evaluated.
use crate::sema::traits::heads::Head as Rigid;
use crate::support::hash::HashMap;
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Target},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    syntax::{
        ast::{Kind, TypeFlavor},
        item::{GenericFlavor, ItemKind},
    },
    types::{ItemId, Primitive},
};
#[cfg(test)]
#[path = "../../../tests/invariants/header_heads.rs"]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Head {
    Known(Rigid),
    Parameter(DefId),
    Other,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Query {
    Alias(DefId),
    Node(NodeKey),
    Argument(NodeKey, DefId),
}
#[derive(Clone, Copy)]
enum State {
    Active,
    Ready(Head),
}
#[derive(Clone, Copy)]
struct Alias {
    source: NodeKey,
    body: NodeKey,
    lifetimes: usize,
}
#[derive(Clone, Copy)]
struct Parameter {
    owner: DefId,
    slot: usize,
    default: Option<NodeKey>,
    type_: bool,
}
enum Work {
    Enter(Query),
    Forward(Query, Query),
    Apply(Query, NodeKey, DefId),
    Default(Query, NodeKey, DefId, Query),
}
pub(super) struct Heads<'a> {
    report: &'a Report,
    facts: &'a Declarations,
    aliases: HashMap<DefId, Alias>,
    parameters: HashMap<DefId, Parameter>,
    states: HashMap<Query, State>,
    work: Vec<Work>,
    steps: usize,
}
impl<'a> Heads<'a> {
    pub(super) fn new(report: &'a Report, facts: &'a Declarations) -> Self {
        Self {
            report,
            facts,
            aliases: HashMap::default(),
            parameters: HashMap::default(),
            states: HashMap::default(),
            work: Vec::new(),
            steps: 0,
        }
    }
    pub(super) fn target(&mut self, key: NodeKey) -> Result<Option<ItemId>, Error> {
        Ok(match self.shape(key)? {
            Some(Rigid::Nominal(item)) => Some(item),
            _ => None,
        })
    }
    /// Only exact resolved outer constructors exclude pending candidates.
    pub(super) fn shape(&mut self, key: NodeKey) -> Result<Option<Rigid>, Error> {
        Ok(match self.demand(key)? {
            Head::Known(head) => Some(head),
            Head::Parameter(_) | Head::Other => None,
        })
    }
    fn demand(&mut self, key: NodeKey) -> Result<Head, Error> {
        let query = Query::Node(key);
        self.work.push(Work::Enter(query));
        while let Some(work) = self.work.pop() {
            self.steps += 1;
            if self.steps > 1_000_000 {
                return Err(self.error(key, "nominal head work budget exceeded"));
            }
            match work {
                Work::Enter(query) => self.enter(query)?,
                Work::Forward(query, dependency) => {
                    self.complete(query, self.value(dependency)?);
                }
                Work::Apply(query, path, owner) => match self.value(Query::Alias(owner))? {
                    Head::Parameter(binding) => self.forward(query, Query::Argument(path, binding)),
                    head => self.complete(query, head),
                },
                Work::Default(query, path, owner, dependency) => {
                    let head = self.value(dependency)?;
                    if let Head::Parameter(binding) = head
                        && self
                            .parameters
                            .get(&binding)
                            .is_some_and(|p| p.owner == owner)
                    {
                        self.forward(query, Query::Argument(path, binding));
                    } else {
                        self.complete(query, head);
                    }
                }
            }
        }
        self.value(query)
    }
    fn error(&self, key: NodeKey, message: &'static str) -> Error {
        Declarations::error(self.report, key, message)
    }
    fn key(&self, query: Query) -> Result<NodeKey, Error> {
        match query {
            Query::Node(key) | Query::Argument(key, _) => Ok(key),
            Query::Alias(id) => self.report.resolution.definitions[id.0 as usize]
                .node
                .ok_or_else(|| {
                    Error::at(crate::resolve::unit::FileId(0), 0, "missing alias source")
                }),
        }
    }
    fn value(&self, query: Query) -> Result<Head, Error> {
        if let Some(State::Ready(head)) = self.states.get(&query) {
            Ok(*head)
        } else {
            Err(self.error(self.key(query)?, "missing completed nominal head"))
        }
    }
    fn complete(&mut self, query: Query, head: Head) {
        self.states.insert(query, State::Ready(head));
    }
    fn forward(&mut self, query: Query, dependency: Query) {
        self.work.push(Work::Forward(query, dependency));
        self.work.push(Work::Enter(dependency));
    }
    fn enter(&mut self, query: Query) -> Result<(), Error> {
        match self.states.get(&query) {
            Some(State::Ready(_)) => return Ok(()),
            Some(State::Active) => {
                return Err(self.error(
                    self.key(query)?,
                    "cycle in nominal head or default dependencies",
                ));
            }
            None => {}
        }
        self.states.insert(query, State::Active);
        match query {
            Query::Alias(id) => {
                let alias = self.alias(id)?;
                self.forward(query, Query::Node(alias.body));
            }
            Query::Node(key) => self.node(key),
            Query::Argument(path, binding) => self.argument(path, binding)?,
        }
        Ok(())
    }
    fn node(&mut self, key: NodeKey) {
        let query = Query::Node(key);
        let ast = &self.report.package.unit(key.file).configured.ast;
        let node = match ast.node(key.node).kind {
            Kind::Group(node) | Kind::Attributed { value: node, .. } => Some(node),
            Kind::Type {
                flavor: TypeFlavor::Path,
                elements,
            } => ast.list(elements).first().copied(),
            _ => None,
        };
        if let Some(node) = node {
            self.forward(query, Query::Node(NodeKey { node, ..key }));
            return;
        }
        if let Some(head) = Self::syntax_shape(ast.node(key.node).kind) {
            self.complete(query, Head::Known(head));
            return;
        }
        let Some(Target::Definition(id)) = self.facts.node(key).and_then(|f| f.reference) else {
            self.complete(query, Head::Other);
            return;
        };
        let head = match self.report.resolution.definitions[id.0 as usize].kind {
            DefKind::Struct | DefKind::Enum => Head::Known(Rigid::Nominal(ItemId(id.0))),
            DefKind::Primitive => Primitive::ALL
                .into_iter()
                .find(|p| {
                    p.name()
                        == self
                            .report
                            .package
                            .names
                            .get(self.report.resolution.definitions[id.0 as usize].name)
                })
                .map_or(Head::Other, |p| Head::Known(Rigid::Primitive(p))),
            DefKind::TypeAlias if self.facts.associated.owner(id).is_none() => {
                self.work.push(Work::Apply(query, key, id));
                self.work.push(Work::Enter(Query::Alias(id)));
                return;
            }
            DefKind::Generic if self.parameters.get(&id).is_some_and(|p| p.type_) => {
                Head::Parameter(id)
            }
            _ => Head::Other,
        };
        self.complete(query, head);
    }
    fn syntax_shape(kind: Kind) -> Option<Rigid> {
        Some(match kind {
            Kind::Type { flavor, .. } => match flavor {
                TypeFlavor::Path => return None,
                TypeFlavor::Tuple => Rigid::Tuple,
                TypeFlavor::Slice => Rigid::Slice,
                TypeFlavor::Array => Rigid::Array,
                TypeFlavor::Ref => Rigid::Reference(false),
                TypeFlavor::RefMut => Rigid::Reference(true),
                TypeFlavor::PointerConst => Rigid::Pointer(false),
                TypeFlavor::PointerMut => Rigid::Pointer(true),
            },
            Kind::FunctionType(_) => Rigid::Function,
            Kind::NeverType => Rigid::Never,
            _ => return None,
        })
    }
    fn alias(&mut self, id: DefId) -> Result<Alias, Error> {
        if let Some(alias) = self.aliases.get(&id) {
            return Ok(*alias);
        }
        let source = self.key(Query::Alias(id))?;
        let ast = &self.report.package.unit(source.file).configured.ast;
        let Kind::Item(index) = ast.node(source.node).kind else {
            return Err(self.error(source, "missing alias head item"));
        };
        let item = ast.items[index as usize];
        let ItemKind::TypeAlias { ty: Some(node), .. } = item.kind else {
            return Err(self.error(source, "missing alias head template"));
        };
        let unit = self.report.package.unit(source.file);
        let mut lifetimes = 0;
        let mut slot = 0;
        for &node in unit.configured.ast.list(item.generics) {
            if !unit.configured.active[node.get() as usize] {
                continue;
            }
            let mut key = NodeKey { node, ..source };
            while let Kind::Attributed { value, .. } = unit.configured.ast.node(key.node).kind {
                key.node = value;
            }
            let Kind::Generic(index) = unit.configured.ast.node(key.node).kind else {
                return Err(self.error(key, "missing alias head parameter"));
            };
            let parameter = unit.configured.ast.generics[index as usize];
            let binding = self
                .facts
                .node(key)
                .and_then(|f| f.binding)
                .ok_or_else(|| self.error(key, "missing resolved alias head parameter"))?;
            lifetimes += usize::from(parameter.flavor == GenericFlavor::Lifetime);
            self.parameters.insert(
                binding,
                Parameter {
                    owner: id,
                    slot,
                    default: parameter.default.map(|node| NodeKey { node, ..key }),
                    type_: parameter.flavor == GenericFlavor::Type,
                },
            );
            slot += 1;
        }
        let alias = Alias {
            source,
            body: NodeKey { node, ..source },
            lifetimes,
        };
        self.aliases.insert(id, alias);
        Ok(alias)
    }
    fn argument(&mut self, path: NodeKey, binding: DefId) -> Result<(), Error> {
        let query = Query::Argument(path, binding);
        let parameter = *self
            .parameters
            .get(&binding)
            .ok_or_else(|| self.error(path, "missing nominal head parameter"))?;
        let alias = self.alias(parameter.owner)?;
        let list = crate::sema::lifetimes::arguments(self.report, path, false)?;
        let ast = &self.report.package.unit(path.file).configured.ast;
        let supplied = list.map_or(&[][..], |list| ast.list(list));
        let explicit = supplied.first().is_some_and(|&node| {
            matches!(ast.node(node).kind, Kind::Name(span)
                if self.report.package.unit(path.file).spelling(span).is_ok_and(|s| s.starts_with('\'')))
        });
        let omitted = if explicit { 0 } else { alias.lifetimes };
        if let Some(&node) = parameter
            .slot
            .checked_sub(omitted)
            .and_then(|i| supplied.get(i))
        {
            // Supplied arguments are simultaneous: an outer occurrence of the
            // same binding must not be substituted by this application twice.
            self.forward(query, Query::Node(NodeKey { node, ..path }));
        } else if let Some(default) = parameter.default {
            self.work.push(Work::Default(
                query,
                path,
                parameter.owner,
                Query::Node(default),
            ));
            self.work.push(Work::Enter(Query::Node(default)));
        } else {
            return Err(self.error(alias.source, "missing required generic argument"));
        }
        Ok(())
    }
}
