cfg_if::cfg_if! {
    if #[cfg(unix)] { pub fn selected() -> u8 { 1 } } else { pub fn selected() -> u8 { 2 } }
}
