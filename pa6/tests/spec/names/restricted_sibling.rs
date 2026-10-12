mod a{mod b{pub(super) fn f(){}}mod c{fn g(){super::b::f();}}}
