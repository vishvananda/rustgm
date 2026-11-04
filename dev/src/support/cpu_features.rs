//! Language requests are distinct from the selected target's enabled CPU features.

/// Pinned non-x86 requests have no attribute consumer on the fixed x86-64 target.
/// Recognition cannot add their spellings to the selected target's feature table.
pub(crate) fn foreign_request(name: &str) -> bool {
    matches!(
        name,
        "aarch64_unstable_target_feature"
            | "aarch64_ver_target_feature"
            | "arm_target_feature"
            | "avr_target_feature"
            | "bpf_target_feature"
            | "csky_target_feature"
            | "hexagon_target_feature"
            | "loongarch_target_feature"
            | "m68k_target_feature"
            | "mips_target_feature"
            | "nvptx_target_feature"
            | "powerpc_target_feature"
            | "riscv_target_feature"
            | "s390x_target_feature"
            | "sparc_target_feature"
            | "wasm_target_feature"
            | "xtensa_target_feature"
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Gate {
    Apx,
    Avx10,
    Clflushopt,
    Ermsb,
    Fma4,
    Lahfsahf,
    Movrs,
    Prfchw,
    Rtm,
    Amx,
    X87,
    Xop,
}
impl Gate {
    pub(crate) const ALL: [Self; 12] = [
        Self::Apx,
        Self::Avx10,
        Self::Clflushopt,
        Self::Ermsb,
        Self::Fma4,
        Self::Lahfsahf,
        Self::Movrs,
        Self::Prfchw,
        Self::Rtm,
        Self::Amx,
        Self::X87,
        Self::Xop,
    ];
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Apx => "apx_target_feature",
            Self::Avx10 => "avx10_target_feature",
            Self::Clflushopt => "clflushopt_target_feature",
            Self::Ermsb => "ermsb_target_feature",
            Self::Fma4 => "fma4_target_feature",
            Self::Lahfsahf => "lahfsahf_target_feature",
            Self::Movrs => "movrs_target_feature",
            Self::Prfchw => "prfchw_target_feature",
            Self::Rtm => "rtm_target_feature",
            Self::Amx => "x86_amx_intrinsics",
            Self::X87 => "x87_target_feature",
            Self::Xop => "xop_target_feature",
        }
    }
    pub(crate) fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|gate| gate.name() == name)
    }
}
