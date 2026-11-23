#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

enum Cache<T> {
    Empty,
    Filled(T),
}

impl<T> Cache<T> {
    fn remember<F: FnOnce() -> T>(&mut self, create: F) -> &mut T {
        if let Self::Empty = self {
            *self = Self::Filled(create());
        }
        match self {
            Self::Filled(value) => value,
            Self::Empty => loop {},
        }
    }
}

struct Cursor<I: Iterator> {
    iter: I,
    saved: Cache<Option<I::Item>>,
}

impl<I: Iterator> Cursor<I> {
    fn peek(&mut self) -> Option<&I::Item> {
        let iter = &mut self.iter;
        match self.saved.remember(|| iter.next()) {
            Some(item) => Some(item),
            None => None,
        }
    }
}

struct Source<'a> {
    item: &'a u16,
    returned: bool,
}

impl<'a> Iterator for Source<'a> {
    type Item = &'a u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.returned {
            None
        } else {
            self.returned = true;
            Some(self.item)
        }
    }
}

fn experiment() -> i32 {
    let value = 7;
    let iter = Source {
        item: &value,
        returned: false,
    };
    let mut cursor = Cursor {
        iter,
        saved: Cache::Empty,
    };
    let Some(first) = cursor.peek() else {
        return 1;
    };
    if **first != 7 {
        return 2;
    }
    let Some(cached) = cursor.peek() else {
        return 3;
    };
    if **cached == 7 { 0 } else { 4 }
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    experiment()
}

#[cfg(not(guest_probe))]
fn main() {
    assert_eq!(experiment(), 0);
}
