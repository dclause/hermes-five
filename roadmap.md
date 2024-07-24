The currently anticipated Roadmap is the following:

### ~~Phase 0: Research~~

- [X] ~~Explore asynchronous task spawning (dynamically spawn tasks, main() should wait for completion).~~
- [X] ~~Explore event system (register/emit) and asynchronous callback~~

### Phase 1: Proof-of-concept

- [X] ~~Simple board connection using Firmata protocol.~~
- [X] ~~Simple led control (on/off) and asynchronous task (blink)~~
- [X] ~~Simple servo control (move to position)~~
- [X] ~~Demonstrate the ability to create simple Animation~~
- [ ] Demonstrate the ability to use Hermes-Five for HermesStudio requirements

### Phase 2: Led / Servo / Animation

- [ ] Implement most features / controls for Led
- [ ] Implement most features / controls for Servo
- [ ] Implement most features for creating Animation

### Current technical task list:

- [X] ~~Investigate for an "abortable" task runner~~
- [X] ~~Make event return TaskResult rather than force ()~~
- [X] ~~Investigate wrap task into spawn to avoid .await on task::run()~~
- [ ] Events callback should be able to return either () or Ok(()) (like task)
- [X] ~~Replace all Mutex/RwLock locking with parking_lot~~
- [ ] Add tracing/log for all method
- [ ] Add https://github.com/colin-kiegel/rust-derive-builder / https://docs.rs/derive-getters/latest/derive_getters/ ?
- [ ] Explore REPL (using: https://rust-script.org/ ?)