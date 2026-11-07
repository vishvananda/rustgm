#![feature(allow_internal_unstable)]
#[macro_export]
#[allow_internal_unstable(cfg_ub_checks, cfg_target_has_atomic)]
macro_rules! checked {
    () => {{
        #[cfg(ub_checks)]
        let checked = 3u8;
        #[cfg(not(ub_checks))]
        let checked = 3u8;
        #[cfg(target_has_atomic_load_store = "8")]
        let atomic = 4u8;
        #[cfg(not(target_has_atomic_load_store = "8"))]
        let atomic = 4u8;
        checked + atomic
    }};
}
#[macro_export]
#[allow_internal_unstable(cfg_ub_checks)]
macro_rules! partial {
    () => {{
        #[cfg(ub_checks)]
        let checked = 3u8;
        #[cfg(not(ub_checks))]
        let checked = 3u8;
        #[cfg(target_has_atomic_load_store = "8")]
        let atomic = 4u8;
        #[cfg(not(target_has_atomic_load_store = "8"))]
        let atomic = 4u8;
        checked + atomic
    }};
}
#[macro_export]
macro_rules! unchecked {
    () => {{
        #[cfg(ub_checks)]
        let checked = 7u8;
        #[cfg(not(ub_checks))]
        let checked = 7u8;
        checked
    }};
}
