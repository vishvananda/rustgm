#![feature(const_trait_impl)]
#![allow(dead_code,unused_variables,unused_unsafe)]
pub const trait Base<U>{fn read(value:U)->u8;}
pub const trait Bridge<U>{unsafe fn read(value:U)->u8;}
const impl<T:[const]Base<U>,U>Bridge<U>for T{unsafe fn read(value:U)->u8{<T as Base<U>>::read(value)}}
pub const fn apply0<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply1<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply2<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply3<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply4<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply5<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply6<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply7<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply8<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply9<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply10<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply11<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply12<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply13<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply14<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply15<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply16<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply17<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply18<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply19<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply20<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply21<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply22<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply23<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply24<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply25<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply26<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply27<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply28<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply29<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply30<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply31<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
fn main(){}
