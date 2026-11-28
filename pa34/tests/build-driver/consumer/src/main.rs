fn main() {
    let total = renamed_gen::compute();
    let len = renamed_gen::tag_len();
    let ok = total == 49 && len == 8;
    let _ = 1u8 / (ok as u8);
}
