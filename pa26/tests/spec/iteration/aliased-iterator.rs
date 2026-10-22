fn check(b:bool){if !b{let z=0;let _=1/z;}}struct C{cur:u8,end:u8}fn counter(end:u8)->C{C{cur:0,end}}use core::iter::Iterator as Iter;impl Iter for C{type Item=u8;fn next(&mut self)->Option<u8>{if self.cur<self.end{let n=self.cur;self.cur+=1u8;Some(n)}else{None}}}fn main(){let mut n=0u8;for x in counter(3){n+=x;}check(n==3);}

