use crate::{mock::*};
use frame_support::{assert_ok};
use crate as faucet_pallet;
use frame_system as system;

#[test]
fn send_some_testnet_token_works() {
	new_test_ext().execute_with(|| {
		let param = faucet_pallet::FaucetData {
			id: 11,
			login: b"test".to_vec(),
			created_at: b"1976-09-24T16:00:00Z".to_vec(),
			address: b"306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20".to_vec(),
		};
		let mut params = Vec::<_>::new();
		params.push(param.clone());
		let acct: <Test as system::Config>::AccountId = Default::default();
		assert_ok!(FaucetPallet::send_some_testnet_token(
			Origin::signed(acct),
			params
		));
		assert_eq!(FaucetPallet::latest_faucet_data(), Some(param));
		let block_number = FaucetPallet::send_list(acct);
		assert_eq!(block_number!=Some(0), true);
	});
}
