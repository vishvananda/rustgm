#[cfg(all(panic="abort", not(debug_assertions), not(test), not(proc_macro)))] fn selected() {}
