fn main() {
    // `args_os` + lossy conversion: `env::args` panics on non-UTF-8 argv.
    std::process::exit(workshop_rs_cli::run(
        std::env::args_os()
            .skip(1)
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect(),
    ));
}
