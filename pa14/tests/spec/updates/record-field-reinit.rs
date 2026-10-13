struct N; struct S{a:N,b:N} fn f(mut x:S){let _a=x.a;x.a=N;let _b=x;}
