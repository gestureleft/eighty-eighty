# eighty-eighty

The [Intel 8080](https://en.wikipedia.org/wiki/Intel_8080) was a 8-bit microprocessor from 1974.

This project aims to emulate it in the Rust programming language.

## Components

### eighty-eighty

This is the Emulator itself. It exposes a basic API for creating, emulating and inspecting the state
of an 8080.

### web-client

This is a small [Yew](https://yew.rs/) web app that can be used to step through an 8080 binary

**Development**:

Requirements:
- Rust
- trunk (`cargo install trunk` to install)
- `wasm32-unknown-unknown` target, the WASM compiler and build target for Rust (`rustup target add wasm32-unknown-unknown` to install)

To serve the web application locally:

`trunk serve --open`

