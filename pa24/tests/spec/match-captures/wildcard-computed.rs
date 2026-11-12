struct Owned(u8);fn make(value:Owned)->impl FnOnce(){move||{let _=match true{true=>value,false=>Owned(7)};}}
