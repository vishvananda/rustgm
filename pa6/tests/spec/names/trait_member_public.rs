mod m {
    pub struct S;
    pub trait A {
        fn get(&self) -> u8 { 7 }
    }
    impl A for S {}
}
fn main() { let _ = m::A::get(&m::S); }
