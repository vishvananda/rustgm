fn f<F>(g:F) where F:for<'a> Fn(&'a str)->&'a str {}
