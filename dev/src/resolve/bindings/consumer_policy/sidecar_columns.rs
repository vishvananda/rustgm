//! Header data borrows either the same private sidecar or decoded artifact columns.
use super::{Dependencies, Error, Rc, ReservationWork, StagedConsumer, Weak, s};
use std::sync::{Arc, Weak as SharedWeak};

pub(super) struct UnpaidHeaderData {
    pub columns: s::HeaderAttributeColumns,
    pub bytes: usize,
    pub names: Option<Box<s::NamespaceColumns>>,
    pub names_bytes: usize,
}
enum Data {
    Unpaid(Rc<UnpaidHeaderData>),
    Encoded {
        inventory: Arc<Dependencies>,
        index: u32,
    },
}
enum Identity {
    Unpaid(Weak<UnpaidHeaderData>),
    Encoded {
        inventory: SharedWeak<Dependencies>,
        index: u32,
    },
}
pub(crate) struct HeaderDataIdentity {
    owner: Identity,
}
pub(crate) struct HeaderDataOwner {
    data: Data,
}
impl HeaderDataOwner {
    pub(super) fn unpaid(data: UnpaidHeaderData) -> Self {
        Self {
            data: Data::Unpaid(Rc::new(data)),
        }
    }
    pub(super) fn encoded(
        inventory: Arc<Dependencies>,
        index: u32,
        work: &mut ReservationWork,
    ) -> Result<Self, Error> {
        work.charge(4)?;
        descriptor(&inventory, index)?;
        Ok(Self {
            data: Data::Encoded { inventory, index },
        })
    }
    pub(crate) fn identity(&self, work: &mut ReservationWork) -> Result<HeaderDataIdentity, Error> {
        work.charge(1)?;
        Ok(HeaderDataIdentity {
            owner: match &self.data {
                Data::Unpaid(data) => Identity::Unpaid(Rc::downgrade(data)),
                Data::Encoded { inventory, index } => Identity::Encoded {
                    inventory: Arc::downgrade(inventory),
                    index: *index,
                },
            },
        })
    }
    pub(crate) fn shared(&self, work: &mut ReservationWork) -> Result<Self, Error> {
        work.charge(1)?;
        Ok(Self {
            data: match &self.data {
                Data::Unpaid(data) => Data::Unpaid(Rc::clone(data)),
                Data::Encoded { inventory, index } => Data::Encoded {
                    inventory: Arc::clone(inventory),
                    index: *index,
                },
            },
        })
    }
    pub(crate) fn columns(&self) -> Result<&s::HeaderAttributeColumns, &'static str> {
        match &self.data {
            Data::Unpaid(data) => Ok(&data.columns),
            Data::Encoded { inventory, index } => descriptor(inventory, *index)
                .map_err(|_| "encoded header data changed its actual profile")?
                .rows()
                .header_attributes()
                .ok_or("encoded header advisory completion is missing"),
        }
    }
    pub(super) fn names(&self) -> Result<Option<&s::NamespaceColumns>, &'static str> {
        match &self.data {
            Data::Unpaid(data) => Ok(data.names.as_deref()),
            Data::Encoded { inventory, index } => Ok(Some(
                &descriptor(inventory, *index)
                    .map_err(|_| "encoded namespace changed its actual profile")?
                    .provided_contracts()?
                    .rows()
                    .namespace,
            )),
        }
    }
    pub(super) fn owned_bytes(&self) -> usize {
        match &self.data {
            Data::Unpaid(data) => data
                .bytes
                .saturating_add(data.names_bytes)
                .saturating_add(
                    size_of::<UnpaidHeaderData>()
                        .saturating_sub(size_of::<s::HeaderAttributeColumns>()),
                )
                .saturating_add(2 * size_of::<usize>()),
            Data::Encoded { .. } => 0,
        }
    }
    pub(super) fn preparation_bytes(&self) -> usize {
        match &self.data {
            Data::Unpaid(data) => data
                .bytes
                .saturating_add(
                    size_of::<UnpaidHeaderData>()
                        .saturating_sub(size_of::<s::HeaderAttributeColumns>()),
                )
                .saturating_add(2 * size_of::<usize>()),
            Data::Encoded { .. } => 0,
        }
    }
    pub(crate) fn require_inventory(
        &self,
        inventory: &Arc<Dependencies>,
        index: u32,
    ) -> Result<(), &'static str> {
        match &self.data {
            Data::Encoded {
                inventory: actual,
                index: owner,
            } => {
                if !Arc::ptr_eq(actual, inventory) || *owner != index {
                    return Err("encoded header data changed its actual inventory/index");
                }
                descriptor(actual, *owner)
                    .map_err(|_| "encoded header data lost its actual checked columns")?;
            }
            Data::Unpaid(_) => {
                if inventory
                    .artifacts()
                    .get(index as usize)
                    .and_then(crate::support::crates::Interface::source_descriptor)
                    .is_none_or(|descriptor| {
                        !matches!(
                            descriptor.profile(),
                            s::Profile::Source2024Provided | s::Profile::Source2024ProvidedCurrent
                        )
                    })
                {
                    return Err(
                        "unpaid header data requires its actual legacy Provided descriptor",
                    );
                }
            }
        }
        Ok(())
    }
}
impl HeaderDataIdentity {
    pub(crate) fn retain(
        &self,
        consumer: &StagedConsumer,
        work: &mut ReservationWork,
    ) -> Result<HeaderDataOwner, Error> {
        self.require(consumer, work)?;
        consumer.admission.headers.shared(work)
    }
    pub(crate) fn require_owner(
        &self,
        owner: &HeaderDataOwner,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        work.charge(2)?;
        if self.same_owner(owner) {
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
        if self.same_owner(&consumer.admission.headers) {
            Ok(())
        } else {
            Err(Error::unlocated(
                "private consumer changed its actual header-data allocation",
            ))
        }
    }
    fn same_owner(&self, owner: &HeaderDataOwner) -> bool {
        match (&self.owner, &owner.data) {
            (Identity::Unpaid(expected), Data::Unpaid(actual)) => {
                Weak::ptr_eq(expected, &Rc::downgrade(actual))
            }
            (
                Identity::Encoded { inventory, index },
                Data::Encoded {
                    inventory: actual,
                    index: owner,
                },
            ) => *index == *owner && SharedWeak::ptr_eq(inventory, &Arc::downgrade(actual)),
            _ => false,
        }
    }
}
fn descriptor(
    inventory: &Dependencies,
    index: u32,
) -> Result<crate::support::crates::SourceDescriptor<'_>, Error> {
    let descriptor = inventory
        .artifacts()
        .get(index as usize)
        .and_then(crate::support::crates::Interface::source_descriptor)
        .ok_or_else(|| Error::unlocated("encoded header data lost its actual provider"))?;
    descriptor.provided_contracts().map_err(Error::unlocated)?;
    descriptor
        .rows()
        .header_attributes()
        .ok_or_else(|| Error::unlocated("encoded header advisory completion is missing"))?;
    Ok(descriptor)
}
