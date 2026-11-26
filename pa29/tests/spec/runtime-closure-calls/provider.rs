pub fn shared(input: u8, argument: u8) -> u8 {
    let mix = |operand: u8| input ^ operand;
    mix(3) ^ mix(argument)
}

pub fn update(input: u8) -> u8 {
    let mut value = input;
    let reference = &mut value;
    let mut update = || {
        let previous = **&reference;
        *reference = match previous {
            7 => 11,
            _ => 23,
        };
    };
    update();
    value
}

pub fn consume(input: u8, argument: u8) -> u8 {
    struct Payload(u8, u8);
    let payload = Payload(input, 2);
    let consume = move |operand: u8| {
        let owned = payload;
        owned.0 ^ owned.1 ^ operand
    };
    consume(argument)
}
