const N:usize=base::id::<usize>(3);const A:[u8;N]=base::array::<N>();fn fail(x:u8)->u8{1/x}fn main(){let [a,b,c]=A;if a!=7||b!=7||c!=7{let _=fail(0);}let []=base::array::<0>();}
