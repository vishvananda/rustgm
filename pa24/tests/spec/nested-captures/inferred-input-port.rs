trait Code {
    fn value(self) -> u32;
}

impl Code for &u8 {
    fn value(self) -> u32 {
        *self as u32
    }
}

fn with_code(offset: u32, code: impl Code) -> u32 {
    offset + code.value()
}

fn run(task: impl FnOnce() -> u32) -> u32 {
    task()
}

fn divide(value: u32) -> u32 {
    1 / value
}

fn main() {
    let offset = 10;
    let value = 3u8;
    let convert = |code| with_code(offset, code);
    let result = run(|| convert(&value));
    if result != 13 {
        let _ = divide(0);
    }
}
