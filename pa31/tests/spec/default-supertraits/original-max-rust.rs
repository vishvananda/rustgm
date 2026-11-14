#![feature(const_trait_impl,const_cmp)]
#![no_std]
use core::cmp::{Ordering,PartialEq,Eq,PartialOrd,Ord};
pub struct Rank(u8);
const impl PartialEq for Rank{fn eq(&self,other:&Self)->bool{self.0==other.0}}
const impl Eq for Rank{}
const impl PartialOrd for Rank{fn partial_cmp(&self,other:&Self)->Option<Ordering>{Some(self.cmp(other))}}
const impl Ord for Rank{fn cmp(&self,other:&Self)->Ordering{self.0.cmp(&other.0)}}
pub const MAX:u8=Ord::max(Rank(7),Rank(41)).0;
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{MAX}
#[unsafe(no_mangle)]pub extern "C" fn probe(a:u8,b:u8)->u8{Ord::max(Rank(a),Rank(b)).0}
