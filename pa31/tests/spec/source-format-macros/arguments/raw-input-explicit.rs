fn main() {
    let value = 7;
    let raw = &value as *const i32;
    println!("{}", format_args!("{:?}", unsafe { *raw }));
}
