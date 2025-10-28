use cyrup::app::run;

fn main() {
    if let Err(e) = run() {
        log::error!("Application error: {e}");
        std::process::exit(1);
    }
}
