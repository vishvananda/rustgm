struct Owned(u8);fn make(value:Owned)->impl FnOnce(bool)->Owned{move|flag|match flag{true=>value,false=>Owned(7)}}
