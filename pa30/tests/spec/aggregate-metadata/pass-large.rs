fn fail(x:u8)->u8{1/x}fn main(){let r=aggregates::metadata_large(aggregates::Large{cells:[1,2,3,4,5]});if r.cells[0]!=1||r.cells[4]!=12{let _=fail(0);}}
