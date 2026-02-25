fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = snaphu_rs::run_cli(args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
