#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
use codec::{Decode, Encode};
use frame_support::traits::{schedule::Named, LockIdentifier, Randomness};
use pallet_matchmaker::MatchFunc;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::{Dispatchable, Hash, TrailingZeroInput};
use sp_std::vec::Vec;

use scale_info::TypeInfo;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

const RPSONLINE_ID: LockIdentifier = *b"rps-fire";

/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod rpscore;
use rpscore::{Direction, Logic, Weapon};

#[derive(Debug, Encode, Decode, Clone, PartialEq, TypeInfo)]
pub enum PhaseState<AccountId> {
	None,
	Move,
	Choose(Vec<AccountId>),
	Reveal(Vec<AccountId>),
}

impl<AccountId> Default for PhaseState<AccountId> {
	fn default() -> Self {
		Self::None
	}
}

#[derive(Debug, Encode, Decode, Clone, PartialEq, TypeInfo)]
pub enum GameState<AccountId> {
	None,
	Initiate(Vec<AccountId>),
	Prepare(Vec<AccountId>),
	Running(AccountId),
	Finished(AccountId),
}
impl<AccountId> Default for GameState<AccountId> {
	fn default() -> Self {
		Self::None
	}
}

#[derive(Debug, Encode, Decode, Clone, PartialEq, TypeInfo)]
pub enum NinjaState<Hash> {
	None,
	Stealth(Hash),
	Reveal(Weapon),
	Dead,
}
impl<Hash> Default for NinjaState<Hash> {
	fn default() -> Self {
		Self::None
	}
}

/// RPS Online board structure containing two players and the board
#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Game<Hash, AccountId, BlockNumber> {
	id: Hash,
	players: Vec<AccountId>,
	ninjas: [Vec<NinjaState<Hash>>; 2],
	board: [[u8; 6]; 7],
	last_move: [u8; 5],
	last_action: BlockNumber,
	phase_state: PhaseState<AccountId>,
	game_state: GameState<AccountId>,
}

impl<Hash, AccountId, BlockNumber> Game<Hash, AccountId, BlockNumber> {
	pub fn initialize(
		game_id: Hash,
		block_number: BlockNumber,
		players: Vec<AccountId>,
		game_state: GameState<AccountId>,
	) -> Self {
		return Game {
			id: game_id,
			players,
			ninjas: [Vec::new(), Vec::new()],
			board: Logic::initialize(),
			last_move: [u8::MAX, u8::MAX, u8::MAX, u8::MAX, u8::MAX],
			last_action: block_number,
			phase_state: PhaseState::None,
			game_state,
		}
	}
}

const PLAYER_1: u8 = 1;
const PLAYER_2: u8 = 2;
const MAX_GAMES_PER_BLOCK: u8 = 10;
const MAX_BLOCKS_PER_TURN: u8 = 10;
const CLEANUP_BOARDS_AFTER: u8 = 20;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	// important to use outside structs and consts
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The generator used to supply randomness to contracts through `seal_random`.
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		type Proposal: Parameter + Dispatchable<Origin = Self::Origin> + From<Call<Self>>;

		type Scheduler: Named<Self::BlockNumber, Self::Proposal, Self::PalletsOrigin>;

		type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>>;

		type MatchMaker: MatchFunc<Self::AccountId>;
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

	#[pallet::storage]
	#[pallet::getter(fn founder_key)]
	pub type FounderKey<T: Config> = StorageValue<_, T::AccountId>;

	// Default value for Nonce
	#[pallet::type_value]
	pub fn NonceDefault<T: Config>() -> u64 {
		0
	}
	// Nonce used for generating a different seed each time.
	#[pallet::storage]
	pub type Nonce<T: Config> = StorageValue<_, u64, ValueQuery, NonceDefault<T>>;

	#[pallet::storage]
	#[pallet::getter(fn games)]
	/// Store all games that are currently being played.
	pub type Games<T: Config> =
		StorageMap<_, Identity, T::Hash, Game<T::Hash, T::AccountId, T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn player_game)]
	/// Store players active games, currently only one game per player allowed.
	pub type PlayerGame<T: Config> = StorageMap<_, Identity, T::AccountId, T::Hash, ValueQuery>;

	// The genesis config type.
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub founder_key: T::AccountId,
	}

	// The default value for the genesis config type.
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { founder_key: Default::default() }
		}
	}

	// The build of genesis for the pallet.
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<FounderKey<T>>::put(&self.founder_key);
		}
	}

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		/// A new game got created.
		NewGame(T::Hash),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Only founder is allowed to do this.
		OnlyFounderAllowed,
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
		/// Bad setup, trying to cheat?
		BadSetup,
		/// Bad reveal, trying to cheat?
		BadReveal,
		/// Player is already queued.
		AlreadyQueued,
		/// Wrong phase state for action.
		WrongPhaseState,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// `on_initialize` is executed at the beginning of the block before any extrinsic are
		// dispatched.
		//
		// This function must return the weight consumed by `on_initialize` and `on_finalize`.
		fn on_initialize(_: T::BlockNumber) -> Weight {
			// Anything that needs to be done at the start of the block.
			// We don't do anything here.

			// initial weights
			let mut tot_weights = 10_000;
			for _i in 0..MAX_GAMES_PER_BLOCK {
				// try to create a match till we reached max games or no more matches available
				let result = T::MatchMaker::try_match();
				// if result is not empty we have a valid match
				if !result.is_empty() {
					// Create new game
					let _game_id = Self::create_game(result);
					// weights need to be adjusted
					tot_weights = tot_weights + T::DbWeight::get().reads_writes(1, 1);
					continue
				}
				break
			}

			// return standard weigth for trying to fiond a match
			return tot_weights
		}

		// `on_finalize` is executed at the end of block after all extrinsic are dispatched.
		fn on_finalize(_n: BlockNumberFor<T>) {
			// Perform necessary data/state clean up here.
		}

		// A runtime code run after every block and have access to extended set of APIs.
		//
		// For instance you can generate extrinsics for the upcoming produced block.
		fn offchain_worker(_n: T::BlockNumber) {
			// We don't do anything here.
			// but we could dispatch extrinsic (transaction/unsigned/inherent) using
			// sp_io::submit_extrinsic.
			// To see example on offchain worker, please refer to example-offchain-worker pallet
			// accompanied in this repository.
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}

		/// Create game for two players
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn new_game(origin: OriginFor<T>, opponent: T::AccountId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Don't allow playing against yourself.
			ensure!(sender != opponent, Error::<T>::NoFakePlay);

			// Don't allow queued player to create a game.
			ensure!(!T::MatchMaker::is_queued(sender.clone()), Error::<T>::AlreadyQueued);
			ensure!(!T::MatchMaker::is_queued(opponent.clone()), Error::<T>::AlreadyQueued);

			// Make sure players have no board open.
			ensure!(!PlayerGame::<T>::contains_key(&sender), Error::<T>::PlayerHasGame);
			ensure!(!PlayerGame::<T>::contains_key(&opponent), Error::<T>::PlayerHasGame);

			let mut players = Vec::new();
			players.push(sender.clone());
			players.push(opponent.clone());

			// Create new game
			let _game_id = Self::create_game(players);

			Ok(())
		}

		/// Queue sender up for a game, ranking brackets
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn queue(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Make sure player has no board open.
			ensure!(!PlayerGame::<T>::contains_key(&sender), Error::<T>::PlayerHasGame);

			let bracket: u8 = 0;
			// Add player to queue, duplicate check is done in matchmaker.
			if !T::MatchMaker::add_queue(sender, bracket) {
				return Err(Error::<T>::AlreadyQueued)?
			}

			Ok(())
		}

		/// Empty all brackets, this is a founder only extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn empty_queue(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Make sure sender is founder.
			ensure!(sender == Self::founder_key().unwrap(), Error::<T>::OnlyFounderAllowed);

			// Empty queues
			T::MatchMaker::all_empty_queue();

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn initiate(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Make sure player has a running game.
			ensure!(PlayerGame::<T>::contains_key(&sender), Error::<T>::GameDoesntExist);
			let game_id = Self::player_game(&sender);

			// Make sure game exists.
			ensure!(Games::<T>::contains_key(&game_id), Error::<T>::GameDoesntExist);

			// get players game
			let game = Self::games(&game_id);

			// check if we have correct state
			if let GameState::Initiate(_) = game.game_state {
				// check we have the correct state
			} else {
				Err(Error::<T>::BadBehaviour)?
			}

			// game state change
			if !Self::game_state_change(sender, game) {
				Err(Error::<T>::BadBehaviour)?
			}

			Ok(())
		}

		// TODO: Remove salt and replace through already computed hashes, once everything is running.
		// pub fn prepare(origin: OriginFor<T>, setup: [T::Hash;14]) -> DispatchResult {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn prepare(origin: OriginFor<T>, setup: [u8; 14], salt: [u8; 32]) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Make sure player has a running game.
			ensure!(PlayerGame::<T>::contains_key(&sender), Error::<T>::GameDoesntExist);
			let game_id = Self::player_game(&sender);

			// Make sure game exists.
			ensure!(Games::<T>::contains_key(&game_id), Error::<T>::GameDoesntExist);

			// get players game
			let mut game = Self::games(&game_id);

			// get index of current player
			let index = game.players.iter().position(|p| *p == sender).unwrap();

			// setup ninjas for fight
			let mut ninjas: Vec<NinjaState<T::Hash>> = Vec::new();
			let mut check: [u8; 14] = [0u8; 14];
			for i in 0..14 {
				match setup[i] {
					0 => ninjas.push(NinjaState::Stealth(Self::hash_choice(salt, i, Weapon::King))),
					1 => ninjas.push(NinjaState::Stealth(Self::hash_choice(salt, i, Weapon::Trap))),
					2 | 3 | 4 | 5 =>
						ninjas.push(NinjaState::Stealth(Self::hash_choice(salt, i, Weapon::Rock))),
					6 | 7 | 8 | 9 =>
						ninjas.push(NinjaState::Stealth(Self::hash_choice(salt, i, Weapon::Paper))),
					10 | 11 | 12 | 13 => ninjas.push(NinjaState::Stealth(Self::hash_choice(
						salt,
						i,
						Weapon::Scissor,
					))),
					// out of range u8 leads to error
					_ => Err(Error::<T>::BadSetup)?,
				}
				// check occurance maximum one, otherwise leads to error
				if check[setup[i] as usize] == 0 {
					check[setup[i] as usize] = 1;
				} else {
					Err(Error::<T>::BadSetup)?
				}
			}

			// set player ninjas in game.
			game.ninjas[index] = ninjas;

			// check if we have correct state
			if let GameState::Prepare(_) = game.game_state {
				// check we have the correct state
			} else {
				Err(Error::<T>::BadBehaviour)?
			}

			// game state change and persisting logic
			if !Self::game_state_change(sender, game) {
				Err(Error::<T>::BadBehaviour)?
			}

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn play_move(
			origin: OriginFor<T>,
			position: [u8; 2],
			direction: Direction,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Make sure position is in range
			ensure!(Logic::position(position), Error::<T>::BadBehaviour);

			// Make sure player has a running game.
			ensure!(PlayerGame::<T>::contains_key(&sender), Error::<T>::GameDoesntExist);
			let game_id = Self::player_game(&sender);

			// Make sure game exists.
			ensure!(Games::<T>::contains_key(&game_id), Error::<T>::GameDoesntExist);

			// get players game
			let mut game = Self::games(&game_id);

			// Make sure game is in correct phase.
			ensure!(game.phase_state == PhaseState::Move, Error::<T>::WrongPhaseState);

			game.last_move = [position[0], position[1], direction as u8, u8::MAX, u8::MAX];

			// check if we have correct state
			if let GameState::Running(_) = game.game_state {
				// check we have the correct state
			} else {
				Err(Error::<T>::BadBehaviour)?
			}

			// game state change and persisting logic
			if !Self::game_state_change(sender, game) {
				Err(Error::<T>::BadBehaviour)?
			}

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn reveal_position(
			origin: OriginFor<T>,
			ninja: u8,
			weapon: Weapon,
			salt: [u8; 32],
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Make sure ninja is in range
			ensure!(ninja < 14, Error::<T>::BadBehaviour);

			// Make sure player has a running game.
			ensure!(PlayerGame::<T>::contains_key(&sender), Error::<T>::GameDoesntExist);
			let game_id = Self::player_game(&sender);

			// Make sure game exists.
			ensure!(Games::<T>::contains_key(&game_id), Error::<T>::GameDoesntExist);

			// get players game
			let mut game = Self::games(&game_id);

			// Make sure game is in correct phase.
			ensure!(matches!(game.phase_state, PhaseState::Reveal(_)), Error::<T>::WrongPhaseState);

			// get index of current player
			let index = game.players.iter().position(|p| *p == sender).unwrap();

			// check if we have correct state
			if let GameState::Running(current_player) = game.game_state.clone() {
				let ninja_number;
				// check we have the correct state
				if current_player == sender {
					ninja_number =
						game.board[game.last_move[0] as usize][game.last_move[1] as usize];
				} else {
					ninja_number =
						game.board[game.last_move[3] as usize][game.last_move[4] as usize];
				}

				// check if ninja is from player
				if !Self::is_ninja_from_player(ninja_number, index) {
					Err(Error::<T>::BadBehaviour)?
				}

				// check if it's the correct ninja
				if ninja != Self::get_ninja_index(ninja_number, index) {
					Err(Error::<T>::BadBehaviour)?
				}

				let ninja_state = game.ninjas[index][ninja as usize].clone();
				if let NinjaState::Stealth(hash) = ninja_state {
					if hash == Self::hash_choice(salt, ninja as usize, weapon.clone()) {
						// correct reaveal add new state
						game.ninjas[index][ninja as usize] = NinjaState::Reveal(weapon);
					} else {
						Err(Error::<T>::BadReveal)?
					}
				} else {
					Err(Error::<T>::BadBehaviour)?
				}
			} else {
				Err(Error::<T>::BadBehaviour)?
			}

			// game state change and persisting logic
			if !Self::game_state_change(sender, game) {
				Err(Error::<T>::BadBehaviour)?
			}

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn choose_weapon(
			origin: OriginFor<T>,
			ninja: u8,
			weapon: Weapon,
			salt: [u8; 32],
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Make sure ninja is in range
			ensure!(ninja < 14, Error::<T>::BadBehaviour);

			// Make sure player has a running game.
			ensure!(PlayerGame::<T>::contains_key(&sender), Error::<T>::GameDoesntExist);
			let game_id = Self::player_game(&sender);

			// Make sure game exists.
			ensure!(Games::<T>::contains_key(&game_id), Error::<T>::GameDoesntExist);

			// get players game
			let mut game = Self::games(&game_id);

			// Make sure game is in correct phase.
			ensure!(matches!(game.phase_state, PhaseState::Choose(_)), Error::<T>::WrongPhaseState);

			// get index of current player
			let index = game.players.iter().position(|p| *p == sender).unwrap();

			// check if we have correct state
			if let GameState::Running(current_player) = game.game_state.clone() {
				let ninja_number;
				// check we have the correct state
				if current_player == sender {
					ninja_number =
						game.board[game.last_move[0] as usize][game.last_move[1] as usize];
				} else {
					ninja_number =
						game.board[game.last_move[3] as usize][game.last_move[4] as usize];
				}

				// check if ninja is from player
				if !Self::is_ninja_from_player(ninja_number, index) {
					Err(Error::<T>::BadBehaviour)?
				}

				// check if it's the correct ninja
				if ninja != Self::get_ninja_index(ninja_number, index) {
					Err(Error::<T>::BadBehaviour)?
				}

				let ninja_state = game.ninjas[index][ninja as usize].clone();
				if let NinjaState::Reveal(weapon) = ninja_state {
					game.ninjas[index][ninja as usize] =
						NinjaState::Stealth(Self::hash_choice(salt, ninja as usize, weapon));
				} else {
					Err(Error::<T>::BadBehaviour)?
				}
			} else {
				Err(Error::<T>::BadBehaviour)?
			}

			// game state change and persisting logic
			if !Self::game_state_change(sender, game) {
				Err(Error::<T>::BadBehaviour)?
			}

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Update nonce once used.
	fn encode_and_update_nonce() -> Vec<u8> {
		let nonce = <Nonce<T>>::get();
		<Nonce<T>>::put(nonce.wrapping_add(1));
		nonce.encode()
	}

	/// Generates a random hash out of a seed.
	fn generate_random_hash(phrase: &[u8], sender: T::AccountId) -> T::Hash {
		let (seed, _) = T::Randomness::random(phrase);
		let seed = <[u8; 32]>::decode(&mut TrailingZeroInput::new(seed.as_ref()))
			.expect("input is padded with zeroes; qed");
		return (seed, &sender, Self::encode_and_update_nonce()).using_encoded(T::Hashing::hash)
	}

	fn hash_choice(salt: [u8; 32], position: usize, choice: Weapon) -> T::Hash {
		let mut choice_value = salt;
		choice_value[30] = position as u8;
		choice_value[31] = choice as u8;
		let choice_hashed = blake2_256(&choice_value);
		// return hashed choice
		choice_hashed.using_encoded(T::Hashing::hash)
	}

	fn create_game(players: Vec<T::AccountId>) -> T::Hash {
		// get a random hash as board id
		let game_id = Self::generate_random_hash(b"create", players[0].clone());

		// get current blocknumber
		let block_number = <frame_system::Pallet<T>>::block_number();

		// create a new empty game
		let game: Game<T::Hash, T::AccountId, T::BlockNumber> = Game::initialize(
			game_id,
			block_number,
			players.clone(),
			GameState::Initiate(players.clone()),
		);

		// insert the new game into the storage
		<Games<T>>::insert(game_id, game);

		// insert conenction for each player with the game
		for player in &players {
			<PlayerGame<T>>::insert(player, game_id);
		}

		// emit event for a new game creation
		Self::deposit_event(Event::NewGame(game_id));

		game_id
	}

	fn try_remove(player: T::AccountId, players: &mut Vec<T::AccountId>) -> bool {
		if let Some(p) = players.iter().position(|x| *x == player) {
			// remove player from vec
			players.swap_remove(p);
			return true
		}

		false
	}

	fn game_state_change(
		player: T::AccountId,
		mut game: Game<T::Hash, T::AccountId, T::BlockNumber>,
	) -> bool {
		match game.game_state.clone() {
			GameState::Initiate(mut players) => {
				if !Self::try_remove(player, &mut players) {
					return false
				}
				// check if all players have initiated
				if players.is_empty() {
					game.game_state = GameState::Prepare(game.players.clone());
				} else {
					game.game_state = GameState::Initiate(players);
				}
			},

			GameState::Prepare(mut players) => {
				if !Self::try_remove(player, &mut players) {
					return false
				}
				// check if all players have choosen
				if players.is_empty() {
					game.phase_state = PhaseState::Move;
					// TODO: Randomly choose starting player
					game.game_state = GameState::Running(game.players[0].clone());
				} else {
					game.game_state = GameState::Prepare(players);
				}
			},

			// turn by turn logic
			GameState::Running(player_at_turn) => {
				// get index of current player
				let index = game.players.iter().position(|p| *p == player).unwrap();
				// get index of opponent
				let opponent_index = ((index + 1) % 2) as usize;

				match game.phase_state.clone() {
					// phase move state
					PhaseState::Move => {
						// Make sure it's current players turn
						if player != player_at_turn {
							return false
						}

						// check if player has a ninja at that position, previously check for legit position Logic::position = true
						let current_position =
							game.board[game.last_move[0] as usize][game.last_move[1] as usize];
						if current_position / 16 != index as u8 {
							return false
						}
						let current_index = Self::get_ninja_index(current_position, index);

						// check if player can move in that direction and get target position
						let mut destination = [game.last_move[0], game.last_move[1]];
						if !Logic::destination(index as u8, &mut destination, game.last_move[2]) {
							return false
						}

						// check target position for own ninja
						let target_position =
							game.board[destination[0] as usize][destination[1] as usize];
						if target_position / 16 == index as u8 {
							return false
						}
						let target_index = Self::get_ninja_index(target_position, opponent_index);

						// set move destination
						game.last_move[3] = destination[0];
						game.last_move[4] = destination[1];

						// move ninja to target position, if empty
						if target_position == u8::MAX {
							game.board[destination[0] as usize][destination[1] as usize] =
								current_position;
							game.board[game.last_move[0] as usize][game.last_move[1] as usize] =
								u8::MAX;
						} else {
							// opponent has a ninja at target position
							let ninja = game.ninjas[index][current_index as usize].clone();
							let opponent_ninja =
								game.ninjas[opponent_index][target_index as usize].clone();

							let mut reveal_players: Vec<T::AccountId> = Vec::new();
							let mut weapon_players: Vec<Weapon> = Vec::new();

							// if opponent ninja go into reveal mode if ninjas aren't revealed or in semi reveal mode if one of the is already revealed
							match ninja {
								NinjaState::Stealth(_) =>
									reveal_players.push(game.players[index].clone()),
								NinjaState::Reveal(weapon) => weapon_players.push(weapon), //
								_ => return false,
							}

							match opponent_ninja {
								NinjaState::Stealth(_) =>
									reveal_players.push(game.players[opponent_index].clone()),
								NinjaState::Reveal(weapon) => weapon_players.push(weapon), //
								_ => return false,
							}

							// both ninjas are already unreavelead and can fight.
							if reveal_players.len() > 0 {
								game.phase_state = PhaseState::Reveal(reveal_players);
							} else {
								match Logic::combat(&weapon_players[0], &weapon_players[1]) {
									0 => {
										// attacker won the combat and moves into new position
										game.ninjas[opponent_index][target_index as usize] =
											NinjaState::Dead;
										game.board[game.last_move[0] as usize]
											[game.last_move[1] as usize] = u8::MAX;
										game.board[game.last_move[3] as usize]
											[game.last_move[4] as usize] = current_position;
										game.phase_state = PhaseState::Move;
									},
									1 => {
										// defender won the combat and stys in position
										game.ninjas[index][current_index as usize] =
											NinjaState::Dead;
										game.board[game.last_move[0] as usize]
											[game.last_move[1] as usize] = u8::MAX;
										game.phase_state = PhaseState::Move;
									},
									_ => {
										// even combat new weapon choose for both
										game.phase_state = PhaseState::Choose(game.players.clone());
									},
								}
							}
						}
					},

					// phase choose state
					PhaseState::Choose(mut players) => {
						if !Self::try_remove(player.clone(), &mut players) {
							return false
						}
						// check if all players have initiated
						if players.is_empty() {
							game.phase_state = PhaseState::Reveal(game.players.clone());
						} else {
							game.phase_state = PhaseState::Choose(players);
						}
					},

					// phase reveal state
					PhaseState::Reveal(mut players) => {
						if !Self::try_remove(player.clone(), &mut players) {
							return false
						}
						// check if all players have initiated
						if players.is_empty() {
							let mut attacker = index;
							let mut defender = opponent_index;
							if let GameState::Running(current_player) = game.game_state.clone() {
								if player != current_player {
									attacker = opponent_index;
									defender = index;
								}
							}

							let attacker_posiion = game.board[game.last_move[0] as usize]
								[game.last_move[1] as usize]
								.clone();
							let attacker_index = Self::get_ninja_index(attacker_posiion, attacker);
							let defender_position = game.board[game.last_move[3] as usize]
								[game.last_move[4] as usize]
								.clone();
							let defender_index = Self::get_ninja_index(defender_position, defender);

							// opponent has a ninja at target position
							let attacker_ninja =
								game.ninjas[attacker][attacker_index as usize].clone();
							let defender_ninja =
								game.ninjas[defender][defender_index as usize].clone();

							if let NinjaState::Reveal(attacker_weapon) = attacker_ninja {
								if let NinjaState::Reveal(defender_weapon) = defender_ninja {
									match Logic::combat(&attacker_weapon, &defender_weapon) {
										0 => {
											// attacker won the combat and moves into new position
											game.ninjas[defender][defender_index as usize] =
												NinjaState::Dead;
											game.board[game.last_move[0] as usize]
												[game.last_move[1] as usize] = u8::MAX;
											game.board[game.last_move[3] as usize]
												[game.last_move[4] as usize] = attacker_posiion;
											game.phase_state = PhaseState::Move;
										},
										1 => {
											// defender won the combat and stys in position
											game.ninjas[attacker][attacker_index as usize] =
												NinjaState::Dead;
											game.board[game.last_move[0] as usize]
												[game.last_move[1] as usize] = u8::MAX;
											game.phase_state = PhaseState::Move;
										},
										_ => {
											// even combat new weapon choose for both
											game.phase_state =
												PhaseState::Choose(game.players.clone());
										},
									}
								} else {
									return false
								}
							} else {
								return false
							}
						} else {
							game.phase_state = PhaseState::Reveal(players);
						}
					},

					_ => {},
				}

				// after successull play change current player to next
				if game.phase_state == PhaseState::Move {
					game.game_state = GameState::Running(game.players[(index + 1) % 2].clone());
				}
			},

			GameState::Finished(_) => {},

			_ => return false,
		}

		// get current blocknumber
		let block_number = <frame_system::Pallet<T>>::block_number();
		game.last_action = block_number;

		// persist game
		<Games<T>>::insert(game.id, game);

		true
	}

	fn get_ninja_index(position_value: u8, player_index: usize) -> u8 {
		position_value - (player_index as u8 * 16)
	}

	fn is_ninja_from_player(position_value: u8, player_index: usize) -> bool {
		position_value / 16 == player_index as u8
	}
}
