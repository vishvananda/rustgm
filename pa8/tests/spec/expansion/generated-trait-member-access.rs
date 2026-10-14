macro_rules! method {
    () => { fn get(&self) -> u8 { 7 } }
}
mod m {
    pub struct S;
    pub trait A { method!(); }
    impl A for S {}
}
macro_rules! import {
    () => { use m::A as _; }
}
fn main() {
    import!();
    let _ = m::A::get(&m::S);
}
