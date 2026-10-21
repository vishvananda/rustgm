struct ManuallyDrop<T>(T);struct S(u8);union U{x:ManuallyDrop<S>}fn main(){}
