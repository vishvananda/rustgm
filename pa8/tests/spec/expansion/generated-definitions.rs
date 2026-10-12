macro_rules! m{()=>{macro_rules! n{()=>{42}}}} m!(); fn f()->i32{n!()}
