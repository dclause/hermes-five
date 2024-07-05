# Hermes-Five

### The Rust Robotics &amp; IoT Platform

[![License](https://img.shields.io/github/license/dclause/hermes-five)](https://github.com/dclause/hermes-five/blob/develop/LICENSE)
[![build](https://github.com/dclause/hermes-five/workflows/Build/badge.svg)](https://github.com/dclause/hermes-five/actions/workflows/build.yml)
[![test](https://github.com/dclause/hermes-five/workflows/Test/badge.svg)](https://github.com/dclause/hermes-five/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/dclause/hermes-five/graph/badge.svg?token=Q39BP43C5P)](https://codecov.io/gh/dclause/hermes-five)

**Hermes-Five is an Open Source, [Firmata Protocol](https://github.com/firmata/protocol) based, IoT and Robotics
programming framework.**

_The Hermes-Five project is done on my spare time; it does not intend to compete with any other solutions you might want
to try,_

_Hermes-Five reason to be is for [the author](https://github.com/dclause) to learn more about Rust and to create the
foundation for HermesIO platform._

_The original intend is to mimic the functionality of [Johnny-Five](https://johnny-five.io/) framework where is finds
its inspiration._

## Features

Hermes-Five is a Rust library designed to control Arduino or supported boards remotely using Rust code.

## Instructions

_@todo

## Examples

All available examples and details can be found in
the [examples](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples) folder.

## Contributing

All contributions are more than welcomed though PR and issue queue.

- Fork the repository
- Create a new branch (git checkout -b feature-branch)
- Commit your changes (git commit -am 'Add new feature')
- Push to the branch (git push origin feature-branch)
- Create a new Pull Request

_The author does not claim to know everything about Rust programming or IoT, and all ideas are welcome as long as they
respect the project's original philosophy._

## Roadmap

### Research

- [X] Explore asynchronous task spawning (dynamically spawn tasks, main() should wait for completion).
- [ ] Explore event system (register/emit) and asynchronous callback

### Proof-of-concept

- [ ] Simple board connection using Firmata protocol.
- [ ] Simple led control (on/off) and asynchronous task (blink)
- [ ] Demonstrate the ability to use Hermes-Five for HermesIO requirements

### Development

- [ ] _to be defined_

## License

This project is licensed under the MIT License - see
the [LICENSE](https://github.com/dclause/hermes-five/blob/develop/LICENSE) file for details.

## Contact

For support, please open an issue or reach out to the [author](https://github.com/dclause).