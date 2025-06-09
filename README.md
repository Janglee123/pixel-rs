# pixel-rs

A naive **rendering engine** using [WGPU](https://wgpu.rs/) and an **Entity Component System** (ECS) library in Rust.

# Overview

This project is my exploration into building a rendering engine from scratch in Rust using WGPU, with a focus on using an Entity Component System (ECS) architecture. 
A lot of the inspiration for the architecture comes from the awesome Bevy Engine.

# Features

- 2D Sprite Rendering
- Sprite Batching
- Lightweight Entity Component System (ECS)
- Asset Loader
- Input System

# How to run
The project requires [Rust](https://rustup.rs/) installed.
1. Clone the repo
```
git clone https://github.com/Janglee123/pixel-rs.git
cd pixel-rs
```
2. Run
```
cargo run
```
(Optionally, use the -r argument for a release build with full performance optimizations):
```
cargo run -r
```
