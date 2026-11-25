#![feature(no_core)]
#![no_core]

extern crate base;

#[repr(transparent)]
struct ClientPtr<T: ?base::Sized> {
    raw: base::NonNull<T>,
}

impl<T: ?base::Sized + base::Unsize<U>, U: ?base::Sized> base::CoerceUnsized<ClientPtr<U>>
    for ClientPtr<T>
{
}

impl<T: ?base::Sized + base::Unsize<U>, U: ?base::Sized> base::DispatchFromDyn<ClientPtr<U>>
    for ClientPtr<T>
{
}

struct Gauge(u8);

impl base::Meter for Gauge {
    fn read(&self) -> u8 {
        self.0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_nonnull_dispatch() -> u8 {
    let gauge = Gauge(41);
    let meter: &dyn base::Meter = &gauge;
    let _ = base::check_pattern_dispatch::<*const Gauge, *const dyn base::Meter>();
    meter.read()
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_client_dispatch() -> u8 {
    let _ = base::require_dispatch::<base::NonNull<Gauge>, base::NonNull<dyn base::Meter>>();
    base::require_dispatch::<ClientPtr<Gauge>, ClientPtr<dyn base::Meter>>()
}
