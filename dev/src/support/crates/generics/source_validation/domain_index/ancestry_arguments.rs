//! Validate the sparse parent forest and the child's substitution for each parent.
use super::{Context, DomainIndex, HeaderRef, Work, s};
use crate::support::{crates::owner_catalog::DeclarationRef, polarity::ImplPolarity};

impl DomainIndex<'_, '_> {
    pub(super) fn specialization_roots(&mut self, work: &mut Work) -> Result<(), &'static str> {
        let rows = self.headers.rows();
        let Some(graph) = rows.cold().specialization.as_deref() else {
            return Ok(());
        };
        if !self.declaration_policies || graph.parents.is_empty() {
            return Err("source specialization requires checked parents and the extended profile");
        }
        let mut previous = None;
        for edge in &graph.parents {
            work.charge(1)?;
            if previous.is_some_and(|prior| prior >= edge.child.0) {
                return Err("source specialization children are duplicated or out of order");
            }
            previous = Some(edge.child.0);
            let (child, child_trait) = self.specialization_impl(edge.child, work)?;
            let (parent, parent_trait) = self.specialization_impl(edge.parent, work)?;
            if child_trait != parent_trait {
                return Err("source specialization parent implements a different trait");
            }
            self.specialization_arguments(*edge, child, parent, work)?;
        }
        self.scratch(
            graph.parents.len().saturating_mul(1 + size_of::<usize>()),
            work,
        )?;
        acyclic(&graph.parents, work)
    }

    fn specialization_impl(
        &self,
        owner: DeclarationRef,
        work: &mut Work,
    ) -> Result<(s::ImplHeader, s::DeclarationAddress), &'static str> {
        let rows = self.headers.rows();
        let Some(HeaderRef::Implementation(index)) = self.headers.header(owner, work)? else {
            return Err("source specialization endpoint has no implementation header");
        };
        let header = rows.implementations[index as usize];
        let application = header
            .trait_application
            .and_then(|application| rows.trait_applications.get(application.0 as usize));
        let Some(application) = application.filter(|_| header.polarity == ImplPolarity::Positive)
        else {
            return Err("source specialization endpoint is not a positive trait implementation");
        };
        Ok((header, application.declaration))
    }

    fn specialization_arguments(
        &mut self,
        edge: s::SpecializationParent,
        child: s::ImplHeader,
        parent: s::ImplHeader,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let rows = self.headers.rows();
        let context = Context {
            owner: child.scope,
            frame: Some(self.binders.declaration(child.self_definition, work)?),
            body: None,
        };
        let arguments = edge.arguments.slice(&rows.arguments)?;
        let (regions, arguments) = arguments
            .split_at_checked(parent.region_captures as usize)
            .ok_or("source specialization parent has too few region arguments")?;
        for &argument in regions {
            let s::Argument::Region(region) = argument else {
                return Err("source specialization parent changed its region argument kind");
            };
            self.types
                .region(self.headers, &self.binders, context, region, work)?;
        }
        let formals = self
            .headers
            .scopes()
            .declared_slots(parent.scope, work)?
            .slice(&rows.slots)?;
        work.charge(formals.len())?;
        let mut arguments = arguments.iter().copied();
        for formal in formals {
            if matches!(formal.kind, s::ParameterKind::Region) {
                continue;
            }
            let argument = arguments
                .next()
                .ok_or("source specialization parent changed its argument count")?;
            match (argument, formal.kind) {
                (s::Argument::Type(ty), s::ParameterKind::Type { .. }) => {
                    self.types.root(self.headers, context, ty, work)?;
                    if let Some(s::Type::FunctionItem(item)) = rows.types.get(ty.0 as usize) {
                        self.function_use(context, *item, work)?;
                    }
                }
                (s::Argument::Const(term), s::ParameterKind::Const { ty }) => self
                    .types
                    .const_parameter_root(self.headers, context, term, ty, work)?,
                _ => return Err("source specialization parent changed its parameter kind"),
            }
        }
        if arguments.next().is_some() {
            return Err("source specialization parent changed its argument count");
        }
        Ok(())
    }
}

fn acyclic(edges: &[s::SpecializationParent], work: &mut Work) -> Result<(), &'static str> {
    work.charge(edges.len())?;
    let mut colours = vec![0_u8; edges.len()];
    let mut path = Vec::with_capacity(edges.len());
    let lookup = edges.len().checked_ilog2().unwrap_or(0) as usize + 1;
    for root in 0..edges.len() {
        work.charge(1)?;
        let mut next = Some(root);
        while let Some(index) = next {
            work.charge(2)?;
            match colours[index] {
                2 => break,
                1 => return Err("source specialization parent cycle"),
                _ => {}
            }
            colours[index] = 1;
            path.push(index);
            work.charge(lookup)?;
            next = edges
                .binary_search_by_key(&edges[index].parent.0, |edge| edge.child.0)
                .ok();
        }
        for index in path.drain(..) {
            colours[index] = 2;
        }
    }
    Ok(())
}
