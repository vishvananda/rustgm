fn main() { let hits = std::cell::Cell::new(0); assert!(true, "{}", { hits.set(hits.get() + 1); hits.get() }); println!("{}", hits.get()); }
