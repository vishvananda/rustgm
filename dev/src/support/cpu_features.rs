//! Language requests for unstable x86 attributes; these are not enabled CPU features.
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
