//! `workshop-rs-cli` — standalone tooling for the canonical Workshop core.
//!
//! Commands: `parse`, `emit`, `convert`, `locales`, `version`. See the
//! repository README for usage.

fn main() {
    std::process::exit(workshop_rs_cli::run(std::env::args().skip(1).collect()));
}
