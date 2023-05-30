# MPMP - Assembler

## Installation
### Precompiled Binaries
Download the latest precompiled binaries for your system (Architecture + OS) from `Releases` tab.

### Manual
#### Requirements
- rust
- cargo
#### Compilation
Clone this repository and checkout the desired release:
```sh
git clone https://gitlab.com/moseschmiedel/rasm.git
cd rasm
git checkout <desired-release-branch>
```
Compile and install the assembler locally via cargo:
```sh
cargo install --path .
```

## Usage
Using the assembler is straight-forward. It takes one input-assembly-file and produces a `.hex` output-file.
The name of the output-file can be specified.
```sh
Usage: rasm [OPTIONS] <INPUT_PATH>

Arguments:
  <INPUT_PATH>

Options:
  -o, --output <OUTPUT_PATH>  Output file where binary is stored
  -d, --debug                 Enable debug output to stdout
  -h, --help                  Print help
  -V, --version               Print version
```

## Author
Mose Schmiedel

## Copyright and License

This software is copyright (c) 2023 by Mose Schmiedel
rasm is licensed under the [MIT](LICENSE.TXT) license.
