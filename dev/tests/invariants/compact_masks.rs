//! The last compact consumer bit preserves every preexisting assignment.
use super::KnownFeature;

#[test]
fn cfg_thread_local_uses_the_last_consumer_bit_without_widening_the_mask() {
    use crate::resolve::bindings::policy::{Feature, Features};
    assert_eq!(size_of::<KnownFeature>(), 1);
    assert_eq!(size_of::<crate::expand::config::CfgGate>(), 1);
    assert_eq!(size_of::<Feature>(), 1);
    assert_eq!(size_of::<Features>(), 16);
    assert_eq!(Feature::ThreadLocal as usize, 80);
    assert_eq!(Feature::CfgTargetThreadLocal as usize, 81);
    assert_eq!(Feature::PanicRuntime as usize, 82);
    assert_eq!(KnownFeature::ALL.len(), u8::BITS as usize);
    let mut mask = 0_u8;
    for (index, feature) in KnownFeature::ALL.into_iter().enumerate() {
        assert_eq!(feature.mask(), 1_u8 << index);
        assert_eq!(mask & feature.mask(), 0);
        mask |= feature.mask();
    }
    assert_eq!(mask, u8::MAX);
    assert_eq!(KnownFeature::CfgTargetThreadLocal.mask(), 0x80);
    assert_eq!(KnownFeature::CfgFloatReliability.mask(), 0x40);
}
