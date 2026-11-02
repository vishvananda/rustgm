macro_rules! make{($m:meta)=>{#[$m]mod docs{}}}make!(rustc_doc_primitive="bool");
