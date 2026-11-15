mod bringup;
mod budget_options;
mod census;
mod crates;
mod driver;
mod emission;
mod link_manifest;
mod modules;

fn main() -> std::process::ExitCode {
    driver::run(std::env::args().skip(1))
}
