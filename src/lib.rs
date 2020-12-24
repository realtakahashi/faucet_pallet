#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, 
	traits::Currency,
	traits::ExistenceRequirement,
	dispatch::DispatchResult,
	debug,
};

use frame_system::{
	self as system, 
	offchain::{SignedPayload,SigningTypes},
	ensure_none,
};
use parity_scale_codec::{Decode, Encode};
use hex_literal::hex;
use sp_runtime::{AccountId32,
	RuntimeDebug,
	transaction_validity::{
		InvalidTransaction, TransactionSource, TransactionValidity,
		ValidTransaction,
	},
};
use sp_runtime::SaturatedConversion;

pub const NUM_VEC_LEN: usize = 10;
pub const UNSIGNED_TXS_PRIORITY: u64 = 100;
// own settings of your network.
// How many block numbers do you wait to allow remittances from fauce?
pub const WAIT_BLOCK_NUMBER: u32 = 100; 
// How much token do you transfer at onece? this mean 100 unit token
pub const TOKEN_AMOUNT: u64 = 100000000000000000; 
// Default faucet address
pub const ACCOUNT_ID_HEX: [u8; 32] = hex!["d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"];

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Payload<Public> {
	number: u64,
	public: Public
}

impl <T: SigningTypes> SignedPayload<T> for Payload<T::Public> {
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type Balance<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Currency: Currency<Self::AccountId>;

}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as TemplateModule {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Something get(fn something): Option<u32>;
		FaucetAddress get(fn faucet_address): Option<T::AccountId>;
		Sendlist: map hasher(blake2_128_concat) T::AccountId => Option<<T as frame_system::Trait>::BlockNumber>;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, AccountId),
		/// ->->-> 
		SetFaucetAddress(AccountId),
		SentSomeToken(AccountId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// ->->-> 
		TimeHasNotPassed,
		NotSetFaucetAddress,
		TransferError,
		TooManyTokenAmount,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		// #[weight = 10000]
		// pub fn set_faucet_address(origin) -> DispatchResult {
		// 	let caller = ensure_signed(origin)?;
		// 	<FaucetAddress<T>>::put(caller.clone());
		// 	Self::deposit_event(RawEvent::SetFaucetAddress(caller.clone()));
		// 	Ok(())
		// }

		#[weight = 10000]
		pub fn get_some_token(origin, to_address: T::AccountId) -> DispatchResult {
			let mut block_number;
			match <Sendlist<T>>::get(to_address.clone()) {
                Some(result) => {
					block_number = result + WAIT_BLOCK_NUMBER.into();
					if block_number > <frame_system::Module<T>>::block_number() {
						return Err(Error::<T>::TimeHasNotPassed)?;
					}
					else{
						block_number = <frame_system::Module<T>>::block_number();
					}
				},
                None => block_number = <frame_system::Module<T>>::block_number(),
			};

			let account32: AccountId32 = ACCOUNT_ID_HEX.into();
			let mut from32 = AccountId32::as_ref(&account32);
			let from_address : T::AccountId = T::AccountId::decode(&mut from32).unwrap_or_default();

			let token_amoount : Balance<T> = TOKEN_AMOUNT.saturated_into();
			debug::info!("$$$$$$$$$$$$$$$$$$$ token_amoount: {:?}", token_amoount);

			if T::Currency::transfer(&from_address, &to_address, token_amoount, ExistenceRequirement::KeepAlive) != Ok(()) {
				return Err(Error::<T>::TransferError)?;
			}			
			Self::deposit_event(RawEvent::SentSomeToken(to_address.clone()));
			<Sendlist<T>>::insert(to_address.clone(), block_number);
			Ok(())
		}
	}
}

impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
	type Call = Call<T>;

	fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
		let valid_tx = |provide| ValidTransaction::with_tag_prefix("faucet-pallet")
			.priority(UNSIGNED_TXS_PRIORITY)
			.and_provides([&provide])
			.longevity(3)
			.propagate(true)
			.build();

		match call {
			Call::get_some_token(_to_address) => valid_tx(b"get_some_token".to_vec()),
			// Call::submit_number_unsigned_with_signed_payload(ref payload, ref signature) => {
			// 	if !SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone()) {
			// 		return InvalidTransaction::BadProof.into();
			// 	}
			// 	valid_tx(b"submit_number_unsigned_with_signed_payload".to_vec())
			// },
			_ => InvalidTransaction::Call.into(),
		}
	}
}
