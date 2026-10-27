#![feature(core_intrinsics, discriminant_kind)]
#![allow(internal_features, dead_code)]
use core::intrinsics::discriminant_value;
use core::marker::DiscriminantKind;
fn actual<T>(value:&T)-><T as DiscriminantKind>::Discriminant {discriminant_value(value)}
trait View {type Target;}
fn projected<X:View>(value:&X::Target)-><X::Target as DiscriminantKind>::Discriminant {discriminant_value(value)}
fn main(){let _:u8=actual(&5u16);}
