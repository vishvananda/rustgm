extern crate base;

fn escape() -> base::Marker<'static> {
    let local = 42;
    base::from_ref(&local)
}
