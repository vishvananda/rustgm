pub struct ArrayStorage<T> {
    pub values: T,
}

// Reduced from the const-generic array nested in core's InnerSized alias.
type InnerSized<T, const N: usize> = ArrayStorage<[T; N]>;
pub type ExportedArray<T, const N: usize> = ArrayStorage<[T; N]>;

pub fn preserve_three(value: InnerSized<u32, 3>) -> InnerSized<u32, 3> {
    value
}
