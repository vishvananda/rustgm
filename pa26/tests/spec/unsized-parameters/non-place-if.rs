#![feature(unsized_fn_params)]
fn take(_:str){}fn forward(x:str,y:str,c:bool){take(if c{x}else{y});}fn main(){}
