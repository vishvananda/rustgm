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
pub const fn convert8<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert9<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert10<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert11<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert12<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert13<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert14<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert15<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert16<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert17<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert18<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert19<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert20<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert21<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert22<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert23<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert24<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert25<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert26<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert27<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert28<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert29<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert30<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub const fn convert31<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
fn main(){}
