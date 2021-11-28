// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tests for the module.

use super::*;
use mock::*;

use frame_support::{assert_ok, assert_noop};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(DotMogModule::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(DotMogModule::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(
			DotMogModule::cause_error(Origin::signed(1)),
			Error::<Test>::NoneValue
		);
	});
}

#[test]
fn test_dotmog_breeding() {
	new_test_ext().execute_with(|| {
		assert_eq!(DotMogModule::all_mogwais_count(), 0);
		assert_ok!(DotMogModule::create_mogwai(Origin::signed(1)));
		assert_eq!(DotMogModule::all_mogwais_count(), 1);
		
		// test create
		assert_ok!(DotMogModule::create_mogwai(Origin::signed(1)));
		assert_eq!(DotMogModule::all_mogwais_count(), 2);
		let mogwai_hash_1 = DotMogModule::mogwai_by_index(0);
		let mogwai_hash_2 = DotMogModule::mogwai_by_index(1);
		let mogwai_1 = DotMogModule::mogwai(mogwai_hash_1);
		let mogwai_2 = DotMogModule::mogwai(mogwai_hash_2);

		// test hybrid mogwais of gen 0
		assert_eq!(mogwai_1.gen, 0);
		assert_eq!(mogwai_1.gen, mogwai_2.gen);

		// test morph
		assert_ok!(DotMogModule::morph_mogwai(Origin::signed(1), mogwai_hash_1));
		
		// test breed
		assert_eq!(DotMogModule::all_game_events_count(), 0);
		assert_ok!(DotMogModule::breed_mogwai(Origin::signed(1), mogwai_hash_1, mogwai_hash_2));
		assert_eq!(DotMogModule::all_game_events_count(), 1);
		assert_eq!(DotMogModule::all_mogwais_count(), 3);

		// create real mogwai by breeding
		let mogwai_hash_3 = DotMogModule::mogwai_by_index(2);
		let mogwai_3 = DotMogModule::mogwai(mogwai_hash_3);
		assert_eq!(mogwai_3.gen, 1);

		let mogwai_bios_3 = DotMogModule::mogwai_bios(mogwai_hash_3);
		assert_eq!(mogwai_bios_3.level, 0);

		// run forward 100 blocks to make the egg hatch
		assert_eq!(System::block_number(), 0);
		run_to_block(101);
		assert_eq!(System::block_number(), 101);

		// test if game event triggered
		assert_eq!(DotMogModule::all_game_events_count(), 0);

		// test if mogwai hatched
		let mogwai_bios_3 = DotMogModule::mogwai_bios(mogwai_hash_3);
		assert_eq!(mogwai_bios_3.level, 1);
	});
}