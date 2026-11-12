#![feature(no_core,lang_items,rustc_attrs,fundamental,staged_api,const_trait_impl,const_destruct,unboxed_closures,tuple_trait,fn_traits)]
#![unstable(feature="source_fn_sugar_fixture",issue="none")]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang = "pointee_sized"]
#[diagnostic::on_unimplemented(
    message = "values of type `{Self}` may or may not have a size",
    label = "may or may not have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_coinductive]
pub trait PointeeSized {
    // Empty
}
#[lang = "meta_sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known",
    label = "doesn't have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
// `MetaSized` being coinductive, despite having supertraits, is okay for the same reasons as
// `Sized` above.
#[rustc_coinductive]
pub trait MetaSized: PointeeSized {
    // Empty
}
#[lang = "sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known at compilation time",
    label = "doesn't have a size known at compile-time"
)]
#[fundamental] // for Default, for example, which requires that `[T]: !Default` be evaluatable
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
// `Sized` being coinductive, despite having supertraits, is okay as there are no user-written impls,
// and we know that the supertraits are always implemented if the subtrait is just by looking at
// the builtin impls.
#[rustc_coinductive]
pub trait Sized: MetaSized {
    // Empty.
}
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang = "clone"]
pub trait Clone: Sized { #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"]
pub trait Copy: Clone {}
#[unstable(feature="const_destruct",issue="133214")]
#[rustc_const_unstable(feature="const_destruct",issue="133214")]
#[lang="destruct"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub const trait Destruct:PointeeSized{}
impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for i32{fn clone(&self)->Self{*self}}impl Copy for i32{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}

#[unstable(feature = "tuple_trait", issue = "157987")]
#[lang = "tuple_trait"]
#[diagnostic::on_unimplemented(message = "`{Self}` is not a tuple")]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub trait Tuple {}

#[lang = "fn"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_paren_sugar]
#[rustc_on_unimplemented(
    on(
        Args = "()",
        note = "wrap the `{Self}` in a closure with no arguments: `|| {{ /* code */ }}`"
    ),
    on(
        Self = "unsafe fn",
        note = "unsafe function cannot be called generically without an unsafe block",
        // SAFETY: tidy is not smart enough to tell that the below unsafe block is a string
        label = "call the function in a closure: `|| unsafe {{ /* code */ }}`"
    ),
    message = "expected an `{This:resolved}` closure, found `{Self}`",
    label = "expected an `{This:resolved}` closure, found `{Self}`"
)]
#[fundamental] // so that regex can rely that `&str: !FnMut`
#[must_use = "closures are lazy and do nothing unless called"]
#[rustc_const_unstable(feature = "const_trait_impl", issue = "143874")]
pub const trait Fn<Args: Tuple>: [const] FnMut<Args> {
    /// Performs the call operation.
    #[unstable(feature = "fn_traits", issue = "29625")]
    extern "rust-call" fn call(&self, args: Args) -> Self::Output;
}

#[lang = "fn_mut"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_paren_sugar]
#[rustc_on_unimplemented(
    on(
        Args = "()",
        note = "wrap the `{Self}` in a closure with no arguments: `|| {{ /* code */ }}`"
    ),
    on(
        Self = "unsafe fn",
        note = "unsafe function cannot be called generically without an unsafe block",
        // SAFETY: tidy is not smart enough to tell that the below unsafe block is a string
        label = "call the function in a closure: `|| unsafe {{ /* code */ }}`"
    ),
    message = "expected an `{This:resolved}` closure, found `{Self}`",
    label = "expected an `{This:resolved}` closure, found `{Self}`"
)]
#[fundamental] // so that regex can rely that `&str: !FnMut`
#[must_use = "closures are lazy and do nothing unless called"]
#[rustc_const_unstable(feature = "const_trait_impl", issue = "143874")]
pub const trait FnMut<Args: Tuple>: FnOnce<Args> {
    /// Performs the call operation.
    #[unstable(feature = "fn_traits", issue = "29625")]
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output;
}

#[lang = "fn_once"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_paren_sugar]
#[rustc_on_unimplemented(
    on(
        Args = "()",
        note = "wrap the `{Self}` in a closure with no arguments: `|| {{ /* code */ }}`"
    ),
    on(
        Self = "unsafe fn",
        note = "unsafe function cannot be called generically without an unsafe block",
        // SAFETY: tidy is not smart enough to tell that the below unsafe block is a string
        label = "call the function in a closure: `|| unsafe {{ /* code */ }}`"
    ),
    message = "expected an `{This:resolved}` closure, found `{Self}`",
    label = "expected an `{This:resolved}` closure, found `{Self}`"
)]
#[fundamental] // so that regex can rely that `&str: !FnMut`
#[must_use = "closures are lazy and do nothing unless called"]
#[rustc_const_unstable(feature = "const_trait_impl", issue = "143874")]
pub const trait FnOnce<Args: Tuple> {
    /// The returned type after the call operator is used.
    #[lang = "fn_once_output"]
    #[stable(feature = "fn_once_output", since = "1.12.0")]
    type Output;

    /// Performs the call operation.
    #[unstable(feature = "fn_traits", issue = "29625")]
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output;
}

enum Option<T>{None,Some(T)}
impl<T> Option<T>{
fn and_then<U,F>(self,f:F)->Option<U>where F:FnOnce(T)->Option<U>{loop{}}
fn map<U,F>(self,f:F)->Option<U>where F:FnOnce(T)->U{loop{}}
fn or(self,other:Self)->Self{self}
}
trait Iterator{type Item;fn last(self)->Option<Self::Item>where Self:Sized{loop{}}}
fn make<I:Iterator>(_x0:Option<I::Item>,_x1:Option<I::Item>,_x2:Option<I::Item>,_x3:Option<I::Item>,_x4:Option<I::Item>,_x5:Option<I::Item>,_x6:Option<I::Item>,_x7:Option<I::Item>,_x8:Option<I::Item>,_x9:Option<I::Item>,_x10:Option<I::Item>,_x11:Option<I::Item>,_x12:Option<I::Item>,_x13:Option<I::Item>,_x14:Option<I::Item>,_x15:Option<I::Item>,_x16:Option<I::Item>,_x17:Option<I::Item>,_x18:Option<I::Item>,_x19:Option<I::Item>,_x20:Option<I::Item>,_x21:Option<I::Item>,_x22:Option<I::Item>,_x23:Option<I::Item>,_x24:Option<I::Item>,_x25:Option<I::Item>,_x26:Option<I::Item>,_x27:Option<I::Item>,_x28:Option<I::Item>,_x29:Option<I::Item>,_x30:Option<I::Item>,_x31:Option<I::Item>,_x32:Option<I::Item>,_x33:Option<I::Item>,_x34:Option<I::Item>,_x35:Option<I::Item>,_x36:Option<I::Item>,_x37:Option<I::Item>,_x38:Option<I::Item>,_x39:Option<I::Item>,_x40:Option<I::Item>,_x41:Option<I::Item>,_x42:Option<I::Item>,_x43:Option<I::Item>,_x44:Option<I::Item>,_x45:Option<I::Item>,_x46:Option<I::Item>,_x47:Option<I::Item>,_x48:Option<I::Item>,_x49:Option<I::Item>,_x50:Option<I::Item>,_x51:Option<I::Item>,_x52:Option<I::Item>,_x53:Option<I::Item>,_x54:Option<I::Item>,_x55:Option<I::Item>,_x56:Option<I::Item>,_x57:Option<I::Item>,_x58:Option<I::Item>,_x59:Option<I::Item>,_x60:Option<I::Item>,_x61:Option<I::Item>,_x62:Option<I::Item>,_x63:Option<I::Item>,_x64:Option<I::Item>,_x65:Option<I::Item>,_x66:Option<I::Item>,_x67:Option<I::Item>,_x68:Option<I::Item>,_x69:Option<I::Item>,_x70:Option<I::Item>,_x71:Option<I::Item>,_x72:Option<I::Item>,_x73:Option<I::Item>,_x74:Option<I::Item>,_x75:Option<I::Item>,_x76:Option<I::Item>,_x77:Option<I::Item>,_x78:Option<I::Item>,_x79:Option<I::Item>,_x80:Option<I::Item>,_x81:Option<I::Item>,_x82:Option<I::Item>,_x83:Option<I::Item>,_x84:Option<I::Item>,_x85:Option<I::Item>,_x86:Option<I::Item>,_x87:Option<I::Item>,_x88:Option<I::Item>,_x89:Option<I::Item>,_x90:Option<I::Item>,_x91:Option<I::Item>,_x92:Option<I::Item>,_x93:Option<I::Item>,_x94:Option<I::Item>,_x95:Option<I::Item>,_x96:Option<I::Item>,_x97:Option<I::Item>,_x98:Option<I::Item>,_x99:Option<I::Item>,_x100:Option<I::Item>,_x101:Option<I::Item>,_x102:Option<I::Item>,_x103:Option<I::Item>,_x104:Option<I::Item>,_x105:Option<I::Item>,_x106:Option<I::Item>,_x107:Option<I::Item>,_x108:Option<I::Item>,_x109:Option<I::Item>,_x110:Option<I::Item>,_x111:Option<I::Item>,_x112:Option<I::Item>,_x113:Option<I::Item>,_x114:Option<I::Item>,_x115:Option<I::Item>,_x116:Option<I::Item>,_x117:Option<I::Item>,_x118:Option<I::Item>,_x119:Option<I::Item>,_x120:Option<I::Item>,_x121:Option<I::Item>,_x122:Option<I::Item>,_x123:Option<I::Item>,_x124:Option<I::Item>,_x125:Option<I::Item>,_x126:Option<I::Item>,_x127:Option<I::Item>,_iter:I)->Option<I::Item>{loop{}}
fn drive<U:Iterator,F>(f:F)->Option<U::Item>where F:FnMut(Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,Option<U::Item>,U)->Option<U::Item>{loop{}}
fn code<U:Iterator>()->Option<U::Item>{drive::<U,_>(make)}
