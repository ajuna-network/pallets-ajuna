use super::*;
use crate::{mock::*, Error};

use frame_support::{assert_noop, assert_ok};

#[test]
fn test_game_creation() {
	new_test_ext().execute_with(|| {
		// Test player can not play against himself
		assert_noop!(ConnectFour::new_game(Origin::signed(1), 1), Error::<Test>::NoFakePlay);

		// Test game creation between to different players
		assert_ok!(ConnectFour::new_game(Origin::signed(1), 2));
		run_to_block(1);

		let board_id_1 = ConnectFour::player_board(1);
		let board_id_2 = ConnectFour::player_board(2);

		assert_eq!(board_id_1, board_id_2);

		assert_noop!(ConnectFour::new_game(Origin::signed(1), 3), Error::<Test>::PlayerBoardExists);

		assert_noop!(ConnectFour::new_game(Origin::signed(3), 2), Error::<Test>::PlayerBoardExists);

		let board = ConnectFour::boards(board_id_1);

		assert_eq!(board.last_turn, 0);
	});
}

#[test]
fn test_game_play() {
	new_test_ext().execute_with(|| {
		let mut current_block: u64 = 100;

		// start from block 100
		run_to_block(current_block);

		// Test game creation between to different players
		assert_ok!(ConnectFour::new_game(Origin::signed(PLAYER_1 as u64), PLAYER_2 as u64));
		let board_id = ConnectFour::player_board(PLAYER_1 as u64);
		let board = ConnectFour::boards(board_id);
		assert_eq!(board.last_turn, current_block);

		run_next_block();
		current_block = current_block + 1;

		assert_eq!(System::block_number(), current_block);

		if board.next_player == PLAYER_1 {
			assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_1 as u64), 0));
			let board = ConnectFour::boards(board_id);
			assert!(board.board_state == BoardState::Running);
			assert!(board.next_player == PLAYER_2);
			assert_eq!(board.last_turn, current_block);

			run_next_block();
			current_block = current_block + 1;
		}

		assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_2 as u64), 1));
		let board = ConnectFour::boards(board_id);
		assert_eq!(board.last_turn, current_block);
		assert!(board.board_state == BoardState::Running);
		assert!(board.next_player == PLAYER_1);

		run_next_block();
		current_block = current_block + 1;

		assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_1 as u64), 2));
		let board = ConnectFour::boards(board_id);
		assert!(board.board_state == BoardState::Running);

		run_next_block();
		current_block = current_block + 1;

		assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_2 as u64), 1));
		let board = ConnectFour::boards(board_id);
		assert!(board.board_state == BoardState::Running);

		run_next_block();
		current_block = current_block + 1;

		assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_1 as u64), 3));
		let board = ConnectFour::boards(board_id);
		assert!(board.board_state == BoardState::Running);

		run_next_block();
		current_block = current_block + 1;

		assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_2 as u64), 1));
		let board = ConnectFour::boards(board_id);
		assert!(board.board_state == BoardState::Running);

		run_next_block();
		current_block = current_block + 1;

		assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_1 as u64), 4));
		let board = ConnectFour::boards(board_id);
		assert!(board.board_state == BoardState::Running);

		run_next_block();
		current_block = current_block + 1;

		assert_ok!(ConnectFour::play_turn(Origin::signed(PLAYER_2 as u64), 1));
		let board = ConnectFour::boards(board_id);
		assert!(board.board_state == BoardState::Finished(board.blue));
		assert_eq!(board.last_turn, current_block);
	});
}
