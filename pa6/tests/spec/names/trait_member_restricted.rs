mod outer {
    mod inner {
        pub struct S;
        pub(super) trait A {
            fn get(&self) -> u8 { 7 }
        }
        impl A for S {}
    }
    pub fn call() { let _ = inner::A::get(&inner::S); }
}
fn main() { outer::call(); }
