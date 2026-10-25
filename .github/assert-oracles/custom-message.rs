fn payload(value: Box<dyn std::any::Any + Send>) -> String {
    if let Some(text) = value.downcast_ref::<&str>() { return (*text).to_owned(); }
    if let Some(text) = value.downcast_ref::<String>() { return text.clone(); }
    "unknown payload".to_owned()
}
fn observe(run: impl FnOnce() + std::panic::UnwindSafe) -> String {
    std::panic::set_hook(Box::new(|_| {}));
    match std::panic::catch_unwind(run) { Ok(()) => "unexpected success".to_owned(), Err(value) => payload(value) }
}
fn main() { let hits = std::cell::Cell::new(0); let message = observe(std::panic::AssertUnwindSafe(|| assert!(false, "custom {}", { hits.set(hits.get() + 1); 7 }))); println!("{} {}", hits.get(), message); }
