# Terramine

A game Engine built with ideas of both Minecraft and Terraria

![image](https://github.com/user-attachments/assets/1c9d19be-fea0-4d07-946f-3bc2faa1f52d)

## Brief

*Terramine* is a voxel game engine featuring multithreaded chunk loading and LOD generation.

## Build

Building Terramine requires nightly build of the Rust toolchain.

```shell
rustup override set nightly
cargo build --release
```

## Run

```shell
target/release/terramine
```
