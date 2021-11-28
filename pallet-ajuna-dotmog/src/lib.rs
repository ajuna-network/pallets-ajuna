// DOT Mog, Susbstrate Gamification Project with C# .NET Standard & Unity3D
// Copyright (C) 2020-2021 DOT Mog Team, darkfriend77 & metastar77
//
// DOT Mog is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License.
// DOT Mog is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::{
	codec::{Decode, Encode},
	decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
	traits::{
		Currency, ExistenceRequirement, Get, OnUnbalanced, Randomness, ReservableCurrency,
		WithdrawReasons,
	},
	PalletId,
};
use frame_system::ensure_signed;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AccountIdConversion, Hash, TrailingZeroInput, Zero},
	SaturatedConversion,
};
use sp_std::{prelude::*, vec::Vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Implementations of some helper traits passed into runtime modules as associated types.
pub mod general;
use general::{BreedType, Breeding, FeeType, Generation, Pricing, RarityType};

pub mod game_event;
use game_event::GameEventType;

pub mod game_config;
use game_config::GameConfig;

const MAX_AUCTIONS_PER_BLOCK: usize = 2;
const MAX_EVENTS_PER_BLOCK: usize = 10;

#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MogwaiStruct<Hash, BlockNumber, Balance, RarityType> {
	id: Hash,
	dna: Hash,
	genesis: BlockNumber,
	price: Balance,
	gen: u32,
	rarity: RarityType,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MogwaiBios<Hash, BlockNumber, Balance> {
	mogwai_id: Hash,
	state: u32,
	metaxy: Vec<[u8; 16]>,
	intrinsic: Balance,
	level: u8,
	phases: Vec<BlockNumber>,
	adaptations: Vec<Hash>,
}

//#[derive(Encode, Decode, Default, Clone, PartialEq)]
//#[cfg_attr(feature = "std", derive(Debug))]
//pub struct MogwaiArt<Hash, BlockNumber, Balance> {
//	mogwai_id: Hash,
//}

#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct GameEvent<Hash, BlockNumber, GameEventType> {
	id: Hash,
	begin: BlockNumber,
	duration: u16,
	event_type: GameEventType,
	hashes: Vec<Hash>,
	value: u64,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Auction<Hash, Balance, BlockNumber, AccountId> {
	mogwai_id: Hash,
	mogwai_owner: AccountId,
	expiry: BlockNumber,
	min_bid: Balance,
	high_bid: Balance,
	high_bidder: AccountId,
}

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
	/// The dotmog's module id, is used for deriving its mogwai account ID's.
	type PalletId: Get<PalletId>;

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

	/// The currency mechanism.
	type Currency: ReservableCurrency<Self::AccountId>;

	/// Something that provides randomness in the runtime.
	type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

	/// Handler for price payments.
	type PricePayment: OnUnbalanced<NegativeImbalanceOf<Self>>;

	// Weight information for extrinsics in this pallet.
	//type WeightInfo: WeightInfo;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Config> as DotMogModule {

		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Something get(fn something): Option<u32>;

		/// The `AccountId` of the dot mog founder.
		Key get(fn key) config(): T::AccountId;

		/// A map of the current configuration of an account.
		AccountConfig get(fn account_config): map hasher(blake2_128_concat) T::AccountId => Option<Vec<u8>>;

		/// A map of mogwais accessible by the mogwai hash.
		Mogwais get(fn mogwai): map hasher(identity) T::Hash => MogwaiStruct<T::Hash, T::BlockNumber, BalanceOf<T>, RarityType>;
		/// A map of mogwai bios accessible by the mogwai hash.
		MogwaisBios get(fn mogwai_bios): map hasher(identity) T::Hash => MogwaiBios<T::Hash, T::BlockNumber, BalanceOf<T>>;
		/// A map of mogwai owners accessible by the mogwai hash.
		MogwaiOwner get(fn owner_of): map hasher(identity) T::Hash => Option<T::AccountId>;

		/// A map of all existing mogwais accessible by the index.
		AllMogwaisArray get(fn mogwai_by_index): map hasher(blake2_128_concat) u64 => T::Hash;
		/// A count over all existing mogwais in the system.
		AllMogwaisCount get(fn all_mogwais_count): u64;
		/// A map of the index of the mogwai accessible by the mogwai hash.
		AllMogwaisIndex: map hasher(identity) T::Hash => u64;

		/// A map of all mogwai hashes associated with an account.
		OwnedMogwaisArray get(fn mogwai_of_owner_by_index): map hasher(blake2_128_concat) (T::AccountId, u64) => T::Hash;
		/// A count over all existing mogwais owned by one account.
		OwnedMogwaisCount get(fn owned_mogwais_count): map hasher(blake2_128_concat) T::AccountId => u64;
		/// A map of the owned mogwais index accessible by the mogwai hash.
		OwnedMogwaisIndex: map hasher(identity) T::Hash => u64;

		/// A map of mogwai auctions accessible by the mogwai hash.
		MogwaiAuction get(fn auction_of): map hasher(blake2_128_concat) T::Hash => Option<Auction<T::Hash, BalanceOf<T>, T::BlockNumber, T::AccountId>>;
		/// A vec of mogwai auctions accessible by the expiry block number.
		Auctions get(fn auctions_expire_at): map hasher(blake2_128_concat) T::BlockNumber => Vec<Auction<T::Hash, BalanceOf<T>, T::BlockNumber, T::AccountId>>;
		/// Current auction period max limit.
		AuctionPeriodLimit get(fn auction_period_limit): T::BlockNumber = (1000 as u32).into();

		/// A map of bids accessible by account id and mogwai hash.
		Bids get(fn bid_of): map hasher(blake2_128_concat) (T::Hash, T::AccountId) => BalanceOf<T>;
		/// A vec of accounts accessible by mogwai hash.
		BidAccounts get(fn bid_accounts): map hasher(blake2_128_concat) T::Hash => Vec<T::AccountId>;

		/// A map of game events accessible by the game event id (hash).
		GameEvents get(fn game_events): map hasher(identity) T::Hash => GameEvent<T::Hash, T::BlockNumber, GameEventType>;

		/// A map of all existing game events accessible by the index.
		AllGameEventsArray get(fn game_event_by_index): map hasher(blake2_128_concat) u64 => T::Hash;
		/// A count over all existing game events in the system.
		AllGameEventsCount get(fn all_game_events_count): u64;
		/// A map of the index of the game events accessible by the game event id (hash).
		AllGameEventsIndex: map hasher(identity) T::Hash => u64;

		/// A map of all game event ids (hash) associated with an game event type (indexed).
		GameEventsArray get(fn game_event_of_type_by_index): map hasher(blake2_128_concat) (GameEventType, u64) => T::Hash;
		/// A count over all existing game events of one particular game event type.
		GameEventsCount get(fn game_event_of_type_count): map hasher(blake2_128_concat) GameEventType => u64;
		/// A map of the game event type index accessible by the game event id (hash).
		GameEventsIndex: map hasher(identity) T::Hash => u64;

		/// A vec of game event ids (hash) accessible by the triggering block number.
		GameEventsAtBlock get(fn game_events_at_block): map hasher(blake2_128_concat) T::BlockNumber => Vec<T::Hash>;

		/// A vec of game event ids (hash) accessible by the corresponding mogwai.
		GameEventsOfMogwai get(fn game_events_of_mogwai): map hasher(identity) T::Hash => Vec<T::Hash>;

		/// The nonce used for randomness.
		Nonce: u64 = 0;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where
		<T as frame_system::Config>::AccountId,
		<T as frame_system::Config>::Hash,
		<T as frame_system::Config>::BlockNumber,
		Balance = BalanceOf<T>,
	{

		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, AccountId),

		// A dot mog sudo just took place.
		//DotMogDid(DispatchResult),

		// A account configuration has been changed.
		AccountConfigChanged(AccountId, Vec<u8>),

		/// A mogwai has been created.
		MogwaiCreated(AccountId, Hash),

		/// A mogwai has been removed. (R.I.P.)
		MogwaiRemoved(AccountId, Hash),

		/// A price has been set for a mogwai.
		PriceSet(AccountId, Hash, Balance),

		/// A mogwai changed his owner.
		Transferred(AccountId, AccountId, Hash),

		/// A mogwai has been was bought.
		Bought(AccountId, AccountId, Hash, Balance),

		/// A auction has been created
		AuctionCreated(Hash, Balance, BlockNumber),

		/// A bid has been placed.
		Bid(Hash, Balance, AccountId),

		/// A auction hash been finalized.
		AuctionFinalized(Hash, Balance, BlockNumber),

		/// A game event hash been created.
		GameEventCreated(AccountId, Hash),

		/// A game event hash been executed.
		GameEventExecuted(Hash),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {

		/// Error names should be descriptive.
		NoneValue,

		// Sender must be the dot mog sudo account
		//RequireDotMogSudo,

		/// A Storage overflow, has occured make sure to validate first.
		StorageOverflow,

		/// The mogwai id (hash) already exists.
		MogwaiAlreadyExists,

		/// The mogwais hash doesn't exist.
		MogwaiDoesntExists,

		/// The mogwai has pending game events.
		MogwaiHasGameEvents,

		/// The mogwai isn't owned by the sender.
		MogwaiNotOwned,

		/// Same mogwai choosen for extrinsic.
		MogwaiSame,

		/// Maximum Mogwais in account reached.
		MaxMogwaisInAccount,

		/// The submitted index is out of range.
		ConfigIndexOutOfRange,

		/// Invalid or unimplemented config update.
		ConfigUpdateInvalid,

		/// Incompatible generation
		MogwaiIncompatibleGeneration,

		// Mogwai doesn't have a bios code.
		MogwaiHasNoBios,

		/// The game event id (hash) already exists.
		GameEventAlreadyExists,
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

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn do_something(origin, something: u32) -> dispatch::DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			Something::put(something);

			// Emit an event.
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			// Return a successful DispatchResult
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		fn cause_error(origin) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match Something::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					Something::put(new);
					Ok(())
				},
			}
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn update_config(origin, index: u8, value_opt: Option<u8>) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			ensure!(index < GameConfig::PARAM_COUNT, Error::<T>::ConfigIndexOutOfRange);

			let config_opt = <AccountConfig<T>>::get(&sender);
			let mut game_config = GameConfig::new();
			if config_opt.is_some() {
				game_config.parameters = config_opt.unwrap();
			}

			// TODO: add rules (min, max) for different configurations
			let update_value:u8 = GameConfig::verify_update(index, game_config.parameters[usize::from(index)], value_opt);

			ensure!(update_value > 0, Error::<T>::ConfigUpdateInvalid);

			let price = Pricing::config_update_price(index, update_value);
			if price > 0 {
				Self::pay_founder(sender.clone(), price.saturated_into())?;
			}

			//if value_opt.is_some() {
			//	game_config.parameters[usize::from(index)] = value_opt.unwrap();
			//} else {
			//	let config_value = game_config.parameters[usize::from(index)].checked_add(1)
			//		.ok_or("Overflow adding a one to index")?;
			//		game_config.parameters[usize::from(index)] = config_value;
			//}

			game_config.parameters[usize::from(index)] = update_value;

			// updating to the new configuration
			<AccountConfig<T>>::insert(&sender, &game_config.parameters);

			// Emit an event.
			Self::deposit_event(RawEvent::AccountConfigChanged(sender, game_config.parameters));

			// Return a successful DispatchResult
			Ok(())
		}

		/// Set price of mogwai.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn set_price(origin, mogwai_id: T::Hash, new_price: BalanceOf<T>) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			ensure!(Mogwais::<T>::contains_key(mogwai_id), Error::<T>::MogwaiDoesntExists);

			let owner = Self::owner_of(mogwai_id).ok_or("No owner for this mogwai")?;

			ensure!(owner == sender, "You don't own this mogwai");

			let mut mogwai = Self::mogwai(mogwai_id);
			mogwai.price = new_price;

			<Mogwais<T>>::insert(mogwai_id, mogwai);

			Self::deposit_event(RawEvent::PriceSet(sender, mogwai_id, new_price));

			Ok(())
		}

		/// Create a new mogwai.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn create_mogwai(origin) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			// ensure that we have enough space
			ensure!(Self::ensure_not_max_mogwais(sender.clone()), Error::<T>::MaxMogwaisInAccount);

			//let data_hash = T::Hashing::hash(random_bytes.as_bytes());
			let block_number = <frame_system::Pallet<T>>::block_number();
			//let block_hash = <frame_system::Pallet<T>>::block_hash(block_number);

			let random_hash = Self::generate_random_hash(b"create_mogwai", sender.clone());

			let new_mogwai = MogwaiStruct {
							id: random_hash,
							dna: random_hash,
							genesis: block_number,
							price: Zero::zero(),
							gen: 0, // straight created mogwais are hybrid mogwais of gen 0
							rarity: RarityType::Minor,
			};

			Self::mint(sender, random_hash, new_mogwai, None)?;

			Ok(())
		}

		/// Remove an old mogwai.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn remove_mogwai(origin, mogwai_id: T::Hash) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			ensure!(sender == Self::key(), "only the dot mog founder can remove mogwais, without sacrificing them.");

			let owner = Self::owner_of(mogwai_id).ok_or("No owner for this mogwai")?;

			ensure!(owner == sender, "You don't own this mogwai");

			Self::remove(sender, mogwai_id)?;

			Ok(())
		}

		/// Transfer mogwai to a new account.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn transfer(origin, to: T::AccountId, mogwai_id: T::Hash) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			ensure!(sender == Self::key(), "only the dot mog founder can transfer mogwais, without paying for them.");

			let owner = Self::owner_of(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(owner == sender, Error::<T>::MogwaiNotOwned);

			// ensure that we have enough space
			ensure!(Self::ensure_not_max_mogwais(to.clone()), Error::<T>::MaxMogwaisInAccount);

			Self::transfer_from(sender, to, mogwai_id)?;

			Ok(())
		}

		/// Sacrifice mogwai to an other mogwai.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn sacrifice(origin, mogwai_id_1: T::Hash) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			let owner = Self::owner_of(mogwai_id_1).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(owner == sender, Error::<T>::MogwaiNotOwned);

			// make sure that there is no pending game event on the mogwai before sacrificing it.
			if GameEventsOfMogwai::<T>::contains_key(&mogwai_id_1) {
				let open_game_events = Self::game_events_of_mogwai(&mogwai_id_1);
				ensure!(open_game_events.is_empty(), Error::<T>::MogwaiHasGameEvents);
			}

			// TODO this needs to be check, reworked and corrected, add dynasty feature !!!
			let mogwai_1 = Self::mogwai(mogwai_id_1);
			if mogwai_1.gen == 0 {
				Self::pay_fee(sender.clone(), Pricing::fee_price(FeeType::Remove).saturated_into())?;
				Self::remove(sender, mogwai_id_1)?;
			} else {
				ensure!(MogwaisBios::<T>::contains_key(mogwai_id_1), Error::<T>::MogwaiHasNoBios);
				let mogwai_bios_1 = Self::mogwai_bios(mogwai_id_1);
				let intrinsic = mogwai_bios_1.intrinsic / Pricing::intrinsic_return(mogwai_bios_1.phases.len()).saturated_into();
				Self::remove(sender.clone(), mogwai_id_1)?;
				let _ = T::Currency::deposit_into_existing(&sender, intrinsic)?;
			}

			Ok(())
		}

		/// Sacrifice mogwai to an other mogwai.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn sacrifice_into(origin, mogwai_id_1: T::Hash, mogwai_id_2: T::Hash) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			let owner1 = Self::owner_of(mogwai_id_1).ok_or(Error::<T>::MogwaiDoesntExists)?;
			let owner2 = Self::owner_of(mogwai_id_2).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(owner1 == owner2, Error::<T>::MogwaiNotOwned);
			ensure!(owner1 == sender, Error::<T>::MogwaiNotOwned);

			// asacrificing into the same mogwai isn't allowed
			ensure!(mogwai_id_1 != mogwai_id_2, Error::<T>::MogwaiSame);

			// make sure that there is no pending game event on the mogwai before sacrificing it.
			if GameEventsOfMogwai::<T>::contains_key(&mogwai_id_1) {
				let open_game_events = Self::game_events_of_mogwai(&mogwai_id_1);
				ensure!(open_game_events.is_empty(), Error::<T>::MogwaiHasGameEvents);
			}

			if GameEventsOfMogwai::<T>::contains_key(&mogwai_id_2) {
				let open_game_events = Self::game_events_of_mogwai(&mogwai_id_2);
				ensure!(open_game_events.is_empty(), Error::<T>::MogwaiHasGameEvents);
			}

			// TODO this needs to be check, reworked and corrected, add dynasty feature !!!
			let mogwai_1 = Self::mogwai(mogwai_id_1);
			let mut mogwai_2 = Self::mogwai(mogwai_id_2);

			ensure!((mogwai_1.rarity as u8 * mogwai_2.rarity as u8) > 0, "Sacrifice into is only available for normal and higher rarity!");

			ensure!(MogwaisBios::<T>::contains_key(mogwai_id_1), Error::<T>::MogwaiHasNoBios);
			ensure!(MogwaisBios::<T>::contains_key(mogwai_id_2), Error::<T>::MogwaiHasNoBios);

			let mogwai_bios_1 = Self::mogwai_bios(mogwai_id_1);
			let mut mogwai_bios_2 = Self::mogwai_bios(mogwai_id_2);

			let gen_jump = Breeding::sacrifice(mogwai_1.gen, mogwai_1.rarity as u32, mogwai_bios_1.metaxy.clone(), mogwai_2.gen, mogwai_2.rarity as u32, mogwai_bios_2.metaxy.clone());
			if gen_jump > 0 && (mogwai_2.gen + gen_jump) <= 16 {
				mogwai_2.gen += gen_jump;
				<Mogwais<T>>::insert(mogwai_id_2, mogwai_2);
			}

			if mogwai_bios_1.intrinsic > Zero::zero() {
				mogwai_bios_2.intrinsic += mogwai_bios_1.intrinsic; // TODO check overflow
				<MogwaisBios<T>>::insert(mogwai_id_2, mogwai_bios_2);
			}

			Self::remove(sender.clone(), mogwai_id_1)?;

			Ok(())
		}

		/// Buy a mogwai.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn buy_mogwai(origin, mogwai_id: T::Hash, max_price: BalanceOf<T>) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			ensure!(Mogwais::<T>::contains_key(mogwai_id), Error::<T>::MogwaiDoesntExists);

			let owner = Self::owner_of(mogwai_id).ok_or(Error::<T>::MogwaiDoesntExists)?;
			ensure!(owner != sender, "You already own this mogwai");

			let mut mogwai = Self::mogwai(mogwai_id);

			let mogwai_price = mogwai.price;

			ensure!(!mogwai_price.is_zero(), "You can't buy this mogwai, there is no price");

			ensure!(mogwai_price <= max_price, "You can't buy this mogwai, price exceeds your max price limit");

			// ensure that we have enough space
			ensure!(Self::ensure_not_max_mogwais(sender.clone()), Error::<T>::MaxMogwaisInAccount);

			T::Currency::transfer(&sender, &owner, mogwai_price, ExistenceRequirement::KeepAlive)?;

			// Transfer the mogwai using `transfer_from()` including a proof of why it cannot fail
			Self::transfer_from(owner.clone(), sender.clone(), mogwai_id)
				.expect("`owner` is shown to own the mogwai; \
				`owner` must have greater than 0 mogwai, so transfer cannot cause underflow; \
				`all_mogwai_count` shares the same type as `owned_mogwai_count` \
				and minting ensure there won't ever be more than `max()` mogwais, \
				which means transfer cannot cause an overflow; \
				qed");

			// Reset mogwai price back to zero, and update the storage
			mogwai.price = Zero::zero();

			<Mogwais<T>>::insert(mogwai_id, mogwai);

			Self::deposit_event(RawEvent::Bought(sender, owner, mogwai_id, mogwai_price));

			Ok(())
		}

		/// Morph a gen 0 mogwai
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn morph_mogwai(origin, mogwai_id: T::Hash) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			ensure!(Mogwais::<T>::contains_key(mogwai_id), Error::<T>::MogwaiDoesntExists);

			let owner = Self::owner_of(mogwai_id).ok_or("No owner for this mogwai")?;
			ensure!(owner == sender, "You don't own this mogwai");

			let mut mogwai = Self::mogwai(mogwai_id);
			ensure!(mogwai.gen == 0, Error::<T>::MogwaiIncompatibleGeneration);

			let block_number = <frame_system::Pallet<T>>::block_number();

			let breed_type : BreedType = Self::calculate_breedtype(block_number);

			let mut dx: [u8;16] = Default::default();
			let mut dy: [u8;16] = Default::default();
			dx.copy_from_slice(&mogwai.dna.as_ref()[0..16]);
			dy.copy_from_slice(&mogwai.dna.as_ref()[16..32]);

			let final_dna : [u8;32] = Breeding::pairing(breed_type, dx, dy);

			// don't know a better way
			for i in 0..32 {
				mogwai.dna.as_mut()[i] = final_dna[i];
			}

			<Mogwais<T>>::insert(mogwai_id, mogwai);

			Ok(())
		}

		/// Breed a mogwai.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn breed_mogwai(origin, mogwai_id_1: T::Hash, mogwai_id_2: T::Hash) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			ensure!(Mogwais::<T>::contains_key(mogwai_id_1), Error::<T>::MogwaiDoesntExists);
			ensure!(Mogwais::<T>::contains_key(mogwai_id_2), Error::<T>::MogwaiDoesntExists);

			let owner = Self::owner_of(mogwai_id_1).ok_or("No owner for this mogwai")?;
			ensure!(owner == sender, "You don't own the first mogwai");

			// breeding into the same mogwai isn't allowed
			ensure!(mogwai_id_1 != mogwai_id_2, Error::<T>::MogwaiSame);

			// ensure that we have enough space
			ensure!(Self::ensure_not_max_mogwais(sender.clone()), Error::<T>::MaxMogwaisInAccount);

			let parents = [Self::mogwai(mogwai_id_1) , Self::mogwai(mogwai_id_2)];

			ensure!(parents[0].gen + parents[1].gen != 1, Error::<T>::MogwaiIncompatibleGeneration);
			ensure!(parents[0].gen == 0 ||  MogwaisBios::<T>::contains_key(mogwai_id_1), Error::<T>::MogwaiHasNoBios);
			ensure!(parents[1].gen == 0 ||  MogwaisBios::<T>::contains_key(mogwai_id_2), Error::<T>::MogwaiHasNoBios);

			let mogwai_id = Self::generate_random_hash(b"breed_mogwai", sender.clone());
			let event_id = Self::generate_random_hash(b"breed_event", sender.clone());

			let (rarity, next_gen) = Generation::next_gen(parents[0].gen, parents[0].rarity, parents[1].gen, parents[1].rarity, mogwai_id.as_ref());

			let block_number = <frame_system::Pallet<T>>::block_number();
			let breed_type : BreedType = Self::calculate_breedtype(block_number);

			let mut dx: [u8;16] = Default::default();
			let mut dy: [u8;16] = Default::default();
			if parents[0].gen + parents[1].gen == 0 {
				dx.copy_from_slice(&parents[0].dna.as_ref()[0..16]);
				dy.copy_from_slice(&parents[1].dna.as_ref()[16..32]);
			} else {
				let mogwai_bios_1 = Self::mogwai_bios(mogwai_id_1);
				let mogwai_bios_2 = Self::mogwai_bios(mogwai_id_2);
				dx = mogwai_bios_1.metaxy[0];
				dy = mogwai_bios_2.metaxy[0];

				// add pairing price to mogwai intrinsic value TODO
				let pairing_price:BalanceOf<T> = Pricing::pairing(parents[0].rarity, parents[1].rarity).saturated_into();
				Self::tip_mogwai(sender.clone(), pairing_price, mogwai_id_2, mogwai_bios_2)?;
			}

			let final_dna : [u8;32] = Breeding::pairing(breed_type, dx, dy);

			// don't know a better way, then using a clone.
			let mut final_dna_hash = mogwai_id.clone();
			for i in 0..32 {
				final_dna_hash.as_mut()[i] = final_dna[i];
			}

			// TODO: still no clue how to get the Hash type filled straight
			//let mut test_u8 : [u8;32] = [0;32];
			//let mut test_h256 = H256::from_slice(&test_u8);
			//let mut test_hash : T::Hash = test_h256.into();

			//let mut final_dna = parents[0].dna;
			//for (i, (dna_2_element, r)) in parents[1].dna.as_ref().iter().zip(random_hash.as_ref().iter()).enumerate() {
			//	if r % 2 == 0 {
			//		final_dna.as_mut()[i] = *dna_2_element;
			//	}
			//}

			//let block_hash = <frame_system::Pallet<T>>::block_hash(block_number);

			let mut mogwai_ids: Vec<T::Hash> = Vec::new();
			mogwai_ids.push(mogwai_id);

			let mogwai_struct = MogwaiStruct {
				id: mogwai_id,
				dna: final_dna_hash,
				genesis: block_number,
				price: Zero::zero(),
				gen: next_gen,
				rarity: rarity,
			};

			let game_event = GameEvent {
				id: event_id,
				begin: block_number + GameEventType::time_till(GameEventType::Hatch).into(),
				duration: GameEventType::duration(GameEventType::Hatch).into(),
				event_type: GameEventType::Hatch,
				hashes: mogwai_ids,
				value: 0,
			};

			// mint mogwai
			Self::mint(sender, mogwai_id, mogwai_struct, Some(game_event))?;

			Ok(())
		}

		/// Create a new auction.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn create_auction(origin, mogwai_id: T::Hash, min_bid: BalanceOf<T>, expiry: T::BlockNumber) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			ensure!(Mogwais::<T>::contains_key(mogwai_id), Error::<T>::MogwaiDoesntExists);

			let owner = Self::owner_of(mogwai_id).ok_or("No owner for this mogwai")?;
			ensure!(owner == sender, "You can't set an auction for a mogwai you don't own");

			ensure!(expiry > <frame_system::Pallet<T>>::block_number(), "The expiry has to be greater than the current block number");
			ensure!(expiry <= <frame_system::Pallet<T>>::block_number() + Self::auction_period_limit(), "The expiry has be lower than the limit block number");

			let auctions = Self::auctions_expire_at(expiry);
			ensure!(auctions.len() < MAX_AUCTIONS_PER_BLOCK, "Maximum number of auctions is reached for the target block, try another block");

			let new_auction = Auction {
				mogwai_id,
				mogwai_owner: owner,
				expiry,
				min_bid,
				high_bid: min_bid,
				high_bidder: sender,
			};

			<MogwaiAuction<T>>::insert(mogwai_id, &new_auction);
			<Auctions<T>>::mutate(expiry, |auctions| auctions.push(new_auction.clone()));

			Self::deposit_event(RawEvent::AuctionCreated(mogwai_id, min_bid, expiry));

			Ok (())
		}

		/// Bid on an auction.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn bid_auction(origin, mogwai_id: T::Hash, bid: BalanceOf<T>) -> dispatch::DispatchResult {

			let sender = ensure_signed(origin)?;

			ensure!(Mogwais::<T>::contains_key(mogwai_id), Error::<T>::MogwaiDoesntExists);

			let owner = Self::owner_of(mogwai_id).ok_or("No owner for this mogwai")?;
			ensure!(owner != sender, "You can't bid for your own mogwai");

			let mut auction = Self::auction_of(mogwai_id).ok_or("No auction for this mogwai")?;

			ensure!(<frame_system::Pallet<T>>::block_number() < auction.expiry, "This auction is expired.");

			ensure!(bid > auction.high_bid, "Your bid has to be greater than the highest bid.");

			ensure!(T::Currency::free_balance(&sender) >= bid, "You don't have enough free balance for this bid");

			auction.high_bid = bid;
			auction.high_bidder = sender.clone();

			<MogwaiAuction<T>>::insert(mogwai_id, &auction);
			<Auctions<T>>::mutate(auction.expiry, |auctions| {
				for stored_auction in auctions {
					if stored_auction.mogwai_id == mogwai_id {
						*stored_auction = auction.clone();
					}
				}
			});

			if <Bids<T>>::contains_key((mogwai_id, sender.clone())) {
				let escrow_balance = Self::bid_of((mogwai_id, sender.clone()));
				T::Currency::reserve(&sender, bid - escrow_balance)?;
			} else {
				T::Currency::reserve(&sender, bid)?;
			}
			<Bids<T>>::insert((mogwai_id, sender.clone()), bid);
			<BidAccounts<T>>::mutate(mogwai_id, |accounts| accounts.push(sender.clone()));

			Self::deposit_event(RawEvent::Bid(mogwai_id, auction.high_bid, auction.high_bidder));

			Ok (())
		}

		/// On finalize
		fn on_finalize() {

			let block_number = <frame_system::Pallet<T>>::block_number();

			let auctions = Self::auctions_expire_at(block_number);
			Self::finalize_auctions(auctions);

			let game_events = Self::game_events_at_block(block_number);
			Self::finalize_events(block_number, game_events);
		}
	}
}

impl<T: Config> Module<T> {
	/// Create technical accounts, currently not needed
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn account_id(mogwai_id: T::Hash) -> T::AccountId {
		T::PalletId::get().into_sub_account(mogwai_id)
	}

	/// Reads the nonce from storage, increments the stored nonce, and returns
	/// the encoded nonce to the caller.
	fn encode_and_update_nonce() -> Vec<u8> {
		let nonce = Nonce::get();
		Nonce::put(nonce.wrapping_add(1));
		nonce.encode()
	}

	/// pay fee
	fn pay_fee(who: T::AccountId, amount: BalanceOf<T>) -> dispatch::DispatchResult {
		let _ = T::Currency::withdraw(
			&who,
			amount,
			WithdrawReasons::FEE,
			ExistenceRequirement::KeepAlive,
		)?;

		Ok(())
	}

	/// pay founder
	fn pay_founder(who: T::AccountId, amount: BalanceOf<T>) -> dispatch::DispatchResult {
		let founder: T::AccountId = Self::key();
		let _ = T::Currency::transfer(&who, &founder, amount, ExistenceRequirement::KeepAlive)?;

		Ok(())
	}

	/// tiping mogwai
	fn tip_mogwai(
		who: T::AccountId,
		amount: BalanceOf<T>,
		mogwai_id: T::Hash,
		mut mogwai_bios: MogwaiBios<T::Hash, T::BlockNumber, BalanceOf<T>>,
	) -> dispatch::DispatchResult {
		Self::pay_fee(who, amount)?;

		mogwai_bios.intrinsic += amount; // TODO check overflow
		<MogwaisBios<T>>::insert(mogwai_id, mogwai_bios);

		Ok(())
	}

	fn config_value(who: T::AccountId, index: u8) -> u32 {
		let config_opt = <AccountConfig<T>>::get(&who);
		let result: u32;
		if config_opt.is_some() {
			let value = config_opt.unwrap()[usize::from(index)];
			result = GameConfig::config_value(index, value);
		} else {
			result = GameConfig::config_value(index, 0);
		}
		result
	}

	fn ensure_not_max_mogwais(who: T::AccountId) -> bool {
		Self::owned_mogwais_count(&who) < Self::config_value(who.clone(), 1) as u64
	}

	fn mint(
		to: T::AccountId,
		mogwai_id: T::Hash,
		new_mogwai: MogwaiStruct<T::Hash, T::BlockNumber, BalanceOf<T>, RarityType>,
		game_event_opt: Option<GameEvent<T::Hash, T::BlockNumber, GameEventType>>,
	) -> dispatch::DispatchResult {
		ensure!(!MogwaiOwner::<T>::contains_key(&mogwai_id), Error::<T>::MogwaiAlreadyExists);

		let owned_mogwais_count = Self::owned_mogwais_count(&to);
		let new_owned_mogwais_count = owned_mogwais_count
			.checked_add(1)
			.ok_or("Overflow adding a new mogwai to account balance")?;

		let all_mogwais_count = Self::all_mogwais_count();
		let new_all_mogwais_count = all_mogwais_count
			.checked_add(1)
			.ok_or("Overflow adding a new mogwai to total supply")?;

		// if there is an event, ensure it is successfull
		if game_event_opt.is_some() {
			Self::create_event(to.clone(), game_event_opt.unwrap())?;
		}

		// Update maps.
		<Mogwais<T>>::insert(mogwai_id, new_mogwai);
		<MogwaiOwner<T>>::insert(mogwai_id, &to);

		<AllMogwaisArray<T>>::insert(all_mogwais_count, mogwai_id);
		AllMogwaisCount::put(new_all_mogwais_count);
		<AllMogwaisIndex<T>>::insert(mogwai_id, all_mogwais_count);

		<OwnedMogwaisArray<T>>::insert((to.clone(), owned_mogwais_count), mogwai_id);
		<OwnedMogwaisCount<T>>::insert(&to, new_owned_mogwais_count);
		<OwnedMogwaisIndex<T>>::insert(mogwai_id, owned_mogwais_count);

		// Emit an event.
		Self::deposit_event(RawEvent::MogwaiCreated(to, mogwai_id));

		Ok(())
	}

	fn create_event(
		to: T::AccountId,
		new_game_event: GameEvent<T::Hash, T::BlockNumber, GameEventType>,
	) -> dispatch::DispatchResult {
		let event_id = new_game_event.id.clone();
		let event_type = new_game_event.event_type.clone();

		ensure!(!GameEvents::<T>::contains_key(&event_id), Error::<T>::GameEventAlreadyExists);

		let event_type_events_count = Self::game_event_of_type_count(&event_type);
		let new_event_type_events_count = event_type_events_count
			.checked_add(1)
			.ok_or("Overflow adding a new event to the event type events map")?;

		let all_events_count = Self::all_game_events_count();
		let new_all_events_count = all_events_count
			.checked_add(1)
			.ok_or("Overflow adding a new event to all events map")?;

		let game_events = Self::game_events_at_block(&new_game_event.begin);
		ensure!(
			game_events.len() < MAX_EVENTS_PER_BLOCK,
			"Maximum number of events is reached for target block, operation blocked."
		);

		// updated event maps.
		<GameEventsAtBlock<T>>::mutate(new_game_event.begin, |game_events| {
			game_events.push(event_id.clone())
		});

		// add the game event for all affected mogwais
		for hash in &new_game_event.hashes {
			<GameEventsOfMogwai<T>>::mutate(hash, |mogwai_game_events| {
				mogwai_game_events.push(event_id.clone())
			});
		}

		<GameEvents<T>>::insert(event_id, new_game_event);

		<AllGameEventsArray<T>>::insert(all_events_count, event_id);
		AllGameEventsCount::put(new_all_events_count);
		<AllGameEventsIndex<T>>::insert(event_id, all_events_count);

		<GameEventsArray<T>>::insert((event_type.clone(), event_type_events_count), event_id);
		GameEventsCount::insert(&event_type, new_event_type_events_count);
		<GameEventsIndex<T>>::insert(event_id, event_type_events_count);

		// Emit an event.
		Self::deposit_event(RawEvent::GameEventCreated(to, event_id));

		Ok(())
	}

	fn remove(from: T::AccountId, mogwai_id: T::Hash) -> dispatch::DispatchResult {
		ensure!(MogwaiOwner::<T>::contains_key(&mogwai_id), Error::<T>::MogwaiDoesntExists);

		// make sure that there is no pending game event on the mogwai before removing it.
		if GameEventsOfMogwai::<T>::contains_key(&mogwai_id) {
			let open_game_events = Self::game_events_of_mogwai(&mogwai_id);
			ensure!(open_game_events.is_empty(), Error::<T>::MogwaiHasGameEvents);
		}

		let owned_mogwais_count = Self::owned_mogwais_count(&from);
		let new_owned_mogwai_count = owned_mogwais_count
			.checked_sub(1)
			.ok_or("Overflow removing an old mogwai from account balance")?;

		let all_mogwais_count = Self::all_mogwais_count();
		let new_all_mogwais_count = all_mogwais_count
			.checked_sub(1)
			.ok_or("Overflow removing an old mogwai to total supply")?;

		// Update maps.
		<Mogwais<T>>::remove(mogwai_id);
		<MogwaisBios<T>>::remove(mogwai_id);
		<MogwaiOwner<T>>::remove(mogwai_id);

		<GameEventsOfMogwai<T>>::remove(mogwai_id);

		let all_mogwai_index = <AllMogwaisIndex<T>>::get(mogwai_id);
		if all_mogwai_index != new_all_mogwais_count {
			let all_last_mogwai_id = <AllMogwaisArray<T>>::get(new_all_mogwais_count);
			<AllMogwaisArray<T>>::insert(all_mogwai_index, all_last_mogwai_id);
			<AllMogwaisIndex<T>>::insert(all_last_mogwai_id, all_mogwai_index);
		}

		<AllMogwaisArray<T>>::remove(new_all_mogwais_count);
		AllMogwaisCount::put(new_all_mogwais_count);
		<AllMogwaisIndex<T>>::remove(mogwai_id);

		let mogwai_index = <OwnedMogwaisIndex<T>>::get(mogwai_id);
		if mogwai_index != new_owned_mogwai_count {
			let last_mogwai_id =
				<OwnedMogwaisArray<T>>::get((from.clone(), new_owned_mogwai_count));
			<OwnedMogwaisArray<T>>::insert((from.clone(), mogwai_index), last_mogwai_id);
			<OwnedMogwaisIndex<T>>::insert(last_mogwai_id, mogwai_index);
		}

		<OwnedMogwaisArray<T>>::remove((from.clone(), new_owned_mogwai_count));
		<OwnedMogwaisCount<T>>::insert(&from, new_owned_mogwai_count);
		<OwnedMogwaisIndex<T>>::remove(mogwai_id);

		// Emit an event.
		Self::deposit_event(RawEvent::MogwaiRemoved(from, mogwai_id));

		Ok(())
	}

	fn transfer_from(
		from: T::AccountId,
		to: T::AccountId,
		mogwai_id: T::Hash,
	) -> dispatch::DispatchResult {
		let owner = Self::owner_of(mogwai_id).ok_or("No owner for this mogwai")?;

		ensure!(owner == from, "You don't own this mogwai");

		ensure!(!<MogwaiAuction<T>>::contains_key(mogwai_id), "This mogwai has an open auction.");

		let owned_mogwai_count_from = Self::owned_mogwais_count(&from);
		let owned_mogwai_count_to = Self::owned_mogwais_count(&to);

		let new_owned_mogwai_count_from = owned_mogwai_count_from
			.checked_sub(1)
			.ok_or("Overflow removing a mogwai from account")?;
		let new_owned_mogwai_count_to = owned_mogwai_count_to
			.checked_add(1)
			.ok_or("Overflow adding a mogwai to account")?;

		// NOTE: This is the "swap and pop" algorithm we have added for you
		//       We use our storage items to help simplify the removal of elements from the OwnedMogwaisArray
		//       We switch the last element of OwnedMogwaisArray with the element we want to remove
		let mogwai_index = <OwnedMogwaisIndex<T>>::get(mogwai_id);
		if mogwai_index != new_owned_mogwai_count_from {
			let last_mogwai_id =
				<OwnedMogwaisArray<T>>::get((from.clone(), new_owned_mogwai_count_from));
			<OwnedMogwaisArray<T>>::insert((from.clone(), mogwai_index), last_mogwai_id);
			<OwnedMogwaisIndex<T>>::insert(last_mogwai_id, mogwai_index);
		}

		// Now we can remove this item by removing the last element
		<MogwaiOwner<T>>::insert(mogwai_id, &to);
		<OwnedMogwaisIndex<T>>::insert(mogwai_id, owned_mogwai_count_to);

		<OwnedMogwaisArray<T>>::remove((from.clone(), new_owned_mogwai_count_from));
		<OwnedMogwaisArray<T>>::insert((to.clone(), owned_mogwai_count_to), mogwai_id);

		// Update the OwnedMogwaisCount for `from` and `to`
		<OwnedMogwaisCount<T>>::insert(&from, new_owned_mogwai_count_from);
		<OwnedMogwaisCount<T>>::insert(&to, new_owned_mogwai_count_to);

		// Emit an event.
		Self::deposit_event(RawEvent::Transferred(from, to, mogwai_id));

		Ok(())
	}

	///
	fn generate_random_hash(phrase: &[u8], sender: T::AccountId) -> T::Hash {
		// we'll need a random seed here.
		// TODO: deal with randomness freshness
		// https://github.com/paritytech/substrate/issues/8312
		let (seed, _) = T::Randomness::random(phrase);
		let seed = <[u8; 32]>::decode(&mut TrailingZeroInput::new(seed.as_ref()))
			.expect("input is padded with zeroes; qed");
		//let mut rng = ChaChaRng::from_seed(seed);
		return (seed, &sender, Self::encode_and_update_nonce()).using_encoded(T::Hashing::hash)
	}

	///
	fn calculate_breedtype(block_number: T::BlockNumber) -> BreedType {
		// old breed type calculations changed on each block
		//let breed_type = BreedType::from_u32((block_number % 4.into()).saturated_into::<u32>());
		//return breed_type;

		let mod_value: u32 = 80;
		let modulo80 = (block_number % mod_value.into()).saturated_into::<u32>();
		if modulo80 < 20 {
			return BreedType::DomDom
		} else if modulo80 < 40 {
			return BreedType::DomRez
		} else if modulo80 < 60 {
			return BreedType::RezDom
		} else {
			return BreedType::RezRez
		}
	}

	fn finalize_auctions(
		auctions: Vec<Auction<T::Hash, BalanceOf<T>, T::BlockNumber, T::AccountId>>,
	) -> () {
		for auction in &auctions {
			let owned_mogwais_count_from = Self::owned_mogwais_count(&auction.mogwai_owner);
			let owned_mogwais_count_to = Self::owned_mogwais_count(&auction.high_bidder);

			if owned_mogwais_count_to.checked_add(1).is_some() &&
				owned_mogwais_count_from.checked_sub(1).is_some() &&
				auction.mogwai_owner != auction.high_bidder
			{
				<MogwaiAuction<T>>::remove(auction.mogwai_id);
				let _ = T::Currency::unreserve(&auction.high_bidder, auction.high_bid);
				let _currency_transfer = T::Currency::transfer(
					&auction.high_bidder,
					&auction.mogwai_owner,
					auction.high_bid,
					ExistenceRequirement::AllowDeath,
				);
				match _currency_transfer {
					Err(_e) => continue,
					Ok(_v) => {
						let _mogwai_transfer = Self::transfer_from(
							auction.mogwai_owner.clone(),
							auction.high_bidder.clone(),
							auction.mogwai_id,
						);
						match _mogwai_transfer {
							Err(_e) => continue,
							Ok(_v) => {
								Self::deposit_event(RawEvent::AuctionFinalized(
									auction.mogwai_id,
									auction.high_bid,
									auction.expiry,
								));
							},
						}
					},
				}
			}
		}

		for auction in &auctions {
			<Auctions<T>>::remove(<frame_system::Pallet<T>>::block_number());
			let bid_accounts = Self::bid_accounts(auction.mogwai_id);
			for account in bid_accounts {
				let bid_balance = Self::bid_of((auction.mogwai_id, account.clone()));
				let _ = T::Currency::unreserve(&account, bid_balance);
				<Bids<T>>::remove((auction.mogwai_id, account));
			}
			<BidAccounts<T>>::remove(auction.mogwai_id);
		}
	}

	fn finalize_events(block_number: T::BlockNumber, game_event_hashes: Vec<T::Hash>) -> () {
		// removing all events on this block
		<GameEventsAtBlock<T>>::remove(block_number);

		for game_event_hash in &game_event_hashes {
			let game_event = Self::game_events(game_event_hash);

			// clean-up game events
			<GameEvents<T>>::remove(&game_event.id);

			// remove the game event for all affected mogwais, removing mogwais with pending game events is forbidden
			// and should be cleared in any function that removes them, like sacrifice and remove.
			// TODO remove empty entries to avoid storage getting to big.
			for hash in &game_event.hashes {
				<GameEventsOfMogwai<T>>::mutate(&hash, |mogwai_game_events| {
					mogwai_game_events.retain(|&x| x != game_event.id)
				});
				let open_game_events = Self::game_events_of_mogwai(&hash);
				if open_game_events.is_empty() {
					<GameEventsOfMogwai<T>>::remove(hash);
				}
			}

			let all_events_count = Self::all_game_events_count();
			let all_count_sub_opt = all_events_count.checked_sub(1);
			if all_count_sub_opt.is_some() {
				let new_all_events_count = all_count_sub_opt.unwrap();
				let all_events_index = <AllGameEventsIndex<T>>::get(&game_event.id);
				if all_events_index != new_all_events_count {
					let all_last_event = <AllGameEventsArray<T>>::get(new_all_events_count);
					<AllGameEventsArray<T>>::insert(all_events_index, all_last_event);
					<AllGameEventsIndex<T>>::insert(all_last_event, all_events_index);
				}
				<AllGameEventsArray<T>>::remove(new_all_events_count);
				AllGameEventsCount::put(new_all_events_count);
				<AllGameEventsIndex<T>>::remove(&game_event.id);
			}

			let event_type_events_count = Self::game_event_of_type_count(&game_event.event_type);
			let event_count_sub_opt = event_type_events_count.checked_sub(1);
			if event_count_sub_opt.is_some() {
				let new_event_type_events_count = event_count_sub_opt.unwrap();
				let event_index = <GameEventsIndex<T>>::get(&game_event.id);
				if event_index != new_event_type_events_count {
					let last_event_id = <GameEventsArray<T>>::get((
						game_event.event_type.clone(),
						new_event_type_events_count,
					));
					<GameEventsArray<T>>::insert(
						(game_event.event_type.clone(), event_index),
						last_event_id,
					);
					<GameEventsIndex<T>>::insert(last_event_id, event_index);
				}
				<GameEventsArray<T>>::remove((
					game_event.event_type.clone(),
					new_event_type_events_count,
				));
				GameEventsCount::insert(&game_event.event_type, new_event_type_events_count);
				<GameEventsIndex<T>>::remove(&game_event.id);
			}

			// finally execute the event at the end of the clean up
			match game_event.event_type {
				GameEventType::Hatch => Self::execute_event_hatch(game_event.clone()),
				GameEventType::Default => {},
			};

			Self::deposit_event(RawEvent::GameEventExecuted(game_event.id));
		}
	}

	/// TODO: check if it is more optimzed when multiple hatching events are gathered in one event, instead of each in one of it's own.
	fn execute_event_hatch(game_event: GameEvent<T::Hash, T::BlockNumber, GameEventType>) -> () {
		for mogwai_id in game_event.hashes.iter() {
			if !Mogwais::<T>::contains_key(mogwai_id) || MogwaisBios::<T>::contains_key(mogwai_id) {
				// if there is no mogwai or it has already a bios we skip this part, as something bad happend
				continue
			}

			let mogwai_struct = Self::mogwai(mogwai_id);
			let block_hash = <frame_system::Pallet<T>>::block_hash(mogwai_struct.genesis);

			let mogwai_bio = Self::segment(mogwai_struct, block_hash, game_event.begin);

			<MogwaisBios<T>>::insert(mogwai_id, mogwai_bio);
		}
	}

	/// do the segmentation
	fn segment(
		mogwai_struct: MogwaiStruct<T::Hash, T::BlockNumber, BalanceOf<T>, RarityType>,
		block_hash: T::Hash,
		phase: T::BlockNumber,
	) -> MogwaiBios<T::Hash, T::BlockNumber, BalanceOf<T>> {
		let mut dna: [u8; 32] = Default::default();
		let mut blk: [u8; 32] = Default::default();

		dna.copy_from_slice(&mogwai_struct.dna.as_ref()[0..32]);
		blk.copy_from_slice(&block_hash.as_ref()[0..32]);

		// segmenting the hatched mogwai
		let (dna, evo) = Breeding::segmenting(dna, blk);

		let mut metaxy = Vec::new();
		metaxy.push(dna);
		metaxy.push(evo);

		let mut phases = Vec::new();
		phases.push(phase);

		MogwaiBios {
			mogwai_id: mogwai_struct.id.clone(),
			state: 0,
			metaxy,
			intrinsic: Zero::zero(),
			level: 1,
			phases,
			adaptations: Vec::new(),
		}
	}
}
