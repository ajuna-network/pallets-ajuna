# Ajuna Network Pallet GameRegistry

This is a a first draft of the ajuna game registry.

## Purpose

This pallet acts as a game registry for games between L1 and L2, with Ajuna TEE.

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
pallet-gameregistry = {default-features = false, version = '3.0.0', git = 'https://https://github.com/ajuna-network/pallets-ajuna.git', tag = 'monthly-2021-10' }
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    'pallet-gameregistry/std',
]
```

### Runtime `lib.rs`

You should implement it's trait like so:

```rust
parameter_types! {

}

impl pallet_gameregistry::Config for Test {
	type Event = Event;
}
```

and include it in your `construct_runtime!` macro:

```rust
Registry: pallet_gameregistry::{Pallet, Call, Storage, Event<T>},
```

### Genesis Configuration

This matchmaker pallet does not have any genesis configuration.

### Types

Additional types used in the matchmaker pallet

```json
{

}
```

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```
