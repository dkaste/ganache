# ganache [![](https://img.shields.io/crates/v/ganache.svg)](https://crates.io/crates/ganache) [![](https://docs.rs/ganache/badge.svg)](https://docs.rs/ganache)
A simple GUI library for Rust

This library draws its main inspiration from the [Godot](https://godotengine.org/) game engine's GUI solution.

### Features
* Proportional anchors for resolution independent placement of UI elements
* Automatic vertical and horizontal layout with minimum sizes
* Rendering backend agnostic; returns a list of draw commands
* Widget styling support
* Generic over lots of things (draw commands, theme resources, input events, etc.)

### Known issues
* Not optimized and probably not very fast yet
* Widget styles would be more ergonomic as structs
* In need of polish and documentation
