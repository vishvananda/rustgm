fn first()->impl Copy{7u8}fn second()->impl Copy{first()}fn main(){let a=second();let _=(a,a);}
