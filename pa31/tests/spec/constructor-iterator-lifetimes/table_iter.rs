#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

struct Requirement(u16);

struct Located {
    value: Requirement,
}

struct CopyInputs {
    goals: [Requirement; 2],
}

struct BuiltinInputs {
    predicates: [Located; 2],
    source_copy: Option<CopyInputs>,
}

struct BodyInputs {
    type_assumptions: [Located; 2],
    builtins: Option<BuiltinInputs>,
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

fn number(input: &BodyInputs) -> u16 {
    let requirements = input
        .type_assumptions
        .iter()
        .map(|row| &row.value)
        .chain(
            input
                .builtins
                .iter()
                .flat_map(|builtins| builtins.predicates.iter().map(|row| &row.value)),
        )
        .chain(
            input
                .builtins
                .iter()
                .filter_map(|builtins| builtins.source_copy.as_ref())
                .flat_map(|copy| &copy.goals),
        );
    let mut number = 0u16;
    for requirement in requirements {
        number = number.wrapping_mul(10).wrapping_add(requirement.0);
    }
    number
}

fn check() -> i32 {
    let mut input = BodyInputs {
        type_assumptions: [
            Located { value: Requirement(1) },
            Located { value: Requirement(2) },
        ],
        builtins: Some(BuiltinInputs {
            predicates: [
                Located { value: Requirement(3) },
                Located { value: Requirement(4) },
            ],
            source_copy: Some(CopyInputs {
                goals: [Requirement(5), Requirement(6)],
            }),
        }),
    };
    if number(&input) != 12_345_u16.wrapping_mul(10).wrapping_add(6) {
        return 1;
    }
    if let Some(builtins) = &mut input.builtins {
        builtins.source_copy = None;
    }
    if number(&input) != 1_234 {
        return 2;
    }
    input.builtins = None;
    if number(&input) != 12 {
        return 3;
    }
    0
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
