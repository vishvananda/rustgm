#![feature(no_core, const_trait_impl, const_destruct)]
#![no_core]

extern crate base;

use base::{Destruct, Show, Sync, Text, Value, Visible};

fn read_show<S: Show>(item: S) -> u32 {
    item.show()
}

fn read_sync_show<S: Show + Sync>(item: S) -> u32 {
    item.show()
}

const fn read_const<V: [const] Value + [const] Destruct>(item: V) -> u32 {
    item.get()
}

const BOXED_CONST: u32 = read_const(base::const_box(19));

fn precise_after_short_borrow() -> u32 {
    let kept = Visible(27);
    let shown = {
        let temp = 99;
        base::precise(&kept, &temp)
    };
    read_show(shown)
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_rpit(
    a: &mut u32,
    b: &mut u32,
    c: &mut u32,
    d: &mut u32,
    e: &mut u32,
) {
    let text = Text::new(42);
    *a = read_show(text.display());
    *b = read_show(base::make_visible(7));
    *c = precise_after_short_borrow();
    *d = BOXED_CONST;
    *e = read_sync_show(base::auto_bounded(88));
}
