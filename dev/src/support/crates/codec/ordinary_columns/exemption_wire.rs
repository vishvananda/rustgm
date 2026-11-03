//! An optional version-one trailer preserves old packets and rejects unknown contracts.
use super::{Input, Writer, id, s, write};
use crate::support::crates::{MAX_COMPONENTS, owner_catalog::DeclarationRef};
const TAG: u8 = 0xEC;
pub(super) fn encode(
    out: &mut Writer,
    contracts: Option<&s::ConstBodyContracts>,
) -> Result<(), &'static str> {
    let Some(contracts) = contracts else {
        return Ok(());
    };
    out.u8(TAG)?;
    out.u8(1)?;
    write(out, &contracts.implicit, |out, owner| id(out, owner.0))?;
    write(out, &contracts.grants, |out, grant| {
        id(out, grant.owner.0)?;
        out.string(&grant.feature)
    })
}
pub(super) fn decode(
    read: &mut Input<'_, '_>,
) -> Result<Option<Box<s::ConstBodyContracts>>, &'static str> {
    if read.reader.remaining() == 0 {
        return Ok(None);
    }
    if read.reader.u8()? != TAG || read.reader.u8()? != 1 {
        return Err("unknown const body contract extension/version");
    }
    read.reserve(size_of::<s::ConstBodyContracts>())?;
    Ok(Some(Box::new(s::ConstBodyContracts {
        implicit: read.list(MAX_COMPONENTS, 4, |read| Ok(DeclarationRef(read.id()?)))?,
        grants: read.list(MAX_COMPONENTS, 8, |read| {
            Ok(s::ConstBodyGrant {
                owner: DeclarationRef(read.id()?),
                feature: super::availability_wire::text(read)?,
            })
        })?,
    })))
}
