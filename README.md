# Molecule

[![License]](#license)
[![GitHub Actions]](https://github.com/nervosnetwork/molecule/actions)

Another serialization system: minimalist and canonicalization.

[License]: https://img.shields.io/badge/License-MIT-blue.svg
[GitHub Actions]: https://github.com/nervosnetwork/molecule/workflows/CI/badge.svg

## Documents

- [Encoding Spec](docs/encoding_spec.md)
- [Schema Language](docs/schema_language.md)
- [Real-World Examples](docs/real_world_examples.md)
- [API](docs/molecule_api.md)

## Features
* `default` — Default features: `std`, utilizes `faster-hex` for hexadecimal operations and enables [bytes] standard features.
* `std` (enabled by default)  — Default features: `std`, utilizes `faster-hex` for hexadecimal operations and enables [bytes] standard features.
* `bytes_vec` - Introduced in version 0.8, the 0.8 molecule defaults to [bytes], which has implications for use in the CKB runtime. The `bytes_vec` feature provides users with a compatibility option to maintain consistency with previous versions.

## Use in CKB scripts
When used in CKB scripts, no-std needs to be specified.

```toml
molecule = { version = "0.7", default-features = false }
```

Particularly, for versions later than 0.8, you need to additionally specify the bytes_vec feature.

```toml
molecule = { version = "0.8.0", default-features = false, features = ["bytes_vec"] }
```


## Tools

### Schema Compiler and Code Generator

#### Install

The official schema compiler and code generator are written in [Rust], so
you can install it via [Cargo]:

```sh
cargo install moleculec --locked
```

**Note: the official code generator is only support two languages: [Rust] and [C].**

#### Usage

- You can use the follow command to generate the code:

  ```sh
  moleculec --language <language> --schema-file <schema-file>
  ```

- More details can be found by the follow command:

  ```sh
  moleculec --help
  ```

### Other Languages

Molecule's reference implementation is in Rust and C.

Implementations in other languages are maintained by respective authors.

- [Go](https://github.com/driftluo/moleculec-go)
- [Modern JavaScript](https://github.com/xxuejie/moleculec-es)

### Plugins for Editors

- [Emacs](https://github.com/yangby-cryptape/emacs-molecule)
- [Vim](https://github.com/yangby-cryptape/vim-molecule)
- [Sublime Text](https://github.com/yangby-cryptape/sublimetext-molecule)

## Benchmark

- [Benchmark in Rust with serde](https://github.com/nervosnetwork/serde_bench)

## Supported Rust Versions

The minimum supported version is 1.85.1.
The current Molecule version is not guaranteed to build on Rust versions earlier than the
minimum supported version.

## License

Licensed under [MIT License].

[MIT License]: LICENSE

[Rust]: https://www.rust-lang.org/
[Cargo]: https://doc.rust-lang.org/cargo/
[C]: https://en.wikipedia.org/wiki/C_%28programming_language%29
[bytes]: https://github.com/tokio-rs/bytes
