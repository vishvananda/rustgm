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
mod compact;
pub(super) use compact::Provider;

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
    Provider(Provider),
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
    fn descriptor(&self, provider: &CrateIdentity) -> Result<SourceDescriptor<'_>, &'static str> {
        self.dependencies
            .artifacts()
            .get(self.provider_index as usize)
            .and_then(crate::support::crates::Interface::source_descriptor)
            .filter(|descriptor| descriptor.identity() == provider)
            .ok_or("imported role lost its actual provider descriptor")
    }

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
    /// Select the source-client role catalog independently of callable policy.
    pub(crate) const fn uses_imported_roles(&self) -> bool {
        matches!(self.bindings, Bindings::Imported(_))
    }

    pub(super) fn bodyless_headers(
        &self,
        work: &mut usize,
    ) -> Result<[Option<Role>; s::BodylessIntrinsic::ALL.len()], &'static str> {
        let mut roles = [None; s::BodylessIntrinsic::ALL.len()];
        let Bindings::Imported(imported) = &self.bindings else {
            return Ok(roles);
        };
        let descriptor = imported.descriptor(&self.provider)?;
        if !descriptor.profile().declaration_policies() {
            return Ok(roles);
        }
        let rows = descriptor.rows();
        charge(work, rows.functions.len())?;
        for header in &rows.functions {
            let s::FunctionBody::Intrinsic(operation) = header.body else {
                continue;
            };
            charge(work, 3)?;
            let definition = imported
                .origins
                .definition_at(descriptor.provider(), header.declaration)?;
            let declaration = imported.origins.declaration(definition)?;
            let origin = imported.origins.validate_declaration(
                declaration,
                definition,
                DefKind::Function,
            )?;
            roles[operation as usize] = Some(Role {
                definition,
                parent: None,
                origin,
            });
        }
        Ok(roles)
    }

    pub(super) fn declared_intrinsics(
        &self,
        work: &mut usize,
    ) -> Result<Vec<(Role, crate::support::bindings::SourceIntrinsic, bool)>, &'static str> {
        let Bindings::Imported(imported) = &self.bindings else {
            return Ok(Vec::new());
        };
        let descriptor = imported.descriptor(&self.provider)?;
        if !descriptor.profile().declaration_policies() {
            return Ok(Vec::new());
        }
        let rows = descriptor.rows();
        let entries = &rows.cold().declared_intrinsics;
        if entries.is_empty() {
            return Ok(Vec::new());
        }
        charge(work, entries.len())?;
        let mut result = Vec::with_capacity(entries.len());
        for entry in entries {
            charge(work, 4)?;
            let decl = rows
                .declarations
                .get(entry.declaration.0 as usize)
                .ok_or("imported declared intrinsic lacks its declaration row")?;
            let kind = decl
                .name
                .as_deref()
                .and_then(crate::support::bindings::SourceIntrinsic::from_name)
                .ok_or("imported declared intrinsic names an unknown intrinsic")?;
            let definition = imported
                .origins
                .definition_at(descriptor.provider(), entry.declaration)?;
            let declaration = imported.origins.declaration(definition)?;
            let origin = imported.origins.validate_declaration(
                declaration,
                definition,
                DefKind::Function,
            )?;
            result.push((
                Role {
                    definition,
                    parent: None,
                    origin,
                },
                kind,
                entry.source_fallback,
            ));
        }
        Ok(result)
    }

    #[cfg(test)]
    pub(crate) const fn binding_layouts() -> (usize, usize, usize) {
        (
            size_of::<Bindings>(),
            size_of::<Imported>(),
            size_of::<ImportedRole>(),
        )
    }
    #[cfg(test)]
    pub(crate) const fn provider_binding_bytes() -> usize {
        size_of::<Provider>()
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
            Bindings::Provider(languages) => Ok(languages.get(role).map(Role::from)),
            Bindings::Imported(imported) => imported.role(role, &self.provider),
        }
    }

    pub(in crate::sema) fn imported_member_named(
        &self,
        origin: DeclarationOrigin,
        names: &[&str],
    ) -> Result<bool, &'static str> {
        let (Bindings::Imported(imported), DeclarationOrigin::Imported(declaration)) =
            (&self.bindings, origin)
        else {
            return Ok(false);
        };
        let (owner, ordinal, _) = imported.origins.declaration_coordinate(declaration)?;
        if owner != &self.provider {
            return Ok(false);
        }
        let descriptor = imported.descriptor(&self.provider)?;
        Ok(descriptor.profile().declaration_policies()
            && descriptor
                .rows()
                .declarations
                .get(ordinal.0 as usize)
                .and_then(|row| row.name.as_deref())
                .is_some_and(|name| names.contains(&name)))
    }

    pub(super) fn imported_diagnostic(
        &self,
        wanted: s::DiagnosticRole,
        kind: DefKind,
        work: &mut usize,
    ) -> Result<Option<Role>, &'static str> {
        let Bindings::Imported(imported) = &self.bindings else {
            return Err("imported diagnostic identity requires its actual provider");
        };
        let descriptor = imported.descriptor(&self.provider)?;
        for row in descriptor.rows().diagnostics() {
            charge(work, 1)?;
            if row.role != wanted {
                continue;
            }
            let definition = imported
                .origins
                .definition_at(descriptor.provider(), row.declaration.declaration)?;
            let declaration = imported.origins.declaration(definition)?;
            imported
                .origins
                .validate_declaration(declaration, definition, kind)?;
            return Ok(Some(Role {
                definition,
                parent: None,
                origin: DeclarationOrigin::Imported(declaration),
            }));
        }
        Ok(None)
    }

    /// Compare declaration identities in the caller's precedence order.
    /// Imported claims still authenticate their remapped origin at each lookup.
    pub(in crate::sema) fn matching_role<T>(
        &self,
        definition: DefId,
        candidates: impl Iterator<Item = (T, LangItem)>,
    ) -> Result<Option<(T, LangItem)>, &'static str> {
        match &self.bindings {
            Bindings::Provider(languages) => Ok(languages.matching_role(definition, candidates)),
            Bindings::Imported(imported) => {
                for (value, role) in candidates {
                    if imported
                        .role(role, &self.provider)?
                        .is_some_and(|declaration| declaration.definition == definition)
                    {
                        return Ok(Some((value, role)));
                    }
                }
                Ok(None)
            }
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
        async_destruction_registration: None,
        traits: None,
        call_effects: None,
        call_contexts: None,
        value_reborrows: None,
        intrinsics: crate::support::hash::HashMap::default(),
        attributes,
        runtime_checks: super::runtime_checks::RuntimeCheckSession::new(&report.package),
        native_layouts: None,
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
    diagnostic_roles(descriptor, origins, work)?;
    // Version thirteen: `Send`'s diagnostic identity fills this consumer's own
    // `LangItem::Send` slot, as a provider's reservations do.
    let send = rows
        .diagnostics()
        .iter()
        .filter(|row| row.role == s::DiagnosticRole::Send)
        .map(|row| s::RoleBinding {
            role: LangItem::Send,
            declaration: row.declaration,
        });
    for role in roles.iter().copied().chain(send) {
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
            charge(work, 6)?;
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
                    (s::TraitFlags::MARKER, HeaderAttribute::Marker),
                    (
                        s::TraitFlags::UNSAFE_SPECIALIZATION,
                        HeaderAttribute::UnsafeSpecialization,
                    ),
                ]
                .into_iter()
                .filter_map(|(bit, attribute)| (flags & bit != 0).then_some(attribute)),
            );
        }
    }
    Ok((result, attributes))
}

/// Each transported diagnostic identity re-proves the local rule over this
/// consumer's own declaration remap: the provider's own struct for `NonNull`,
/// its own trait for `Send`. Its packet row is the record a later consumer
/// reads.
fn diagnostic_roles(
    descriptor: SourceDescriptor<'_>,
    origins: &Origins,
    work: &mut usize,
) -> Result<(), &'static str> {
    for row in descriptor.rows().diagnostics() {
        charge(work, 1)?;
        if row.declaration.artifact != s::ArtifactRef(0) {
            return Err("source diagnostic identity belongs to a foreign provider");
        }
        let definition =
            origins.definition_at(descriptor.provider(), row.declaration.declaration)?;
        let declaration = origins.declaration(definition)?;
        let (kind, message) = match row.role {
            s::DiagnosticRole::NonNull => (
                DefKind::Struct,
                "imported NonNull diagnostic item requires its nominal declaration profile",
            ),
            s::DiagnosticRole::Send => (
                DefKind::Trait,
                "imported Send diagnostic item requires its unsafe auto trait declaration",
            ),
            s::DiagnosticRole::Eq => (
                DefKind::Trait,
                "imported Eq diagnostic item requires its actual trait declaration",
            ),
            s::DiagnosticRole::Display => (
                DefKind::Trait,
                "imported Display diagnostic item requires its actual trait declaration",
            ),
            s::DiagnosticRole::Result => (
                DefKind::Enum,
                "imported Result diagnostic item requires its actual enum declaration",
            ),
        };
        origins
            .validate_declaration(declaration, definition, kind)
            .map_err(|_| message)?;
    }
    Ok(())
}

fn role_parent(
    rows: s::ValidatedView<'_>,
    role: LangItem,
    declaration: DeclarationRef,
    work: &mut usize,
) -> Result<Option<DeclarationRef>, &'static str> {
    charge(work, 1)?;
    if matches!(
        role,
        LangItem::FutureOutput
            | LangItem::FnOnceOutput
            | LangItem::MetadataType
            | LangItem::DiscriminantType
            | LangItem::DerefTarget
            | LangItem::ReceiverTarget
            | LangItem::FieldBase
            | LangItem::FieldType
    ) || role.resumable_associated()
    {
        let Some(s::HeaderRef::AssociatedType(index)) = rows.header(declaration)? else {
            return Err("imported Output role lacks its actual associated header");
        };
        return Ok(Some(rows.associated_types[index as usize].parent));
    }
    if matches!(
        role,
        LangItem::RangeMin
            | LangItem::RangeMax
            | LangItem::AsyncGenPending
            | LangItem::AsyncGenFinished
            | LangItem::FieldOffset
    ) {
        let parent = rows
            .declarations
            .get(declaration.0 as usize)
            .and_then(|row| row.definition.parent)
            .ok_or("imported associated constant lacks its actual owner")?;
        return Ok(Some(parent));
    }
    if matches!(role, LangItem::MemSizeConst | LangItem::MemAlignConst) {
        return rows
            .declarations
            .get(declaration.0 as usize)
            .and_then(|row| row.definition.parent)
            .map(Some)
            .ok_or("imported layout constant lost its actual trait owner");
    }
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
    if let Some(profile) = role.declaration_profile() {
        return crate::resolve::bindings::language_kind(profile.target);
    }
    match role {
        LangItem::FuturePoll
        | LangItem::AsyncDropInPlace
        | LangItem::RangeSub
        | LangItem::CloneFn
        | LangItem::DropGlue
        | LangItem::Panic
        | LangItem::PanicFmt
        | LangItem::PanicImpl
        | LangItem::PanicDisplay
        | LangItem::PanicNounwind
        | LangItem::ConstPanicFmt
        | LangItem::TryFromResidual
        | LangItem::TryFromOutput
        | LangItem::TryBranch
        | LangItem::SliceLen
        | LangItem::OffsetOf
        | LangItem::IntoTryType
        | LangItem::IteratorNext
        | LangItem::IntoIter
        | LangItem::ContractBuildCheckEnsures
        | LangItem::ContractCheckRequires
        | LangItem::ContractCheckEnsures => DefKind::Function,
        LangItem::ControlFlowContinue
        | LangItem::ControlFlowBreak
        | LangItem::ResultOk
        | LangItem::ResultErr
        | LangItem::OptionSome
        | LangItem::OptionNone => DefKind::Variant,
        LangItem::Ordering | LangItem::Option => DefKind::Enum,
        LangItem::FormatArguments
        | LangItem::FormatArgument
        | LangItem::DynMetadata
        | LangItem::PhantomData
        | LangItem::ManuallyDrop
        | LangItem::MaybeDangling
        | LangItem::MaybeUninit
        | LangItem::TransmuteOpts
        | LangItem::TypeInfo
        | LangItem::TypeId
        | LangItem::PanicLocation
        | LangItem::PanicInfo
        | LangItem::AllocLayout => DefKind::Struct,
        LangItem::MemSizeConst
        | LangItem::MemAlignConst
        | LangItem::RangeMin
        | LangItem::RangeMax => DefKind::Const,
        LangItem::FutureOutput
        | LangItem::FnOnceOutput
        | LangItem::MetadataType
        | LangItem::DiscriminantType => DefKind::TypeAlias,
        _ => DefKind::Trait,
    }
}
