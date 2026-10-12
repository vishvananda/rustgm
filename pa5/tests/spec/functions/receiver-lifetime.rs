struct S; impl S { fn f<'a>(&'a self)->&'a Self {self} }
