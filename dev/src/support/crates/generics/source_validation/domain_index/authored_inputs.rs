//! Raw original inputs retain complete ownership; they are requests for reproof.
#[path = "input_ports/obligation_ports.rs"]
mod obligations;
use super::{Context, DomainIndex, HeaderRef, Types, Work, s};

impl DomainIndex<'_, '_> {
    pub(super) fn input_ports(
        &mut self,
        context: Context,
        original: &s::Original,
        input: &s::BodyInputs,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        self.environment(context, input.environment, work)?;
        if input.parameters.len()
            != self
                .headers
                .scopes()
                .complete_type_slots(context.owner, work)? as usize
        {
            return Err("source original omitted unused type parameter identities");
        }
        for (index, &parameter) in input.parameters.iter().enumerate() {
            if self.headers.scopes().original_type_parameter_position(
                context.owner,
                parameter,
                work,
            )? as usize
                != index
            {
                return Err(
                    "source original parameter identities are foreign, missing or reordered",
                );
            }
        }
        for &bound in &input.assumptions {
            self.bound(context, bound, work)?;
        }
        for input in &input.type_assumptions {
            self.input_site(original, input.site, work)?;
            if matches!(input.value, s::Requirement::Trait { .. }) {
                return Err("source regional assumptions cannot carry trait permission");
            }
            self.requirement(context, input.value, work)?;
        }
        self.allocations(context, original, input, work)?;
        for input in &input.obligations {
            self.input_site(original, input.site, work)?;
            self.obligation(context, original, input.kind, work)?;
        }
        if let Some(builtins) = &input.builtins {
            if matches!(
                input.environment,
                s::EnvironmentOrigin::ConstTraitDefault { .. }
            ) && builtins.environment != input.environment
            {
                return Err(
                    "source default builtins lost their same defining environment construction",
                );
            }
            self.environment(context, builtins.environment, work)?;
            for requirement in &builtins.predicates {
                self.input_site(original, requirement.site, work)?;
                self.requirement(context, requirement.value, work)?;
            }
            if let Some(copy) = &builtins.source_copy {
                if let Some(declaration) = copy.declaration
                    && (declaration.artifact != s::ArtifactRef(0)
                        || !matches!(
                            self.headers.header(declaration.declaration, work)?,
                            Some(HeaderRef::Trait(_))
                        ))
                {
                    return Err("source Copy query has no exact supported declaration");
                }
                for &goal in &copy.goals {
                    self.requirement(context, goal, work)?;
                }
            }
        }
        Ok(())
    }
    fn environment(
        &self,
        context: Context,
        environment: s::EnvironmentOrigin,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let header = self.function(context.owner, work)?;
        let default = if let Some(parent) = header.parent {
            match self.headers.header(parent, work)? {
                Some(HeaderRef::Trait(index))
                    if self.headers.rows().traits[index as usize].flags.bits()
                        & s::TraitFlags::CONST
                        != 0 =>
                {
                    Some(parent)
                }
                _ => None,
            }
        } else {
            None
        };
        match (environment, default) {
            (s::EnvironmentOrigin::ConstTraitDefault { member, trait_ }, Some(parent))
                if member == context.owner
                    && trait_
                        == (s::DeclarationAddress {
                            artifact: s::ArtifactRef(0),
                            declaration: parent,
                        }) =>
            {
                Ok(())
            }
            (_, Some(_)) => {
                Err("source provided default lost its actual defining environment domain")
            }
            (s::EnvironmentOrigin::Declaration(owner), None) if owner == context.owner => Ok(()),
            (s::EnvironmentOrigin::Absent | s::EnvironmentOrigin::Contract, None) => Ok(()),
            _ => Err("source environment construction belongs to another exact owner"),
        }
    }
    pub(super) fn input_site(
        &self,
        original: &s::Original,
        site: s::InputSite,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        match site {
            s::InputSite::Body(site) => Self::site(original, site, work),
            s::InputSite::Declaration { owner, site } => {
                if self
                    .headers
                    .rows()
                    .declarations
                    .get(owner.0 as usize)
                    .is_none_or(|declaration| site >= declaration.sites)
                {
                    return Err("source original input has a foreign declaration location");
                }
                Ok(())
            }
        }
    }
    fn allocations(
        &mut self,
        context: Context,
        original: &s::Original,
        input: &s::BodyInputs,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        self.scratch(original.variables.len(), work)?;
        work.charge(original.variables.len())?;
        let mut owned = vec![false; original.variables.len()];
        for (index, normalization) in input.normalizations.iter().enumerate() {
            self.input_site(original, normalization.site, work)?;
            for &site in &normalization.sources {
                self.input_site(original, site, work)?;
            }
            for range in &normalization.owned {
                let variables = range.slice(&original.variables)?;
                for (offset, variable) in variables.iter().enumerate() {
                    work.charge(1)?;
                    let slot = range.start as usize + offset;
                    if variable.allocation
                        != s::RegionAllocation::Normalization(
                            u32::try_from(index)
                                .map_err(|_| "source normalization owner ordinal overflow")?,
                        )
                        || std::mem::replace(&mut owned[slot], true)
                    {
                        return Err("source normalization interval is foreign or multiply owned");
                    }
                }
            }
            for &bound in normalization
                .equalities
                .iter()
                .chain(&normalization.outlives)
            {
                self.bound(context, bound, work)?;
            }
            for input in &normalization.types {
                self.types.root(self.headers, context, input.ty, work)?;
                Types::region(self.headers, &self.binders, context, input.shorter, work)?;
            }
        }
        for (index, variable) in original.variables.iter().enumerate() {
            Self::site(original, variable.site, work)?;
            if matches!(variable.allocation, s::RegionAllocation::Normalization(_)) != owned[index]
            {
                return Err("source normalization allocation has no complete disjoint input owner");
            }
        }
        Ok(())
    }
}
