pub extern "C" fn add(a:u32,b:u32)->u32{a+b}
pub extern "C" fn real(a:f64,b:f64)->f64{a+b}
pub unsafe extern "C" fn guarded(v:u32)->u32{v+1}
pub extern "C" fn identity(v:&u8)->&u8{v}
pub extern "C" fn update(v:&mut u8)->&mut u8{*v+=1;v}
pub extern fn implicit(v:u32)->u32{v+2}
pub fn ordinary(v:u32)->u32{v+3}
#[unsafe(export_name="rustgm_metadata_exact")]pub extern "C" fn exact(v:u32)->u32{v+4}

