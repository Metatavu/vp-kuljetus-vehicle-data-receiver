mod driver_one_card_event_handler;
mod driver_one_drive_state_event_handler;
mod odometer_reading_event_handler;
mod speed_event_handler;
mod teltonika_event_handlers;
mod temperature_sensors_reading_event_handler;

use std::slice::Iter;

pub use driver_one_card_event_handler::DriverOneCardEventHandler;
pub use driver_one_drive_state_event_handler::DriverOneDriveStateEventHandler;
pub use odometer_reading_event_handler::OdometerReadingEventHandler;
pub use speed_event_handler::SpeedEventHandler;
pub use teltonika_event_handlers::TeltonikaEventHandlers;
pub use temperature_sensors_reading_event_handler::TemperatureSensorsReadingEventHandler;

use crate::Listener;

/// Enumeration of possible Teltonika temperature sensors
#[derive(Debug)]
pub enum TeltonikaTemperatureSensors {
    Sensor1,
    Sensor2,
    Sensor3,
    Sensor4,
    Sensor5,
    Sensor6,
}

/// Implementation of [TeltonikaTemperatureSensors]
///
/// Provides utilities to iterate over all possible temperature sensors and get the corresponding [nom_teltonika::AVLEventIO] ids.
impl TeltonikaTemperatureSensors {
    pub fn iterator() -> Iter<'static, TeltonikaTemperatureSensors> {
        static SENSORS: [TeltonikaTemperatureSensors; 6] = [
            TeltonikaTemperatureSensors::Sensor1,
            TeltonikaTemperatureSensors::Sensor2,
            TeltonikaTemperatureSensors::Sensor3,
            TeltonikaTemperatureSensors::Sensor4,
            TeltonikaTemperatureSensors::Sensor5,
            TeltonikaTemperatureSensors::Sensor6,
        ];
        SENSORS.iter()
    }

    /// Get sensor event ids used by Teltonika FMC234 devices
    fn fmc234_hardware_sensor_io_event_id(&self) -> u16 {
        match self {
            TeltonikaTemperatureSensors::Sensor1 => 76,
            TeltonikaTemperatureSensors::Sensor2 => 77,
            TeltonikaTemperatureSensors::Sensor3 => 79,
            TeltonikaTemperatureSensors::Sensor4 => 71,
            TeltonikaTemperatureSensors::Sensor5 => 0,
            TeltonikaTemperatureSensors::Sensor6 => 0,
        }
    }

    /// Get sensor event ids used by Teltonika FMC650 devices
    fn fmc650_hardware_sensor_io_event_id(&self) -> u16 {
        match self {
            TeltonikaTemperatureSensors::Sensor1 => 62,
            TeltonikaTemperatureSensors::Sensor2 => 63,
            TeltonikaTemperatureSensors::Sensor3 => 64,
            TeltonikaTemperatureSensors::Sensor4 => 65,
            TeltonikaTemperatureSensors::Sensor5 => 5,
            TeltonikaTemperatureSensors::Sensor6 => 6,
        }
    }

    /// Get the [nom_teltonika::AVLEventIO] id for the hardware sensor event
    pub fn hardware_sensor_io_event_id(&self, listener: &Listener) -> u16 {
        match listener {
            Listener::TeltonikaFMC234 => self.fmc234_hardware_sensor_io_event_id(),
            Listener::TeltonikaFMC650 => self.fmc650_hardware_sensor_io_event_id(),
        }
    }

    /// Get the [nom_teltonika::AVLEventIO] id for the temperature reading event
    pub fn temperature_reading_io_event_id(&self) -> u16 {
        match self {
            TeltonikaTemperatureSensors::Sensor1 => 72,
            TeltonikaTemperatureSensors::Sensor2 => 73,
            TeltonikaTemperatureSensors::Sensor3 => 74,
            TeltonikaTemperatureSensors::Sensor4 => 75,
            TeltonikaTemperatureSensors::Sensor5 => 6,
            TeltonikaTemperatureSensors::Sensor6 => 8,
        }
    }
}
