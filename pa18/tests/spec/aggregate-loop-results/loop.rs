enum Maybe<T> {
    Some(T),
    None,
}

enum Outcome<T, E> {
    Ok(T),
    Err(E),
}

enum Input {
    Ready(u8),
    Pending,
    Failed(&'static u8),
}

static FAILURE: u8 = 19;

fn from_loop(input: Input) -> Outcome<Maybe<u8>, &'static u8> {
    let result = loop {
        match input {
            Input::Ready(properties) => break Outcome::Ok(Maybe::Some(properties)),
            Input::Pending => break Outcome::Ok(Maybe::None),
            Input::Failed(error) => break Outcome::Err(error),
        }
    };
    result
}

fn from_block(input: Input) -> Outcome<Maybe<u8>, &'static u8> {
    let result = 'result: {
        match input {
            Input::Ready(properties) => break 'result Outcome::Ok(Maybe::Some(properties)),
            Input::Pending => break 'result Outcome::Ok(Maybe::None),
            Input::Failed(error) => Outcome::Err(error),
        }
    };
    result
}

fn observe(result: Outcome<Maybe<u8>, &'static u8>) -> u8 {
    match result {
        Outcome::Ok(Maybe::Some(value)) => value,
        Outcome::Ok(Maybe::None) => 0,
        Outcome::Err(error) => *error,
    }
}

#[cfg(escaping)]
fn escape(value: &u8) -> Outcome<Maybe<u8>, &'static u8> {
    let result = loop {
        if *value == 0 {
            break Outcome::Ok(Maybe::None);
        }
        break Outcome::Err(value);
    };
    result
}

fn fail(value: u8) -> u8 {
    1 / value
}

fn check(actual: u8, expected: u8) {
    if actual != expected {
        let _ = fail(0);
    }
}

fn main() {
    check(observe(from_loop(Input::Ready(42))), 42);
    check(observe(from_loop(Input::Pending)), 0);
    check(observe(from_loop(Input::Failed(&FAILURE))), 19);
    check(observe(from_block(Input::Ready(41))), 41);
    check(observe(from_block(Input::Pending)), 0);
    check(observe(from_block(Input::Failed(&FAILURE))), 19);
}
