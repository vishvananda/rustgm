//! Literal body boundaries, expression suffix restrictions and typed values.
use super::{
    escapes::{self, Encoding, Flavor},
    value::Value,
};
use crate::lex::token::Kind;

pub(super) fn decode(spelling: &str, kind: Kind) -> Result<Value<'_>, &'static str> {
    let flavor = Flavor {
        encoding: match kind {
            Kind::Byte | Kind::ByteString | Kind::RawByteString => Encoding::Bytes,
            Kind::CString | Kind::RawCString => Encoding::CString,
            _ => Encoding::Unicode,
        },
        single: matches!(kind, Kind::Character | Kind::Byte),
        raw: matches!(
            kind,
            Kind::RawString | Kind::RawByteString | Kind::RawCString
        ),
    };
    let quote = if flavor.single { '\'' } else { '"' };
    let open = spelling.find(quote).ok_or("missing literal delimiter")?;
    let close = spelling
        .rfind(quote)
        .filter(|close| *close > open)
        .ok_or("missing literal delimiter")?;
    let hashes = if flavor.raw {
        spelling[..open].bytes().filter(|b| *b == b'#').count()
    } else {
        0
    };
    if close + 1 + hashes != spelling.len() {
        return Err("suffix on non-numeric literal");
    }
    let content = &spelling[open + 1..close];
    if flavor.single {
        let scalar = escapes::scalar(content, flavor)?;
        return if kind == Kind::Byte {
            u8::try_from(scalar)
                .map(Value::Byte)
                .map_err(|_| "non-ASCII character in byte literal")
        } else {
            Ok(Value::Character(scalar))
        };
    }
    let bytes = escapes::decode(content, flavor)?;
    Ok(if flavor.encoding == Encoding::Bytes {
        Value::ByteString(bytes)
    } else if flavor.encoding == Encoding::CString {
        Value::CString(bytes)
    } else {
        Value::String(bytes)
    })
}
