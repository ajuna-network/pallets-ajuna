use super::*;
use crate::{Error, mock::*};

use frame_support::{assert_ok, assert_noop};

#[test]
fn test_game_creation() {
	new_test_ext().execute_with(|| {

		let player_1:u64 = 1;
		let player_2:u64 = 2;
		let player_3:u64 = 3;

		// Test player can not play against himself
		assert_noop!(
			RockPaperScissor::new_game(Origin::signed(player_1), player_1),
			Error::<Test>::NoFakePlay
		);

		// Test game creation between to different players
		assert_ok!(RockPaperScissor::new_game(Origin::signed(player_1), player_2));
		run_to_block(1);

		let game_id_1 = RockPaperScissor::player_game(player_1);
		let game_id_2 = RockPaperScissor::player_game(player_2);

		assert_eq!(game_id_1, game_id_2);

		assert_noop!(
			RockPaperScissor::new_game(Origin::signed(player_1), player_3),
			Error::<Test>::PlayerHasGame
		);

		assert_noop!(
			RockPaperScissor::new_game(Origin::signed(player_3), player_2),
			Error::<Test>::PlayerHasGame
		);

		let _game = RockPaperScissor::games(game_id_1);

	});
}

#[test]
fn try_rps_player_1_win() {
	new_test_ext().execute_with(|| {

		let player_1:u64 = 1;
		let salt_1: [u8; 32] = [1u8;32];

		let player_2:u64 = 2;
		let salt_2: [u8; 32] = [2u8;32];

		// Create game
		assert_ok!(RockPaperScissor::new_game(Origin::signed(player_1), player_2));
		let game_id = RockPaperScissor::player_game(player_1);
		let game = RockPaperScissor::games(game_id);
		matches!(game.states[0], MatchState::Choose);
		matches!(game.states[1], MatchState::Choose);

		run_next_block();

		// Choose phase
		assert_ok!(RockPaperScissor::choose(Origin::signed(player_2), WeaponType::Paper, salt_2));
		let game = RockPaperScissor::games(game_id);
		matches!(game.states[0], MatchState::Choose);
		matches!(game.states[1], MatchState::Reveal);

		run_next_block();

		assert_ok!(RockPaperScissor::choose(Origin::signed(player_1), WeaponType::Scissors, salt_1));
		let game = RockPaperScissor::games(game_id);
		matches!(game.states[0], MatchState::Reveal);
		matches!(game.states[1], MatchState::Reveal);

		run_next_block();

		// Reveal phase
		assert_ok!(RockPaperScissor::reveal(Origin::signed(player_1), WeaponType::Scissors, salt_1));
		let game = RockPaperScissor::games(game_id);
		matches!(game.states[0], MatchState::Resolution);
		matches!(game.states[1], MatchState::Reveal);

		run_next_block();

		// trying to cheat !!!
		assert_noop!(RockPaperScissor::reveal(Origin::signed(player_2), WeaponType::Rock, salt_2),
			Error::<Test>::BadBehaviour);
		assert_noop!(RockPaperScissor::reveal(Origin::signed(player_2), WeaponType::Paper, salt_1),
			Error::<Test>::BadBehaviour);

		assert_ok!(RockPaperScissor::reveal(Origin::signed(player_2), WeaponType::Paper, salt_2));
		let game = RockPaperScissor::games(game_id);

		matches!(game.states[0], MatchState::Won);
		matches!(game.states[1], MatchState::Lost);

	});
}

#[test]
fn try_rps_player_2_win() {
	new_test_ext().execute_with(|| {

		let player_1:u64 = 1;
		let salt_1: [u8; 32] = [1u8;32];

		let player_2:u64 = 2;
		let salt_2: [u8; 32] = [2u8;32];

		// Create game
		assert_ok!(RockPaperScissor::new_game(Origin::signed(player_1), player_2));
		let game_id = RockPaperScissor::player_game(player_1);
		let game = RockPaperScissor::games(game_id);
		matches!(game.states[0], MatchState::Choose);
		matches!(game.states[1], MatchState::Choose);

		run_next_block();

		// Choose phase
		assert_ok!(RockPaperScissor::choose(Origin::signed(player_1), WeaponType::Rock, salt_1));
		let game = RockPaperScissor::games(game_id);
		matches!(game.states[0], MatchState::Reveal);
		matches!(game.states[1], MatchState::Reveal);

		run_next_block();

		assert_ok!(RockPaperScissor::choose(Origin::signed(player_2), WeaponType::Paper, salt_2));
		let game = RockPaperScissor::games(game_id);
		matches!(game.states[0], MatchState::Choose);
		matches!(game.states[1], MatchState::Reveal);

		run_next_block();

		// Reveal phase
		assert_ok!(RockPaperScissor::reveal(Origin::signed(player_1), WeaponType::Rock, salt_1));
		let game = RockPaperScissor::games(game_id);
		matches!(game.states[0], MatchState::Resolution);
		matches!(game.states[1], MatchState::Reveal);

		run_next_block();

		assert_ok!(RockPaperScissor::reveal(Origin::signed(player_2), WeaponType::Paper, salt_2));
		let game = RockPaperScissor::games(game_id);

		matches!(game.states[0], MatchState::Lost);
		matches!(game.states[1], MatchState::Won);

	});
}

#[test]
fn try_rps_players_draw() {
	new_test_ext().execute_with(|| {

		let player_1:u64 = 1;
		let salt_1: [u8; 32] = [1u8;32];

		let player_2:u64 = 2;
		let salt_2: [u8; 32] = [2u8;32];

		// Create game
		assert_ok!(RockPaperScissor::new_game(Origin::signed(player_1), player_2));
		let game_id = RockPaperScissor::player_game(player_1);
		let game = RockPaperScissor::games(game_id);
		matches!(game.states[0], MatchState::Choose);
		matches!(game.states[1], MatchState::Choose);

		run_next_block();

		// Choose phase
		assert_ok!(RockPaperScissor::choose(Origin::signed(player_1), WeaponType::Rock, salt_1));
		let game = RockPaperScissor::games(game_id);
		matches!(game.states[0], MatchState::Reveal);
		matches!(game.states[1], MatchState::Reveal);

		run_next_block();

		assert_ok!(RockPaperScissor::choose(Origin::signed(player_2), WeaponType::Rock, salt_2));
		let game = RockPaperScissor::games(game_id);
		matches!(game.states[0], MatchState::Choose);
		matches!(game.states[1], MatchState::Reveal);

		run_next_block();

		// Reveal phase
		assert_ok!(RockPaperScissor::reveal(Origin::signed(player_1), WeaponType::Rock, salt_1));
		let game = RockPaperScissor::games(game_id);
		matches!(game.states[0], MatchState::Resolution);
		matches!(game.states[1], MatchState::Reveal);

		run_next_block();

		assert_ok!(RockPaperScissor::reveal(Origin::signed(player_2), WeaponType::Rock, salt_2));
		let game = RockPaperScissor::games(game_id);

		matches!(game.states[0], MatchState::Draw);
		matches!(game.states[1], MatchState::Draw);

	});
}