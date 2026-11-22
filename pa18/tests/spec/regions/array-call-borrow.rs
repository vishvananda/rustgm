fn identity<T>(value: T) -> T {
    value
}

fn read(value: &u32) -> u32 {
    let [element] = identity([value]);
    *element
}

fn main() {
    let value = 19;
    if read(&value) != 19 {
        loop {}
    }
}
