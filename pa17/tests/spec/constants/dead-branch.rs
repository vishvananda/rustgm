struct S{n:usize}impl Drop for S{fn drop(&mut self){}}const C:usize=if true{3}else{S{n:3}.n};fn main(){let _=C;}
