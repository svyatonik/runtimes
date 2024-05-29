// Copyright Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

use crate::xcm_config;
use sp_runtime::traits::Zero;
use xcm::latest::prelude::*;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			if !(n % 10u32.into()).is_zero() {
				return Weight::zero()
			}

			let send_result = Self::send_ping(n);
			log::trace!(
				target: "runtime::bridge-ping",
				"Sent message to Polkadot Pong: {:?}",
				send_result,
			);

			if send_result.is_ok() {
				Pallet::<T>::deposit_event(Event::PingSent { id: n });
			}

			// don't bother with weights, because we only use this pallet in test environment
			Weight::zero()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::zero())]
		pub fn receive_pong(_origin: OriginFor<T>, id: BlockNumberFor<T>) -> DispatchResult {
			Pallet::<T>::deposit_event(Event::PongReceived { id });
			Ok(())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Ping has been sent to the Pong chain.
		PingSent {
			/// Unique ping ID.
			id: BlockNumberFor<T>,
		},
		/// Pong has been received from the Pong chain.
		PongReceived {
			/// Unique ping ID.
			id: BlockNumberFor<T>,
		},
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn send_ping(id: BlockNumberFor<T>) -> Result<(), SendError> {
			let receive_ping_prefix = hex_literal::hex!("6400");

			let mut encoded_receive_ping_call = Vec::new();
			encoded_receive_ping_call.extend_from_slice(&receive_ping_prefix);
			id.encode_to(&mut encoded_receive_ping_call);

			let receive_ping_call_weight = Weight::from_parts(20_000_000_000, 8000);

			let destination = xcm_config::bridging::to_polkadot::PongPolkadot::get();
			let msg = sp_std::vec![Transact {
				origin_kind: OriginKind::Superuser,
				call: encoded_receive_ping_call.into(),
				require_weight_at_most: receive_ping_call_weight,
			}]
			.into();

			send_xcm::<xcm_config::XcmRouter>(destination, msg).map(drop)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{ParachainSystem, PolkadotXcm, RuntimeOrigin};
	use codec::Encode;

	#[test]
	fn message_to_pong_chain_is_sent() {
		sp_io::TestExternalities::new(Default::default()).execute_with(|| {
			PolkadotXcm::force_xcm_version(
				RuntimeOrigin::root(),
				Box::new(crate::xcm_config::bridging::SiblingBridgeHub::get()),
				XCM_VERSION,
			)
			.unwrap();
			PolkadotXcm::force_xcm_version(
				RuntimeOrigin::root(),
				Box::new(crate::xcm_config::bridging::to_polkadot::PongPolkadot::get()),
				XCM_VERSION,
			)
			.unwrap();
			ParachainSystem::open_outbound_hrmp_channel_for_benchmarks_or_tests(1002.into());
			Pallet::<crate::Runtime>::send_ping(100).unwrap();
		});
	}

	#[test]
	fn receive_pong_encoding() {
		// 6400aaaaaaaa
		// 6400ffffffff
		let encoded_call_aa: crate::RuntimeCall =
			Call::<crate::Runtime>::receive_pong { id: 0xAAAAAAAA }.into();
		let encoded_call_ff: crate::RuntimeCall =
			Call::<crate::Runtime>::receive_pong { id: 0xFFFFFFFF }.into();
		println!("{}", hex::encode(encoded_call_aa.encode()));
		println!("{}", hex::encode(encoded_call_ff.encode()));
	}
}
