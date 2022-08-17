# koi

`koi` is a game engine that's fun, quick to build, and easy to understand.

It serves as a light-weight core that can be modified and adapted to a project, but it doesn't aim to handle every scenario out of the box.

**WARNING:** `koi` is very work-in-progress. There's a lot left to do!

## Quick to build

### M1 Mac build of `examples/random.rs`

Mode | Clean | Incremental
:-- | --- | ---
**Debug** | 5.22s | 0.41s
**Release** | 7.09s | 0.76s

`koi` accomplishes these build times with narrowly scoped crates that build significantly faster than the typical Rust-ecosystem equivalents.

## Easy to understand

`koi`'s simulation is built upon the ECS (Entity-Component System) paradigm.

`koi` aims to balance code simplicity with performance. If a small performance gain introduces significant complexity (for the user or within the library) it is excluded.
