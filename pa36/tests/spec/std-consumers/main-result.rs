// `main` returns an error: the runtime prints it to standard error and the
// process exits with status 1 after running destructors.
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum ConfigError {
    Missing(&'static str),
    Invalid { key: &'static str, source: ParseIntError },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing(key) => write!(f, "missing key {key}"),
            ConfigError::Invalid { key, .. } => write!(f, "invalid value for {key}"),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::Missing(_) => None,
            ConfigError::Invalid { source, .. } => Some(source),
        }
    }
}

struct Guard(&'static str);

impl Drop for Guard {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn lookup(settings: &[(&'static str, &str)], key: &'static str) -> Result<u32, ConfigError> {
    let (_, value) = settings.iter().find(|(name, _)| *name == key).ok_or(ConfigError::Missing(key))?;
    value.parse().map_err(|source| ConfigError::Invalid { key, source })
}

fn chain(error: &dyn Error) -> Vec<String> {
    let mut messages = vec![error.to_string()];
    let mut current = error.source();
    while let Some(inner) = current {
        messages.push(inner.to_string());
        current = inner.source();
    }
    messages
}

fn total(settings: &[(&'static str, &str)]) -> Result<u32, Box<dyn Error>> {
    let width = lookup(settings, "width")?;
    let height = lookup(settings, "height")?;
    let depth: u32 = "3".parse()?;
    width.checked_mul(height).and_then(|area| area.checked_mul(depth)).ok_or_else(|| "volume overflow".into())
}

fn main() -> Result<(), Box<dyn Error>> {
    let _guard = Guard("main guard");
    let good = [("width", "6"), ("height", "7")];
    println!("total {:?}", total(&good).map_err(|error| error.to_string()));
    println!("overflow {:?}", total(&[("width", "4000000000"), ("height", "2")]).map_err(|error| error.to_string()));
    for settings in [&[("width", "6")][..], &[("width", "6"), ("height", "tall")][..]] {
        match lookup(settings, "height") {
            Ok(value) => println!("unexpected {value}"),
            Err(error) => println!("error {} | {:?} | chain {:?}", error, error, chain(&error)),
        }
    }
    let boxed: Box<dyn Error + Send + Sync> = From::from("plain message");
    let downcast = total(&[("width", "x")]).unwrap_err().downcast::<ConfigError>().map(|error| error.to_string());
    println!("boxed {} {:?} {:?}", boxed, boxed.source().is_none(), downcast);
    let converted: Result<u8, Box<dyn Error>> = "300".parse::<u8>().map_err(Into::into);
    println!("converted {:?}", converted.map_err(|error| error.to_string()));
    println!("returning the error");
    let area = total(&[("width", "6"), ("height", "-1")])?;
    println!("unreachable {area}");
    Ok(())
}
