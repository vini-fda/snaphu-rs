fn main() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .try_init();

    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = snaphu_rs::run_cli(args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
