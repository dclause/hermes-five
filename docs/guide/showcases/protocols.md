---
outline: deep
---

# Protocols & transports

All protocols within the application _<small>(see [Protocol Layer](../introduction/concepts#protocol-layer))</small>_
must implement `IoProtocol` trait.
This page lists the protocols currently available.

| Methods of a Protocol | Description                                                               |                                                                                                          |
|-----------------------|---------------------------------------------------------------------------|:--------------------------------------------------------------------------------------------------------:|
| **get_io**            | Returns the IoData associated with the protocol.                          |     [doc](https://docs.rs/hermes_five/0.1.0/hermes_five/io/trait.IoProtocol.html#tymethod.get_data)      |
| **get_protocol_name** | Returns the procol name (RemoteIo, RaspiIo, etc.)                         | [doc](https://docs.rs/hermes_five/0.1.0/hermes_five/io/trait.IoProtocol.html#tymethod.get_protocol_name) |
| **open**              | Opens the communication using the underlying protocol.                    |       [doc](https://docs.rs/hermes_five/0.1.0/hermes_five/io/trait.IoProtocol.html#tymethod.open)        |
| **close**             | Gracefully shuts down the communication.                                  |       [doc](https://docs.rs/hermes_five/0.1.0/hermes_five/io/trait.IoProtocol.html#tymethod.close)       |
| **is_connected**      | Checks if the communication is opened using the underlying protocol.      |   [doc](https://docs.rs/hermes_five/0.1.0/hermes_five/io/trait.IoProtocol.html#tymethod.is_connected)    |
| **set_pin_mode**      | Sets the `mode` of the specified `pin`.                                   |   [doc](https://docs.rs/hermes_five/0.1.0/hermes_five/io/trait.IoProtocol.html#tymethod.set_pin_mode)    |
| **digital_write**     | Writes `level` to the digital `pin`.                                      |   [doc](https://docs.rs/hermes_five/0.1.0/hermes_five/io/trait.IoProtocol.html#tymethod.digital_write)   |
| **analog_write**      | Writes `level` to the analog `pin`.                                       |   [doc](https://docs.rs/hermes_five/0.1.0/hermes_five/io/trait.IoProtocol.html#tymethod.analog_write)    |
| **report_analog**     | Sets the analog reporting `state` of the specified analog `pin`.          |   [doc](https://docs.rs/hermes_five/0.1.0/hermes_five/io/trait.IoProtocol.html#tymethod.report_analog)   |
| **report_digital**    | Sets the digital reporting `state` of the specified digital `pin`.        |  [doc](https://docs.rs/hermes_five/0.1.0/hermes_five/io/trait.IoProtocol.html#tymethod.report_digital)   |
| **sampling_interval** | Set the sampling `interval` (in ms).                                      | [doc](https://docs.rs/hermes_five/0.1.0/hermes_five/io/trait.IoProtocol.html#tymethod.sampling_interval) |
| **servo_config**      | Configures the servo pwm `range`.                                         |   [doc](https://docs.rs/hermes_five/0.1.0/hermes_five/io/trait.IoProtocol.html#tymethod.servo_config)    |
| **i2c_config**        | Sets a `delay` in microseconds between I2C devices write/read operations. |    [doc](https://docs.rs/hermes_five/0.1.0/hermes_five/io/trait.IoProtocol.html#tymethod.i2c_config)     |
| **i2c_read**          | Reads `size` bytes from I2C device at the specified `address`.            |     [doc](https://docs.rs/hermes_five/0.1.0/hermes_five/io/trait.IoProtocol.html#tymethod.i2c_read)      |
| **i2c_write**         | Writes `data` to the I2C device at the specified `address`.               |     [doc](https://docs.rs/hermes_five/0.1.0/hermes_five/io/trait.IoProtocol.html#tymethod.i2c_write)     |

## RemoteIo

`RemoteIo` is the default protocol for a board created with no special configuration.

```rust
let board = Board::run();
let board = Board::default ().open();
```

Its purpose is to control a board remotely connected to the backend that run the software. The remote connection is
established via the transport layer which `RemoteIo` must use.
All transports within the application _<small>(see [Transport Layer](../introduction/concepts#transport-layer))</small>_
must implement `IoTransport` trait.

### Serial

![RemoteIo+Serial](/communication/RemoteIo_Serial.png)

The `Serial` transport layer lets you control a remote board connected via a serial cable to your backend. That is the
simplest solution you could image: an Arduino board for instance, cable-connected to your computer.

```rust
let board = Board::from(Serial::default ()); // RemoteIo + serial with default port.
let board = Board::new(RemoteIo::new("COM3")); // custom port
let board = Board::new(RemoteIo::from(Serial::new("COM3"))); // custom transport
```

### Ethernet (coming soon)

_(coming soon)_

## RaspiIo (coming soon)

![RaspiIo](/communication/RaspiIo.png#center){width=500}

_(coming soon)_    
The purpose of RaspiIo is to control the input output pins of a RaspberryPi collocated to where your software would run.
Meaning in that case: everything from your [Backend Layer](../introduction/concepts#backend-layer) to
your [Hardware Layer](../introduction/concepts#hardware-layer) is squashed into a single RaspberryPi

```rust
let board = Board::new(RaspiIo::default ());
```
