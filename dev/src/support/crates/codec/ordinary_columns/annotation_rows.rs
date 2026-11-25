//! Version-one complete columns share the ordinary decoder's retained envelope.
use super::{Input, Writer, id, s, write};
use crate::support::crates::{MAX_FUNCTIONS, owner_catalog::DeclarationRef};

/// Version fourteen packs safe `#[target_feature]` policy into the top bit of
/// the declared-feature count; unsafe rows keep bit 31 clear and their exact
/// version-thirteen bytes.
const SAFE_TARGET_FEATURE_FLAG: u32 = 1 << 31;

pub(super) fn encode(
    out: &mut Writer,
    columns: &s::HeaderAttributeColumns,
) -> Result<(), &'static str> {
    write(out, &columns.functions, row_write)?;
    write(out, &columns.traits, row_write)
}
pub(super) fn decode(read: &mut Input<'_, '_>) -> Result<s::HeaderAttributeColumns, &'static str> {
    read.reserve(size_of::<s::HeaderAttributeColumns>())?;
    Ok(s::HeaderAttributeColumns {
        functions: read.list(MAX_FUNCTIONS, 10, row_read)?,
        traits: read.list(MAX_FUNCTIONS, 10, row_read)?,
    })
}
pub(super) fn row_write(out: &mut Writer, row: &s::HeaderContract) -> Result<(), &'static str> {
    id(out, row.declaration.0)?;
    id(out, row.declaration_site)?;
    out.u8(u8::from(row.must_use.is_some()))?;
    if let Some(value) = &row.must_use {
        id(out, value.owner_site)?;
        out.u8(u8::from(value.reason.is_some()))?;
        if let Some(reason) = &value.reason {
            out.string(reason)?;
        }
    }
    // Version thirteen widens the inline presence byte into a flag set: bit 0
    // is the inline hint, bit 1 the cold advisory, bit 2 caller tracking and
    // bit 3 declared target features. Version fourteen adds optimization in
    // bits 4 and 5, the formatting panic policy in bit 6 and promotion in bit 7.
    // Unmarked rows keep their exact earlier bytes.
    out.u8(u8::from(row.inline.is_some())
        | u8::from(row.cold) << 1
        | u8::from(row.caller_tracking) << 2
        | u8::from(row.target_features.is_some()) << 3
        | optimization_bits(row.optimization) << 4
        | u8::from(row.format_panic) << 6
        | u8::from(row.promotable.is_some()) << 7)?;
    if let Some(value) = row.inline {
        id(out, value.owner_site)?;
        out.u8(match value.hint {
            s::InlineHint::Hint => 0,
            s::InlineHint::Always => 1,
            s::InlineHint::Never => 2,
        })?;
        out.u8(match value.applicability {
            s::InlineApplicability::FunctionBody => 0,
            s::InlineApplicability::DiagnosedNoBody => 1,
        })?;
    }
    if let Some(site) = row.promotion_site() {
        id(out, site)?;
    }
    // Written as declared: the reader, not the writer, owns canonical form.
    if let Some(features) = &row.target_features {
        let count = u32::try_from(features.names.len())
            .map_err(|_| "source target-feature set exceeds transport profile limit")?;
        if count & SAFE_TARGET_FEATURE_FLAG != 0 {
            return Err("source target-feature set exceeds transport profile limit");
        }
        let encoded = count
            | if features.safe {
                SAFE_TARGET_FEATURE_FLAG
            } else {
                0
            };
        out.bytes(&encoded.to_le_bytes())?;
        for name in &features.names {
            out.string(name)?;
        }
    }
    Ok(())
}
pub(super) fn row_read(read: &mut Input<'_, '_>) -> Result<s::HeaderContract, &'static str> {
    let declaration = DeclarationRef(read.id()?);
    let declaration_site = read.id()?;
    let must_use = if read.boolean()? {
        Some(s::MustUseContract {
            owner_site: read.id()?,
            reason: if read.boolean()? {
                Some(reason_read(read)?)
            } else {
                None
            },
        })
    } else {
        None
    };
    let flags = read.reader.u8()?;
    if flags > 15 && !read.profile.declaration_policies() {
        return Err("unknown transported inline or callable advisory flags");
    }
    let optimization = optimization_read((flags >> 4) & 3)?;
    if flags & 8 != 0 && !read.profile.body_nodes() {
        return Err("source target-feature rows require the version-thirteen header profile");
    }
    if flags & 6 != 0 && !read.profile.body_nodes() {
        return Err(
            "source cold and caller tracking rows require the version-thirteen header profile",
        );
    }
    let inline = if flags & 1 == 1 {
        Some(inline_read(read)?)
    } else {
        None
    };
    let promotable = if flags & 128 != 0 {
        Some(s::HeaderContract::pack_promotion_site(read.id()?)?)
    } else {
        None
    };
    let target_features = if flags & 8 == 0 {
        None
    } else {
        // Pinned rustc rejects the combination in the provider.
        if inline.is_some_and(|value| value.hint == s::InlineHint::Always) {
            return Err("source target-feature function cannot use an always-inline hint");
        }
        Some(Box::new(features_read(read)?))
    };
    Ok(s::HeaderContract {
        declaration,
        declaration_site,
        must_use,
        inline,
        cold: flags & 2 != 0,
        caller_tracking: flags & 4 != 0,
        target_features,
        optimization,
        format_panic: flags & 64 != 0,
        promotable,
    })
}

fn optimization_bits(preference: Option<s::Optimization>) -> u8 {
    match preference {
        None => 0,
        Some(s::Optimization::Disabled) => 1,
        Some(s::Optimization::Size) => 2,
        Some(s::Optimization::Speed) => 3,
    }
}

fn optimization_read(bits: u8) -> Result<Option<s::Optimization>, &'static str> {
    match bits {
        0 => Ok(None),
        1 => Ok(Some(s::Optimization::Disabled)),
        2 => Ok(Some(s::Optimization::Size)),
        3 => Ok(Some(s::Optimization::Speed)),
        _ => Err("unknown transported optimization intent"),
    }
}
/// Syntax only: support cannot see sema's feature table, so each consumer
/// resolves the names when it checks its import ledger.
fn features_read(read: &mut Input<'_, '_>) -> Result<s::DeclaredFeatures, &'static str> {
    let raw = u32::from_le_bytes(
        read.reader
            .bytes(4)?
            .try_into()
            .map_err(|_| "truncated metadata u32")?,
    );
    let safe = raw & SAFE_TARGET_FEATURE_FLAG != 0;
    if safe && !read.profile.declaration_policies() {
        return Err(
            "source safe target-feature function requires the version-fourteen header profile",
        );
    }
    let count = usize::try_from(raw & !SAFE_TARGET_FEATURE_FLAG)
        .map_err(|_| "metadata count exceeds host index")?;
    if count == 0 {
        return Err("source target-feature set is empty");
    }
    if count > s::MAX_TARGET_FEATURES {
        return Err("source target-feature set exceeds transport profile limit");
    }
    read.reader.atoms(count)?;
    read.reserve(size_of::<s::DeclaredFeatures>() + count * size_of::<Box<str>>())?;
    let mut names: Vec<Box<str>> = Vec::with_capacity(count);
    for _ in 0..count {
        let length = read.reader.u32()?;
        if length > s::MAX_TARGET_FEATURE_NAME {
            return Err("source target-feature name exceeds transport profile limit");
        }
        if length == 0 {
            return Err("unknown source target feature");
        }
        read.reader.atoms(length)?;
        read.reserve(length)?;
        let name = std::str::from_utf8(read.reader.bytes(length)?)
            .map_err(|_| "unknown source target feature")?;
        if names.last().is_some_and(|last| **last >= *name) {
            return Err("non-canonical source target-feature set");
        }
        names.push(Box::from(name));
    }
    Ok(s::DeclaredFeatures {
        names: names.into_boxed_slice(),
        safe,
    })
}
fn inline_read(read: &mut Input<'_, '_>) -> Result<s::InlineContract, &'static str> {
    Ok(s::InlineContract {
        owner_site: read.id()?,
        hint: match read.reader.u8()? {
            0 => s::InlineHint::Hint,
            1 => s::InlineHint::Always,
            2 => s::InlineHint::Never,
            _ => return Err("unknown transported inline hint"),
        },
        applicability: match read.reader.u8()? {
            0 => s::InlineApplicability::FunctionBody,
            1 => s::InlineApplicability::DiagnosedNoBody,
            _ => return Err("unknown transported inline applicability"),
        },
    })
}
fn reason_read(read: &mut Input<'_, '_>) -> Result<String, &'static str> {
    let length = read.reader.u32()?;
    if length > s::MAX_REASON_BYTES {
        return Err("source must_use reason exceeds transport profile limit");
    }
    read.reader.atoms(length)?;
    read.reserve(length)?;
    let value = std::str::from_utf8(read.reader.bytes(length)?)
        .map_err(|_| "source must_use reason is not UTF-8")?;
    Ok(value.to_owned())
}
