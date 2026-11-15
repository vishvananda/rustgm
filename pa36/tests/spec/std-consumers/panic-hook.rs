// Negative control: a custom panic hook reports the message and location on
// standard output, then the process aborts.
use std::panic;

fn parse(text: &str) -> u32 {
    text.parse().expect("the text was not a number")
}

fn main() {
    panic::set_hook(Box::new(|info| {
        let location = info.location().map(|l| (l.file().ends_with("panic-hook.rs"), l.line(), l.column()));
        println!("hook: {:?} at {:?}", info.payload_as_str(), location);
    }));
    println!("parsed {}", parse("42"));
    println!("before the panic");
    let value = parse(std::hint::black_box("forty-two"));
    println!("unreachable {value}");
}
