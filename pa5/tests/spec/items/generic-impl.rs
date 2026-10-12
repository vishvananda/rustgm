struct W<T>(T); impl<T:Clone> W<T> where T:Send {fn f(&self)->T{self.0.clone()}}
