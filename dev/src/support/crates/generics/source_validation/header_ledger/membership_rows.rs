//! Associated members retain required/default/override ownership independently.
use super::{DefinitionKind, HeaderRef, SourceRows, Work, declaration, s};

pub(super) fn validate(
    rows: &SourceRows,
    headers: &[Option<HeaderRef>],
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(
        rows.declarations
            .len()
            .saturating_mul(2)
            .saturating_add(rows.members.len())
            .saturating_add(rows.member_links.len()),
    )?;
    let mut ledger = Members {
        rows,
        headers,
        traits: vec![false; rows.declarations.len()],
        stamps: vec![0_u32; rows.declarations.len()],
        listed: vec![false; rows.members.len()],
        links: vec![false; rows.member_links.len()],
    };
    ledger.traits(work)?;
    for (index, implementation) in rows.implementations.iter().enumerate() {
        ledger.implementation(index, implementation, work)?;
    }
    ledger.finish(work)
}
struct Members<'a> {
    rows: &'a SourceRows,
    headers: &'a [Option<HeaderRef>],
    traits: Vec<bool>,
    stamps: Vec<u32>,
    listed: Vec<bool>,
    links: Vec<bool>,
}
impl<'a> Members<'a> {
    fn traits(&mut self, work: &mut Work) -> Result<(), &'static str> {
        for trait_ in &self.rows.traits {
            for (offset, &member) in trait_.members.slice(&self.rows.members)?.iter().enumerate() {
                declaration(self.rows, member, DefinitionKind::Function, work)?;
                let Some(HeaderRef::Function(index)) = self.headers[member.0 as usize] else {
                    return Err("source trait member has no actual function header");
                };
                let function = &self.rows.functions[index as usize];
                if function.parent != Some(trait_.declaration)
                    || self.rows.declarations[member.0 as usize].definition.parent
                        != Some(trait_.declaration)
                    || std::mem::replace(&mut self.traits[member.0 as usize], true)
                    || std::mem::replace(
                        &mut self.listed[trait_.members.start as usize + offset],
                        true,
                    )
                {
                    return Err("source trait member has a duplicate or foreign associated owner");
                }
            }
        }
        Ok(())
    }
    fn implementation(
        &mut self,
        index: usize,
        implementation: &s::ImplHeader,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let stamp = u32::try_from(index)
            .ok()
            .and_then(|index| index.checked_add(1))
            .ok_or("source implementation member stamp overflow")?;
        let trait_ = self.application(implementation, work)?;
        for (offset, link) in implementation
            .members
            .slice(&self.rows.member_links)?
            .iter()
            .enumerate()
        {
            work.charge(1)?;
            if std::mem::replace(
                &mut self.links[implementation.members.start as usize + offset],
                true,
            ) {
                return Err("source implementation member mapping is repeated");
            }
            self.link(implementation, trait_, stamp, link, work)?;
        }
        Ok(())
    }
    fn application(
        &self,
        implementation: &s::ImplHeader,
        work: &mut Work,
    ) -> Result<Option<&'a s::TraitApplication>, &'static str> {
        work.charge(1)?;
        let trait_ = implementation
            .trait_application
            .map(|reference| {
                self.rows
                    .trait_applications
                    .get(reference.0 as usize)
                    .ok_or("source implementation trait application is unknown")
            })
            .transpose()?;
        if let Some(trait_) = trait_ {
            let target = if trait_.declaration.artifact == s::ArtifactRef(0) {
                self.headers
                    .get(trait_.declaration.declaration.0 as usize)
                    .copied()
                    .flatten()
            } else {
                None
            };
            let Some(HeaderRef::Trait(index)) = target else {
                return Err("source implementation requires its exact supported trait header");
            };
            if self.rows.traits[index as usize]
                .members
                .slice(&self.rows.members)?
                .len()
                != implementation.members.slice(&self.rows.member_links)?.len()
            {
                return Err("source implementation omits an actual trait member mapping");
            }
        }
        Ok(trait_)
    }
    fn link(
        &mut self,
        implementation: &s::ImplHeader,
        trait_: Option<&s::TraitApplication>,
        stamp: u32,
        link: &s::MemberLink,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        match (trait_, link.trait_member) {
            (Some(trait_), Some(member)) if member.artifact == s::ArtifactRef(0) => {
                declaration(
                    self.rows,
                    member.declaration,
                    DefinitionKind::Function,
                    work,
                )?;
                if trait_.declaration.artifact != s::ArtifactRef(0)
                    || self.rows.declarations[member.declaration.0 as usize]
                        .definition
                        .parent
                        != Some(trait_.declaration.declaration)
                    || !self.traits[member.declaration.0 as usize]
                    || std::mem::replace(&mut self.stamps[member.declaration.0 as usize], stamp)
                        == stamp
                {
                    return Err("source implementation maps a foreign actual trait member");
                }
                if link.implementation.is_none() {
                    let Some(HeaderRef::Function(index)) =
                        self.headers[member.declaration.0 as usize]
                    else {
                        return Err("source default member header is absent");
                    };
                    if self.rows.functions[index as usize].body == s::FunctionBody::Absent {
                        return Err("source required member has no actual implementation");
                    }
                }
            }
            (None, None) if link.implementation.is_some() => {}
            _ => return Err("source member conformance loses its exact trait/inherent identity"),
        }
        if let Some(member) = link.implementation {
            declaration(self.rows, member, DefinitionKind::Function, work)?;
            let Some(HeaderRef::Function(index)) = self.headers[member.0 as usize] else {
                return Err("source impl member header is absent");
            };
            if self.rows.functions[index as usize].parent != Some(implementation.self_definition)
                || self.rows.functions[index as usize].body == s::FunctionBody::Absent
                || std::mem::replace(&mut self.stamps[member.0 as usize], stamp) == stamp
            {
                return Err("source implementation member belongs to another actual Self owner");
            }
        }
        Ok(())
    }
    fn finish(&self, work: &mut Work) -> Result<(), &'static str> {
        work.charge(self.listed.len().saturating_add(self.links.len()))?;
        if self.listed.iter().chain(&self.links).any(|used| !used) {
            return Err("source associated member ranges leave uncovered rows");
        }
        for function in &self.rows.functions {
            work.charge(1)?;
            if let Some(parent) = function.parent {
                match self.headers.get(parent.0 as usize).copied().flatten() {
                    Some(HeaderRef::Trait(_)) if self.traits[function.declaration.0 as usize] => {}
                    Some(HeaderRef::Implementation(_))
                        if self.stamps[function.declaration.0 as usize] != 0 => {}
                    _ => return Err("source function has an unauthenticated associated parent"),
                }
            }
        }
        Ok(())
    }
}
