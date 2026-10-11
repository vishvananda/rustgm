#[cfg(all(target_has_atomic="ptr",target_has_atomic="64",not(target_has_atomic="128")))] fn selected() {}
