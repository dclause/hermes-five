# Roadmap

The project is currently moving towards **release version 0.1**.

## Technical task list (not release related)

- [X] ~~Investigate for an "abortable" task runner~~
- [X] ~~Make events return TaskResult rather than force ()~~
- [X] ~~Investigate wrap task into spawn to avoid .await on task::run()~~
- [ ] Events callback should be able to return either () or Ok(()) (like task)
- [X] ~~Events on/emit should not need async~~
- [X] ~~Replace all Mutex/RwLock locking with parking_lot~~
- [ ] Explore REPL (using: https://rust-script.org/ ?)

## Release version 0.1

The purpose for this release is to create a proof-of-concept of the project:

- determine the API syntax (which may vary until stable 1.0)
- build the underlying requirements (events, tasks, multi-tasking, etc..)
- validate Firmata protocol as a communication choice
- demonstrate both input/ouput device communication with simple examples (led, servo, button, potentiometer)

### ~~Phase 0: Research~~

- [X] ~~Explore asynchronous task spawning (dynamically spawn tasks, main() should wait for completion).~~
- [X] ~~Explore event system (register/emit) and asynchronous callback~~

### ~~Phase 1: Proof-of-concept~~

- [X] ~~Simple board connection using Firmata protocol.~~
- [X] ~~Simple led control (on/off) and asynchronous task (blink)~~
- [X] ~~Simple servo control (move to position)~~
- [X] ~~Simple input control (button)~~
- [X] ~~Demonstrate the ability to create simple Animation~~
- [X] ~~Demonstrate the ability to use Hermes-Five for HermesStudio requirements~~

### ~~Phase 2: Led / Servo / Animation~~

- [X] ~~Implement most features / controls for Led~~
- [X] ~~Implement most features / controls for Servo~~
- [X] ~~Implement most features for creating Animation~~
- [X] ~~Implement most features for creating Button input~~
- [X] ~~Implement most features for creating AnalogInput input~~
- [X] ~~Implement most features for creating DigitalInput input~~
- [X] ~~Implement most features for creating DigitalOutput output~~
- [X] ~~Implement most features for creating PwmOutput output~~

### Pre-release phase:

- [ ] Write appropriate documentation

## Release version 0.2

The purpose for this release is elaborate on the project and show-casing how to swap the underlying elements:

- multi-protocol compatibility: use serial, wifi, etc. to communicate with a board
- multi-board compatibility: control devices attached to arduino, raspberry, etc.
- *to be clarified:* introduce a new layer between the board and devices ? (ex. PCA9685 for servo)
