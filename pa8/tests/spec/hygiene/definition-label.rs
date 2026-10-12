fn f(){'outer: loop{macro_rules! m{()=>{break 'outer;}} loop{m!();}}}
