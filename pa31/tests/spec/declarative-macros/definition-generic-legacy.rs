pub fn identity<T>(value: T) -> T {
    macro_rules! keep { ($value:expr) => {{ let result: T = $value; result }} }
    keep!(value)
}
