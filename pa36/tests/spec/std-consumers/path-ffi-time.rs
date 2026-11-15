use std::borrow::Cow;
use std::ffi::{CStr, CString, OsStr, OsString};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn main() {
    let path = Path::new("/usr/local/lib/rustlib/src/lib.rs");
    println!("{} {:?}", path.display(), path);
    println!("parts {:?} {:?} {:?} {:?}", path.parent(), path.file_name(), path.file_stem(), path.extension());
    println!("kind {} {} {} {}", path.is_absolute(), path.is_relative(), path.has_root(), path.starts_with("/usr/local"));
    println!("ends {} {} {}", path.ends_with("src/lib.rs"), path.ends_with("b.rs"), path.starts_with("/usr/loc"));
    println!("strip {:?} {:?}", path.strip_prefix("/usr/local"), path.strip_prefix("lib").is_err());
    println!("with {:?} {:?}", path.with_extension("rlib"), path.with_file_name("main.rs"));
    println!("components {:?}", path.components().collect::<Vec<_>>());
    println!("ancestors {:?}", path.ancestors().map(|a| a.to_str().unwrap_or("?")).collect::<Vec<_>>());
    let odd = Path::new("./a//b/../c/./d.tar.gz");
    println!("odd {:?} {:?} {:?}", odd.components().collect::<Vec<_>>(), odd.file_stem(), odd.extension());
    let normal: Vec<&OsStr> = odd.components().filter_map(|c| if let Component::Normal(name) = c { Some(name) } else { None }).collect();
    println!("normal {:?} {:?}", normal, odd.iter().count());
    println!("join {:?} {:?} {:?}", Path::new("a").join("b/c"), Path::new("a/b").join("/abs"), Path::new("").join("x"));
    println!("edge {:?} {:?} {:?} {:?}", Path::new("/").parent(), Path::new("/").file_name(), Path::new("dir/").file_name(), Path::new(".hidden").extension());
    println!("edge {:?} {:?} {:?}", Path::new("a/..").file_name(), Path::new("noext").extension(), Path::new("x.").extension());
    println!("compare {} {} {:?}", Path::new("a/b") == Path::new("a//b/"), Path::new("a/b") < Path::new("a/c"), Path::new("a").cmp(Path::new("B")));

    let mut buffer = PathBuf::from("/var");
    buffer.push("log");
    buffer.push("app.log");
    println!("buf {:?}", buffer);
    println!("buf {} {:?}", buffer.set_extension("txt"), buffer);
    println!("buf {} {:?}", buffer.pop(), buffer);
    buffer.set_file_name("cache");
    buffer.push("/absolute/replaces");
    let collected: PathBuf = ["x", "y", "z.rs"].iter().collect();
    println!("buf {:?} {:?} {:?}", buffer, collected, collected.clone().into_os_string());
    println!("lossy {:?} {:?}", Path::new("naïve/日本.txt").to_string_lossy(), Path::new("a/b").to_str());

    let os = OsStr::new("os string");
    let mut owned = OsString::from("grow");
    owned.push("ing");
    owned.push(os);
    println!("os {:?} {} {:?} {:?}", os, os.len(), owned, owned.to_str());
    println!("os {:?} {} {}", os.to_ascii_uppercase(), os.eq_ignore_ascii_case("OS STRING"), os.is_empty());
    #[cfg(unix)]
    {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        let raw = OsString::from_vec(vec![b'b', b'a', 0xff, b'd']);
        println!("unix {:?} {:?} {:?} {:?}", raw, raw.to_str(), raw.to_string_lossy(), OsStr::from_bytes(b"ok").as_bytes());
        println!("unix {:?}", matches!(raw.to_string_lossy(), Cow::Owned(_)));
    }

    let c_string = CString::new("for C").unwrap();
    println!("cstring {:?} {} {:?} {:?}", c_string, c_string.as_bytes().len(), c_string.as_bytes_with_nul(), c_string.to_str());
    let interior = CString::new("in\0side").unwrap_err();
    println!("cstring error {} | {} | {:?}", interior, interior.nul_position(), interior.clone().into_vec());
    let literal: &CStr = c"literal";
    let from_bytes = CStr::from_bytes_with_nul(b"bytes\0").unwrap();
    let until = CStr::from_bytes_until_nul(b"stop\0here\0").unwrap();
    println!("cstr {:?} {:?} {:?} {} {}", literal, from_bytes, until, literal.count_bytes(), literal.is_empty());
    println!("cstr {:?} {:?}", CStr::from_bytes_with_nul(b"no nul").is_err(), CStr::from_bytes_with_nul(b"a\0b\0").is_err());
    let pointer = c_string.as_ptr();
    let round_trip = unsafe { CStr::from_ptr(pointer) };
    println!("cstr {:?} {:?} {:?}", round_trip.to_string_lossy(), round_trip.to_owned() == c_string, c_string.into_string());

    let start = Instant::now();
    let mut spin = 0u64;
    for step in 0..200_000u64 {
        spin = spin.wrapping_mul(31).wrapping_add(step);
    }
    let middle = Instant::now();
    let elapsed = start.elapsed();
    println!("instant {} {} {} {}", spin, middle >= start, elapsed >= middle - start, middle.duration_since(start) < Duration::from_secs(60));
    println!("instant {:?} {:?}", start.checked_duration_since(middle).is_none() || start == middle, (start + Duration::from_secs(5)) - start);
    println!("instant {:?} {}", start.checked_add(Duration::from_millis(1)).map(|later| later > start), middle.saturating_duration_since(middle + elapsed) == Duration::ZERO);
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).unwrap();
    println!("system {} {} {:?}", since_epoch.as_secs() > 1_700_000_000, since_epoch.as_secs() < 4_102_444_800, UNIX_EPOCH.duration_since(now).is_err());
    let later = UNIX_EPOCH + Duration::new(86_400 * 365, 5);
    println!("system {:?} {:?} {}", later.duration_since(UNIX_EPOCH), UNIX_EPOCH.checked_sub(Duration::from_secs(1)).map(|earlier| earlier < UNIX_EPOCH), later > UNIX_EPOCH);
    println!("system {:?}", UNIX_EPOCH.duration_since(later).map_err(|error| error.duration()));
}
