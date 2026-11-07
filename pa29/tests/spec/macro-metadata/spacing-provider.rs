// Definition-site token spacing must survive export: stringify! sees it.
#[macro_export]
macro_rules! spelled {
    (0) => { stringify!(a+=b) };
    (1) => { stringify!(a + = b) };
    (2) => { stringify!(a/**/+=/* c */b) };
    (3) => { stringify!(x< -y, x<-y) };
    (4) => { stringify!(f(a)[b]{c}{ d }) };
    (5) => { stringify!(&'a ('b') r#"s /* t */"#) };
    (6) => { stringify!(a- -b, a--b) };
    (7) => { stringify!(#[a] #![b] # ![c]) };
    (8) => { stringify!(a::b, a :: b, a:: b) };
}
