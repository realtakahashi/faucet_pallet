# Substrate Pallet Template

This is a faucet pallet for sending tokens.
It has the following functions.
-You can execute a remittance transaction to the address specified as an unsigned transaction.
-The token remittance source can be defined as a constant of the program.
-You can define the quantity to be remitted at one time as a constant of the program.
-You can define as a program constant how long you want to stop sending money after sending it once.

## Purpose

This pallet acts as a faucet.

## Dependencies

### Traits

This pallet  depend on "frame_support::unsigned::ValidateUnsigned".

### Pallets

This pallet does not depend on any other FRAME pallet or externally developed modules.

## Installation

### Runtime `Cargo.toml`

To add this pallet to your runtime, simply include the following to your runtime's `Cargo.toml` file:

```TOML
[dependencies.substrate-pallet-template]
default_features = false
git = 'https://github.com/realtakahashi/faucet_pallet.git'
```

### Runtime `lib.rs`

You should implement it's trait like so:

```rust
impl faucet_pallet::Trait for Runtime {
	type Event = Event;
	type Currency = Balances;
}
```

and include it in your `construct_runtime!` macro:

```rust
FaucetPallet: faucet_pallet::{Module, Call, Storage, Event<T>,ValidateUnsigned},
```

### Genesis Configuration

You need to set the following constants:

```rust
// How many block numbers do you wait to allow remittances from fauce?
pub const WAIT_BLOCK_NUMBER: u32 = 100; 
// How much token do you transfer at onece? this mean 100 unit token
pub const TOKEN_AMOUNT: u64 = 100000000000000000; 
// Default faucet address
pub const ACCOUNT_ID_HEX: [u8; 32] = hex!["d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"];
```
