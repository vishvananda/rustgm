//! Typed x86 requirements from pinned rustc88d9e12; preserve NOTICE.
use crate::support::{cpu_features::Gate, retained::Growth};
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub(crate) struct FeatureSet(u128);
#[derive(Clone, Copy)]
struct Feature(u8);
struct Specification {
    name: &'static str,
    gate: Option<Gate>,
    implies: &'static [u8],
}
const _: [(); 128 - FEATURES.len()] = [(); 128 - FEATURES.len()];
impl Specification {
    const fn new(name: &'static str, gate: Option<Gate>, implies: &'static [u8]) -> Self {
        Self {
            name,
            gate,
            implies,
        }
    }
}
static FEATURES: &[Specification] = &[
    Specification::new("adx", None, &[]),
    Specification::new("aes", None, &[61]),
    Specification::new("amx-avx512", Some(Gate::Amx), &[10]),
    Specification::new("amx-bf16", Some(Gate::Amx), &[10]),
    Specification::new("amx-complex", Some(Gate::Amx), &[10]),
    Specification::new("amx-fp8", Some(Gate::Amx), &[10]),
    Specification::new("amx-fp16", Some(Gate::Amx), &[10]),
    Specification::new("amx-int8", Some(Gate::Amx), &[10]),
    Specification::new("amx-movrs", Some(Gate::Amx), &[10]),
    Specification::new("amx-tf32", Some(Gate::Amx), &[10]),
    Specification::new("amx-tile", Some(Gate::Amx), &[]),
    Specification::new("apxf", Some(Gate::Apx), &[]),
    Specification::new("avx", None, &[64]),
    Specification::new("avx2", None, &[12]),
    Specification::new(
        "avx10.1",
        Some(Gate::Avx10),
        &[16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 29],
    ),
    Specification::new("avx10.2", Some(Gate::Avx10), &[14, 32, 33, 34]),
    Specification::new("avx512bf16", None, &[18]),
    Specification::new("avx512bitalg", None, &[18]),
    Specification::new("avx512bw", None, &[21]),
    Specification::new("avx512cd", None, &[21]),
    Specification::new("avx512dq", None, &[21]),
    Specification::new("avx512f", None, &[13, 41, 40]),
    Specification::new("avx512fp16", None, &[18]),
    Specification::new("avx512ifma", None, &[21]),
    Specification::new("avx512vbmi", None, &[18]),
    Specification::new("avx512vbmi2", None, &[18]),
    Specification::new("avx512vl", None, &[21]),
    Specification::new("avx512vnni", None, &[21]),
    Specification::new("avx512vp2intersect", None, &[21]),
    Specification::new("avx512vpopcntdq", None, &[21]),
    Specification::new("avxifma", None, &[13]),
    Specification::new("avxneconvert", None, &[13]),
    Specification::new("avxvnni", None, &[13]),
    Specification::new("avxvnniint8", None, &[13]),
    Specification::new("avxvnniint16", None, &[13]),
    Specification::new("bmi1", None, &[]),
    Specification::new("bmi2", None, &[]),
    Specification::new("clflushopt", Some(Gate::Clflushopt), &[]),
    Specification::new("cmpxchg16b", None, &[]),
    Specification::new("ermsb", Some(Gate::Ermsb), &[]),
    Specification::new("f16c", None, &[12]),
    Specification::new("fma", None, &[12]),
    Specification::new("fma4", Some(Gate::Fma4), &[12, 65]),
    Specification::new("fxsr", None, &[]),
    Specification::new("gfni", None, &[61]),
    Specification::new("kl", None, &[61]),
    Specification::new("lahfsahf", Some(Gate::Lahfsahf), &[]),
    Specification::new("lzcnt", None, &[]),
    Specification::new("movbe", None, &[]),
    Specification::new("movrs", Some(Gate::Movrs), &[]),
    Specification::new("pclmulqdq", None, &[61]),
    Specification::new("popcnt", None, &[]),
    Specification::new("prfchw", Some(Gate::Prfchw), &[]),
    Specification::new("rdrand", None, &[]),
    Specification::new("rdseed", None, &[]),
    Specification::new("rtm", Some(Gate::Rtm), &[]),
    Specification::new("sha", None, &[61]),
    Specification::new("sha512", None, &[13]),
    Specification::new("sm3", None, &[12]),
    Specification::new("sm4", None, &[13]),
    Specification::new("sse", None, &[]),
    Specification::new("sse2", None, &[60]),
    Specification::new("sse3", None, &[61]),
    Specification::new("sse4.1", None, &[66]),
    Specification::new("sse4.2", None, &[63]),
    Specification::new("sse4a", None, &[62]),
    Specification::new("ssse3", None, &[62]),
    Specification::new("tbm", None, &[]),
    Specification::new("vaes", None, &[13, 1]),
    Specification::new("vpclmulqdq", None, &[12, 50]),
    Specification::new("widekl", None, &[45]),
    Specification::new("x87", Some(Gate::X87), &[]),
    Specification::new("xop", Some(Gate::Xop), &[42, 12, 65]),
    Specification::new("xsave", None, &[]),
    Specification::new("xsavec", None, &[73]),
    Specification::new("xsaveopt", None, &[73]),
    Specification::new("xsaves", None, &[73]),
];
impl FeatureSet {
    #[cfg(test)]
    pub(super) const fn uses_upper_word(self) -> bool {
        self.0 >> 64 != 0
    }
    pub(crate) const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    pub(crate) const fn covers(self, required: Self) -> bool {
        self.0 & required.0 == required.0
    }
    pub(super) fn insert<E>(
        &mut self,
        name: &str,
        mut pay: impl FnMut(Growth) -> Result<(), E>,
        mut allowed: impl FnMut(Gate) -> Result<bool, E>,
        error: impl Fn(&'static str) -> E,
    ) -> Result<(), E> {
        pay(Growth {
            work: name
                .len()
                .saturating_mul(FEATURES.len().ilog2() as usize + 2),
            bytes: 0,
        })?;
        let feature = named(name)
            .ok_or_else(|| error("target feature is invalid or forbidden for this target"))?;
        let spec = &FEATURES[feature.0 as usize];
        if let Some(gate) = spec.gate {
            pay(Growth { work: 1, bytes: 0 })?;
            if !allowed(gate)? {
                return Err(error(
                    "target feature requires its actual crate language gate",
                ));
            }
        }
        let mut pending = 1_u128 << feature.0;
        while pending != 0 {
            let bit = pending.trailing_zeros();
            pending &= !(1_u128 << bit);
            if self.0 & (1_u128 << bit) != 0 {
                continue;
            }
            let spec = FEATURES
                .get(bit as usize)
                .ok_or_else(|| error("invalid target feature identity"))?;
            pay(Growth {
                work: 1 + spec.implies.len(),
                bytes: 0,
            })?;
            *self = self.union(Self(1_u128 << bit));
            for &parent in spec.implies {
                pending |= 1_u128 << parent;
            }
        }
        Ok(())
    }
}
fn named(name: &str) -> Option<Feature> {
    FEATURES
        .binary_search_by_key(&name, |spec| spec.name)
        .ok()
        .and_then(|index| u8::try_from(index).ok())
        .map(Feature)
}
