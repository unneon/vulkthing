pub struct Args {
    pub benchmark: bool,
    pub disable_validation: bool,
    pub window_protocol: WindowProtocol,
}

pub enum WindowProtocol {
    Wayland,
    X11,
}

const DEFAULT_WINDOW_PROTOCOL: WindowProtocol = WindowProtocol::Wayland;

impl Args {
    pub fn parse() -> Args {
        let wayland = std::env::args().any(|arg| arg == "--wayland");
        let x11 = std::env::args().any(|arg| arg == "--x11");
        let window_protocol = if wayland && x11 {
            panic!("can't specify both --wayland and --x11");
        } else if wayland {
            WindowProtocol::Wayland
        } else if x11 {
            WindowProtocol::X11
        } else {
            DEFAULT_WINDOW_PROTOCOL
        };
        Args {
            benchmark: std::env::args().any(|arg| arg == "--benchmark"),
            disable_validation: std::env::args().any(|arg| arg == "--disable-validation"),
            window_protocol,
        }
    }
}
