
//! Autogenerated weights for `pallet_bridge_grandpa`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-11-13, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `MusXroom`, CPU: `13th Gen Intel(R) Core(TM) i7-13650HX`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("bh-polkadot-local-raw.json")`, DB CACHE: 1024

// Executed Command:
// ../polkadot-sdk/target/release/polkadot-parachain-benchmarks
// benchmark
// pallet
// --chain
// bh-polkadot-local-raw.json
// --pallet
// pallet-bridge-grandpa
// --extrinsic
// *
// --output=system-parachains/bridge-hubs/bridge-hub-polkadot/src/weights
// --no-median-slopes
// --no-min-squares

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_bridge_grandpa`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bridge_grandpa::WeightInfo for WeightInfo<T> {
	/// Storage: `BridgeKusamaGrandpa::PalletOperatingMode` (r:1 w:0)
	/// Proof: `BridgeKusamaGrandpa::PalletOperatingMode` (`max_values`: Some(1), `max_size`: Some(1), added: 496, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaGrandpa::BestFinalized` (r:1 w:1)
	/// Proof: `BridgeKusamaGrandpa::BestFinalized` (`max_values`: Some(1), `max_size`: Some(36), added: 531, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaGrandpa::CurrentAuthoritySet` (r:1 w:0)
	/// Proof: `BridgeKusamaGrandpa::CurrentAuthoritySet` (`max_values`: Some(1), `max_size`: Some(50250), added: 50745, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaGrandpa::ImportedHashesPointer` (r:1 w:1)
	/// Proof: `BridgeKusamaGrandpa::ImportedHashesPointer` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaGrandpa::ImportedHashes` (r:1 w:1)
	/// Proof: `BridgeKusamaGrandpa::ImportedHashes` (`max_values`: Some(1200), `max_size`: Some(36), added: 1521, mode: `MaxEncodedLen`)
	/// Storage: `BridgeKusamaGrandpa::ImportedHeaders` (r:0 w:2)
	/// Proof: `BridgeKusamaGrandpa::ImportedHeaders` (`max_values`: Some(1200), `max_size`: Some(68), added: 1553, mode: `MaxEncodedLen`)
	/// The range of component `p` is `[1, 838]`.
	/// The range of component `v` is `[50, 100]`.
	fn submit_finality_proof(p: u32, v: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `302 + p * (60 ±0)`
		//  Estimated: `51735`
		// Minimum execution time: 258_945_000 picoseconds.
		Weight::from_parts(178_045_123, 0)
			.saturating_add(Weight::from_parts(0, 51735))
			// Standard Error: 160_419
			.saturating_add(Weight::from_parts(30_791_204, 0).saturating_mul(p.into()))
			// Standard Error: 2_675_623
			.saturating_add(Weight::from_parts(19_628_247, 0).saturating_mul(v.into()))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(5))
	}
}
