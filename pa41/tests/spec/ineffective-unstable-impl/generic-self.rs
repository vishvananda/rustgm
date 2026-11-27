#![feature(staged_api)]
#![stable(feature="base",since="1.0.0")]
#![allow(internal_features,dead_code)]
#[stable(feature="base",since="1.0.0")]
trait Tr {}
#[expect(ineffective_unstable_trait_impl)]
#[unstable(feature="sample",issue="none")]
impl<T> Tr for T {}
