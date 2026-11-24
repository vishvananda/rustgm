#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang = "phantom_data"]
pub struct PhantomData<T: PointeeSized>;

pub struct Context<'a> {
    pub value: &'a u8,
    _marker: PhantomData<fn(&'a ()) -> &'a ()>,
}

impl<'a> Context<'a> {
    pub fn new(value: &'a u8) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    pub fn read(&self) -> u8 {
        *self.value
    }
}

pub struct Hooks {
    pub inspect: for<'b> fn(&'b u8) -> u8,
}

fn next_tag<'b>(x: &'b u8) -> u8 {
    match *x {
        6 => 7,
        _ => 0,
    }
}

pub fn default_hooks() -> Hooks {
    Hooks { inspect: next_tag }
}

pub fn run_hook(hooks: Hooks, value: &u8) -> u8 {
    (hooks.inspect)(value)
}

pub fn apply_borrowed<'a>(f: fn(&'a u8) -> u8, value: &'a u8) -> u8 {
    f(value)
}
