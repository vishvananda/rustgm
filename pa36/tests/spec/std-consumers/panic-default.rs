// Negative control: a panic with the default hook. The message goes to standard
// error, the process aborts, and neither destructors nor buffered partial output run.
struct Loud(&'static str);

impl Drop for Loud {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn checked_index(values: &[u32], index: usize) -> u32 {
    if index >= values.len() {
        panic!("index {} is out of range for {} values", index, values.len());
    }
    values[index]
}

fn main() {
    let _guard = Loud("never printed");
    let values = vec![10, 20, 30];
    println!("value {}", checked_index(&values, 2));
    println!("before the panic");
    print!("a partial line that is never flushed");
    let value = checked_index(&values, std::hint::black_box(3));
    println!("unreachable {value}");
}
