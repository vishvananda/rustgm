#![feature(impl_restriction,const_trait_impl,auto_traits)]
pub impl(crate) const unsafe trait Restricted {}
pub impl(crate) unsafe auto trait AutoRestricted {}
