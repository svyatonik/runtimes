// Copyright (C) Parity Technologies and the various Polkadot contributors, see Contributions.md
// for a list of specific contributors.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//! Autogenerated weights for `runtime_parachains::initializer`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2024-03-10, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `ggwpez-ref-hw`, CPU: `Intel(R) Xeon(R) CPU @ 2.60GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("./kusama-chain-spec.json")`, DB CACHE: 1024

// Executed Command:
// ./target/production/polkadot
// benchmark
// pallet
// --chain=./kusama-chain-spec.json
// --steps=50
// --repeat=20
// --pallet=runtime_parachains::initializer
// --extrinsic=*
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./kusama-weights/
// --header=./file_header.txt

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `runtime_parachains::initializer`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> runtime_parachains::initializer::WeightInfo for WeightInfo<T> {
	/// Storage: `System::Digest` (r:1 w:1)
	/// Proof: `System::Digest` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `d` is `[0, 65536]`.
	fn force_approve(d: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0 + d * (11 ±0)`
		//  Estimated: `1480 + d * (11 ±0)`
		// Minimum execution time: 2_690_000 picoseconds.
		Weight::from_parts(2_760_000, 0)
			.saturating_add(Weight::from_parts(0, 1480))
			// Standard Error: 16
			.saturating_add(Weight::from_parts(2_212, 0).saturating_mul(d.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(Weight::from_parts(0, 11).saturating_mul(d.into()))
	}
}
