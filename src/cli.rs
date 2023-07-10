pub struct Args {
    pub benchmark: bool,
    pub disable_validation: bool,
    pub x11: bool,
}

impl Args {
    pub fn parse() -> Args {
        Args {
            benchmark: std::env::args().any(|arg| arg == "--benchmark"),
            disable_validation: std::env::args().any(|arg| arg == "--disable-validation"),
            x11: std::env::args().any(|arg| arg == "--x11"),
        }
    }
}
