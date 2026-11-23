#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[derive(Clone, Copy)]
struct Key(u32);

#[derive(Clone, Copy)]
enum Subject {
    Direct(u32),
    Indirect(usize),
}

#[derive(Clone, Copy)]
struct Syntax {
    subject: Subject,
    source: Key,
}

struct Checker<'a> {
    rows: &'a [Syntax],
    definitions: &'a [u32],
}

impl Checker<'_> {
    #[inline(never)]
    fn declared_self_bounds(
        &self,
        owner: usize,
        definition: u32,
    ) -> impl Iterator<Item = (Key, u32)> + '_ {
        self.rows.iter().filter_map(move |syntax| {
            let subject = match syntax.subject {
                Subject::Direct(id) => Some(id),
                Subject::Indirect(key) => self.definitions.get(key).copied(),
            };
            (owner == 0 && subject == Some(definition)).then_some((syntax.source, definition))
        })
    }
}

fn checker() -> Checker<'static> {
    const ROWS: &[Syntax] = &[
        Syntax {
            subject: Subject::Direct(11),
            source: Key(30),
        },
        Syntax {
            subject: Subject::Indirect(0),
            source: Key(42),
        },
        Syntax {
            subject: Subject::Direct(11),
            source: Key(17),
        },
    ];
    Checker {
        rows: ROWS,
        definitions: &[19],
    }
}

#[inline(never)]
fn bounds(checker: &Checker<'_>, owner: usize, definition: u32) -> (usize, Option<usize>, u32) {
    let rows = checker
        .declared_self_bounds(owner, definition)
        .map(|(source, _)| source);
    let (minimum, maximum) = rows.size_hint();
    let mut sum = 0;
    for row in rows {
        sum += row.0;
    }
    (minimum, maximum, sum)
}

fn check() -> i32 {
    let source = checker();
    if bounds(&source, 0, 11).2 != 47 || bounds(&source, 0, 19).2 != 42 {
        return 1;
    }
    let (minimum, maximum, sum) = bounds(&source, 1, 11);
    if minimum != 0 || maximum != Some(3) || sum != 0 {
        return 2;
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
