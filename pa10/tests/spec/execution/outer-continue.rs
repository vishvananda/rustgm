fn entry()->u64{let mut x=0;'a:loop{x+=1;loop{if x<3{continue 'a;}break 'a x;}}}
