The currently anticipated Roadmap is the following:

### ~~Phase 0: Research~~

- [X] ~~Explore asynchronous task spawning (dynamically spawn tasks, main() should wait for completion).~~
- [X] ~~Explore event system (register/emit) and asynchronous callback~~

### Phase 1: Proof-of-concept

- [X] ~~Simple board connection using Firmata protocol.~~
- [X] ~~Simple led control (on/off) and asynchronous task (blink)~~
- [ ] Simple servo control (move to position)
- [ ] Demonstrate the ability to use Hermes-Five for HermesStudio requirements

### Phase 2: Led / Servo / Animation

- [ ] Implement most features / controls for Led
- [ ] Implement most features / controls for Servo
- [ ] Demonstrate the ability to create simple Animation

Current technical task list:

- [X] ~~Investigate for an "abortable" task runner~~
- [ ] Make event return TaskResult rather than force ()
- [ ] Investigate wrap task into spawn to avoid .await on task::run()
- [ ] Replace all Mutex/RwLock locking with parking_lot
- [ ] Explore if all async work can be switchable using create feature
- [ ] Add tracing/log for all method