struct OnlyDisplay;impl std::fmt::Display for OnlyDisplay{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("x")}}fn main(){let _=format_args!("{:?}",OnlyDisplay);}
