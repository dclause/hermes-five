#![cfg_attr(coverage_nightly, coverage(off))]

//! Defines mock structure to test the implementations (requires `mock` feature flag).

use crate::hardware::{Pin, PinMode, PinModeId};
use std::collections::HashMap;
use std::sync::Arc;

pub fn create_analog_pin(id: u8, value: u16) -> Pin {
    let mut pin = Pin::default();
    pin.id = id;
    pin.supported_modes = vec![
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
    ];
    pin.set_pin_mode(PinModeId::ANALOG).unwrap();
    pin.channel.set(Some(id)).unwrap();
    pin.set_value(value);

    pin
}

pub fn create_digital_pin(id: u8, value: u16) -> Pin {
    let mut pin = Pin::default();
    pin.id = id;
    pin.supported_modes = vec![
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
    ];
    pin.set_pin_mode(PinModeId::OUTPUT).unwrap();
    pin.set_value(value);

    pin
}

pub fn create_input_pin(id: u8, value: u16) -> Pin {
    let mut pin = Pin::default();
    pin.id = id;
    pin.supported_modes = vec![
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
    ];
    pin.set_pin_mode(PinModeId::INPUT).unwrap();
    pin.set_value(value);

    pin
}

pub fn create_pwm_pin(id: u8, value: u16) -> Pin {
    let mut pin = Pin::default();
    pin.id = id;
    pin.supported_modes = vec![
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
    ];
    pin.set_pin_mode(PinModeId::PWM).unwrap();
    pin.set_value(value);

    pin
}

pub fn create_shift_pin(id: u8, value: u16) -> Pin {
    let mut pin = Pin::default();
    pin.id = id;
    pin.supported_modes = vec![
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
    ];
    pin.set_pin_mode(PinModeId::SHIFT).unwrap();
    pin.set_value(value);

    pin
}

pub fn create_servo_pin(id: u8, value: u16) -> Pin {
    let mut pin = Pin::default();
    pin.id = id;
    pin.supported_modes = vec![
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
    ];
    pin.set_pin_mode(PinModeId::SERVO).unwrap();
    pin.set_value(value);

    pin
}

pub fn create_unsupported_pin(id: u8) -> Pin {
    let mut pin = Pin::default();
    pin.id = id;
    pin.supported_modes = vec![
        PinMode {
            id: Default::default(),
            resolution: 1,
        },
        PinMode {
            id: PinModeId::ANALOG,
            resolution: 8,
        },
    ];
    pin.set_pin_mode(PinModeId::UNSUPPORTED).unwrap();
    pin.set_value(0);

    pin
}

pub fn create_test_pins() -> HashMap<u8, Arc<Pin>> {
    HashMap::from([
        (0, Arc::new(create_unsupported_pin(0))),
        (1, Arc::new(create_unsupported_pin(0))),
        (2, Arc::new(create_digital_pin(2, 2))),
        (3, Arc::new(create_digital_pin(3, 3))),
        (4, Arc::new(create_digital_pin(4, 4))),
        (5, Arc::new(create_digital_pin(5, 0))),
        (6, Arc::new(create_digital_pin(6, 0))),
        (7, Arc::new(create_digital_pin(7, 0))),
        (8, Arc::new(create_pwm_pin(8, 8))),
        (9, Arc::new(create_shift_pin(9, 9))),
        (10, Arc::new(create_input_pin(10, 10))),
        (11, Arc::new(create_pwm_pin(11, 11))),
        (12, Arc::new(create_servo_pin(12, 12))),
        (13, Arc::new(create_digital_pin(13, 13))),
        (14, Arc::new(create_analog_pin(14, 100))),
        (15, Arc::new(create_analog_pin(15, 200))),
        (22, Arc::new(create_analog_pin(22, 222))),
    ])
}
