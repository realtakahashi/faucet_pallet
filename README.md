# Faucet Pallet

This is a faucet pallet for sending tokens.
The usage is as follows.

- You can apply for this Faucet by entering the "user name", "address", and "account ID derived from the address" in a specific Github Issue.
- This pallet uses an off-chain worker to get this Github Issue at regular intervals.
- Check the format of the address on Off-Chain and check if the AccountId set by the user is correct.
- Also check on the Off-chain that the same address has not been requested multiple times within a certain period of time.
- Set each program const value in lib.rs.
  - FAUCET_CHECK_INTERVAL: Definition of the interval at which data is retrieved from the Github Issue site used as a faucet front end.
  - TOKEN_AMOUNT:Amount of test net token to send at once.
  - KEY_TYPE:KeyType definition.
  - WAIT_BLOCK_NUMBER:Specify the block number as the interval until the account that received the test net token can receive it again.
  - HTTP_HEADER_USER_AGENT:Specify any character string.
  - HTTP_REMOTE_REQUEST:URL of Github Issue to use as front end.

## Purpose

This pallet acts as a faucet.

## Dependencies

### Traits

This pallet does not depend on any externally defined traits.

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
impl faucet_pallet::Config for Runtime {
	type AuthorityId = faucet_pallet::crypto::TestAuthId;
	type Call = Call;
	type Event = Event;
	type Currency = Balances;
}
```

and include it in your `construct_runtime!` macro:

```rust
Faucet: faucet_pallet::{Module, Call, Storage, Event<T>},
```

### Genesis Configuration

You need to set the following constants:

```rust
// genesis settings.
// Definition of the interval at which data is retrieved from the Github Issue site used as a faucet front end.
const FAUCET_CHECK_INTERVAL: u64 = 60000;
// Amount of test net token to send at once.
pub const TOKEN_AMOUNT: u64 = 1000000000000000;
// KeyType definition. 
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"shin");
// Specify the block number as the interval until the account that received the test net token can receive.
pub const WAIT_BLOCK_NUMBER: u32 = 1000; 
// HTTP_USER_AGENT string.
const HTTP_HEADER_USER_AGENT: &str = "realtakahashi";
// URL of Github Issue to use as front end.
const HTTP_REMOTE_REQUEST: &str = "https://api.github.com/repos/realtakahashi/faucet_pallet/issues/2/comments";
```
