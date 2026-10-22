fn fail(x:u8)->u8{1/x}fn main(){let r=aggregates::metadata_nested(aggregates::Nested{cells:[[1,2],[3,4]],tag:5});if r.cells[1][1]!=5||r.tag!=7{let _=fail(0);}}
