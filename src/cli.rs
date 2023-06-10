pub struct Args {
    pub disable_validation: bool,
    pub x11: bool,
}

impl Args {
    pub fn parse() -> Args {
        Args {
            disable_validation: std::env::args().any(|arg| arg == "--disable-validation"),
            x11: std::env::args().any(|arg| arg == "--x11"),
        }
    }
}
