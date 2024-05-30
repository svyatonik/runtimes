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
use sp_std::vec::{self, Vec};
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
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::zero())]
		pub fn receive_ping(_origin: OriginFor<T>, id: BlockNumberFor<T>) -> DispatchResult {
			Pallet::<T>::deposit_event(Event::PingReceived { id });
			let send_result = Self::send_pong(id);
			log::trace!(
				target: "runtime::bridge-ping",
				"Sent message to Kusama Ping: {:?}",
				send_result,
			);

			if send_result.is_ok() {
				Pallet::<T>::deposit_event(Event::PongSent { id });
			}
			Ok(())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Ping has been received from the Ping chain.
		PingReceived {
			/// Unique ping ID.
			id: BlockNumberFor<T>,
		},
		/// Pong has been sent to the Ping chain.
		PongSent {
			/// Unique ping ID.
			id: BlockNumberFor<T>,
		},
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn send_pong(id: BlockNumberFor<T>) -> Result<(), SendError> {
			let receive_pong_prefix = hex_literal::hex!("6400");

			let mut encoded_receive_pong_call = Vec::new();
			encoded_receive_pong_call.extend_from_slice(&receive_pong_prefix);
			id.encode_to(&mut encoded_receive_pong_call);

			let receive_pong_call_weight = Weight::from_parts(20_000_000_000, 8000);

			let destination = xcm_config::bridging::to_kusama::PingKusama::get();
			let msg = sp_std::vec![Transact {
				origin_kind: OriginKind::Superuser,
				call: encoded_receive_pong_call.into(),
				require_weight_at_most: receive_pong_call_weight,
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
	fn message_to_ping_chain_is_sent() {
		sp_io::TestExternalities::new(Default::default()).execute_with(|| {
			PolkadotXcm::force_xcm_version(
				RuntimeOrigin::root(),
				Box::new(crate::xcm_config::bridging::SiblingBridgeHub::get()),
				XCM_VERSION,
			)
			.unwrap();
			PolkadotXcm::force_xcm_version(
				RuntimeOrigin::root(),
				Box::new(crate::xcm_config::bridging::to_kusama::PingKusama::get()),
				XCM_VERSION,
			)
			.unwrap();
			ParachainSystem::open_outbound_hrmp_channel_for_benchmarks_or_tests(1002.into());
			Pallet::<crate::Runtime>::send_pong(100).unwrap();
		});
	}

	#[test]
	fn receive_ping_encoding() {
		// 6400aaaaaaaa
		// 6400ffffffff
		let encoded_call_aa: crate::RuntimeCall =
			Call::<crate::Runtime>::receive_ping { id: 0xAAAAAAAA }.into();
		let encoded_call_ff: crate::RuntimeCall =
			Call::<crate::Runtime>::receive_ping { id: 0xFFFFFFFF }.into();
		println!("{}", hex::encode(encoded_call_aa.encode()));
		println!("{}", hex::encode(encoded_call_ff.encode()));
	}
}
