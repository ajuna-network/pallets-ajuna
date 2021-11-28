#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

use codec::{Decode, Encode};

use sp_runtime::{
	traits::{Hash, TrailingZeroInput}
};

use scale_info::TypeInfo;

use sp_std::vec::{
	Vec
};

use sp_io::hashing::blake2_256;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum MatchState {
	None,
	Choose,
	Reveal,
	Resolution,
	Won,
	Draw,
	Lost
}
impl Default for MatchState { fn default() -> Self { Self::None } }

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum WeaponType {
	None,
	Rock,
	Paper,
	Scissors,
}
impl Default for WeaponType { fn default() -> Self { Self::None } }

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum Choice<Hash> {
	None,
	Choose(Hash),
	Reveal(WeaponType),
}
impl<Hash> Default for Choice<Hash> { fn default() -> Self { Self::None } }

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Game<Hash, AccountId> {
	pub id: Hash,
	pub players: [AccountId; 2],
	pub choices: [Choice<Hash>; 2],
	pub states:  [MatchState; 2],
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Default value for Nonce
	#[pallet::type_value]
	pub fn NonceDefault<T: Config>() -> u64 { 0 }
	// Nonce used for generating a different seed each time.
	#[pallet::storage]
	pub type Nonce<T: Config> = StorageValue<_, u64, ValueQuery, NonceDefault<T>>;

	#[pallet::storage]
	#[pallet::getter(fn games)]
	/// Store all games that are currently being played.
	pub type Games<T: Config> = StorageMap<_, Identity, T::Hash, Game<T::Hash, T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn player_game)]
	/// Store players active games, currently only one game per player allowed.
	pub type PlayerGame<T: Config> = StorageMap<_, Identity, T::AccountId, T::Hash, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new game got created.
		NewGame(T::Hash),
		/// A games match state changed.
		MatchStateChange(T::Hash, T::AccountId, MatchState),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Player can't play against them self.
		NoFakePlay,
		/// Player has already a game.
		PlayerHasGame,
		/// Player has no active game or there is no such game.
		GameDoesntExist,
		/// Player choice already exist.
		PlayerChoiceExist,
		/// Player choice doesn't exist.
		PlayerChoiceDoesntExist,
		/// Bad behaviour, trying to cheat?
		BadBehaviour,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Create game for two players
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn new_game(origin: OriginFor<T>, opponent: T::AccountId) -> DispatchResult {

			let sender = ensure_signed(origin)?;

			// Don't allow playing against yourself.
			ensure!(sender != opponent, Error::<T>::NoFakePlay);

			// Make sure players have no board open.
			ensure!(!PlayerGame::<T>::contains_key(&sender), Error::<T>::PlayerHasGame);
			ensure!(!PlayerGame::<T>::contains_key(&opponent), Error::<T>::PlayerHasGame);

			// Create new game
			let _game_id = Self::create_game([sender, opponent]);

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn choose(origin: OriginFor<T>, choice: WeaponType, salt: [u8; 32]) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Make sure player has a running game.
			ensure!(PlayerGame::<T>::contains_key(&sender), Error::<T>::GameDoesntExist);
			let game_id = Self::player_game(&sender);

			// Make sure game exists.
			ensure!(Games::<T>::contains_key(&game_id), Error::<T>::GameDoesntExist);

			// get players game
			let mut game = Self::games(&game_id);

			// get index of current player
			let mut me = 0;
			if game.players[1] == sender {
				me = 1;
			} else  {
				ensure!(game.players[0] == sender, Error::<T>::BadBehaviour);
			}

			// Make sure player has not already choosen in this game.
			ensure!(game.states[me] == MatchState::Choose, Error::<T>::PlayerChoiceExist);

			// add choice to game.
			game.choices[me] = Choice::Choose(Self::hash_choice(salt, choice as u8));

			// change player state
			game.states[me] = MatchState::Reveal;

			// write states and choices back to storage
			Games::<T>::insert(game.id, game);

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn reveal(origin: OriginFor<T>, choice: WeaponType, salt: [u8; 32]) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Make sure player has a running game.
			ensure!(PlayerGame::<T>::contains_key(&sender), Error::<T>::GameDoesntExist);
			let game_id = Self::player_game(&sender);

			// Make sure game exists.
			ensure!(Games::<T>::contains_key(&game_id), Error::<T>::GameDoesntExist);

			// get players game
			let mut game = Self::games(&game_id);

			// get index of current player
			let mut me = 0;
			if game.players[1] == sender {
				me = 1;
			} else  {
				ensure!(game.players[0] == sender, Error::<T>::BadBehaviour);
			}
			let he = (me + 1) % 2;

			// Make sure both players have made their choice and are in the reveal state.
			ensure!(game.states[me] == MatchState::Reveal && (game.states[he] == MatchState::Reveal || game.states[he] == MatchState::Resolution), Error::<T>::BadBehaviour);

			// get choice of player
			let player_choice = game.choices[me].clone();

			// check if the hash of the choice matches with the weapon unrevealed
			match player_choice {
				Choice::Choose(org_hash) => {
					// compare persisted hash with revealing value
					if org_hash == Self::hash_choice(salt, choice.clone() as u8)  {
						game.choices[me] = Choice::Reveal(choice);
					} else {
						Err(Error::<T>::BadBehaviour)?
					}
				},
				_ => Err(Error::<T>::BadBehaviour)?,
			}

			// change player state
			game.states[me] = MatchState::Resolution;

			// resolve game if both players waiting for resolution
			if game.states[0] == MatchState::Resolution && game.states[1] == MatchState::Resolution {

				let mut me_weapon: WeaponType = WeaponType::None;
				if let Choice::Reveal(weapon) = game.choices[me].clone() {
					me_weapon = weapon;
				}
				let mut he_weapon: WeaponType = WeaponType::None;
				if let Choice::Reveal(weapon) = game.choices[he].clone() {
					he_weapon = weapon;
				}

				match Self::game_logic(&me_weapon, &he_weapon) {
					0 => {
						game.states[me] = MatchState::Draw;
						game.states[he] = MatchState::Draw;
					},
					1 => {
						game.states[me] = MatchState::Won;
						game.states[he] = MatchState::Lost;
					},
					2 => {
						game.states[me] = MatchState::Lost;
						game.states[he] = MatchState::Won;
					},
					_ => {
						game.states[me] = MatchState::Draw;
						game.states[he] = MatchState::Draw;
					},
				}
			}

			// write states and choices back to storage
			Games::<T>::insert(game.id, game);

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {

	/// Update nonce once used.
	fn encode_and_update_nonce(
	) -> Vec<u8> {
		let nonce = <Nonce<T>>::get();
		<Nonce<T>>::put(nonce.wrapping_add(1));
		nonce.encode()
	}

	/// Generates a random hash out of a seed.
	fn generate_random_hash(
		phrase: &[u8],
		sender: T::AccountId
	) -> T::Hash {
		// FIXME: fake random for now
		let mut seed = <frame_system::Pallet<T>>::block_number().encode();
		seed.append(&mut phrase.to_vec());
		let seed: T::Hash = seed.using_encoded(T::Hashing::hash);
		let seed = <[u8; 32]>::decode(&mut TrailingZeroInput::new(seed.as_ref()))
			.expect("input is padded with zeroes; qed");
		return (seed, &sender, Self::encode_and_update_nonce()).using_encoded(T::Hashing::hash);
	}

	fn create_game(
		players: [T::AccountId; 2]
	) -> T::Hash {

		// get a random hash as board id
		let game_id = Self::generate_random_hash(b"create", players[0].clone());

		// create a new empty game
		let game = Game {
			id: game_id,
			players: players.clone(),
			choices: [Choice::None, Choice::None],
			states: [ MatchState::Choose, MatchState::Choose],
		};

		// insert the new board into the storage
		<Games<T>>::insert(game_id, game);

		// insert connection for each player with the game
		for player in &players {
			<PlayerGame<T>>::insert(player, game_id);
		}

		// emit event for a new game creation
		Self::deposit_event(Event::NewGame(game_id));

		game_id
	}

	fn hash_choice(
		salt: [u8; 32],
		choice: u8
	) -> T::Hash {
		let mut choice_value = salt;
		choice_value[31] = choice as u8;
		let choice_hashed = blake2_256(&choice_value);
		// return hashed choice
		choice_hashed.using_encoded(T::Hashing::hash)
	}

	fn game_logic(
		a: &WeaponType,
		b: &WeaponType
	) -> u8 {
		match a {
			WeaponType::None => {
				if a == b {
					return 0;
				} else {
					return 2;
				}
			},
			WeaponType::Rock => {
				if a == b {
					return 0;
				} else if let WeaponType::Paper = b {
					return 2;
				} else {
					return 1;
				}
			},
			WeaponType::Paper => {
				if a == b {
					return 0;
				} else if let WeaponType::Scissors = b {
					return 2;
				} else {
					return 1;
				}
			},
			WeaponType::Scissors => {
				if a == b {
					return 0;
				} else if let WeaponType::Rock = b {
					return 2;
				} else {
					return 1;
				}
			},
		}
	}
}