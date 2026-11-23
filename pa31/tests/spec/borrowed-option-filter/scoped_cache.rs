#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

#[derive(Clone, Copy)]
struct FileId(u32);

struct Config {
    flag: u32,
}

impl Config {
    fn for_crate(&self, guest: Option<&str>) -> (usize, &Self) {
        let index = guest.map_or(0, |name| name.len());
        (index, self)
    }
}

struct Package {
    names: [&'static str; 3],
    uses: usize,
}

impl Package {
    fn crate_config<'c>(&self, config: &'c Config, id: FileId) -> (usize, &'c Config) {
        let guest = (id.0 != 0).then(|| self.names[id.0 as usize]);
        config.for_crate(guest)
    }
}

struct Loader<'a, S> {
    package: Package,
    sources: &'a mut S,
    config: &'a Config,
    cache: [Option<FileId>; 2],
}

impl<S> Loader<'_, S> {
    fn module(&mut self, file: FileId, slot: usize) -> u32 {
        let parsed = Some(self.cache);
        let (scope, config) = self.package.crate_config(self.config, file);
        let cached = parsed
            .and_then(|units| units[slot])
            .filter(|&id| self.package.crate_config(self.config, id).0 == scope);
        self.package.uses += 1;
        match cached {
            Some(id) => id.0 + config.flag,
            None => config.flag,
        }
    }
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

fn check() -> i32 {
    let mut source = ();
    let mut loader = Loader {
        package: Package { names: ["", "short", "longer"], uses: 0 },
        sources: &mut source,
        config: &Config { flag: 10 },
        cache: [Some(FileId(1)), Some(FileId(2))],
    };
    if loader.module(FileId(1), 0) != 11 || loader.module(FileId(1), 1) != 10 {
        return 1;
    }
    if loader.module(FileId(2), 1) != 12 || loader.package.uses != 3 {
        return 2;
    }
    0
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    check()
}

#[cfg(not(guest_probe))]
fn main() {
    std::process::exit(check());
}
