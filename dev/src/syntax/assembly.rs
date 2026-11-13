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
pub(crate) use crate::support::initial_assembly::{
    ATT_SYNTAX, NOMEM, NORETURN, NOSTACK, PRESERVES_FLAGS, PURE, RAW, READONLY,
};
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
    /// `ymm_reg`, which needs `avx`. Only typing admits it; the initial
    /// assembly profile rejects it before lowering or export.
    Ymm,
    /// `zmm_reg`, which needs `avx512f`. Only typing admits it; the initial
    /// assembly profile rejects it before lowering or export.
    Zmm,
    /// `kreg`, an AVX-512 mask register, which needs `avx512f`, and
    /// `avx512bw` for 32- and 64-bit operands. Only typing admits it.
    Kreg,
    Explicit {
        base: u8,
        bytes: u8,
        xmm: bool,
    },
}
pub(crate) fn register(name: &str, explicit: bool) -> Result<Register, &'static str> {
    if !explicit {
        return match name {
            "reg" => Ok(Register::General),
            "reg_abcd" => Ok(Register::Abcd),
            "reg_byte" => Ok(Register::Byte),
            "xmm_reg" => Ok(Register::Xmm),
            "ymm_reg" => Ok(Register::Ymm),
            "zmm_reg" => Ok(Register::Zmm),
            "kreg" => Ok(Register::Kreg),
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
