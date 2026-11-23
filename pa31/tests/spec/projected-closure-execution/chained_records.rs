#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

use core::iter::once;

#[derive(Clone, Copy)]
struct TypeRef(u16);

struct Signature {
    parameters: [TypeRef; 2],
    result: TypeRef,
}

struct Field(TypeRef);

impl Field {
    fn ty(&self) -> TypeRef {
        self.0
    }
}

struct Record {
    fields: [Field; 2],
}

struct Storage {
    statics: [Field; 2],
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

fn scan(roots: impl Iterator<Item = TypeRef>) -> u16 {
    let mut total = 0_u16;
    for root in roots {
        total = total.wrapping_mul(10).wrapping_add(root.0);
    }
    total
}

fn read(functions: &[Signature], records: &[Record], storage: Option<&Storage>) -> u16 {
    let roots = functions
        .iter()
        .flat_map(|function| function.parameters.iter().copied().chain(once(function.result)));
    scan(
        roots
            .chain(records.iter().flat_map(|record| record.fields.iter().map(Field::ty)))
            .chain(storage.into_iter().flat_map(|storage| storage.statics.iter().map(Field::ty))),
    )
}

fn check() -> i32 {
    let functions = [Signature {
        parameters: [TypeRef(1), TypeRef(2)],
        result: TypeRef(3),
    }];
    let records = [Record {
        fields: [Field(TypeRef(4)), Field(TypeRef(5))],
    }];
    let storage = Storage {
        statics: [Field(TypeRef(6)), Field(TypeRef(7))],
    };
    if read(&functions, &records, Some(&storage)) == (1_234_567_u32 as u16)
        && read(&functions, &records, None) == 12_345
        && read(&[], &records, None) == 45
        && read(&[], &[], Some(&storage)) == 67
    {
        0
    } else {
        1
    }
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    check()
}

#[cfg(not(guest_probe))]
fn main() {
    std::process::exit(check());
}
