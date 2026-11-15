// `main` returns an `ExitCode`; destructors run first and standard output is flushed.
use std::process::{ExitCode, Termination};

struct Guard(&'static str);

impl Drop for Guard {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn checked(values: &[u8]) -> Result<u8, String> {
    values.iter().try_fold(0u8, |total, value| total.checked_add(*value)).ok_or_else(|| format!("{values:?} overflows u8"))
}

fn main() -> ExitCode {
    let _guard = Guard("main guard");
    println!("success {:?} failure {:?}", ExitCode::SUCCESS == ExitCode::from(0), ExitCode::FAILURE == ExitCode::from(1));
    println!("unit reports {:?}", ().report() == ExitCode::SUCCESS);
    match checked(&[20, 22]) {
        Ok(total) => {
            print!("exit code {total}");
            ExitCode::from(total)
        }
        Err(message) => {
            println!("{message}");
            ExitCode::FAILURE
        }
    }
}
