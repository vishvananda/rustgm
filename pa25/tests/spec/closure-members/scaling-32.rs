#![feature(const_closures,const_trait_impl,const_destruct)]
#![allow(dead_code,unused_variables)]
use core::marker::Destruct;
pub const trait Compare{fn compare(&self,other:&Self)->u8;}
const impl Compare for u8{fn compare(&self,_other:&Self)->u8{*self}}
impl Compare for u16{fn compare(&self,_other:&Self)->u8{0}}
pub const fn make0<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make1<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make2<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make3<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make4<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make5<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make6<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make7<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make8<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make9<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make10<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make11<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make12<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make13<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make14<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make15<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make16<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make17<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make18<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make19<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make20<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make21<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make22<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make23<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make24<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make25<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make26<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make27<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make28<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make29<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make30<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
pub const fn make31<T:[const]Compare>()->impl [const]Fn(&T,&T)->u8+[const]Destruct{const |a:&T,b:&T| Compare::compare(a,b)}
fn main(){}
