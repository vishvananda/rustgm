#![feature(no_core,lang_items,intrinsics,rustc_attrs,pattern_types,auto_traits,decl_macro,const_trait_impl)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables,incomplete_features)]
// Authored reducer of core's io/error/repr_bitpacked.rs static_asserts and slice/cmp.rs
// `as_underlying` (pinned core 88d9e12a): NonNull, PhantomData, Result and ControlFlow
// shapes follow ptr/non_null.rs, marker.rs, result.rs and ops/control_flow.rs.
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[lang="destruct"]pub const trait Destruct:PointeeSized{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for u32{fn clone(&self)->Self{*self}}impl Copy for u32{}
impl Clone for i32{fn clone(&self)->Self{*self}}impl Copy for i32{}

#[diagnostic::on_unimplemented(message = "collecting {A}")]
trait Collect<A> {}
struct Sink<T>(T);
impl<T> Collect<T> for Sink<T> {}

struct Range<T>(T);
trait HasItem { type Item; }
trait MiniStep {}
impl MiniStep for i32 {}
impl MiniStep for u32 {}
impl MiniStep for u8 {}
impl<T: MiniStep> HasItem for Range<T> { type Item = T; }

trait Call<A> { type Output; }
struct Identity;
impl<T> Call<T> for Identity { type Output = T; }
struct Map<I, F>(I, F);
impl<B, I: HasItem, F: Call<I::Item, Output = B>> HasItem for Map<I, F> {
    type Item = B;
}

fn collect<I: HasItem, B: Collect<I::Item>>(_i: I) -> B { loop {} }

fn main() { let _: Sink<u32> = collect(Map(Range(1u32), Identity)); }
