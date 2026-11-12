struct Owned(u8);fn code(){let value=Owned(7);let run=move|flag|match flag{true=>value,false=>Owned(7)};let _=run(true);let _=run(false);}
