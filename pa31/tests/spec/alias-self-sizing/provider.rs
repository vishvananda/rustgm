#![feature(no_core,lang_items,rustc_attrs,trait_alias)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[lang="meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang="sized"] pub trait Sized: MetaSized {}
trait ViaPS = PointeeSized;
trait Direct = PointeeSized;
trait InWhere = where Self: PointeeSized;
trait Mediated = ViaPS;
trait WithMeta = MetaSized;
trait WithSized = Sized;
trait Unused<T> = PointeeSized;
trait RelaxUnused<T> = Sized where T: PointeeSized;
trait ProbeDirect = Direct;
trait ProbeWhere = InWhere;
trait ProbeMediated = Mediated;
trait ProbeMeta = WithMeta;
trait ProbeSized = WithSized;
trait ProbeUnused<X> = Unused<X>;
trait ProbeRelax<X: PointeeSized> = RelaxUnused<X>;
mod pretend { pub trait PointeeSized: super::PointeeSized {} }
trait Lookalike = pretend::PointeeSized;
trait ProbeLookalike = Lookalike;
