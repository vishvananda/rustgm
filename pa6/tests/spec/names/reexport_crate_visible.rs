mod a{pub(crate) fn f(){}}pub(crate) use a::f;fn g(){f();}
