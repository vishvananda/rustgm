static A:[u32;4]=[2,3,5,7];const R:&[u32]={let[_,ref r @ ..,_]=A;r};fn main(){check(match R{[_,_]=>true,_=>false});check(R[0]==3);check(R[1]==5);}
