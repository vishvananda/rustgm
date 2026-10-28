#![feature(staged_api)]
#![stable(feature="base",since="1.0.0")]
#![allow(internal_features,dead_code)]
#[stable(feature="base",since="1.0.0")]
trait Tr {}
#[stable(feature="base",since="1.0.0")]
struct S;
#[expect(ineffective_unstable_trait_impl)]
#[stable(feature="base",since="1.0.0")]
impl Tr for S {}
