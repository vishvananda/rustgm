#![feature(staged_api,decl_macro,allow_internal_unstable,rustc_attrs,offset_of_enum)]
#![unstable(feature="offset_fixture",issue="none")]
#[stable(feature = "offset_of", since = "1.77.0")]
#[diagnostic::on_unmatched_args(
    note = "this macro expects a container type and a (nested) field path, like `offset_of!(Type, field)`"
)]
#[doc(alias = "memoffset")]
#[allow_internal_unstable(builtin_syntax, core_intrinsics)]
pub macro offset_of($Container:ty, $($fields:expr)+ $(,)?) {
    // The `{}` is for better error messages
    const {builtin # offset_of($Container, $($fields)+)}
}
#[repr(C)]struct I{a:u8,b:u32}#[repr(C)]struct N{z:u8,i:I}#[repr(C,packed)]struct P{a:u8,b:u32}#[repr(C)]union U{a:u8,b:u64}fn main(){println!("{} {} {}",offset_of!(N,i.b),offset_of!(P,b),offset_of!(U,b));}
