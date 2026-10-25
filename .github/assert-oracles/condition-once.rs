fn main() { let hits = std::cell::Cell::new(0); assert!({ hits.set(hits.get() + 1); true }); println!("{}", hits.get()); }
