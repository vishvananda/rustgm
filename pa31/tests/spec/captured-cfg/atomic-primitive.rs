#![feature(decl_macro,rustc_attrs,cfg_target_has_atomic,staged_api)]
#![stable(feature = "core", since = "1.6.0")]
unsafe trait AtomicPrimitive{type Storage;}
mod private{pub struct Align1<T>(T);pub struct Align2<T>(T);pub struct Align4<T>(T);pub struct Align8<T>(T);pub struct Align16<T>(T);}
macro impl_atomic_primitive(
    [$($T:ident)?] $Primitive:ty as $Storage:ident<$Operand:ty>, size($size:literal)
) {
    #[unstable(
        feature = "atomic_internals",
        reason = "implementation detail which may disappear or be replaced at any time",
        issue = "none"
    )]
    #[cfg(target_has_atomic_load_store = $size)]
    unsafe impl $(<$T>)? AtomicPrimitive for $Primitive {
        type Storage = private::$Storage<$Operand>;
    }
}

impl_atomic_primitive!([] bool as Align1<u8>, size("8"));
impl_atomic_primitive!([] i8 as Align1<i8>, size("8"));
impl_atomic_primitive!([] u8 as Align1<u8>, size("8"));
impl_atomic_primitive!([] i16 as Align2<i16>, size("16"));
impl_atomic_primitive!([] u16 as Align2<u16>, size("16"));
impl_atomic_primitive!([] i32 as Align4<i32>, size("32"));
impl_atomic_primitive!([] u32 as Align4<u32>, size("32"));
impl_atomic_primitive!([] i64 as Align8<i64>, size("64"));
impl_atomic_primitive!([] u64 as Align8<u64>, size("64"));
impl_atomic_primitive!([] i128 as Align16<i128>, size("128"));
impl_atomic_primitive!([] u128 as Align16<u128>, size("128"));

fn main(){}
