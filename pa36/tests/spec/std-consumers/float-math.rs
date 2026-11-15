// Floating-point functions that std provides on top of core. Exactly rounded
// operations print every digit; library functions print twelve significant digits.
fn main() {
    let values = [0.0f64, 0.5, 1.0, 2.0, 2.5, -2.5, 10.0, 1e10, 1e-10, -7.75];
    for value in values {
        println!(
            "{:>12} floor {} ceil {} round {} trunc {} fract {} even {} abs {} sqrt {}",
            value,
            value.floor(),
            value.ceil(),
            value.round(),
            value.trunc(),
            value.fract(),
            value.round_ties_even(),
            value.abs(),
            value.sqrt()
        );
    }
    println!("mul_add {} {} {}", 2.0f64.mul_add(3.0, 4.0), 0.1f64.mul_add(10.0, -1.0), 1e16f64.mul_add(1e16, -1e32));
    println!("powi {} {} {} {}", 2.0f64.powi(10), 2.0f64.powi(-2), (-3.0f64).powi(3), 1.5f64.powi(0));
    println!("euclid {} {} {} {}", 7.5f64.div_euclid(2.0), (-7.5f64).div_euclid(2.0), 7.5f64.rem_euclid(2.0), (-7.5f64).rem_euclid(2.0));
    println!("sign {} {} {} {}", (-2.0f64).signum(), 0.0f64.signum(), (-0.0f64).signum(), 3.0f64.copysign(-0.0));
    println!("special {} {} {} {}", (-1.0f64).sqrt(), f64::INFINITY.floor(), f64::NAN.round(), (1.0f64 / 0.0).recip());
    println!("clamp {} {} {} {}", 5.5f64.clamp(0.0, 1.0), (-5.5f64).max(f64::NAN), 2.0f64.min(1.0), 1e308f64 * 10.0);
    println!("f32 {} {} {} {} {}", 2.0f32.sqrt(), 7.9f32.floor(), (-7.5f32).round(), 0.1f32 + 0.2f32, 16777216.0f32 + 1.0);
    println!("f32 {} {} {}", 3.0f32.mul_add(2.0, 1.0), 1.0e20f32.powi(2), (2.0f32).powi(-126));

    println!("exp {:.12e} {:.12e} {:.12e} {:.12e}", 1.0f64.exp(), 0.0f64.exp(), (-1.0f64).exp(), 10.5f64.exp());
    println!("ln {:.12e} {:.12e} {:.12e} {}", 10.0f64.ln(), 2.0f64.ln(), 0.5f64.ln(), 0.0f64.ln());
    println!("log {:.12e} {:.12e} {:.12e} {:.12e}", 1000.0f64.log10(), 1024.0f64.log2(), 81.0f64.log(3.0), 1e-3f64.ln_1p());
    println!("pow {:.12e} {:.12e} {:.12e} {:.12e}", 2.0f64.powf(0.5), 10.0f64.powf(-2.5), 0.5f64.exp2(), 1e-5f64.exp_m1());
    println!("trig {:.12e} {:.12e} {:.12e}", 1.0f64.sin(), 1.0f64.cos(), 1.0f64.tan());
    println!("trig {:.12e} {:.12e} {:.12e}", std::f64::consts::FRAC_PI_6.sin(), std::f64::consts::FRAC_PI_3.cos(), std::f64::consts::FRAC_PI_4.tan());
    println!("inverse {:.12e} {:.12e} {:.12e} {:.12e}", 0.5f64.asin(), 0.5f64.acos(), 1.0f64.atan(), (-1.0f64).atan2(-1.0));
    println!("hyperbolic {:.12e} {:.12e} {:.12e} {:.12e}", 1.0f64.sinh(), 1.0f64.cosh(), 1.0f64.tanh(), 1.0f64.asinh());
    println!("roots {:.12e} {:.12e} {:.12e}", 27.0f64.cbrt(), 3.0f64.hypot(4.0), 2.0f64.sqrt().powi(2));
    let (sine, cosine) = 2.0f64.sin_cos();
    println!("identity {:.12e} {}", sine * sine + cosine * cosine, (1.0f64.exp().ln() - 1.0).abs() < 1e-15);
    println!("f32 lib {:.6e} {:.6e} {:.6e} {:.6e}", 1.0f32.exp(), 2.0f32.ln(), 0.5f32.sin(), 2.0f32.powf(1.5));
    println!("degrees {:.12e} {:.12e}", 180.0f64.to_radians(), std::f64::consts::PI.to_degrees());

    let samples: Vec<f64> = (1..=10).map(|n| f64::from(n) * 1.5).collect();
    let mean = samples.iter().sum::<f64>() / samples.len() as f64;
    let variance = samples.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / samples.len() as f64;
    println!("statistics {} {} {:.12e}", mean, variance, variance.sqrt());
    let mut estimate = 0.0f64;
    for k in 0..2000 {
        estimate += (if k % 2 == 0 { 4.0 } else { -4.0 }) / f64::from(2 * k + 1);
    }
    println!("leibniz {} {:.3e}", estimate, (estimate - std::f64::consts::PI).abs());
    let roots = {
        let (a, b, c) = (1.0f64, -3.0, 2.0);
        let discriminant = (b * b - 4.0 * a * c).sqrt();
        ((-b + discriminant) / (2.0 * a), (-b - discriminant) / (2.0 * a))
    };
    println!("quadratic {:?} {}", roots, 1e300f64.hypot(1e300).is_finite());
    println!("rounding {} {} {} {}", (2.5f64).round(), (3.5f64).round(), (-0.4f64).round(), 0.1 + 0.2 == 0.3);
    println!("to int {} {} {}", 2.9f64.floor() as i64, (-2.9f64).ceil() as i64, (1e19f64).round() as u64);
}
