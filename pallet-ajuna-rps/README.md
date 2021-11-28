# Ajuna Network Pallet Rock-Paper-Scissor (RPS)

This is a rock-paper-scissor Substrate pallet which lives as its own crate so it can be imported into multiple runtimes.  

*Important: For a real usage, the choose extrinsic needs to take hash and not a salt, the salt should only be provided in the last reveal step.*

## Purpose

This pallet implements the rock-paper-scissor game as a pattern for obfuscating player informations and reavel them later in game.

## Dependencies

### Traits

This pallet does not depend on any externally defined traits.

### Pallets

This pallet does not depend on any other FRAME pallet or externally developed modules.

## Installation

### Runtime `Cargo.toml`

To add this pallet to your runtime, simply include the following to your runtime's `Cargo.toml` file:

```TOML
# external pallets
pallet-rps = {default-features = false, version = '0.1.0', git = 'https://github.com/ajuna-network/pallet-jton-rps.git'}
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    'pallet-rps/std',
]
```

### Runtime `lib.rs`

You should implement it's trait like so:

```rust
/// pallet rps main logic
impl pallet_rps::Config for Runtime {
    type Event = Event;
}
```

and include it in your `construct_runtime!` macro:

```rust
    RockPaperScissor: pallet_rps::{Pallet, Call, Storage, Event<T>},
```

### Genesis Configuration

This rps pallet does not have any genesis configuration.

## Reference Docs

You can view the reference docs for this pallet by running:

```shell
cargo doc --open
```
