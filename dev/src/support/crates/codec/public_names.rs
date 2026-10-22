//! Canonical public bindings reuse declared owners, never resolver arena IDs.
use super::{CrateIdentity, ExternalDef, ExternalKind, Reader, Writer};
use crate::support::crates::{MAX_FUNCTIONS, PublicBinding, PublicModule, PublicNames};
use crate::support::hash::HashMap;

pub(super) fn encode(
    out: &mut Writer,
    names: &PublicNames,
    identity: &CrateIdentity,
    dependencies: &[CrateIdentity],
) -> Result<(), &'static str> {
    let owners: HashMap<_, _> = std::iter::once(identity)
        .chain(dependencies)
        .enumerate()
        .map(|(index, owner)| (owner, index))
        .collect();
    out.u32(names.modules().len())?;
    for module in names.modules() {
        out.path(module.path())?;
        out.u32(module.bindings().len())?;
        for binding in module.bindings() {
            out.string(binding.name())?;
            let target = binding.target();
            out.u32(
                *owners
                    .get(target.owner())
                    .ok_or("missing public metadata owner")?,
            )?;
            out.u8(match target.kind() {
                ExternalKind::Module => 0,
                ExternalKind::Function => 1,
                ExternalKind::Constant => 2,
                ExternalKind::Record => 3,
                ExternalKind::Static => 4,
            })?;
            out.path(target.path())?;
        }
    }
    Ok(())
}
pub(super) fn decode(
    input: &mut Reader<'_>,
    identity: &CrateIdentity,
    dependencies: &[CrateIdentity],
) -> Result<PublicNames, &'static str> {
    let count = input.count(MAX_FUNCTIONS, 6)?;
    input.atoms(count)?;
    let mut modules: Vec<PublicModule> = Vec::with_capacity(count);
    for _ in 0..count {
        let path = input.module_path()?;
        if modules
            .last()
            .is_some_and(|last| last.path() >= path.as_slice())
        {
            return Err("public metadata modules are duplicate or noncanonical");
        }
        let count = input.count(MAX_FUNCTIONS, 12)?;
        input.atoms(count)?;
        let mut bindings: Vec<PublicBinding> = Vec::with_capacity(count);
        for _ in 0..count {
            let name = input.string(256)?;
            let ordinal = input.u32()?;
            let owner = if ordinal == 0 {
                identity
            } else {
                dependencies
                    .get(ordinal - 1)
                    .ok_or("unknown public metadata owner")?
            };
            let kind = match input.u8()? {
                0 => ExternalKind::Module,
                1 => ExternalKind::Function,
                2 => ExternalKind::Constant,
                3 => ExternalKind::Record,
                4 => ExternalKind::Static,
                _ => return Err("unknown public metadata target kind"),
            };
            let target = ExternalDef::new(owner.clone(), input.module_path()?, kind)?;
            let binding = PublicBinding::new(name, target)?;
            if bindings.last().is_some_and(|last| {
                (last.name(), last.target().namespace())
                    >= (binding.name(), binding.target().namespace())
            }) {
                return Err("public metadata bindings are duplicate or noncanonical");
            }
            bindings.push(binding);
        }
        modules.push(PublicModule::new(path, bindings)?);
    }
    PublicNames::new(modules)
}
