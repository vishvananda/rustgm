#![feature(const_trait_impl)]
pub const trait Base<R>{fn make(value:R)->Self;}
pub const trait Maker<R>{type Output:[const]Base<R>;}
pub const fn convert0<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert1<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert2<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert3<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert4<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert5<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert6<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert7<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
fn main(){}
