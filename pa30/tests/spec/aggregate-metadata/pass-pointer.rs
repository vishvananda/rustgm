fn fail(x:u8)->u8{1/x}fn main(){let f:extern "C" fn(aggregates::Mixed)->aggregates::Mixed=aggregates::metadata_mixed;let r=f(aggregates::Mixed{n:8,x:0.5});if r.n!=11||r.x!=3.0{let _=fail(0);}}
