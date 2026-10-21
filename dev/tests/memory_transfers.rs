//! Sized pointer memory operations preserve ownership snapshots and raw authority.
#[path = "memory_transfers/capabilities.rs"]
mod capabilities;
#[path = "harness/checked_sources.rs"]
mod checked;
#[path = "harness/native_references.rs"]
mod native;
#[path = "harness/reconstruction_work.rs"]
mod work;
use std::path::Path;
const CASES: &[(&str, i32, [i32; 3], &str)] = &[
    ("const-generic", 0, [0, 0, 0], ""),
    ("wide-transfer", 0, [0, 0, 0], ""),
    ("packed-aggregate", 0, [0, 0, 0], ""),
    ("union-image-copy", 0, [0, 0, 0], ""),
    (
        "union-image-mask",
        1,
        [1, 1, 1],
        "constant union scalar read uses uninitialized bytes",
    ),
    ("scalar-runtime", 0, [0, 0, 0], ""),
    ("const-scalar", 0, [0, 0, 0], ""),
    ("const-product", 0, [0, 0, 0], ""),
    ("aggregate-runtime", 0, [0, 0, 0], ""),
    ("array-const", 0, [0, 0, 0], ""),
    ("generic", 0, [0, 0, 0], ""),
    ("reified", 0, [0, 0, 0], ""),
    ("alias-core", 0, [0, 0, 0], ""),
    ("std-import", 0, [0, 0, 0], ""),
    ("shadow", 0, [0, 0, 0], ""),
    ("zst-live", 0, [0, 0, 0], ""),
    (
        "zst-null",
        0,
        [1, 1, 1],
        "constant pointer has no allocation provenance",
    ),
    (
        "zst-unaligned",
        1,
        [1, 1, 1],
        "constant pointer has no allocation provenance",
    ),
    (
        "read-unsafe",
        1,
        [1, 1, 1],
        "call to unsafe function requires an unsafe context",
    ),
    (
        "write-unsafe",
        1,
        [1, 1, 1],
        "call to unsafe function requires an unsafe context",
    ),
    ("wrong-arity", 1, [1, 1, 1], "call argument count mismatch"),
    ("wrong-type", 1, [1, 1, 1], "incompatible types"),
    (
        "shared-derived",
        1,
        [1, 1, 1],
        "constant raw pointer write is derived from a shared reference",
    ),
    ("const-cast-writable", 0, [0, 0, 0], ""),
    (
        "stale-read",
        1,
        [1, 1, 1],
        "constant pointer refers to expired storage",
    ),
    (
        "stale-write",
        1,
        [1, 1, 1],
        "constant pointer refers to expired storage",
    ),
    (
        "packed-field",
        1,
        [1, 1, 1],
        "constant raw pointer read is unaligned",
    ),
    (
        "invalid-bool",
        1,
        [1, 1, 1],
        "constant union scalar read has invalid bool representation",
    ),
    ("union-alias", 0, [0, 0, 0], ""),
    ("static-read", 0, [0, 0, 0], ""),
    (
        "static-write",
        1,
        [1, 1, 1],
        "constant execution cannot write static storage",
    ),
    ("pointer-value", 0, [0, 0, 0], ""),
    ("noncopy-boundary", 0, [0, 0, 0], ""),
    (
        "const-reified",
        1,
        [1, 1, 1],
        "non-const calls are not allowed in constant initializers",
    ),
    (
        "zst-null-write",
        0,
        [1, 1, 1],
        "constant pointer has no allocation provenance",
    ),
    (
        "zst-expired",
        0,
        [1, 1, 1],
        "constant pointer refers to expired storage",
    ),
    ("union-repair", 0, [0, 0, 0], ""),
    ("union-initialize", 0, [0, 0, 0], ""),
    ("copied-write-authority", 0, [0, 0, 0], ""),
    (
        "copied-shared-authority",
        1,
        [1, 1, 1],
        "constant raw pointer write is derived from a shared reference",
    ),
    (
        "wide-value",
        0,
        [1, 1, 1],
        "constant slice projection is not implemented",
    ),
    ("ordinary-scalar", 0, [0, 0, 0], ""),
];
#[test]
fn pointer_memory_matches_independent_objects() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa28/pointer-memory-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&out)?;
    for &(name, rust, statuses, reason) in CASES {
        let path = root.join(format!("pa28/tests/spec/pointer-memory/{name}.rs"));
        if statuses == [statuses[0]; 3] {
            checked::check(&path, &out, name, (rust, statuses[0], reason))?;
        } else {
            checked::check_modes(&path, &out, name, (rust, statuses, reason))?;
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned rustc and CPPGM backend"]
fn pointer_memory_native_snapshots_and_reified_abi() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "parallel/pa28/pointer-memory-native",
        &[
            (
                "const-generic",
                "const unsafe fn take<T>(p:*const T)->T{unsafe{core::ptr::read(p)}}const unsafe fn put<T>(p:*mut T,x:T){unsafe{core::ptr::write(p,x)}}const X:u32={let mut x=(3u8,7u32);unsafe{let y=take(&raw const x);put(&raw mut x,(y.0,37));x.1}};fn main(){check(X==37);}",
                false,
            ),
            (
                "wide-bound-panic",
                "fn index()->usize{3}fn main(){let a=[3u32,7,37];let mut r:&[u32]=&a;unsafe{let s=core::ptr::read(&raw const r);core::ptr::write(&raw mut r,s);}let _=r[index()];}",
                true,
            ),
            (
                "packed-aggregate",
                "#[derive(Copy,Clone)]#[repr(C,packed)]struct P{a:u8,b:u32}fn main(){let mut p=P{a:3,b:7};unsafe{let mut q=core::ptr::read(&raw const p);q.b=37;core::ptr::write(&raw mut p,q);}check(p.a==3&&p.b==37);}",
                false,
            ),
            (
                "operand-evaluation",
                "fn pointer(x:&mut u32,n:&mut u32)->*mut u32{*n=*n+1;x as *mut u32}fn argument(n:&mut u32)->u32{*n=*n+1;37}fn main(){let mut x=7u32;let mut n=0u32;unsafe{core::ptr::write(pointer(&mut x,&mut n),argument(&mut n));let y=core::ptr::read(pointer(&mut x,&mut n));check(y==37);}check(x==37&&n==3);}",
                false,
            ),
            (
                "whole-union-image",
                "#[derive(Copy,Clone)]union U{a:u8,b:bool}const X:u8={let mut u=U{a:2};unsafe{let v=core::ptr::read(&raw const u);core::ptr::write(&raw mut u,v);u.a}};fn main(){check(X==2);}",
                false,
            ),
            (
                "scalars",
                "fn main(){let mut x=7u128;unsafe{let y=core::ptr::read(&raw const x);core::ptr::write(&raw mut x,y+30);}check(x==37);let mut f=1.5f64;unsafe{let g=core::ptr::read(&raw const f);core::ptr::write(&raw mut f,g+2.0);}check(f==3.5);}",
                false,
            ),
            (
                "padded-snapshots",
                "#[derive(Copy,Clone)]#[repr(C)]struct P{a:u8,b:u64}fn main(){let mut p=P{a:3,b:7};unsafe{let q=core::ptr::read(&raw const p);core::ptr::write(&raw mut p,P{a:5,b:37});check(q.a==3&&q.b==7);core::ptr::write(&raw mut p,p);check(p.a==5&&p.b==37);}}",
                false,
            ),
            (
                "reified",
                "#[derive(Copy,Clone)]#[repr(C)]struct P{a:u8,b:u64}unsafe fn take<T>(p:*const T)->T{unsafe{core::ptr::read(p)}}unsafe fn put<T>(p:*mut T,x:T){unsafe{core::ptr::write(p,x)}}fn main(){let mut p=P{a:3,b:37};let f:unsafe fn(*const P)->P=take::<P>;let g:unsafe fn(*mut P,P)=put::<P>;let r:unsafe fn(*const P)->P=core::ptr::read::<P>;let w:unsafe fn(*mut P,P)=core::ptr::write::<P>;unsafe{let q=f(&raw const p);g(&raw mut p,q);let q=r(&raw const p);w(&raw mut p,q);}check(p.a==3&&p.b==37);}",
                false,
            ),
            (
                "arrays-and-wide-values",
                "fn main(){let mut a=[3u32,7,37];unsafe{let b=core::ptr::read(&raw const a);core::ptr::write(&raw mut a,[b[2],b[0],b[1]]);}check(a[0]==37&&a[1]==3&&a[2]==7);let mut r:&[u32]=&a;unsafe{let s=core::ptr::read(&raw const r);core::ptr::write(&raw mut r,s);}check(r[0]==37&&r[2]==7);}",
                false,
            ),
            (
                "symbolic-zst",
                "#[derive(Copy,Clone)]#[repr(align(16))]struct Z;const X:Z={let mut z=Z;unsafe{let v=core::ptr::read(&raw const z);core::ptr::write(&raw mut z,v);z}};fn main(){let mut z=X;unsafe{let y=core::ptr::read(&raw const z);core::ptr::write(&raw mut z,y);}}",
                false,
            ),
            (
                "constant-union-bytes",
                "union U{a:u8,b:u32}const X:u32={let mut u=U{a:7};unsafe{let p=&raw const u.b;core::ptr::write(&raw mut u.b,37);core::ptr::read(p)}};union B{a:u8,b:bool}const Y:bool={let mut u=B{a:2};unsafe{core::ptr::write(&raw mut u.b,true);core::ptr::read(&raw const u.b)}};fn main(){check(X==37&&Y);}",
                false,
            ),
        ],
    )
}
#[test]
fn pointer_memory_work_has_bounded_overhead() -> Result<(), Box<dyn std::error::Error>> {
    work::compare("parallel/pa28/pointer-memory-loop", |memory, count| {
        let operation = if memory {
            "unsafe{core::ptr::write(&raw mut x,core::ptr::read(&raw const x)+1);}"
        } else {
            "x=x+1;"
        };
        format!(
            "const fn f(n:u32)->u32{{let mut x=0u32;let mut i=0u32;while i<n{{{operation}i=i+1;}}x}}const X:u32=f({count});fn main(){{let _=X;}}"
        )
    })
}
