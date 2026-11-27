mod hidden {
    pub macro opaque() { provider_value() }

    pub fn provider_value() -> u32 { 41 }

    pub mod inner {
        #[rustc_builtin_macro]
        pub macro line() {}

        pub mod nested {
            #[rustc_builtin_macro]
            pub macro line() {}
        }
    }
}

pub use hidden::inner as route;
pub use hidden::inner as alternate;

pub use hidden::opaque as qualified_opaque;
pub use hidden::provider_value as provider_symbol;

#[rustc_macro_transparency = "semiopaque"]
pub macro forward($value:expr) { $value }

#[rustc_macro_transparency = "transparent"]
pub macro caller_value() { value }
