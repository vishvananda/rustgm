//! Assembly syntax tags and option bits; neither register spelling nor options grant permission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    In,
    Out,
    LateOut,
    InOut,
    InLateOut,
    Const,
    Sym,
}
pub(crate) const PURE: u16 = 1;
pub(crate) const NOMEM: u16 = 2;
pub(crate) const READONLY: u16 = 4;
pub(crate) const PRESERVES_FLAGS: u16 = 8;
pub(crate) const NORETURN: u16 = 16;
pub(crate) const NOSTACK: u16 = 32;
pub(crate) const ATT_SYNTAX: u16 = 64;
pub(crate) const RAW: u16 = 128;
pub(crate) fn option(name: &str) -> Option<u16> {
    Some(match name {
        "pure" => PURE,
        "nomem" => NOMEM,
        "readonly" => READONLY,
        "preserves_flags" => PRESERVES_FLAGS,
        "noreturn" => NORETURN,
        "nostack" => NOSTACK,
        "att_syntax" => ATT_SYNTAX,
        "raw" => RAW,
        _ => return None,
    })
}
/// A checked architectural register class, independent of any runtime value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Register {
    General,
    Abcd,
    Byte,
    Xmm,
    Explicit { base: u8, bytes: u8, xmm: bool },
}
pub(crate) fn register(name: &str, explicit: bool) -> Result<Register, &'static str> {
    if !explicit {
        return match name {
            "reg" => Ok(Register::General),
            "reg_abcd" => Ok(Register::Abcd),
            "reg_byte" => Ok(Register::Byte),
            "xmm_reg" => Ok(Register::Xmm),
            _ => Err("assembly register class is not implemented for this target"),
        };
    }
    for (base, aliases) in [
        (0, ["rax", "eax", "ax", "al"]),
        (1, ["rcx", "ecx", "cx", "cl"]),
        (2, ["rdx", "edx", "dx", "dl"]),
        (6, ["rsi", "esi", "si", "sil"]),
        (7, ["rdi", "edi", "di", "dil"]),
    ] {
        if let Some(index) = aliases.iter().position(|alias| *alias == name) {
            return Ok(Register::Explicit {
                base,
                bytes: [8, 4, 2, 1][index],
                xmm: false,
            });
        }
    }
    if let Some(n) = name
        .strip_prefix("xmm")
        .and_then(|s| s.parse::<u8>().ok())
        .filter(|&n| n < 16)
    {
        return Ok(Register::Explicit {
            base: n,
            bytes: 16,
            xmm: true,
        });
    }
    if let Some(n) = name
        .strip_prefix('r')
        .and_then(|s| s.parse::<u8>().ok())
        .filter(|&n| (8..16).contains(&n))
    {
        return Ok(Register::Explicit {
            base: n,
            bytes: 8,
            xmm: false,
        });
    }
    Err("assembly explicit register is reserved or not implemented")
}
