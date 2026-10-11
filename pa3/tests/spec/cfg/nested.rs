#[cfg(all(unix, any(windows, not(false)), target_pointer_width = "64",))] fn selected() {}
