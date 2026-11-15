// `std::process::exit` flushes standard output, skips destructors and reports
// the requested status.
use std::process;

struct Loud(&'static str);

impl Drop for Loud {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn finish(code: i32) -> ! {
    let _inner = Loud("inner guard, never dropped");
    print!("exiting with {code}");
    process::exit(code)
}

fn main() {
    let _outer = Loud("outer guard, never dropped");
    {
        let _scoped = Loud("scoped guard");
    }
    println!("process id is positive: {}", process::id() > 0);
    let status = std::env::args().count() as i32 + 6;
    finish(status)
}
