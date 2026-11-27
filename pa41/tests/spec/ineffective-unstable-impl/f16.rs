#![feature(staged_api)]
#![stable(feature="base",since="1.0.0")]
#![allow(internal_features,dead_code)]
#![feature(f16)]
#[stable(feature="base",since="1.0.0")]
trait Tr {}
#[stable(feature="base",since="1.0.0")]
struct S;
#[expect(ineffective_unstable_trait_impl)]
#[unstable(feature="sample",issue="none")]
impl Tr for f16 {}
