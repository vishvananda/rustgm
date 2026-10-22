fn fail(x:u8)->u8{1/x}fn main(){let r=aggregates::metadata_mixed(aggregates::Mixed{n:7,x:1.5});if r.n!=10||r.x!=4.0{let _=fail(0);}}
