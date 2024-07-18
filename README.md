# NaNVM

A VM for [FunctionalScript](https://github.com/functionalscript/functionalscript).

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install).
- For Windows, you may need Visual C++. You can get either
  - by installing [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/),
  - or adding [Desktop development with C++](https://learn.microsoft.com/en-us/cpp/build/vscpp-step-0-installation?view=msvc-170) to Visual Studio.

## Installation

To install the latest stable version from [crates.io](https://crates.io/crates/nanvm), run:

```console
cargo install nanvm
```

To install the current version from the `main` branch, run:

```console
cargo install --git https://github.com/functionalscript/nanvm
```

To unininstall the `nanvm`, run:

```console
cargo uninstall nanvm
```

## Command Line Interface

Converting DJS module into one file.

```console
nanvm INPUT_FILE OUTPUT_FILE
```

### Examples

From JSON to JSON:

```console
nanvm notes/sample.json sample.json
```

From ESM module to JSON:

```console
nanvm nanvm-lib/test/test_cache_b.d.mjs sample.json
```

From CommonJS module to ESM module

```console
nanvm nanvm-lib/test/test_import_main.d.cjs sample.d.mjs
```
