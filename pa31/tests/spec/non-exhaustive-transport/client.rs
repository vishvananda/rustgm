#![feature(no_core)]
#![no_core]

extern crate base;

#[cfg(accepted)]
pub fn read_unit() {
    let base::OpenUnit { .. } = base::unit();
}

#[cfg(accepted)]
pub fn read_tuple() -> u8 {
    let base::OpenTuple { 0: value, .. } = base::tuple(34);
    value
}

#[cfg(accepted)]
pub fn read_record() -> u8 {
    let base::OpenRecord { value, .. } = base::record(34);
    value
}

#[cfg(accepted)]
pub fn read_enum() -> u8 {
    match base::first() {
        base::OpenEnum::First => 34,
        _ => 1,
    }
}

#[cfg(unit_value)]
pub fn unit_value() -> base::OpenUnit {
    base::OpenUnit
}

#[cfg(renamed_value)]
pub fn renamed_value() -> base::Renamed {
    base::Renamed
}

#[cfg(unit_literal)]
pub fn unit_literal() -> base::OpenUnit {
    base::OpenUnit {}
}

#[cfg(tuple_value)]
pub fn tuple_value() -> base::OpenTuple {
    base::OpenTuple(34)
}

#[cfg(record_literal)]
pub fn record_literal() -> base::OpenRecord {
    base::OpenRecord { value: 34 }
}

#[cfg(closed_pattern)]
pub fn closed_pattern() -> u8 {
    let base::OpenRecord { value } = base::record(34);
    value
}

#[cfg(exhaustive_match)]
pub fn exhaustive_match() -> u8 {
    match base::first() {
        base::OpenEnum::First => 34,
        base::OpenEnum::Second => 1,
    }
}
