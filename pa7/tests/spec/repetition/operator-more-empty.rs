macro_rules! m { ($($i:ident),*) => {$(fn $i(){})+} }
m!();
