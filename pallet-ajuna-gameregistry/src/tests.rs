use super::*;

use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(Registry::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(Registry::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(Registry::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
	});
}

#[test]
fn regsitry_test() {
	new_test_ext().execute_with(|| {
		let mut current_block: u64 = 100;

		let player1: u64 = 1u64;
		let player2: u64 = 2u64;

		let scheduler: u64 = 5u64;
		let tee: u64 = 7u64;

		let game_engine1: GameEngine = GameEngine { id: 1, version: 1 };

		let mut players = Vec::new();
		players.push(player1.clone());
		players.push(player2.clone());

		// start from block 100
		run_to_block(current_block);

		let queue_test1 = Registry::game_queues(&game_engine1);
		assert_eq!(queue_test1.length(), 0);

		// queue up matchmaker first player
		assert_ok!(Registry::queue(Origin::signed(player1)));

		run_next_block();
		current_block = current_block + 1;
		assert_eq!(System::block_number(), current_block);

		// queue up matchmaker second player
		assert_ok!(Registry::queue(Origin::signed(player2)));

		run_next_block();
		current_block = current_block + 1;
		assert_eq!(System::block_number(), current_block);

		// check if we have something queued
		let queue_test2 = Registry::game_queues(&game_engine1);
		assert_eq!(queue_test2.length(), 1);

		let game_hash = queue_test2.peek().unwrap();
		let mut games = Vec::new();
		games.push(game_hash.clone());

		// check correct game state
		let game_entry1 = Registry::game_registry(&game_hash);
		assert_eq!(game_entry1.game_state, GameState::Waiting);

		// acknowledge game
		assert_ok!(Registry::ack_game(Origin::signed(tee), game_engine1.clone(), games));

		// check if we have something queued
		let queue_test3 = Registry::game_queues(&game_engine1);
		assert_eq!(queue_test3.length(), 0);

		run_next_block();

		// check correct game state
		let game_entry2 = Registry::game_registry(&game_hash);
		assert_eq!(game_entry2.game_state, GameState::Accepted);

		// ready game
		assert_ok!(Registry::ready_game(Origin::signed(tee), game_hash.clone()));

		run_next_block();

		// check correct game state
		let game_entry3 = Registry::game_registry(&game_hash);
		assert_eq!(game_entry3.game_state, GameState::Running);

		run_next_block();

		// finish game
		assert_ok!(Registry::finish_game(Origin::signed(tee), game_hash.clone(), player1.clone()));

		// check correct game state
		let game_entry4 = Registry::game_registry(&game_hash);
		assert_eq!(game_entry4.game_state, GameState::Finished(player1));

		// drop game
		assert_ok!(Registry::drop_game(
			Origin::signed(tee),
			game_hash.clone(),
			game_engine1.clone()
		));

		let game_entry5 = Registry::game_registry(&game_hash);
		assert_eq!(game_entry5.game_state, GameState::None);
	});
}
