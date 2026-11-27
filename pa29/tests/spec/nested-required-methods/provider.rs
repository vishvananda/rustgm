pub fn nested(value: u8) -> u8 {
    trait Convert {
        fn convert(value: u8) -> u8;
    }

    impl Convert for u8 {
        fn convert(value: u8) -> u8 {
            value
        }
    }

    <u8 as Convert>::convert(value)
}
