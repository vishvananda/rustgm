    struct L1Lut {
        l2_luts: [L2Lut; 2],
    }

    struct L2Lut {
        singles: &'static [(Range, i16)],
        multis: &'static [(u16, [u16; 3])],
    }

    #[derive(Copy, Clone)]
    struct Range {
        start: u16,
        len: u8,
        parity: bool,
    }

    impl Range {
        const fn new(range: RangeInclusive<u16>, parity: bool) -> Self {
            let start = *range.start();
            let end = *range.end();
            assert!(start <= end);

            let len = end - start;
            assert!(len <= 255);

            Self { start, len: len as u8, parity }
        }

        const fn singleton(start: u16) -> Self {
            Self::new(start..=start, false)
        }

        const fn step_by_1(range: RangeInclusive<u16>) -> Self {
            Self::new(range, false)
        }

        const fn step_by_2(range: RangeInclusive<u16>) -> Self {
            Self::new(range, true)
        }

        const fn start(&self) -> u16 {
            self.start
        }

        const fn end(&self) -> u16 {
            self.start + self.len as u16
        }
    }

