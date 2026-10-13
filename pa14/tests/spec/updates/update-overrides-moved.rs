struct N; struct S{a:N,b:N} fn f(x:S){let _a=x.a;let _b=S{a:N,..x};}
