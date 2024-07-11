# Hermes-Five

### The Rust Robotics & IoT Platform

[![License](https://img.shields.io/github/license/dclause/hermes-five)](https://github.com/dclause/hermes-five/blob/develop/LICENSE)
[![build](https://github.com/dclause/hermes-five/workflows/Build/badge.svg)](https://github.com/dclause/hermes-five/actions/workflows/build.yml)
[![test](https://github.com/dclause/hermes-five/workflows/Test/badge.svg)](https://github.com/dclause/hermes-five/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/dclause/hermes-five/graph/badge.svg?token=KF8EFDUQ7A)](https://codecov.io/gh/dclause/hermes-five)

**_Hermes-Five_ is an Open Source, [Firmata Protocol](https://github.com/firmata/protocol)-based, IoT and Robotics
programming framework.**

_The ultimate goal of this project is to mimic the functionalities of [Johnny-Five](https://johnny-five.io/) framework
where it finds its inspiration. That being said, the project is done in [my spare time](https://github.com/dclause) and
does not intend
to compete with any other solutions you might want to try,_

## Features

**Hermes-Five** is a Rust library designed to control Arduino or supported boards remotely using Rust code.

**_If you wish to control your Hermes compatible boards using a UI rather than Rust code, please consult
the [HermesStudio](https://github.com/dclause/HermesStudio)
project._**

## Instructions

- Install the
  compatible [Firmata Protocol client](https://github.com/firmata/arduino/blob/main/examples/StandardFirmataPlus/StandardFirmataPlus.ino)
  on your Arduino board.
- Create a new Rust project:

```
cargo new my_project
cd my_project
```

- Add this crate to your dependencies in the _Cargo.toml_
  file

```
[dependencies]
hermes-five = { branch = "develop", git = "https://github.com/dclause/hermes-five" }
```

- Start by exploring the [examples](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples) code and
  the project API

## Examples

All available examples and details can be found in
the [examples](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples) folder.

To start an example, run the following command:

```
cargo run --example examplename
```

To run an example in a file called `examples/folder/my_file.rs` the `examplename` to be used is the concatenation
of `folder_my_file`.

If you want the "full" output you can use:

```
RUST_LOG=DEBUG cargo run --example folder_my_file
```

## Roadmap

For details, see the full [roadmap](https://github.com/dclause/hermes-five/blob/develop/roadmap.md):

- ~~Phase 0: Research~~
- Phase 1: Proof-of-concept
- Phase 2: Animations

## Contribution

All contributions are more than welcomed though [PR](https://github.com/dclause/hermes-five/pulls) and
the [issue queue](https://github.com/dclause/hermes-five/issues).

- Fork the repository
- Create a new branch (git checkout -b feature-branch)
- Commit your changes (git commit -am 'Add new feature')
- Push to the branch (git push origin feature-branch)
- Create a new Pull Request

_The author does not claim to know everything about Rust programming or IoT, and all ideas are welcome as long as they
respect the project's original philosophy._

## License

This project is licensed under the MIT License. See
the [LICENSE](https://github.com/dclause/hermes-five/blob/develop/LICENSE) file for details.

## Contact

For support, please open an issue or reach out to the [author](https://github.com/dclause).