#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use core::fmt;
use frame_support::{
	debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult,
	traits::{ Currency, ExistenceRequirement },
};
use parity_scale_codec::{Decode, Encode};

use frame_system::{
	self as system, ensure_signed,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendSignedTransaction,
		SignedPayload, SigningTypes, Signer,
	},
};
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
	AccountId32,
	RuntimeDebug,
	offchain::{
		http,
		storage::StorageValueRef,
	},
};
use sp_std::{
	prelude::*, str,
};
use alt_serde::{Deserialize, Deserializer};
use chrono::{DateTime, Duration};
use rustc_hex::FromHex;
use sp_runtime::SaturatedConversion;

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

pub mod crypto {
	use crate::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::app_crypto::{app_crypto, sr25519};
	use sp_runtime::{
		traits::Verify,
		MultiSignature, MultiSigner,
	};

	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

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

#[derive(Clone, Eq, PartialEq, Default, Encode, Decode, Hash, Debug)]
pub struct FaucetData {
    pub id: u64,
	pub login: Vec<u8>,
    pub created_at: Vec<u8>,
	pub address: Vec<u8>,
}

type Balance<T> = <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

/// Configure the pallet by specifying the parameters and types on which it depends.
//pub trait Trait: frame_system::Trait {
pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
	type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Call: From<Call<Self>>;
	type Currency: Currency<Self::AccountId>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Config> as TemplateModule {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		// Something get(fn something): Option<u32>;
		LatestFaucetData get(fn latest_faucet_data): Option<FaucetData>;
		Sendlist get(fn send_list): map hasher(blake2_128_concat) T::AccountId => Option<<T as frame_system::Config>::BlockNumber>;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
		TestNetTokenTransfered(AccountId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		HttpFetchingError,
		OffchainSignedTxError, 
		NoLocalAcctForSigning, 
		TimeHasNotPassed,
		StrConvertError,
		TransferTokenError,
		HTTPFetchError,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		fn offchain_worker(block: T::BlockNumber) {
			Self::check_faucet_datas(LatestFaucetData::get());
		}

		#[weight = 10000]
		pub fn send_some_testnet_token(origin, faucet_datas: Vec<FaucetData>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let latest_created_at_str;
			let latest_id;
			let tmp;
			match LatestFaucetData::get(){
				Some(data) => {
					tmp = data.created_at.clone();
					latest_created_at_str = str::from_utf8(&tmp).map_err(|_| Error::<T>::StrConvertError)?;
					latest_id = data.id;
				},
				None => {
					latest_created_at_str = "1976-09-24T16:00:00Z";
					latest_id = 0;	
				},
			};

			for faucet_data in faucet_datas{
				if faucet_data.id != latest_id {
					let param_created_at_str = str::from_utf8(&faucet_data.created_at).map_err(|_| Error::<T>::StrConvertError)?;
					let created_at_for_param_data = DateTime::parse_from_rfc3339(param_created_at_str).unwrap();
					let created_at_for_latest_data = DateTime::parse_from_rfc3339(latest_created_at_str).unwrap();
					let duration: Duration = created_at_for_param_data - created_at_for_latest_data;
					if duration.num_seconds() >= 0 {
						let to_address = Self::convert_vec_to_accountid(faucet_data.address.clone());
						let token_amoount : Balance<T> = TOKEN_AMOUNT.saturated_into();
						// debug::info!("##### from:{:#?}, to:{:#?}, amount:{:#?}",who,to_address,token_amoount);
						match T::Currency::transfer(&who, &to_address, token_amoount, ExistenceRequirement::KeepAlive){
							Ok(())=> debug::info!("faucet transfer token is succeed. address is : {:#?}", to_address),
							Err(e)=> debug::error!("##### transfer token is failed. : {:#?}", e), 
						};
						// debug::info!("##### update LatestFaucetData.");
						LatestFaucetData::put(faucet_data);
						<Sendlist<T>>::insert(to_address.clone(), <frame_system::Module<T>>::block_number());
						Self::deposit_event(RawEvent::TestNetTokenTransfered(to_address.clone()));
					}
				}
			}
			Ok(())
		}
	}
}

#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default)]
struct User{
	#[serde(deserialize_with = "de_string_to_bytes")]
	login: Vec<u8>,
	id: u64,
	#[serde(deserialize_with = "de_string_to_bytes")]
	node_id: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	avatar_url: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	gravatar_id: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	url: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	html_url: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	followers_url: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	following_url: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	gists_url: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	starred_url: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	subscriptions_url: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	organizations_url: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	repos_url: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	events_url: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	received_events_url: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	r#type: Vec<u8>,
	site_admin: bool,
}

#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default)]
struct GithubInfo {
	#[serde(deserialize_with = "de_string_to_bytes")]
	url: Vec<u8>,
	id: u64,
	#[serde(deserialize_with = "de_string_to_bytes")]
	node_id: Vec<u8>,
	user: User,
	#[serde(deserialize_with = "de_string_to_bytes")]
	created_at: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	updated_at: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	body: Vec<u8>,
}

pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(de)?;
	Ok(s.as_bytes().to_vec())
}

impl fmt::Debug for GithubInfo {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{{ url: {}, id: {}, node_id: {}, login: {}, created_at: {}, updated_at: {}, body: {} }}",
			str::from_utf8(&self.url).map_err(|_| fmt::Error)?,
			self.id,
			str::from_utf8(&self.node_id).map_err(|_| fmt::Error)?,
			str::from_utf8(&self.user.login).map_err(|_| fmt::Error)?,
			str::from_utf8(&self.created_at).map_err(|_| fmt::Error)?,
			str::from_utf8(&self.updated_at).map_err(|_| fmt::Error)?,
			str::from_utf8(&self.body).map_err(|_| fmt::Error)?,
		)
	}
}

impl<T: Config> Module<T> {
	fn check_faucet_datas(latest_faucet_data:Option<FaucetData>){
		let last_check_time = StorageValueRef::persistent(b"offchain-test::last_check_time");
		let now = sp_io::offchain::timestamp().unix_millis();
		if let Some(Some(last_check_time)) = last_check_time.get::<u64>() {
			// debug::info!("##### last_check_time: {:?}", last_check_time);
			if now - last_check_time < FAUCET_CHECK_INTERVAL{
				return;
			}
		}		
		// debug::info!("##### faucet executed");
		last_check_time.set(&now);
		let g_infos = match Self::fetch_data(){
			Ok(res)=>res,
			Err(e)=>{
				debug::error!("Off-chain Faucet Error fetch_data: {}", e);
				return
			}
		};
		if g_infos.len() > 0 {
			let target_faucet_datas = match Self::check_fetch_data(g_infos,latest_faucet_data){
				Ok(res)=>res,
				Err(e)=>{
					debug::error!("Off-chain Faucet Error check_data: {}", e);
					return
				}
			};
			if target_faucet_datas.len() > 0 {
				// debug::info!("##### transaction will be executed.");
				match Self::offchain_signed_tx(target_faucet_datas) {
					Ok(res)=> res,
					Err(e)=> debug::error!("Off-chain Faucet offchain_signed_tx is failed.: {:#?}", e),
				}
			}
		}
	}

	fn check_fetch_data(g_infos: Vec<GithubInfo>, latest_faucet_data:Option<FaucetData>) -> Result<Vec<FaucetData>, &'static str>{
		let mut results = Vec::new();
		let latest_created_at_str;
		let latest_id;
		let tmp;
		match latest_faucet_data{
			Some(data)=> {
				tmp = data.created_at;
				latest_created_at_str = str::from_utf8(&tmp).map_err(|_| Error::<T>::StrConvertError)?;
				latest_id = data.id;
			},
			None=>{
				latest_created_at_str = "1976-09-24T16:00:00Z";
				latest_id = 0;
			},
		};
		for g_info in g_infos{
			if g_info.id != latest_id {
				let param_created_at_str = str::from_utf8(&g_info.created_at).map_err(|_| Error::<T>::StrConvertError)?;
				let created_at_for_param_data = DateTime::parse_from_rfc3339(param_created_at_str).unwrap();
				let created_at_for_latest_data = DateTime::parse_from_rfc3339(latest_created_at_str).unwrap();
				let duration: Duration = created_at_for_param_data - created_at_for_latest_data;
				if duration.num_seconds() >= 0 {
					let body_str = str::from_utf8(&g_info.body).map_err(|_| Error::<T>::StrConvertError)?;
					let body_value: Vec<&str> = body_str.split("<br>").collect();
					let mut address_str = "";
					let mut homework_str = "";
					for key_value_str in body_value{
						let key_value: Vec<&str> = key_value_str.split(':').collect();
						if key_value.len() != 2{
							debug::warn!("##### This is not key_value string: {:#?}", key_value);
							continue;
						}
						if key_value_str.find("address") >= Some(0){
							address_str = key_value[1];
						}
						if key_value_str.find("homework") >= Some(0){
							homework_str = key_value[1];
						}
					}
					let mut output = vec![0xFF; 35];
					match bs58::decode(address_str).into(&mut output){
						Ok(_res)=> (), // debug::warn!("##### bs58.decode is succeed."),
						Err(_e)=> {
							debug::warn!("##### This is invalid address : {:#?} : {:#?}", address_str, _e);
							continue;
						},
					};
					let cut_address_vec:Vec<_> = output.drain(1..33).collect();
					let homework_vec:Vec<_> = homework_str.from_hex().unwrap();
					if cut_address_vec != homework_vec {
						debug::warn!("##### This is invalid address or invalid homework: {:#?}", homework_str);
						continue;
					}
					let to_address_vec:Vec<u8> = homework_str.from_hex().unwrap();
					let to_address = Self::convert_vec_to_accountid(to_address_vec.clone());
					match <Sendlist<T>>::get(to_address.clone()) {
						Some(result) => {
							let block_number = result + WAIT_BLOCK_NUMBER.into();
							if block_number > <frame_system::Module<T>>::block_number() {
								return Err(Error::<T>::TimeHasNotPassed)?;
							}
							else{
								<Sendlist<T>>::remove(to_address.clone());
							}
						},
						None => (),
					};
					let result = FaucetData{
						id: g_info.id,
						login: g_info.user.login,
						created_at: g_info.created_at,
						address: to_address_vec.clone(),
					};
					results.push(result);
				}
			}
		}
		Ok(results)
	}

	fn fetch_data() -> Result<Vec<GithubInfo>, &'static str> {
		let pending = http::Request::get(HTTP_REMOTE_REQUEST)
			.add_header("User-Agent", HTTP_HEADER_USER_AGENT)
			.add_header("Accept-Charset", "UTF-8")
			.send()
			.map_err(|_| "Error in sending http GET request")?;
		let response = pending.wait()
			.map_err(|_| "Error in waiting http response back")?;

		if response.code != 200 {
			debug::warn!("Unexpected status code: {}", response.code);
			return Err(<Error<T>>::HttpFetchingError)?;
		} 
		let resp_bytes = response.body().collect::<Vec<u8>>();
		let resp_str = str::from_utf8(&resp_bytes).map_err(|_| <Error<T>>::HttpFetchingError)?;
		let resp_str2 = resp_str.replace(r"\r\n","<br>");
		let gh_info: Vec<GithubInfo> =
			serde_json::from_str(&resp_str2).map_err(|_| <Error<T>>::HttpFetchingError)?;
		Ok(gh_info)
	}

	fn offchain_signed_tx(faucet_datas: Vec<FaucetData>) -> Result<(), Error<T>> {
		let signer = Signer::<T, T::AuthorityId>::any_account();
		let result = signer.send_signed_transaction(|_acct|
			Call::send_some_testnet_token(faucet_datas.clone())
		);

		if let Some((_acc, res)) = result {
			if res.is_err() {
				debug::error!("Execute transaction is failed.AccountId is {:#?}",_acc.id);
				return Err(<Error<T>>::OffchainSignedTxError);
			}
			return Ok(());
		}
		// debug::error!("No local account available");
		Err(<Error<T>>::NoLocalAcctForSigning)
	}

	fn convert_vec_to_accountid(account_vec: Vec<u8>)-> T::AccountId{
		let mut array = [0; 32];
		let bytes = &account_vec[..array.len()]; 
		array.copy_from_slice(bytes); 
		let account32: AccountId32 = array.into();
		let mut to32 = AccountId32::as_ref(&account32);
		let to_address : T::AccountId = T::AccountId::decode(&mut to32).unwrap_or_default();
		to_address
	}
}