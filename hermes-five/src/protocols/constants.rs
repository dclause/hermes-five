//! Firmata protocol constants

/// Received Protocol message.
#[derive(Clone, Debug)]
pub enum Message {
    ProtocolVersion,
    Analog,
    Digital,
    EmptyResponse,
    AnalogMappingResponse,
    CapabilityResponse,
    PinStateResponse,
    ReportFirmware,
    I2CReply,
}

// ########################################
// Protocol related functions

/// For non-compatible changes
pub const PROTOCOL_MAJOR_VERSION: u8 = 2;
/// For backwards-compatible changes
pub const PROTOCOL_MINOR_VERSION: u8 = 5;
/// For bugfix releases
pub const PROTOCOL_BUGFIX_VERSION: u8 = 7;

// ########################################
// Message command bytes (128-255/0x80-0xFF)
/// Send data for a digital port (collection of 8 pins)
pub const DIGITAL_MESSAGE: u8 = 0x90;
/// Send data for an analog pin (or PWM)
pub const ANALOG_MESSAGE: u8 = 0xE0;
/// Enable analog input by pin #
pub const REPORT_ANALOG: u8 = 0xC0;
/// Enable digital input by port pair
pub const REPORT_DIGITAL: u8 = 0xD0;
/// Digital message input range upper byte bound
pub const DIGITAL_MESSAGE_BOUND: u8 = 0x9F;
/// Analog message input range upper byte bound
pub const ANALOG_MESSAGE_BOUND: u8 = 0xEF;
//
/// Set a pint to INPUT/OUTPUT/PWM/etc
pub const SET_PIN_MODE: u8 = 0xF4;
/// Set value of an individual digital pin
pub const SET_DIGITAL_PIN_VALUE: u8 = 0xF5;
//
/// Report protocol version
pub const REPORT_VERSION: u8 = 0xF9;
/// Reset from MIDI
pub const SYSTEM_RESET: u8 = 0xFF;
//
/// Start a MIDI Sysex message
pub const START_SYSEX: u8 = 0xF0;
/// End a MIDI Sysex message
pub const END_SYSEX: u8 = 0xF7;

// Extended command set using sysex (0-127/0x00-0x7F)

/// Communicate with serial devices
pub const SERIAL_DATA: u8 = 0x60;
/// Reply with encoders current positions
pub const ENCODER_DATA: u8 = 0x61;
/// Set max angle, minPulse, maxPulse, freq
pub const SERVO_CONFIG: u8 = 0x70;
/// String message with 14-bits per char
pub const STRING_DATA: u8 = 0x71;
/// Control a stepper motor
pub const STEPPER_DATA: u8 = 0x72;
/// Send an OneWire read/write/reset/select/skip/search request
pub const ONEWIRE_DATA: u8 = 0x73;
/// Bitstream to/from a shift register
pub const SHIFT_DATA: u8 = 0x75;
/// Send an I2C read/write request
pub const I2C_REQUEST: u8 = 0x76;
/// Reply to an I2C read request
pub const I2C_REPLY: u8 = 0x77;
/// Config I2C settings such as delay times and power pins
pub const I2C_CONFIG: u8 = 0x78;
/// Report name and version of the firmware
pub const REPORT_FIRMWARE: u8 = 0x79;
/// Analog write (PWM, Servo, etc) to any pin
pub const EXTENDED_ANALOG: u8 = 0x6F;
/// Ask for a pin's current mode and value
pub const PIN_STATE_QUERY: u8 = 0x6D;
/// Reply with pin's current mode and value
pub const PIN_STATE_RESPONSE: u8 = 0x6E;
/// Ask for supported modes and resolution of all pins
pub const CAPABILITY_QUERY: u8 = 0x6B;
/// Reply with supported modes and resolution
pub const CAPABILITY_RESPONSE: u8 = 0x6C;
/// Ask for mapping of analog to pin numbers
pub const ANALOG_MAPPING_QUERY: u8 = 0x69;
/// Reply with mapping info
pub const ANALOG_MAPPING_RESPONSE: u8 = 0x6A;
/// Set the poll rate of the main loop
pub const SAMPLING_INTERVAL: u8 = 0x7A;
/// Send a createtask/deletetask/addtotask/schedule/querytasks/querytask request to the scheduler
pub const SCHEDULER_DATA: u8 = 0x7B;
/// MIDI Reserved for non-realtime messages
pub const SYSEX_NON_REALTIME: u8 = 0x7E;
/// MIDI Reserved for realtime messages
pub const SYSEX_REALTIME: u8 = 0x7F;

// I2C additions.
pub const I2C_WRITE: u8 = 0x00;
pub const I2C_READ: u8 = 0x01;
pub const I2C_READ_CONTINUOUSLY: u8 = 0x10;
pub const I2C_STOP_READING: u8 = 0x18;
pub const I2C_READ_WRITE_MODE_MASK: u8 = 0x18;
pub const I2C_10BIT_ADDRESS_MODE_MASK: u8 = 0x20;
pub const I2C_END_TX_MASK: u8 = 0x40;

// Other values
/// Default analog resolution value
pub const DEFAULT_ANALOG_RESOLUTION: u8 = 10;
/// Default PWM resolution value
pub const DEFAULT_PWM_RESOLUTION: u8 = 10;
/// Default PWM resolution value
pub const DEFAULT_SERVO_RESOLUTION: u8 = 14;
