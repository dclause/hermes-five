#![cfg(not(tarpaulin_include))]

//! Defines mock structure to test the implementations (requires `mock` feature flag).

use crate::io::{IoData, Pin, PinMode, PinModeId};
use std::collections::HashMap;

pub mod input_device;
pub mod output_device;
pub mod plugin_io;
pub mod serial_port;
pub mod transport_layer;

pub fn create_analog_pin(id: u8, value: u16) -> Pin {
    Pin {
        id,
        name: format!("A{}", id),
        mode: PinMode {
            id: PinModeId::ANALOG,
            resolution: 8,
        },
        supported_modes: vec![
            PinMode {
                id: Default::default(),
                resolution: 1,
            },
            PinMode {
                id: PinModeId::ANALOG,
                resolution: 8,
            },
            PinMode {
                id: PinModeId::INPUT,
                resolution: 1,
            },
            PinMode {
                id: PinModeId::OUTPUT,
                resolution: 1,
            },
        ],
        channel: Some(id as u8),
        value,
    }
}

pub fn create_digital_pin(id: u8, value: u16) -> Pin {
    Pin {
        id,
        name: format!("D{}", id),
        mode: PinMode {
            id: PinModeId::OUTPUT,
            resolution: 1,
        },
        supported_modes: vec![
            PinMode {
                id: Default::default(),
                resolution: 1,
            },
            PinMode {
                id: PinModeId::INPUT,
                resolution: 1,
            },
            PinMode {
                id: PinModeId::PULLUP,
                resolution: 1,
            },
            PinMode {
                id: PinModeId::OUTPUT,
                resolution: 1,
            },
        ],
        channel: None,
        value,
    }
}

pub fn create_input_pin(id: u8, value: u16) -> Pin {
    Pin {
        id,
        name: format!("D{}", id),
        mode: PinMode {
            id: PinModeId::INPUT,
            resolution: 1,
        },
        supported_modes: vec![
            PinMode {
                id: Default::default(),
                resolution: 1,
            },
            PinMode {
                id: PinModeId::INPUT,
                resolution: 1,
            },
            PinMode {
                id: PinModeId::OUTPUT,
                resolution: 1,
            },
        ],
        channel: None,
        value,
    }
}

pub fn create_pwm_pin(id: u8, value: u16) -> Pin {
    Pin {
        id,
        name: format!("D{}", id),
        mode: PinMode {
            id: PinModeId::PWM,
            resolution: 8,
        },
        supported_modes: vec![
            PinMode {
                id: Default::default(),
                resolution: 1,
            },
            PinMode {
                id: PinModeId::INPUT,
                resolution: 1,
            },
            PinMode {
                id: PinModeId::OUTPUT,
                resolution: 1,
            },
            PinMode {
                id: PinModeId::PWM,
                resolution: 8,
            },
        ],
        channel: None,
        value,
    }
}

pub fn create_shift_pin(id: u8, value: u16) -> Pin {
    Pin {
        id,
        name: format!("D{}", id),
        mode: PinMode {
            id: PinModeId::SHIFT,
            resolution: 8,
        },
        supported_modes: vec![
            PinMode {
                id: Default::default(),
                resolution: 1,
            },
            PinMode {
                id: PinModeId::SHIFT,
                resolution: 8,
            },
            PinMode {
                id: PinModeId::OUTPUT,
                resolution: 1,
            },
        ],
        channel: None,
        value,
    }
}

pub fn create_servo_pin(id: u8, value: u16) -> Pin {
    Pin {
        id,
        name: format!("D{}", id),
        mode: PinMode {
            id: PinModeId::SERVO,
            resolution: 8,
        },
        supported_modes: vec![
            PinMode {
                id: Default::default(),
                resolution: 1,
            },
            PinMode {
                id: PinModeId::SERVO,
                resolution: 8,
            },
            PinMode {
                id: PinModeId::OUTPUT,
                resolution: 1,
            },
        ],
        channel: None,
        value,
    }
}

pub fn create_unsupported_pin(id: u8) -> Pin {
    Pin {
        id,
        name: format!("A{}", id),
        mode: PinMode {
            id: PinModeId::UNSUPPORTED,
            resolution: 0,
        },
        supported_modes: vec![PinMode {
            id: PinModeId::ANALOG,
            resolution: 8,
        }],
        channel: None,
        value: 0,
    }
}

pub fn create_test_plugin_io_data() -> IoData {
    IoData {
        pins: HashMap::from([
            (0, create_unsupported_pin(0)),
            (1, create_unsupported_pin(0)),
            (2, create_digital_pin(2, 2)),
            (3, create_digital_pin(3, 3)),
            (4, create_digital_pin(4, 4)),
            (5, create_digital_pin(5, 0)),
            (6, create_digital_pin(6, 0)),
            (7, create_digital_pin(7, 0)),
            (8, create_pwm_pin(8, 8)),
            (9, create_shift_pin(9, 9)),
            (10, create_input_pin(10, 10)),
            (11, create_pwm_pin(11, 11)),
            (12, create_servo_pin(12, 12)),
            (13, create_digital_pin(13, 13)),
            (14, create_analog_pin(14, 100)),
            (15, create_analog_pin(15, 200)),
            (22, create_analog_pin(22, 222)),
        ]),
        i2c_data: vec![],
        digital_reported_pins: vec![],
        analog_reported_channels: vec![],
        protocol_version: "fake.1.0".to_string(),
        firmware_name: "Fake protocol".to_string(),
        firmware_version: "fake.2.3".to_string(),
        connected: false,
    }
}
