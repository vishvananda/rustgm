struct AtomicU32;impl AtomicU32{fn fetch_min(&self,_:u32)->u32{3}fn fetch_max(&self,_:u32)->u32{37}}fn main(){let a=AtomicU32;let _=a.fetch_min(7);let _=a.fetch_max(7);}
