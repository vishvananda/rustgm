fn entry()->u64{let mut out=0;let a=5;{let a=100;out+=a;}let b=7;'outer:loop{let c=11;{let gone=13;out+=gone;}let d=17;out+=a+b+c+d;break 'outer;}out}
