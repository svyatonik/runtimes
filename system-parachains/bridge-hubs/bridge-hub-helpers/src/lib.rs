// Copyright (C) Parity Technologies (UK) Ltd.
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

#![cfg_attr(not(feature = "std"), no_std)]

//! This crate defines several helper structs and traits to alleviate process
//! of adding bridge lanes between community parachains. It should be superseded
//! by the [bridges v2](https://github.com/paritytech/polkadot-sdk/pull/4427) in
//! the future.

use bp_messages::{source_chain::{MessagesBridge, OnMessagesDelivered}, LaneId, MessageNonce};
use bridge_runtime_common::messages_xcm_extension::{LocalXcmQueueManager, SenderAndLane, XcmBlobHauler};
use sp_runtime::traits::{Get, PhantomData};
use xcm::latest::prelude::*;
use xcm_executor::traits::ExportXcm;

pub trait XcmBlobHaulerItem {
	type Hauler;
	type SenderAndLane;

	fn try_on_message_enqueued(lane: LaneId, enqueued_messages: MessageNonce);
	fn try_on_messages_delivered(lane: LaneId, enqueued_messages: MessageNonce);
}

#[impl_trait_for_tuples::impl_for_tuples(1, 30)]
impl XcmBlobHaulerItem for Tuple {
	for_tuples!( type Hauler = ( #( Tuple::Hauler ),* ); );
	for_tuples!( type SenderAndLane = ( #( Tuple::SenderAndLane ),* ); );

	fn try_on_message_enqueued(lane: LaneId, enqueued_messages: MessageNonce) {
		for_tuples!( #(
			Tuple::try_on_message_enqueued(lane, enqueued_messages);
		)* );
	}

	fn try_on_messages_delivered(lane: LaneId, enqueued_messages: MessageNonce) {
		for_tuples!( #(
			Tuple::try_on_messages_delivered(lane, enqueued_messages);
		)* );
	}
}

pub struct XcmBlobHaulerItemAdapter<H, SL>(PhantomData<(H, SL)>);
impl<H: XcmBlobHauler, SL: Get<SenderAndLane>> XcmBlobHaulerItem for XcmBlobHaulerItemAdapter<H, SL> {
	type Hauler = H;
	type SenderAndLane = SL;

	fn try_on_message_enqueued(lane: LaneId, enqueued_messages: MessageNonce) {
		let sender_and_lane = Self::SenderAndLane::get();
		if lane == sender_and_lane.lane {
			// notify XCM queue manager about updated lane state
			LocalXcmQueueManager::<H>::on_bridge_message_enqueued(
				&sender_and_lane,
				enqueued_messages,
			);
		}
	}

	fn try_on_messages_delivered(lane: LaneId, enqueued_messages: MessageNonce) {
		let sender_and_lane = Self::SenderAndLane::get();
		if lane == sender_and_lane.lane {
			// notify XCM queue manager about updated lane state
			LocalXcmQueueManager::<H>::on_bridge_messages_delivered(
				&sender_and_lane,
				enqueued_messages,
			);
		}
	}
}

/// XCM bridge adapter which connects [`XcmBlobHauler`] with [`pallet_bridge_messages`] and
/// makes sure that XCM blob is sent to the outbound lane to be relayed.
///
/// It needs to be used at the source bridge hub.
pub struct XcmBlobHaulerAdapter<H>(PhantomData<H>);

impl<H: XcmBlobHaulerItem> OnMessagesDelivered for XcmBlobHaulerAdapter<H> {
	fn on_messages_delivered(lane: LaneId, enqueued_messages: MessageNonce) {
		H::try_on_messages_delivered(lane, enqueued_messages);
	}
}

type MessagesPallet<T, I> = pallet_bridge_messages::Pallet<T, <T as pallet_xcm_bridge_hub::Config<I>>::BridgeMessagesPalletInstance>;

pub struct OverBridgeXcmExporter<R, I, H>(PhantomData<(R, I, H)>);

impl<R, I, H> ExportXcm for OverBridgeXcmExporter<R, I, H>
where
	R: pallet_xcm_bridge_hub::Config<I>,
	I: 'static,
	H: XcmBlobHaulerItem,
	pallet_xcm_bridge_hub::Pallet<R, I>: ExportXcm<
		Ticket = (
			SenderAndLane,
			<MessagesPallet::<R, I> as MessagesBridge<R::OutboundPayload>>::SendMessageArgs,
			XcmHash,
		),
	>,
{
	type Ticket = <pallet_xcm_bridge_hub::Pallet<R, I> as ExportXcm>::Ticket;

	fn validate(
		network: NetworkId,
		channel: u32,
		universal_source: &mut Option<InteriorLocation>,
		destination: &mut Option<InteriorLocation>,
		message: &mut Option<Xcm<()>>,
	) -> Result<(Self::Ticket, Assets), SendError> {
		pallet_xcm_bridge_hub::Pallet::<R, I>::validate(network, channel, universal_source, destination, message)
	}

	fn deliver((sender_and_lane, bridge_message, id): Self::Ticket) -> Result<XcmHash, SendError> {
		let lane_id = sender_and_lane.lane;
		let artifacts = MessagesPallet::<R, I>::send_message(bridge_message);
/*
		log::info!(
			target: pallext_xcm_bridge_hub::LOG_TARGET,
			"XCM message {:?} has been enqueued at bridge {:?} with nonce {}",
			id,
			lane_id,
			artifacts.nonce,
		);
*/
		// notify XCM queue manager about updated lane state
		H::try_on_message_enqueued(lane_id, artifacts.enqueued_messages);

		Ok(id)
	}
}
