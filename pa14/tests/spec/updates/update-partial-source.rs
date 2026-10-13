struct N; struct S{a:N,b:N} fn f(x:S){let _a=S{a:N,..x};let _b=x;}
