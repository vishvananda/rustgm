// Reduced from the `Chunk` alias in pinned Rust 1.98.0 core/src/slice/memchr.rs.
pub fn block_word() -> u32 {
    type Chunk = u32;
    let word: Chunk = 34;
    word
}
