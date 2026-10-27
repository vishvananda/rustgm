fn main() {
    let args;
    {
        let text = String::from("owned");
        args = format_args!("{:?}", text);
    }
    println!("{args}");
}
