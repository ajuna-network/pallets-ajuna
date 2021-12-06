#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
use codec::{Decode, Encode};
use frame_support::{
	dispatch::DispatchResult,
	log,
	traits::{
		schedule::{DispatchTime, Named},
		LockIdentifier, Randomness,
	},
};

//use frame_system::WeightInfo;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{Dispatchable, Hash, TrailingZeroInput},
	RuntimeDebug,
};
use sp_std::vec::Vec;

use pallet_matchmaker::MatchFunc;

use log::info;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// importing the `weights.rs` here
//pub mod weights;

// importing queues, for game management
mod queues;

use queues::Queue;

/// GameState structure, allowing Client & TEE to determine actions.
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo)]
pub enum GameState<AccountId> {
	None,
	Waiting,
	Accepted,
	Running,
	Finished(AccountId),
}
impl<AccountId> Default for GameState<AccountId> {
	fn default() -> Self {
		Self::None
	}
}

/// Connect four board structure containing two players and the board
#[derive(Encode, Decode, Default, Clone, PartialEq, RuntimeDebug, TypeInfo)]
pub struct GameEngine {
	id: u8,
	version: u8,
}

/// Connect four board structure containing two players and the board
#[derive(Encode, Decode, Default, Clone, PartialEq, RuntimeDebug, TypeInfo)]
pub struct GameEntry<Hash, AccountId, GameEngine, GameState, BlockNumber> {
	id: Hash,
	tee_id: Option<AccountId>,
	game_engine: GameEngine,
	players: Vec<AccountId>,
	game_state: GameState,
	state_change: [BlockNumber; 4],
}

/// GameState structure, allowing Client & TEE to determine actions.
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo)]
pub enum GameRuleType {
	None,
	PlayersPerGame([u8; 2]),
}
impl Default for GameRuleType {
	fn default() -> Self {
		Self::None
	}
}

/// Connect four board structure containing two players and the board
#[derive(Encode, Decode, Default, Clone, PartialEq, RuntimeDebug, TypeInfo)]
pub struct GameRule<GameRuleType> {
	game_rule_type: GameRuleType,
	game_rule_info: [u8; 16],
}

const GAMEREGISTRY_ID: LockIdentifier = *b"gameregi";
const MAX_GAMES_PER_BLOCK: u8 = 10;
const MAX_QUEUE_SIZE: u8 = 64;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	// important to use outside structs and consts
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Proposal: Parameter + Dispatchable<Origin = Self::Origin> + From<Call<Self>>;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The generator used to supply randomness to contracts through `seal_random`.
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		type Scheduler: Named<Self::BlockNumber, Self::Proposal, Self::PalletsOrigin>;

		type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>>;

		type MatchMaker: MatchFunc<Self::AccountId>;

		// /// Weight information for extrinsics in this pallet.
		//type WeightInfo: WeightInfo;
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
	/// Founder key set in genesis, and maintained only for administration purpose.
	pub type FounderKey<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn game_queues)]
	/// Store all queues for the games.
	pub type GameQueues<T: Config> =
		StorageMap<_, Identity, GameEngine, Queue<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn game_registry)]
	/// Store all queues for the games.
	pub type GameRegistry<T: Config> = StorageMap<
		_,
		Identity,
		T::Hash,
		GameEntry<T::Hash, T::AccountId, GameEngine, GameState<T::AccountId>, T::BlockNumber>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn game_requirements)]
	/// Store all requirements for a sepecific game engine and it's version.
	pub type GameRequirments<T: Config> =
		StorageMap<_, Identity, GameEngine, Vec<GameRule<GameRuleType>>, ValueQuery>;

	// Default value for Nonce
	#[pallet::type_value]
	pub fn NonceDefault<T: Config>() -> u64 {
		0
	}
	// Nonce used for generating a different seed each time.
	#[pallet::storage]
	pub type Nonce<T: Config> = StorageValue<_, u64, ValueQuery, NonceDefault<T>>;

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

		/// Game queued in waiting queue
		GameQueued(GameEngine, T::Hash),

		/// Amount of Games accepted by specific AjunaTEE
		GamesAccepted(T::AccountId, u8),

		/// Game state changed to running, game is ready to play
		GameStateReady(T::AccountId, T::Hash),

		/// Game state changed to finished, with game winner
		GameStateFinished(T::Hash, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// To many games trying to acknowledge at once.
		AckToMany,
		/// During Acknowledge of a waiting games there was an error.
		AckFail,
		/// There is no game queue for the game engine version.
		NoGameQueue,
		/// There is no such game entry
		NoGameEntry,
		/// Player is already queued for a match.
		AlreadyQueued,
	}

	// Pallet implements [`Hooks`] trait to define some logic to execute in some context.
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
					let game_engine = GameEngine { id: 1u8, version: 1u8 };
					// Create new game
					let _game_id = Self::queue_game(game_engine, result);
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

			// Print out log or debug message in the console via log::{error, warn, info, debug, trace},
			// accepting format strings similar to `println!`.
			// https://substrate.dev/rustdocs/v3.0.0/log/index.html
			info!("New value is now: {:?}", something);

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

		/// Queue sender up for a game, ranking brackets
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn queue(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// #TODO[MUST_HAVE, ALLREADY_REGISTRED] check if player is already in the game registry for a game.

			let bracket: u8 = 0;
			// Add player to queue, duplicate check is done in matchmaker.
			if !T::MatchMaker::add_queue(sender, bracket) {
				return Err(Error::<T>::AlreadyQueued)?
			}

			Ok(())
		}

		/// Drop game will remove the game from the queue and the registry.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn drop_game(
			origin: OriginFor<T>,
			game_hash: T::Hash,
			game_engine: GameEngine,
		) -> DispatchResult {
			// #TODO[MUST_HAVE, SIGNATURE_CHECK] check that it's signed by a registred AjunaTEE.
			let _who = ensure_signed(origin)?;

			// retrieve game entry
			if GameRegistry::<T>::contains_key(&game_hash) {
				let _game_entry = GameRegistry::<T>::remove(&game_hash);

				let mut game_queue = Self::game_queues(&game_engine);

				// check if there is any elements queued
				if game_queue.length() > 0 {
					// remove element
					game_queue.remove(game_hash);
					// insert into waiting queue for Ajuna TEE
					<GameQueues<T>>::insert(game_engine, game_queue);
				}
			}

			// #TODO[MUST_HAVE, VEC_REMOVE] remove a game from the queue.

			Ok(())
		}

		/// Acknowledge game will remove from queue and set state to accepted.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn ack_game(
			origin: OriginFor<T>,
			cluster: GameEngine,
			games: Vec<T::Hash>,
		) -> DispatchResult {
			// #TODO[MUST_HAVE, SIGNATURE_CHECK] check that it's signed by a registred AjunaTEE.
			let who = ensure_signed(origin)?;

			// only up to 100 games allowed to acknowledge in one batch.
			if games.len() > 100 {
				return Err(Error::<T>::AckToMany)?
			}

			// #TODO[OPTIMIZATION, STORAGE] optimize storage to use a ringbuffer instead of the vector to avoid to big elements beeing read and written down to the queue.

			// retrieve game queue for asked cluster
			ensure!(GameQueues::<T>::contains_key(&cluster), Error::<T>::NoGameQueue);
			let mut game_queue = Self::game_queues(&cluster);

			let mut games_count = 0;
			for game_hash_tee in games.iter() {
				let game_hash = game_queue.peek();

				// check if peeked game matches acknowledge
				if game_hash == Some(game_hash_tee) {
					// dequeue game hash from waiting queue cluster
					let _ = game_queue.dequeue();

					// insert changed queue back
					<GameQueues<T>>::insert(cluster.clone(), game_queue.clone());

					// retrieve game entry to change state
					let mut game_entry = Self::game_registry(game_hash_tee.clone());

					game_entry.state_change[1] = <frame_system::Pallet<T>>::block_number();
					game_entry.game_state = GameState::Accepted;

					// insert changed game entry back
					<GameRegistry<T>>::insert(game_hash_tee, game_entry);

					// Increase counter
					games_count += 1;
				} else {
					return Err(Error::<T>::AckFail)?
				}
			}

			// Emit an event.
			Self::deposit_event(Event::GamesAccepted(who, games_count));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// Drop game will remove the game from the queue and the registry.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn ready_game(origin: OriginFor<T>, game_hash: T::Hash) -> DispatchResult {
			// #TODO[MUST_HAVE, SIGNATURE_CHECK] check that it's signed by a registred AjunaTEE.
			let who = ensure_signed(origin)?;

			// retrieve game entry
			ensure!(GameRegistry::<T>::contains_key(&game_hash), Error::<T>::NoGameEntry);
			let mut game_entry = Self::game_registry(&game_hash);

			game_entry.tee_id = Some(who.clone());
			game_entry.state_change[2] = <frame_system::Pallet<T>>::block_number();
			game_entry.game_state = GameState::Running;

			// insert changed game entry back
			<GameRegistry<T>>::insert(game_hash, game_entry.clone());

			// Emit an event.
			Self::deposit_event(Event::GameStateReady(who, game_hash));

			Ok(())
		}

		/// Drop game will remove the game from the queue and the registry.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn finish_game(
			origin: OriginFor<T>,
			game_hash: T::Hash,
			winner: T::AccountId,
		) -> DispatchResult {
			// #TODO[MUST_HAVE, SIGNATURE_CHECK] check that it's signed by a registred AjunaTEE.
			let who = ensure_signed(origin)?;

			// retrieve game entry
			ensure!(GameRegistry::<T>::contains_key(&game_hash), Error::<T>::NoGameEntry);
			let mut game_entry = Self::game_registry(&game_hash);

			game_entry.state_change[3] = <frame_system::Pallet<T>>::block_number();
			game_entry.game_state = GameState::Finished(winner.clone());

			// insert changed game entry back
			<GameRegistry<T>>::insert(game_hash, game_entry.clone());

			// Emit an event.
			Self::deposit_event(Event::GameStateFinished(game_hash, winner));

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

	/// Generate a new game between two players.
	fn queue_game(game_engine: GameEngine, players: Vec<T::AccountId>) -> DispatchResult {
		// check if requirements for this game are meet, for all the players.
		let game_rules = Self::game_requirements(&game_engine);
		for _game_rule in game_rules.iter() {
			// #TODO[MUST_HAVE, REQUIRMENTS_CHECK] check if game engine requirments are meet for the players.
		}

		// #TODO[MUST_HAVE, HAS_A_PLAYER] must have at least one player.

		// create new game entry with corresponding informations
		let game_entry = Self::create_game_entry(game_engine.clone(), players);

		// insert game entry into registry.
		<GameRegistry<T>>::insert(game_entry.id.clone(), game_entry.clone());

		// retrieve game queue for asked cluster
		let mut game_queue = Queue::new(MAX_QUEUE_SIZE.into());
		if GameQueues::<T>::contains_key(&game_engine) {
			game_queue = Self::game_queues(&game_engine);
		}

		// enqueue new game id
		game_queue.enqueue(game_entry.id.clone());

		// insert into waiting queue for Ajuna TEE
		<GameQueues<T>>::insert(&game_engine, game_queue);

		// Emit an event.
		Self::deposit_event(Event::GameQueued(game_engine, game_entry.id));

		// Return a successful DispatchResultWithPostInfo
		Ok(())
	}

	/// Generate a new game entry in waiting state.
	fn create_game_entry(
		game_engine: GameEngine,
		players: Vec<T::AccountId>,
	) -> GameEntry<T::Hash, T::AccountId, GameEngine, GameState<T::AccountId>, T::BlockNumber> {
		// get a random hash as game id
		let game_id = Self::generate_random_hash(&GAMEREGISTRY_ID, players[0].clone());

		// get current blocknumber
		let mut state_change: [T::BlockNumber; 4] = [0u8.into(); 4];
		state_change[0] = <frame_system::Pallet<T>>::block_number();

		// create a new empty game
		let game_entry = GameEntry {
			id: game_id,
			tee_id: None,
			game_engine,
			players,
			game_state: GameState::Waiting,
			state_change,
		};

		return game_entry
	}
}
