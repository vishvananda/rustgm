#![feature(no_core)]
#![no_core]
extern crate base;

use base::{Payload, Signal};

const DIRECT: u8 = Signal::<Payload<u8>, 3>::FLAG;
const CALL: u8 = Signal::<Payload<u8>, 4>::flag();
const WIDTH: usize = Signal::<Payload<u8>, 9>::WIDTH;
const WIDTH_CALL: usize = Signal::<Payload<u8>, 11>::width();
const LENGTH: usize = base::length::<13>();
const WAITING: u8 = match Signal::<Payload<u8>, 2>::WAITING {
    Signal::Idle => 42,
    Signal::Payload(_) => 0,
};
const STOPPED: u8 = match Signal::<Payload<u8>, 6>::STOPPED {
    Signal::Payload(Payload::None) => 31,
    _ => 0,
};

pub fn direct() -> u8 { DIRECT }
pub fn call() -> u8 { CALL }
pub fn waiting() -> u8 { WAITING }
pub fn stopped() -> u8 { STOPPED }
pub fn width() -> usize { WIDTH }
pub fn width_call() -> usize { WIDTH_CALL }
pub fn length() -> usize { LENGTH }
