use crate::{Error, mock::*};
use frame_support::{assert_ok , assert_noop};

#[test]
fn it_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(TemplateModule::get_some_token(Origin::signed(1), BOB));
    });
}

#[test]
fn send_to_other_address(){
    new_test_ext().execute_with(|| {
        TemplateModule::get_some_token(Origin::signed(1), BOB);
        assert_ok!(TemplateModule::get_some_token(Origin::signed(1), CHARLIE));
    });
}

#[test]
fn blocktime_error(){
    new_test_ext().execute_with(|| {
        TemplateModule::get_some_token(Origin::signed(1), BOB);
        assert_noop!(TemplateModule::get_some_token(Origin::signed(1), BOB),
        Error::<Test>::TimeHasNotPassed);
    });
}

// #[test]
// fn it_works_for_default_value() {
// 	new_test_ext().execute_with(|| {
// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
// 		// Read pallet storage and assert an expected result.
// 		assert_eq!(TemplateModule::something(), Some(42));
// 	});
// }

// #[test]
// fn correct_error_for_none_value() {
// 	new_test_ext().execute_with(|| {
// 		// Ensure the expected error is thrown when no value is present.
// 		assert_noop!(
// 			TemplateModule::cause_error(Origin::signed(1)),
// 			Error::<Test>::NoneValue
// 		);
// 	});
// }
