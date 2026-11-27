//! An array or slice can depend on Sized in its actual declaration environment.
use super::{HeaderIndex, HeaderRef, Types, Work, g, row_edges::Edges, s};
use crate::support::{bindings::LangItem, crates::owner_catalog::DeclarationRef, hash::HashSet};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum Obligation {
    #[default]
    None,
    Parameter(s::SlotRef),
    Multiple,
}
impl Obligation {
    pub(super) fn union(self, other: Self) -> Self {
        match (self, other) {
            (Self::None, other) | (other, Self::None) => other,
            _ if self == other => self,
            _ => Self::Multiple,
        }
    }
}

impl Types {
    pub(super) fn check_sizing(
        &self,
        headers: &HeaderIndex<'_>,
        owner: DeclarationRef,
        index: usize,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        match self.summaries[index].sizing {
            Obligation::None => Ok(()),
            Obligation::Parameter(slot) => require(headers, owner, slot, work),
            Obligation::Multiple => self.check_multiple_sizes(headers, owner, index, work),
        }
    }

    fn check_multiple_sizes(
        &self,
        headers: &HeaderIndex<'_>,
        owner: DeclarationRef,
        index: usize,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let edges = Edges::new(headers.rows(), self.declaration_policies);
        let mut visited = HashSet::default();
        let mut proven = HashSet::default();
        let mut pending = vec![index];
        while let Some(node) = pending.pop() {
            work.charge(1)?;
            if !visited.insert(node) {
                continue;
            }
            match self.summaries[node].sizing {
                Obligation::None => {}
                Obligation::Parameter(slot) => {
                    if proven.insert(slot) {
                        require(headers, owner, slot, work)?;
                    }
                }
                Obligation::Multiple => {
                    edges.visit(node, work, |child, _| {
                        if self.summaries[child].sizing != Obligation::None {
                            pending.push(child);
                        }
                        Ok(())
                    })?;
                }
            }
        }
        Ok(())
    }
}

fn require(
    headers: &HeaderIndex<'_>,
    owner: DeclarationRef,
    slot: s::SlotRef,
    work: &mut Work,
) -> Result<(), &'static str> {
    let rows = headers.rows();
    let mut proof = Proof {
        sized: None,
        pending: Vec::new(),
    };
    for role in rows.policy.roles.slice(&rows.roles)? {
        work.charge(1)?;
        if role.role == LangItem::Sized && role.declaration.artifact == rows.policy.provider {
            proof.sized = Some(role.declaration);
            break;
        }
    }
    let mut scope = Some(owner);
    while let Some(owner) = scope {
        if let Some((inputs, count)) = declared_inputs(headers, owner, work)?
            && proof.inputs(rows, inputs, count, slot, work)?
        {
            return Ok(());
        }
        scope = headers.scopes().parent(owner, work)?;
    }
    if proof.supertraits(headers, work)? {
        return Ok(());
    }
    Err("source array or slice element lacks its declared Sized requirement")
}

fn declared_inputs(
    headers: &HeaderIndex<'_>,
    owner: DeclarationRef,
    work: &mut Work,
) -> Result<Option<(g::Range, u32)>, &'static str> {
    let rows = headers.rows();
    Ok(Some(match headers.header(owner, work)? {
        Some(HeaderRef::Function(index)) => {
            let header = rows.functions[index as usize];
            (header.inputs, header.declared_inputs)
        }
        Some(HeaderRef::Trait(index)) => {
            let header = rows.traits[index as usize];
            (header.inputs, header.canonical_inputs)
        }
        Some(HeaderRef::Implementation(index)) => {
            let header = rows.implementations[index as usize];
            (header.inputs, header.declared_inputs)
        }
        Some(HeaderRef::Nominal(index)) => {
            let header = &rows.nominals[index as usize];
            (header.inputs, header.declared_inputs)
        }
        _ => return Ok(None),
    }))
}

struct Proof {
    sized: Option<s::DeclarationAddress>,
    pending: Vec<DeclarationRef>,
}
impl Proof {
    fn inputs(
        &mut self,
        rows: &s::SourceRows,
        range: g::Range,
        count: u32,
        slot: s::SlotRef,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        let clauses = range
            .slice(&rows.clauses)?
            .get(..count as usize)
            .ok_or("source declared Sized inputs exceed their actual header range")?;
        let subject = s::Type::Parameter(slot);
        for clause in clauses {
            work.charge(1)?;
            if clause.binder.is_some() {
                continue;
            }
            match clause.requirement {
                s::Requirement::Bootstrap {
                    subject: ty,
                    bound: s::BootstrapBound::Sized,
                } if rows.types.get(ty.0 as usize) == Some(&subject) => return Ok(true),
                s::Requirement::Trait { application, .. } => {
                    let app = rows
                        .trait_applications
                        .get(application.0 as usize)
                        .ok_or("source Sized clause lost its actual trait application")?;
                    if rows.types.get(app.subject.0 as usize) != Some(&subject) {
                        continue;
                    }
                    if Some(app.declaration) == self.sized {
                        return Ok(true);
                    }
                    if app.declaration.artifact == rows.policy.provider {
                        self.pending.push(app.declaration.declaration);
                    }
                }
                _ => {}
            }
        }
        Ok(false)
    }

    fn supertraits(
        &mut self,
        headers: &HeaderIndex<'_>,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        let rows = headers.rows();
        let mut visited = HashSet::default();
        while let Some(owner) = self.pending.pop() {
            work.charge(1)?;
            if !visited.insert(owner) {
                continue;
            }
            let Some(HeaderRef::Trait(index)) = headers.header(owner, work)? else {
                continue;
            };
            let header = rows.traits[index as usize];
            let Some(&s::Type::Parameter(slot)) = rows.types.get(header.self_type.0 as usize)
            else {
                return Err("source Sized supertrait lost its actual Self parameter");
            };
            if self.inputs(rows, header.inputs, header.canonical_inputs, slot, work)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
