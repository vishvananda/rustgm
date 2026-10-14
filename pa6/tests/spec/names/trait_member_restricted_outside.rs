mod outer {
    pub mod inner {
        pub struct S;
        pub(super) trait A {
            fn get(&self) -> u8 { 7 }
        }
        impl A for S {}
    }
}
fn main() { let _ = outer::inner::A::get(&outer::inner::S); }
