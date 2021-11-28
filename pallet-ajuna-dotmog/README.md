![image](https://user-images.githubusercontent.com/17710198/120896646-c3bc0900-c622-11eb-9a9e-9638c2fc4698.png)

# DOTMog Pallet

This is the DOTMog pallet which lives as its own crate so it can be imported into multiple runtimes.

## Purpose

This pallet acts as the main pallet for the game DOTMog.

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
pallet-dotmog = {default-features = false, version = '0.1.0', git = 'https://github.com/dotmog/pallet-dotmog.git'}
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    'pallet-dotmog/std',
]
```

### Runtime `lib.rs`

You should implement it's trait like so:

```rust
parameter_types! {
	pub const DotMogPalletId: PalletId = PalletId(*b"py/dtmog");
}

/// Configure the pallet dotmog in pallets/dotmog.
impl pallet_dotmog::Config for Runtime {
		type PalletId = DotMogPalletId;
		type Event = Event;
		type Currency = Balances;
		type Randomness = RandomnessCollectiveFlip;
		type PricePayment = ();
}
```

and include it in your `construct_runtime!` macro:
```rust
DotMogModule: pallet_dotmog::{Pallet, Call, Storage, Event<T>, Config<T>},
```

### Genesis Configuration

This dotmog pallet does have a genesis configuration.

```rust
use node_template_runtime::{
	..., DotMogModuleConfig, ...
};
```

```rust
	GenesisConfig {
		...
		pallet_dotmog: DotMogModuleConfig {
			key: root_key,
		},
	}
```

### Additional types

```json
{
  "Address": "MultiAddress",
  "AccountInfo": "AccountInfoWithDualRefCount",
  "LookupSource": "MultiAddress",
  "GameEventType": {
    "_enum": [
      "Default",
      "Hatch"
    ]
  },
  "GameEvent": {
    "id": "H256",
    "begin": "BlockNumber",
    "duration": "u16",
    "event_type": "GameEventType",
    "hashes": "Vec<H256>",
    "value": "u64"
  },
  "RarityType": {
    "_enum": [
      "Minor",
      "Normal",
      "Rare",
      "Epic",
      "Legendary"
    ]
  },
  "MogwaiStruct": {
    "id": "H256",
    "dna": "H256",
    "genesis": "BlockNumber",
    "price": "Balance",
    "gen": "u32",
    "rarity": "RarityType"
  },
  "MogwaiBios": {
    "mogwai_id": "Hash",
    "state": "u32",
    "metaxy": "Vec<[u8;16]>",
    "intrinsic": "Balance",
    "level": "u8",
    "phases": "Vec<BlockNumber>",
    "adaptations": "Vec<Hash>"
  }
}
```

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```
