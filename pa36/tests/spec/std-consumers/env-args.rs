use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("count {} program named {}", args.len(), !args[0].is_empty());
    for (index, argument) in args.iter().enumerate().skip(1) {
        println!("arg {} {:?} bytes {} chars {}", index, argument, argument.len(), argument.chars().count());
    }
    let mut iterator = env::args();
    println!("len {} last {:?} nth {:?}", iterator.len(), iterator.next_back(), iterator.nth(2));
    let os_args: Vec<OsString> = env::args_os().skip(1).collect();
    println!("os {} {:?}", os_args.len(), os_args.first());
    let flags: Vec<(&str, &str)> = args.iter().filter_map(|a| a.strip_prefix("--")).filter_map(|a| a.split_once('=')).collect();
    let numbers: Vec<i64> = args.iter().filter_map(|a| a.parse().ok()).collect();
    println!("flags {:?} numbers {:?} sum {}", flags, numbers, numbers.iter().sum::<i64>());

    println!("var {:?} {:?}", env::var("CONSUMER_NAME"), env::var("CONSUMER_EMPTY"));
    match env::var("CONSUMER_MISSING") {
        Ok(value) => println!("unexpected {value}"),
        Err(error) => println!("missing: {} / {:?}", error, error),
    }
    println!("var_os {:?} {:?}", env::var_os("CONSUMER_NAME"), env::var_os("CONSUMER_MISSING"));
    let mut variables: Vec<(String, String)> = env::vars().filter(|(key, _)| key.starts_with("CONSUMER_")).collect();
    variables.sort();
    println!("vars {:?} total {}", variables, env::vars().count());
    unsafe {
        env::set_var("CONSUMER_SET", "set at run time");
    }
    println!("set {:?} {}", env::var("CONSUMER_SET"), env::vars_os().count());
    unsafe {
        env::set_var("CONSUMER_NAME", "replaced");
        env::remove_var("CONSUMER_SET");
    }
    println!("replaced {:?} removed {:?}", env::var("CONSUMER_NAME"), env::var("CONSUMER_SET").is_err());
    let paths: Vec<PathBuf> = env::var_os("CONSUMER_PATH").map(|value| env::split_paths(&value).collect()).unwrap_or_default();
    println!("paths {:?}", paths);
    let joined = env::join_paths(paths.iter().rev()).map(|value| value.into_string());
    println!("joined {:?} {:?}", joined, env::join_paths(["a:b"]).is_err());
    println!("consts {} {} {} {:?} {}", env::consts::OS, env::consts::ARCH, env::consts::FAMILY, env::consts::EXE_SUFFIX, env::consts::DLL_EXTENSION);
    let directory = env::current_dir();
    println!("cwd {:?} {:?}", directory.as_ref().map(|path| path.is_absolute()).ok(), directory.map(|path| path.is_dir()).ok());
    println!("exe {:?}", env::current_exe().map(|path| path.is_absolute()).ok());
    println!("temp {}", env::temp_dir().display());
}
