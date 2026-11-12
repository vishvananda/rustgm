struct Owned(u8);fn code(){let value=Owned(7);let run=move||{let _=match true{true=>value,false=>Owned(7)};};run();run();}
