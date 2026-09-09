fn main() {
    std::process::exit(workshop_rs_cli::run(std::env::args().skip(1).collect()));
}
