macro_rules! m { ($o:ident;$($i:ident),*) => {$(fn $i(){let $o=1;})*} }
m!(unused;a,b);
