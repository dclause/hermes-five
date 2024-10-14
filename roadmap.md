# Roadmap

The currently anticipated Roadmap is the following.

## Release version 0.1

### ~~Phase 0: Research~~

- [X] ~~Explore asynchronous task spawning (dynamically spawn tasks, main() should wait for completion).~~
- [X] ~~Explore event system (register/emit) and asynchronous callback~~

### ~~Phase 1: Proof-of-concept~~

- [X] ~~Simple board connection using Firmata protocol.~~
- [X] ~~Simple led control (on/off) and asynchronous task (blink)~~
- [X] ~~Simple servo control (move to position)~~
- [ ] Simple input control (analog / digital value)
- [X] ~~Demonstrate the ability to create simple Animation~~
- [X] ~~Demonstrate the ability to use Hermes-Five for HermesStudio requirements~~

### ~~Phase 2: Led / Servo / Animation~~

- [X] ~~Implement most features / controls for Led~~
- [X] ~~Implement most features / controls for Servo~~
- [X] ~~Implement most features for creating Animation~~
- [ ] Implement most features for creating Button input

### Current technical task list:

- [X] ~~Investigate for an "abortable" task runner~~
- [X] ~~Make event return TaskResult rather than force ()~~
- [X] ~~Investigate wrap task into spawn to avoid .await on task::run()~~
- [ ] Events callback should be able to return either () or Ok(()) (like task)
- [X] ~~Events on/emit should not need async~~
- [X] ~~Replace all Mutex/RwLock locking with parking_lot~~
- [ ] Explore REPL (using: https://rust-script.org/ ?)