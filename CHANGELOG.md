# Changelog - dustrun (DPL Runtime)

All notable changes to dustrun are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-02-12 (DPL v0.2)

### Added

- **DPL v0.2 Compliance**: Full support for v0.2 specification
- Full runtime support for K Regime v0.2
- Heap allocator implementation
- Memory management runtime
- Exception handling and panics
- Stack unwinding support
- Debug information support
- Memory pool allocation
- Thread local storage runtime

### Changed

- Enhanced memory allocation performance
- Improved error propagation
- Better panic messages
- Pinned time crate to version 0.3.34 for compatibility

### Fixed

- Memory allocation race conditions
- Stack overflow handling

## [0.1.0] - 2026-02-12

### Added

- Initial runtime implementation
- Basic effect execution (emit)
- Simple panic handling

### Known Issues

- Minimal runtime for v0.1 emit-only programs

---

Copyright © 2026 Dust LLC