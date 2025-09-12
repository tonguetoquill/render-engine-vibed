# lib.rs - Typst Wrapper Design Document

Stateless wrapper around Typst's rendering system with embedded assets for USAF memo generation.

## Architecture Overview

- **Stateless Wrapper**: Zero-sized struct with static asset initialization
- **Embedded Assets**: Fonts, templates, and images bundled at compile time
- **Fresh Worlds**: Each render call creates a new Typst world from static resources

## Core Structures

### `TypstWrapper`
Stateless unit struct - all functionality provided through static methods.

### `RenderConfig`  
Optional configuration for output format (SVG/PDF).

## Core Functions

### `new() -> TypstWrapper`
Returns a zero-cost unit struct. All assets pre-initialized at compile time.

### `render(markup: &str, config: Option<RenderConfig>) -> Result<Vec<u8>, TypstWrapperError>`
Primary rendering function - compiles Typst markup to SVG or PDF using embedded assets.


## Error Handling

Standard `TypstWrapperError` enum covering compilation errors, font issues, and output format problems.

## Asset Integration

All assets embedded at compile time using `include_bytes!` and `include_str!`:

- **Fonts**: Arial, Times New Roman, Copperplate CC from `/assets`
- **Templates**: lib.typ and utils.typ from tonguetoquill-usaf-memo:0.0.3
- **Images**: DoD seal and other memo assets

**Benefits**: Self-contained binary, no external file dependencies, fast memory access.

## Implementation Dependencies

### Required Crates
- `typst` (0.13) - Core Typst compiler
- `typst-pdf` (0.13) - PDF output format
- `typst-svg` (0.13) - SVG output format  
- `chrono` - DateTime handling for Typst world
- `thiserror` - Error handling derive macros

### Internal Dependencies
- Font parsing and book management
- File ID and path resolution
- Source code management and compilation
- Output format conversion utilities

## Thread Safety & Performance

### Concurrency Considerations
- Wrapper instances are stateless and thread-safe by design
- Static resources are initialized at program load time (no runtime initialization)
- All static resources are immutable and inherently thread-safe
- Multiple render calls can run concurrently using shared static resources
- No synchronization primitives needed anywhere

### Performance Optimizations
- Zero initialization cost - all setup happens at compile/load time
- Static font book and resolvers eliminate any setup overhead
- All assets loaded directly from embedded memory (no I/O overhead)
- Fresh Typst world creation optimized using pre-initialized static resources
- No lazy evaluation overhead - everything is immediately available
- Typst's internal compilation caches managed automatically
- No bells and whistles. Just implement the core functionality.