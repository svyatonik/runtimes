
//! Autogenerated weights for `pallet_xcm_benchmarks::generic`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-11-13, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `MusXroom`, CPU: `13th Gen Intel(R) Core(TM) i7-13650HX`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("ah-polkadot-local-raw.json")`, DB CACHE: 1024

// Executed Command:
// ../polkadot-sdk/target/release/polkadot-parachain-benchmarks
// benchmark
// pallet
// --chain
// ah-polkadot-local-raw.json
// --pallet
// pallet-xcm-benchmarks::generic
// --extrinsic
// report_holding,buy_execution,query_response,transact,refund_surplus,set_error_handler,set_appendix,clear_error,descend_origin,clear_origin,report_error,claim_asset,trap,subscribe_version,unsubscribe_version,burn_asset,expect_asset,expect_origin,expect_error,expect_transact_status,query_pallet,report_transact_status,clear_transact_status,set_topic,clear_topic,export_message,set_fees_mode,unpaid_execution,universal_origin
// --output=system-parachains/asset-hubs/asset-hub-polkadot/src/weights/xcm
// --no-median-slopes
// --no-min-squares

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_xcm_benchmarks::generic`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo<T> {
	/// Storage: `ParachainInfo::ParachainId` (r:1 w:0)
	/// Proof: `ParachainInfo::ParachainId` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `ParachainSystem::UpwardDeliveryFeeFactor` (r:1 w:0)
	/// Proof: `ParachainSystem::UpwardDeliveryFeeFactor` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::SupportedVersion` (r:1 w:0)
	/// Proof: `PolkadotXcm::SupportedVersion` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::VersionDiscoveryQueue` (r:1 w:1)
	/// Proof: `PolkadotXcm::VersionDiscoveryQueue` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::SafeXcmVersion` (r:1 w:0)
	/// Proof: `PolkadotXcm::SafeXcmVersion` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:2 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `ParachainSystem::HostConfiguration` (r:1 w:0)
	/// Proof: `ParachainSystem::HostConfiguration` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ParachainSystem::PendingUpwardMessages` (r:1 w:1)
	/// Proof: `ParachainSystem::PendingUpwardMessages` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	pub fn  report_holding() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `176`
		//  Estimated: `6196`
		// Minimum execution time: 276_639_000 picoseconds.
		Weight::from_parts(278_663_000, 0)
			.saturating_add(Weight::from_parts(0, 6196))
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	pub fn  buy_execution() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 2_154_000 picoseconds.
		Weight::from_parts(2_329_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: `PolkadotXcm::Queries` (r:1 w:0)
	/// Proof: `PolkadotXcm::Queries` (`max_values`: None, `max_size`: None, mode: `Measured`)
	pub fn  query_response() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `69`
		//  Estimated: `3534`
		// Minimum execution time: 6_547_000 picoseconds.
		Weight::from_parts(6_816_000, 0)
			.saturating_add(Weight::from_parts(0, 3534))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	pub fn  transact() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 6_649_000 picoseconds.
		Weight::from_parts(6_866_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	pub fn  refund_surplus() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 2_249_000 picoseconds.
		Weight::from_parts(2_380_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	pub fn  set_error_handler() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_388_000 picoseconds.
		Weight::from_parts(1_486_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	pub fn  set_appendix() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_361_000 picoseconds.
		Weight::from_parts(1_475_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	pub fn  clear_error() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_357_000 picoseconds.
		Weight::from_parts(1_454_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	pub fn  descend_origin() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_872_000 picoseconds.
		Weight::from_parts(1_906_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	pub fn  clear_origin() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_360_000 picoseconds.
		Weight::from_parts(1_428_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: `ParachainInfo::ParachainId` (r:1 w:0)
	/// Proof: `ParachainInfo::ParachainId` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `ParachainSystem::UpwardDeliveryFeeFactor` (r:1 w:0)
	/// Proof: `ParachainSystem::UpwardDeliveryFeeFactor` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::SupportedVersion` (r:1 w:0)
	/// Proof: `PolkadotXcm::SupportedVersion` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::VersionDiscoveryQueue` (r:1 w:1)
	/// Proof: `PolkadotXcm::VersionDiscoveryQueue` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::SafeXcmVersion` (r:1 w:0)
	/// Proof: `PolkadotXcm::SafeXcmVersion` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:2 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `ParachainSystem::HostConfiguration` (r:1 w:0)
	/// Proof: `ParachainSystem::HostConfiguration` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ParachainSystem::PendingUpwardMessages` (r:1 w:1)
	/// Proof: `ParachainSystem::PendingUpwardMessages` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	pub fn  report_error() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `176`
		//  Estimated: `6196`
		// Minimum execution time: 32_133_000 picoseconds.
		Weight::from_parts(33_020_000, 0)
			.saturating_add(Weight::from_parts(0, 6196))
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `PolkadotXcm::AssetTraps` (r:1 w:1)
	/// Proof: `PolkadotXcm::AssetTraps` (`max_values`: None, `max_size`: None, mode: `Measured`)
	pub fn  claim_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `126`
		//  Estimated: `3591`
		// Minimum execution time: 9_862_000 picoseconds.
		Weight::from_parts(10_248_000, 0)
			.saturating_add(Weight::from_parts(0, 3591))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	pub fn  trap() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_322_000 picoseconds.
		Weight::from_parts(1_439_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: `PolkadotXcm::VersionNotifyTargets` (r:1 w:1)
	/// Proof: `PolkadotXcm::VersionNotifyTargets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `ParachainSystem::UpwardDeliveryFeeFactor` (r:1 w:0)
	/// Proof: `ParachainSystem::UpwardDeliveryFeeFactor` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::SupportedVersion` (r:1 w:0)
	/// Proof: `PolkadotXcm::SupportedVersion` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::VersionDiscoveryQueue` (r:1 w:1)
	/// Proof: `PolkadotXcm::VersionDiscoveryQueue` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::SafeXcmVersion` (r:1 w:0)
	/// Proof: `PolkadotXcm::SafeXcmVersion` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ParachainSystem::HostConfiguration` (r:1 w:0)
	/// Proof: `ParachainSystem::HostConfiguration` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ParachainSystem::PendingUpwardMessages` (r:1 w:1)
	/// Proof: `ParachainSystem::PendingUpwardMessages` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	pub fn  subscribe_version() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `75`
		//  Estimated: `3540`
		// Minimum execution time: 18_358_000 picoseconds.
		Weight::from_parts(19_108_000, 0)
			.saturating_add(Weight::from_parts(0, 3540))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `PolkadotXcm::VersionNotifyTargets` (r:0 w:1)
	/// Proof: `PolkadotXcm::VersionNotifyTargets` (`max_values`: None, `max_size`: None, mode: `Measured`)
	pub fn  unsubscribe_version() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 2_920_000 picoseconds.
		Weight::from_parts(3_088_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	pub fn  burn_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 77_156_000 picoseconds.
		Weight::from_parts(77_735_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	pub fn  expect_asset() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_621_000 picoseconds.
		Weight::from_parts(7_749_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	pub fn  expect_origin() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_411_000 picoseconds.
		Weight::from_parts(1_507_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	pub fn  expect_error() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_346_000 picoseconds.
		Weight::from_parts(1_433_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	pub fn  expect_transact_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_504_000 picoseconds.
		Weight::from_parts(1_578_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: `ParachainInfo::ParachainId` (r:1 w:0)
	/// Proof: `ParachainInfo::ParachainId` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `ParachainSystem::UpwardDeliveryFeeFactor` (r:1 w:0)
	/// Proof: `ParachainSystem::UpwardDeliveryFeeFactor` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::SupportedVersion` (r:1 w:0)
	/// Proof: `PolkadotXcm::SupportedVersion` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::VersionDiscoveryQueue` (r:1 w:1)
	/// Proof: `PolkadotXcm::VersionDiscoveryQueue` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::SafeXcmVersion` (r:1 w:0)
	/// Proof: `PolkadotXcm::SafeXcmVersion` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:2 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `ParachainSystem::HostConfiguration` (r:1 w:0)
	/// Proof: `ParachainSystem::HostConfiguration` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ParachainSystem::PendingUpwardMessages` (r:1 w:1)
	/// Proof: `ParachainSystem::PendingUpwardMessages` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	pub fn  query_pallet() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `176`
		//  Estimated: `6196`
		// Minimum execution time: 36_992_000 picoseconds.
		Weight::from_parts(38_309_000, 0)
			.saturating_add(Weight::from_parts(0, 6196))
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `ParachainInfo::ParachainId` (r:1 w:0)
	/// Proof: `ParachainInfo::ParachainId` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `ParachainSystem::UpwardDeliveryFeeFactor` (r:1 w:0)
	/// Proof: `ParachainSystem::UpwardDeliveryFeeFactor` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::SupportedVersion` (r:1 w:0)
	/// Proof: `PolkadotXcm::SupportedVersion` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::VersionDiscoveryQueue` (r:1 w:1)
	/// Proof: `PolkadotXcm::VersionDiscoveryQueue` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `PolkadotXcm::SafeXcmVersion` (r:1 w:0)
	/// Proof: `PolkadotXcm::SafeXcmVersion` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:2 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `ParachainSystem::HostConfiguration` (r:1 w:0)
	/// Proof: `ParachainSystem::HostConfiguration` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `ParachainSystem::PendingUpwardMessages` (r:1 w:1)
	/// Proof: `ParachainSystem::PendingUpwardMessages` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	pub fn  report_transact_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `176`
		//  Estimated: `6196`
		// Minimum execution time: 32_249_000 picoseconds.
		Weight::from_parts(33_390_000, 0)
			.saturating_add(Weight::from_parts(0, 6196))
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	pub fn  clear_transact_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_394_000 picoseconds.
		Weight::from_parts(1_482_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	pub fn  set_topic() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_345_000 picoseconds.
		Weight::from_parts(1_441_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	pub fn  clear_topic() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_359_000 picoseconds.
		Weight::from_parts(1_434_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: `ParachainInfo::ParachainId` (r:1 w:0)
	/// Proof: `ParachainInfo::ParachainId` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	pub fn  universal_origin() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `1489`
		// Minimum execution time: 3_040_000 picoseconds.
		Weight::from_parts(3_089_000, 0)
			.saturating_add(Weight::from_parts(0, 1489))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	pub fn  set_fees_mode() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_376_000 picoseconds.
		Weight::from_parts(1_441_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	pub fn  unpaid_execution() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_454_000 picoseconds.
		Weight::from_parts(1_622_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	// TODO: https://github.com/paritytech/polkadot-sdk/issues/2284
	// (right now that's a copypaste of previous value)
	pub fn  expect_pallet() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 5_638_000 picoseconds.
		Weight::from_parts(5_756_000, 0)
	}
}
