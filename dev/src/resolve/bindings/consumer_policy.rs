//! A real private consumer owns source policy; unpaid provider data grants no proof.
use super::{
    Context, Source, client,
    policy::{Features, Prepared},
};
use crate::{
    resolve::{
        crates::{ReservationWork, unpaid_provided::Inputs},
        package::{Error, Package},
        unit::{FileId, NodeKey, Unit},
    },
    support::{
        bindings::Mode,
        crates::{
            CrateIdentity, Dependencies,
            generics::{Work, source as s},
        },
        runtime_checks::RuntimeCheckConfig,
        source::OriginalImage,
    },
};
use std::{
    rc::{Rc, Weak},
    sync::Arc,
};

#[cfg(test)]
pub(crate) const fn layouts() -> (usize, usize) {
    (size_of::<StagedConsumer>(), size_of::<Source>())
}
#[cfg(test)]
pub(crate) fn configuration_opponent(package: &mut Package) -> Result<(), &'static str> {
    let actual = package
        .staged_consumer()
        .cloned()
        .ok_or("configuration opponent requires its actual private consumer")?;
    let original = package.runtime_checks;
    package.runtime_checks = RuntimeCheckConfig::with_ub_checks(!original.ub_checks());
    let rejected = actual.require_package(package).is_err();
    package.runtime_checks = original;
    if rejected {
        Ok(())
    } else {
        Err("private consumer accepted a replaced configuration")
    }
}

/// Moved immutable data only. No producer-local arena or catalog survives here.
pub(crate) struct ProviderAdmission {
    inventory: Arc<Dependencies>,
    index: u32,
    headers: Rc<UnpaidHeaderData>,
}
/// Structural rows do not certify source completion, absence or defaultness.
struct UnpaidHeaderData {
    columns: s::HeaderAttributeColumns,
    bytes: usize,
    names: Option<Box<s::NamespaceColumns>>,
    names_bytes: usize,
}
/// Allocation identity only. This carries no source-completion permission.
pub(crate) struct HeaderDataIdentity {
    owner: Weak<UnpaidHeaderData>,
}
/// Strong ownership of the SAME unpaid allocation; this type certifies no completion.
pub(crate) struct HeaderDataOwner {
    data: Rc<UnpaidHeaderData>,
}
impl HeaderDataOwner {
    pub(crate) fn identity(&self, work: &mut ReservationWork) -> Result<HeaderDataIdentity, Error> {
        work.charge(1)?;
        Ok(HeaderDataIdentity {
            owner: Rc::downgrade(&self.data),
        })
    }
    pub(crate) fn columns(&self) -> &s::HeaderAttributeColumns {
        &self.data.columns
    }
}
impl HeaderDataIdentity {
    pub(crate) fn retain(
        &self,
        consumer: &StagedConsumer,
        work: &mut ReservationWork,
    ) -> Result<HeaderDataOwner, Error> {
        self.require(consumer, work)?;
        work.charge(1)?;
        Ok(HeaderDataOwner {
            data: Rc::clone(&consumer.admission.headers),
        })
    }
    pub(crate) fn require_owner(
        &self,
        owner: &HeaderDataOwner,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        work.charge(2)?;
        if Weak::ptr_eq(&self.owner, &Rc::downgrade(&owner.data)) {
            Ok(())
        } else {
            Err(Error::unlocated(
                "retained source columns changed their actual allocation",
            ))
        }
    }
    pub(crate) fn require(
        &self,
        consumer: &StagedConsumer,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        work.charge(2)?;
        if !Weak::ptr_eq(&self.owner, &Rc::downgrade(&consumer.admission.headers)) {
            return Err(Error::unlocated(
                "private consumer changed its actual header-data allocation",
            ));
        }
        Ok(())
    }
}
impl ProviderAdmission {
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(self.headers.bytes)
            .saturating_add(self.headers.names_bytes)
            .saturating_add(
                size_of::<UnpaidHeaderData>()
                    .saturating_sub(size_of::<s::HeaderAttributeColumns>()),
            )
            .saturating_add(2 * size_of::<usize>())
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the genuine numbered collector must move checked unpaid data here before public Provided admission can open"
        )
    )]
    pub(crate) fn from_reserved(
        inputs: Inputs,
        contracts: s::HeaderAttributeColumns,
        work: &mut ReservationWork,
    ) -> Result<Self, Error> {
        let (inventory, index) = inputs.into_provider(work)?;
        Self::from_data(inventory, index, contracts, work)
    }
    pub(crate) fn from_data(
        inventory: Arc<Dependencies>,
        index: u32,
        contracts: s::HeaderAttributeColumns,
        work: &mut ReservationWork,
    ) -> Result<Self, Error> {
        Self::from_data_with_names(inventory, index, contracts, None, work)
    }
    pub(crate) fn from_data_with_names(
        inventory: Arc<Dependencies>,
        index: u32,
        contracts: s::HeaderAttributeColumns,
        mut names: Option<s::NamespaceColumns>,
        work: &mut ReservationWork,
    ) -> Result<Self, Error> {
        let artifact = inventory
            .artifacts()
            .get(index as usize)
            .ok_or_else(|| Error::unlocated("private admission lost its actual provider"))?;
        let descriptor = artifact
            .source_descriptor()
            .ok_or_else(|| Error::unlocated("private admission lost its complete source data"))?;
        work.charge(1)?;
        let mut validation = Work::default();
        let result = contracts.validate_columns(&descriptor.rows(), 0, &mut validation);
        work.charge(validation.steps)?;
        let bytes = result.map_err(Error::unlocated)?;
        let names_bytes = if let Some(names) = names.as_mut() {
            let mut validation = Work {
                steps: work.steps,
                ..Work::default()
            };
            let start = validation.steps;
            let size = names.storage_bytes(&mut validation);
            work.charge(validation.steps.saturating_sub(start))?;
            let size = size.map_err(Error::unlocated)?;
            work.reserve(
                size.saturating_add(names.validation_scratch(descriptor.rows().declarations.len())),
            )?;
            let start = validation.steps;
            let checked = names.validate_columns(&descriptor.rows(), &mut validation);
            work.charge(validation.steps.saturating_sub(start))?;
            let checked = checked.map_err(Error::unlocated)?;
            if checked.retained_bytes != size {
                return Err(Error::unlocated(
                    "private namespace ordering changed its actual retained capacity",
                ));
            }
            size
        } else {
            0
        };
        work.reserve(
            bytes
                .saturating_add(size_of::<Self>())
                .saturating_add(
                    size_of::<UnpaidHeaderData>()
                        .saturating_sub(size_of::<s::HeaderAttributeColumns>()),
                )
                .saturating_add(2 * size_of::<usize>()),
        )?;
        Ok(Self {
            inventory,
            index,
            headers: Rc::new(UnpaidHeaderData {
                columns: contracts,
                bytes,
                names: names.map(Box::new),
                names_bytes,
            }),
        })
    }
    pub(crate) fn header_identity(
        &self,
        work: &mut ReservationWork,
    ) -> Result<HeaderDataIdentity, Error> {
        work.charge(1)?;
        Ok(HeaderDataIdentity {
            owner: Rc::downgrade(&self.headers),
        })
    }
    pub(crate) fn inventory(&self) -> &Arc<Dependencies> {
        &self.inventory
    }
    pub(crate) const fn index(&self) -> u32 {
        self.index
    }
    fn provider(&self) -> Result<&CrateIdentity, &'static str> {
        self.inventory
            .artifacts()
            .get(self.index as usize)
            .map(crate::support::crates::Interface::identity)
            .ok_or("private admission provider is outside its actual inventory")
    }
}

pub(crate) struct Request<'a> {
    admission: ProviderAdmission,
    runtime_checks: RuntimeCheckConfig,
    work: &'a mut ReservationWork,
}
impl<'a> Request<'a> {
    pub(crate) fn new(
        admission: ProviderAdmission,
        config: &crate::expand::config::Config,
        work: &'a mut ReservationWork,
    ) -> Self {
        Self {
            admission,
            runtime_checks: config.runtime_checks(),
            work,
        }
    }
    pub(in crate::resolve) fn into_parts(
        self,
    ) -> (
        ProviderAdmission,
        RuntimeCheckConfig,
        &'a mut ReservationWork,
    ) {
        (self.admission, self.runtime_checks, self.work)
    }
}
pub(crate) struct StagedConsumer {
    current: CrateIdentity,
    admission: ProviderAdmission,
    root: NodeKey,
    image: Arc<OriginalImage>,
    runtime_checks: RuntimeCheckConfig,
    pub(super) features: Features,
}
impl StagedConsumer {
    /// Borrowed data remains unpaid until a distinct semantic receipt is bound.
    pub(crate) fn unpaid_header_columns(&self) -> &s::HeaderAttributeColumns {
        &self.admission.headers.columns
    }
    pub(crate) fn unpaid_namespace(&self) -> Option<&s::NamespaceColumns> {
        self.admission.headers.names.as_deref()
    }
    pub(crate) fn current(&self) -> &CrateIdentity {
        &self.current
    }
    pub(crate) fn inventory(&self) -> &Arc<Dependencies> {
        self.admission.inventory()
    }
    pub(crate) const fn index(&self) -> u32 {
        self.admission.index()
    }
    pub(crate) fn provider(&self) -> Result<&CrateIdentity, &'static str> {
        self.admission.provider()
    }
    pub(crate) fn require_package(&self, package: &Package) -> Result<(), &'static str> {
        let unit = package
            .units
            .first()
            .ok_or("private consumer root disappeared")?;
        if package.crate_name != self.current.name()
            || self.current == *self.provider()?
            || self.root.file != FileId(0)
            || self.root.node != unit.configured.ast.root
            || !Arc::ptr_eq(&self.image, unit.source.original_image())
            || unit.configured.active.get(self.root.node.get() as usize) != Some(&true)
            || package.runtime_checks() != self.runtime_checks
        {
            return Err("private consumer changed its actual current/source/configuration binding");
        }
        let descriptor = self
            .inventory()
            .artifacts()
            .get(self.index() as usize)
            .and_then(crate::support::crates::Interface::source_descriptor)
            .ok_or("private consumer lost its actual provider descriptor")?;
        if descriptor.profile() != s::Profile::Source2024Provided
            || descriptor.identity() != self.provider()?
            || descriptor.provider() != self.provider()?
            || descriptor.edition() != crate::support::edition::Edition::Rust2024
            || self.admission.headers.columns.functions.len() != descriptor.rows().functions.len()
            || self.admission.headers.columns.traits.len() != descriptor.rows().traits.len()
            || self.admission.headers.bytes > crate::support::crates::MAX_BYTES
        {
            return Err("private consumer changed its sealed unpaid provider data");
        }
        Ok(())
    }
}

pub(super) fn prepare(
    unit: &Unit,
    context: &Context,
    no_core: bool,
    features: Features,
    library: Option<Box<crate::resolve::stability::Library>>,
    request: Request<'_>,
) -> Result<Prepared, Error> {
    let error = Error::unlocated;
    let Mode::SourceClient { provider } = &context.mode else {
        return Err(error(
            "private consumer requires its actual requested client context",
        ));
    };
    let current = client::identity(unit, context, provider, no_core, &features)?;
    if context
        .dependencies
        .as_ref()
        .is_none_or(|actual| !Arc::ptr_eq(actual, request.admission.inventory()))
        || request.admission.provider().map_err(error)? != provider
    {
        return Err(error(
            "private consumer context differs from its sealed provider inventory",
        ));
    }
    request.work.charge(4)?;
    request.work.reserve(
        size_of::<StagedConsumer>()
            + 2 * size_of::<usize>()
            + size_of::<Source>()
            + request.admission.headers.bytes
            + size_of::<UnpaidHeaderData>().saturating_sub(size_of::<s::HeaderAttributeColumns>())
            + 2 * size_of::<usize>(),
    )?;
    let macros = features.macros();
    let consumer = StagedConsumer {
        current: current.clone(),
        admission: request.admission,
        root: NodeKey {
            file: FileId(0),
            node: unit.configured.ast.root,
        },
        image: Arc::clone(unit.source.original_image()),
        runtime_checks: request.runtime_checks,
        features,
    };
    Ok(Prepared {
        source: Some(Box::new(Source::Staged(Rc::new(consumer)))),
        library,
        macros,
    })
}
