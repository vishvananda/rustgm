#![feature(no_core, location_fixture)]
#![no_core]

fn require_sized<T: base::Sized>() {}

pub fn rejected() {
    require_sized::<base::ManuallyDrop<[u8]>>();
}
