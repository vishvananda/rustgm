#![no_std]
#![no_main]

use core::fmt::{self, Write};

#[link(name = "c")]
unsafe extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn abort() -> !;
}

/// Buffered standard output through `core::fmt::Write` and libc `write`.
struct Out {
    buf: [u8; 256],
    len: usize,
}

impl Out {
    fn flush(&mut self) {
        let mut done = 0;
        while done < self.len {
            let n = unsafe { write(1, self.buf.as_ptr().add(done), self.len - done) };
            if n <= 0 {
                unsafe { abort() }
            }
            done += n as usize;
        }
        self.len = 0;
    }
}

impl Write for Out {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &b in s.as_bytes() {
            if self.len == self.buf.len() {
                self.flush();
            }
            self.buf[self.len] = b;
            self.len += 1;
        }
        Ok(())
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    unsafe { abort() }
}

#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let mut out = Out { buf: [0; 256], len: 0 };
    let status = match run(&mut out) {
        Ok(()) => 0,
        Err(fmt::Error) => 1,
    };
    out.flush();
    status
}

use core::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4};
use core::time::Duration;

fn run(out: &mut Out) -> fmt::Result {
    let short = Duration::from_millis(1500);
    let long = Duration::new(3661, 500_000_000);
    writeln!(out, "{:?} {:?} {:?} {:?}", short, long, Duration::from_micros(42), Duration::from_nanos(7))?;
    writeln!(out, "{:?} {:?} {:?}", Duration::ZERO, Duration::from_secs(86400), Duration::MAX)?;
    writeln!(out, "{:.2?} {:8.1?}| {:<10?}| {:.0?}", long, short, Duration::from_millis(5), Duration::from_micros(1500))?;
    writeln!(out, "parts {} {} {} {}", long.as_secs(), long.subsec_millis(), long.subsec_nanos(), short.as_millis())?;
    writeln!(out, "float {} {} {}", short.as_secs_f64(), long.as_secs_f32(), Duration::from_secs_f64(2.75).as_micros())?;
    writeln!(out, "arith {:?} {:?} {:?} {:?}", long + short, long - short, short * 4, long / 3)?;
    writeln!(
        out,
        "checked {:?} {:?} {:?}",
        short.checked_sub(long),
        long.checked_sub(short),
        Duration::MAX.checked_add(short)
    )?;
    writeln!(out, "saturating {:?} {:?}", short.saturating_sub(long), Duration::MAX.saturating_add(long) == Duration::MAX)?;
    writeln!(out, "compare {} {} {:?}", short < long, short == Duration::from_secs_f32(1.5), short.cmp(&long))?;
    writeln!(out, "ratio {} {:?} {:?}", long.as_nanos() / short.as_nanos(), short.mul_f64(1.5), long.div_f32(2.0))?;
    writeln!(out, "abs_diff {:?} zero {} sum {:?}", short.abs_diff(long), Duration::ZERO.is_zero(), [short, long, short].iter().sum::<Duration>())?;
    writeln!(out, "try {:?} {}", Duration::try_from_secs_f64(-1.0).is_err(), Duration::try_from_secs_f64(f64::NAN).unwrap_err())?;

    let local = Ipv4Addr::new(127, 0, 0, 1);
    let private = Ipv4Addr::from([192, 168, 10, 250]);
    writeln!(out, "{} {} {:?} {}", local, private, Ipv4Addr::BROADCAST, Ipv4Addr::from(0x0a00_0001u32))?;
    writeln!(out, "[{:>18}] [{:<18}] {:?}", private, local, private.octets())?;
    writeln!(
        out,
        "v4 {} {} {} {} {} {}",
        local.is_loopback(),
        private.is_private(),
        Ipv4Addr::new(224, 0, 0, 5).is_multicast(),
        Ipv4Addr::UNSPECIFIED.is_unspecified(),
        Ipv4Addr::new(169, 254, 1, 1).is_link_local(),
        u32::from(private)
    )?;
    writeln!(out, "v4 order {} {:?}", local < private, private.to_ipv6_mapped())?;
    for text in ["10.1.2.3", "255.255.255.255", "256.1.1.1", "1.2.3", "1.2.3.4.5", "01.2.3.4", " 1.2.3.4", "a.b.c.d", ""] {
        writeln!(out, "parse v4 {:?} -> {:?}", text, text.parse::<Ipv4Addr>().ok())?;
    }
    let v6 = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0x8a2e, 0x7334);
    writeln!(out, "{} {} {} {}", v6, Ipv6Addr::LOCALHOST, Ipv6Addr::UNSPECIFIED, Ipv6Addr::new(1, 0, 0, 2, 0, 0, 0, 3))?;
    writeln!(out, "{} {} {:?}", Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc0a8, 1), Ipv6Addr::new(0xfe80, 0, 0, 0, 1, 2, 3, 4), v6.segments())?;
    writeln!(out, "v6 {} {} {:?} {:?}", Ipv6Addr::LOCALHOST.is_loopback(), v6.is_multicast(), Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x0102, 0x0304).to_ipv4_mapped(), v6.to_ipv4())?;
    for text in ["::1", "2001:db8::8a2e:7334", "::ffff:10.0.0.1", "1:2:3:4:5:6:7:8", "1::2::3", "12345::", "fe80::1%eth0", "::", "1:2:3:4:5:6:7"] {
        writeln!(out, "parse v6 {:?} -> {:?}", text, text.parse::<Ipv6Addr>().ok())?;
    }
    let socket = SocketAddr::new(IpAddr::V4(private), 8080);
    let socket6 = SocketAddr::from((v6, 443));
    writeln!(out, "{} {} {:?} {}", socket, socket6, socket, socket.port())?;
    writeln!(out, "socket {} {} {:?}", socket.is_ipv4(), socket6.ip().is_ipv6(), SocketAddrV4::new(local, 1).ip())?;
    for text in ["127.0.0.1:80", "[::1]:65535", "127.0.0.1", "[::1]", "1.2.3.4:65536", "::1:80"] {
        writeln!(out, "parse socket {:?} -> {:?}", text, text.parse::<SocketAddr>().ok())?;
    }
    let error = "nope".parse::<IpAddr>().unwrap_err();
    writeln!(out, "error {} | {:?} | {}", error, error, "x".parse::<SocketAddr>().unwrap_err())?;
    writeln!(out, "ip {:?} {} {}", "::1".parse::<IpAddr>(), IpAddr::from([1, 2, 3, 4]), IpAddr::V6(v6).is_loopback())?;
    Ok(())
}
