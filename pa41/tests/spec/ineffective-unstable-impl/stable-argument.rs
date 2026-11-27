#![feature(staged_api)]
#![stable(feature="base",since="1.0.0")]
#![allow(internal_features,dead_code)]
#[stable(feature="base",since="1.0.0")]
trait Tr<T> {}
#[stable(feature="base",since="1.0.0")]
struct S;
#[unstable(feature="sample",issue="none")]
struct U;
#[expect(ineffective_unstable_trait_impl)]
#[unstable(feature="sample",issue="none")]
impl Tr<u8> for S {}
