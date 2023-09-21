# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.4.0] - 2023-09-21
### Improved
 - allow underscores in label names

### Added
 - comments with semicolon character ';'


## [1.3.0] - 2023-06-26
### Improved
 - label parsing:
    - forward jump-labels are now possible
    - label definitions start at line start and end with colon ':'
    - label references don't need dot '.' anymore
 - whitespace handling:
    - all types of whitespace before instructions now get recognized
    - whitespace after label definition doesn't effect parsing result

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
