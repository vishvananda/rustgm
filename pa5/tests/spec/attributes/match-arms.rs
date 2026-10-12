fn select(value: u8) -> u8 {
    match value {
        #[cfg(any())]
        _ => missing_name,
        #[cfg(all())]
        binding => binding,
    }
}
