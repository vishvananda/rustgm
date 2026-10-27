//! Neutral role identity is checked catalog data, never implementation or call proof.
use super::{Catalog, CrateIdentity, Declaration, DefId, LangItem, Report};
use crate::{
    resolve::{
        bindings::{HeaderAttribute, HeaderAttributes},
        body_origin::{Catalog as Origins, DeclarationOrigin, ImportedDeclarationId},
        model::DefKind,
    },
    support::crates::{
        Dependencies, SourceDescriptor, generics::source as s, owner_catalog::DeclarationRef,
    },
};
use std::{rc::Rc, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sema) struct Role {
    pub definition: DefId,
    pub parent: Option<DefId>,
    pub origin: DeclarationOrigin,
}
impl From<Declaration> for Role {
    fn from(source: Declaration) -> Self {
        Self {
            definition: source.definition,
            parent: source.parent,
            origin: source.origin.into(),
        }
    }
}
pub(super) enum Bindings {
    Provider([Option<Declaration>; LangItem::ALL.len()]),
    Imported(Imported),
}
#[derive(Clone, Copy)]
struct ImportedRole {
    declaration: ImportedDeclarationId,
    parent: Option<DefId>,
}
pub(super) struct Imported {
    current: CrateIdentity,
    dependencies: Arc<Dependencies>,
    provider_index: u32,
    origins: Rc<Origins>,
    roles: [Option<ImportedRole>; LangItem::ALL.len()],
    installation_work: usize,
    staged: Option<std::rc::Weak<crate::resolve::bindings::consumer_policy::StagedConsumer>>,
}
impl Imported {
    pub(super) fn require_report(
        &self,
        report: &Report,
        provider: &CrateIdentity,
    ) -> Result<(), &'static str> {
        if let Some(identity) = &self.staged {
            let actual = report
                .package
                .staged_consumer()
                .ok_or("staged role catalog requires its actual private consumer report")?;
            if !std::rc::Weak::ptr_eq(identity, &Rc::downgrade(actual)) {
                return Err("staged role catalog belongs to another actual private consumer");
            }
            actual.require_package(&report.package)?;
            if actual.provider()? != provider {
                return Err("staged role catalog changed its actual provider");
            }
            let imported = report
                .resolution
                .crates
                .as_deref()
                .ok_or("staged role catalog lost its actual imported inventory")?;
            return self.require_inventory(imported, actual.current(), actual.index());
        }
        let source = report
            .package
            .source_client()
            .ok_or("imported role catalog requires its actual source client report")?;
        let imported = report
            .resolution
            .crates
            .as_deref()
            .ok_or("source client report has no actual imported inventory")?;
        let index = source.require_inventory(imported.dependencies())?;
        if source.provider() != provider {
            return Err("imported role catalog belongs to another source client inventory");
        }
        self.require_inventory(imported, source.current(), index)
    }
    fn require_inventory(
        &self,
        imported: &crate::resolve::crates::Imported,
        current: &CrateIdentity,
        index: u32,
    ) -> Result<(), &'static str> {
        if current != &self.current
            || index != self.provider_index
            || !std::ptr::eq(imported.dependencies(), self.dependencies.as_ref())
        {
            return Err("imported role catalog belongs to another source client inventory");
        }
        let origins = imported
            .origin_catalog()?
            .ok_or("source client report has no completed origin catalog")?;
        if !Rc::ptr_eq(&origins, &self.origins) {
            return Err("source client role catalog belongs to another actual declaration remap");
        }
        Ok(())
    }

    fn role(&self, kind: LangItem, provider: &CrateIdentity) -> Result<Option<Role>, &'static str> {
        let Some(role) = self.roles[kind.index()] else {
            return Ok(None);
        };
        let (owner, _, _) = self.origins.declaration_coordinate(role.declaration)?;
        if owner != provider {
            return Err("imported language role belongs to a foreign provider artifact");
        }
        let definition = self
            .origins
            .declaration_identity(role.declaration, role_kind(kind))?;
        Ok(Some(Role {
            definition,
            parent: role.parent,
            origin: DeclarationOrigin::Imported(role.declaration),
        }))
    }
}

impl Catalog {
    /// Source clients retain the closed local-header ABI of their import profile.
    pub(crate) const fn local_caller_tracking_closed(&self) -> bool {
        matches!(self.bindings, Bindings::Imported(_))
    }

    #[cfg(test)]
    pub(crate) const fn binding_layouts() -> (usize, usize, usize) {
        (
            size_of::<Bindings>(),
            size_of::<Imported>(),
            size_of::<ImportedRole>(),
        )
    }
    pub(crate) fn binding_work(&self) -> usize {
        match &self.bindings {
            Bindings::Provider(_) => 0,
            Bindings::Imported(imported) => imported.installation_work,
        }
    }
    pub(in crate::sema) fn neutral_attributes(
        &self,
        requested: Role,
    ) -> Result<crate::resolve::bindings::HeaderAttributes, &'static str> {
        for role in LangItem::ALL {
            if let Some(actual) = self.role(role)?
                && actual.definition == requested.definition
            {
                if actual.parent != requested.parent || actual.origin != requested.origin {
                    return Err(
                        "source role attributes belong to another actual declaration origin or parent",
                    );
                }
                return Ok(self.attributes[role.index()]);
            }
        }
        Ok(crate::resolve::bindings::HeaderAttributes::EMPTY)
    }
    /// Neutral semantic identity. Source-only syntax adapters continue using
    /// `language()`; absence there must never select a bootstrap implementation.
    pub(in crate::sema) fn role(&self, role: LangItem) -> Result<Option<Role>, &'static str> {
        match &self.bindings {
            Bindings::Provider(languages) => Ok(languages[role.index()].map(Role::from)),
            Bindings::Imported(imported) => imported.role(role, &self.provider),
        }
    }
}

pub(super) fn client_catalog(report: &Report) -> Result<Box<Catalog>, &'static str> {
    let client = report
        .package
        .source_client()
        .ok_or("source client role installation has no client policy")?;
    let imported = report
        .resolution
        .crates
        .as_deref()
        .ok_or("source client role installation has no imported declarations")?;
    let provider_index = client.require_inventory(imported.dependencies())?;
    let mut work = 0;
    prepare_catalog(
        report,
        client.current(),
        client.provider(),
        provider_index,
        None,
        &mut work,
    )
}

pub(super) fn staged_catalog(
    report: &Report,
    work: &mut crate::resolve::crates::ReservationWork,
) -> Result<Box<Catalog>, crate::resolve::package::Error> {
    let error = crate::resolve::package::Error::unlocated;
    let actual = report
        .package
        .staged_consumer()
        .ok_or_else(|| error("staged role catalog requires its actual private consumer"))?;
    actual.require_package(&report.package).map_err(error)?;
    work.charge(4)?;
    work.reserve(size_of::<Catalog>())?;
    let mut attempted = 0;
    let result = prepare_catalog(
        report,
        actual.current(),
        actual.provider().map_err(error)?,
        actual.index(),
        Some(Rc::downgrade(actual)),
        &mut attempted,
    );
    work.charge(attempted)?;
    result.map_err(error)
}

fn prepare_catalog(
    report: &Report,
    current: &CrateIdentity,
    provider: &CrateIdentity,
    provider_index: u32,
    staged: Option<std::rc::Weak<crate::resolve::bindings::consumer_policy::StagedConsumer>>,
    installation_work: &mut usize,
) -> Result<Box<Catalog>, &'static str> {
    let imported = report
        .resolution
        .crates
        .as_deref()
        .ok_or("source role installation has no actual imported declarations")?;
    let origins = imported
        .origin_catalog()?
        .ok_or("source client has no completed authenticated origin catalog")?;
    let descriptor = imported
        .dependencies()
        .artifacts()
        .get(provider_index as usize)
        .and_then(crate::support::crates::Interface::source_descriptor)
        .ok_or("source client provider descriptor is absent")?;
    let (roles, attributes) = reserve_roles(descriptor, &origins, installation_work)?;
    let dependencies = imported.shared_dependencies();
    let bindings = Imported {
        current: current.clone(),
        dependencies,
        provider_index,
        origins,
        roles,
        installation_work: *installation_work,
        staged,
    };
    bindings.require_report(report, provider)?;
    Ok(Box::new(Catalog {
        provider: provider.clone(),
        bindings: Bindings::Imported(bindings),
        sizing: None,
        destruction_registration: None,
        traits: None,
        call_effects: None,
        call_contexts: None,
        intrinsics: crate::support::hash::HashMap::default(),
        attributes,
        runtime_checks: super::runtime_checks::RuntimeCheckSession::new(&report.package),
    }))
}

type Reserved = (
    [Option<ImportedRole>; LangItem::ALL.len()],
    [crate::resolve::bindings::HeaderAttributes; LangItem::ALL.len()],
);
fn reserve_roles(
    descriptor: SourceDescriptor<'_>,
    origins: &Origins,
    work: &mut usize,
) -> Result<Reserved, &'static str> {
    let rows = descriptor.rows();
    let roles = rows
        .roles
        .get(rows.policy.roles.start as usize..rows.policy.roles.end as usize)
        .ok_or("source role range is outside its validated payload")?;
    let mut result = [None; LangItem::ALL.len()];
    let mut attributes = [crate::resolve::bindings::HeaderAttributes::EMPTY; LangItem::ALL.len()];
    for role in roles {
        charge(work, 1)?;
        if role.declaration.artifact != s::ArtifactRef(0) {
            return Err("source provider role names a foreign defining artifact");
        }
        let definition =
            origins.definition_at(descriptor.provider(), role.declaration.declaration)?;
        let declaration = origins.declaration(definition)?;
        origins.validate_declaration(declaration, definition, role_kind(role.role))?;
        let parent = role_parent(rows, role.role, role.declaration.declaration, work)?
            .map(|parent| origins.definition_at(descriptor.provider(), parent))
            .transpose()?;
        if result[role.role.index()]
            .replace(ImportedRole {
                declaration,
                parent,
            })
            .is_some()
        {
            return Err("source descriptor has duplicate language roles");
        }
        if role_kind(role.role) == DefKind::Trait {
            charge(work, 1)?;
            let Some(s::HeaderRef::Trait(index)) = rows.header(role.declaration.declaration)?
            else {
                return Err("source trait role has no actual canonical header");
            };
            let flags = rows.traits[index as usize].flags.bits();
            charge(work, 4)?;
            attributes[role.role.index()] = HeaderAttributes::retained(
                [
                    (s::TraitFlags::FUNDAMENTAL, HeaderAttribute::Fundamental),
                    (
                        s::TraitFlags::SPECIALIZATION,
                        HeaderAttribute::Specialization,
                    ),
                    (s::TraitFlags::COINDUCTIVE, HeaderAttribute::Coinductive),
                    (
                        s::TraitFlags::DYN_INCOMPATIBLE,
                        HeaderAttribute::DynIncompatible,
                    ),
                ]
                .into_iter()
                .filter_map(|(bit, attribute)| (flags & bit != 0).then_some(attribute)),
            );
        }
    }
    Ok((result, attributes))
}

fn role_parent(
    rows: s::ValidatedView<'_>,
    role: LangItem,
    declaration: DeclarationRef,
    work: &mut usize,
) -> Result<Option<DeclarationRef>, &'static str> {
    charge(work, 1)?;
    if role_kind(role) != DefKind::Function {
        return Ok(None);
    }
    charge(work, 1)?;
    let Some(s::HeaderRef::Function(index)) = rows.header(declaration)? else {
        return Err("source function role has no actual canonical header");
    };
    Ok(rows.functions[index as usize].parent)
}
fn charge(work: &mut usize, count: usize) -> Result<(), &'static str> {
    *work = work.saturating_add(count);
    if *work > crate::support::crates::MAX_COMPONENTS {
        return Err("source role installation work budget exceeded");
    }
    Ok(())
}
fn role_kind(role: LangItem) -> DefKind {
    match role {
        LangItem::CloneFn
        | LangItem::DropGlue
        | LangItem::Panic
        | LangItem::PanicFmt
        | LangItem::PanicImpl
        | LangItem::PanicDisplay
        | LangItem::ConstPanicFmt
        | LangItem::TryFromResidual
        | LangItem::TryFromOutput
        | LangItem::TryBranch
        | LangItem::IntoTryType => DefKind::Function,
        LangItem::ControlFlowContinue | LangItem::ControlFlowBreak => DefKind::Variant,
        LangItem::Ordering => DefKind::Enum,
        LangItem::FormatArguments
        | LangItem::FormatArgument
        | LangItem::DynMetadata
        | LangItem::PhantomData
        | LangItem::PanicLocation
        | LangItem::PanicInfo => DefKind::Struct,
        LangItem::MemSizeConst | LangItem::MemAlignConst => DefKind::Const,
        LangItem::FnOnceOutput | LangItem::MetadataType | LangItem::DiscriminantType => DefKind::TypeAlias,
        _ => DefKind::Trait,
    }
}
