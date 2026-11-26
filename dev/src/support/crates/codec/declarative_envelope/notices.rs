//! The sparse policy column in version six of the macro envelope.
use super::{BUILTIN, DECLARATIVE, MAX_MACROS, Macros, Reader, SCOPED, Writer, notice_fields};
use crate::support::crates::{
    MAX_BYTES,
    exported_macros::{DefinitionRow, MacroDeprecation},
};

pub(super) fn encode(out: &mut Writer, macros: &Macros) -> Result<(), &'static str> {
    out.u32(macros.deprecations().len())?;
    for notice in macros.deprecations() {
        let (kind, index) = match notice.definition {
            DefinitionRow::Declarative(index) => (DECLARATIVE, index),
            DefinitionRow::Builtin(index) => (BUILTIN, index),
            DefinitionRow::Scoped(index) => (SCOPED, index),
        };
        out.u8(kind)?;
        out.u32(index)?;
        out.u8(u8::from(notice.staged))?;
        notice_fields::since_write(out, &notice.since)?;
        notice_fields::string_option_write(out, notice.note.as_deref())?;
        notice_fields::string_option_write(out, notice.suggestion.as_deref())?;
    }
    Ok(())
}

pub(super) fn decode(input: &mut Reader<'_>, macros: Macros) -> Result<Macros, &'static str> {
    let count = input.count(MAX_MACROS, 9)?;
    if count == 0 {
        return Err("version-six exported macro table requires a deprecation notice");
    }
    input.atoms(count)?;
    let mut bytes = macros.bytes();
    let mut reserve = |length| {
        bytes = bytes.saturating_add(length);
        if bytes > MAX_BYTES {
            return Err("exported macros exceed combined profile budget");
        }
        Ok(())
    };
    reserve(count * size_of::<MacroDeprecation>())?;
    let mut notices = Vec::with_capacity(count);
    for _ in 0..count {
        let kind = input.u8()?;
        let index = input.u32()?;
        let definition = match kind {
            DECLARATIVE => DefinitionRow::Declarative(index),
            BUILTIN => DefinitionRow::Builtin(index),
            SCOPED => DefinitionRow::Scoped(index),
            _ => return Err("unknown macro deprecation definition kind"),
        };
        let staged = match input.u8()? {
            0 => false,
            1 => true,
            _ => return Err("invalid macro deprecation staging flag"),
        };
        notices.push(MacroDeprecation {
            definition,
            staged,
            since: notice_fields::since_read(input, &mut reserve)?,
            note: notice_fields::string_option_read(input, &mut reserve)?,
            suggestion: notice_fields::string_option_read(input, &mut reserve)?,
        });
    }
    macros.with_deprecations(notices)
}
