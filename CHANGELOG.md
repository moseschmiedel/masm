# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0] - 2023-06-15
### Added
 - instructions:
    - Load `ld` and Store `st` from/to RAM
 - numbered register identifier (reg0..reg7)

### Removed
 - macOS continuous delivery build

## [1.1.0] - 2023-06-12
### Added
 - instructions:
    - `add3` - addition with 3 operands
    - `nop` - No operation
    - Absolute/relative jump
 - Jump to Label management

## [1.0.0] - 2023-06-05

### Added
 - instructions:
    - ALU
    - `hlt` - HALT
    - `mov` - MOVE register to register
    - `ldcon` - Load 16bit constant to register
