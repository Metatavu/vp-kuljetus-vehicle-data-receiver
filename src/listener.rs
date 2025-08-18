/// Allows for different configurations for different device types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Listener {
    TeltonikaFMC650,
    TeltonikaFMC234,
}

impl Listener {
    /// Gives each device type their own port number
    pub fn port(&self) -> u16 {
        match self {
            Listener::TeltonikaFMC650 => 6500,
            Listener::TeltonikaFMC234 => 2340,
        }
    }
}
