//! The initial inline assembly profile: the eight option bits and the one
//! table of texts for every form outside the profile. Syntax, native
//! lowering, version-thirteen source export and every version-thirteen reader
//! name a refused form by the same text, in the same order, so they can never
//! disagree about a form or about which of two forms is refused first.
pub const PURE: u16 = 1;
pub const NOMEM: u16 = 2;
pub const READONLY: u16 = 4;
pub const PRESERVES_FLAGS: u16 = 8;
pub const NORETURN: u16 = 16;
pub const NOSTACK: u16 = 32;
pub const ATT_SYNTAX: u16 = 64;
pub const RAW: u16 = 128;
/// Every option bit the syntax defines; no other bit has a meaning.
pub const KNOWN: u16 = 0xff;

/// The defined options outside the profile, reported in bit order before any
/// operand.
pub const OPTIONS: [(u16, &str); 6] = [
    (
        PURE,
        "inline assembly pure option is outside the initial assembly profile",
    ),
    (
        NOMEM,
        "inline assembly nomem option is outside the initial assembly profile",
    ),
    (
        READONLY,
        "inline assembly readonly option is outside the initial assembly profile",
    ),
    (
        NORETURN,
        "inline assembly noreturn option is outside the initial assembly profile",
    ),
    (
        ATT_SYNTAX,
        "inline assembly att_syntax option is outside the initial assembly profile",
    ),
    (
        RAW,
        "inline assembly raw option is outside the initial assembly profile",
    ),
];
pub const OUTPUT: &str = "inline assembly output operand is outside the initial assembly profile";
pub const GENERAL: &str =
    "inline assembly reg input other than a raw pointer is outside the initial assembly profile";
pub const ABCD: &str = "inline assembly reg_abcd operand is outside the initial assembly profile";
pub const BYTE: &str = "inline assembly reg_byte operand is outside the initial assembly profile";
pub const EXPLICIT: &str =
    "inline assembly explicit register operand is outside the initial assembly profile";
pub const XMM: &str = "inline assembly xmm_reg input other than a 128-bit vector is outside the initial assembly profile";
pub const MODIFIER: &str =
    "inline assembly template modifier is outside the initial assembly profile";
/// B1's constant-context text, shared by the local checker and every reader.
pub const CONSTANT: &str = "inline assembly is not permitted in constant contexts";

/// The first defined option outside the profile, in bit order.
///
/// # Errors
/// Returns that option's profile text.
pub fn option(options: u16) -> Result<(), &'static str> {
    match OPTIONS.iter().find(|&&(bit, _)| options & bit != 0) {
        Some(&(_, form)) => Err(form),
        None => Ok(()),
    }
}
