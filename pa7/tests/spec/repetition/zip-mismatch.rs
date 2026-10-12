macro_rules! m { ($($i:ident),*;$($j:ident),*) => {$(fn $i(){let $j=1;})*} }
m!(a,b;c);
