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

use bp_messages::{
	source_chain::{MessagesBridge, OnMessagesDelivered},
	target_chain::{DispatchMessage, MessageDispatch, ProvedMessages, SourceHeaderChain},
	LaneId, Message, MessageNonce, VerificationError,
};
use bp_runtime::{messages::MessageDispatchResult, Chain};
use bp_xcm_bridge_hub_router::XcmChannelStatusProvider as _;
use bridge_runtime_common::{
	messages::{target::FromBridgedChainMessagesProof, BridgedChain, HashOf, MessageBridge},
	messages_call_ext::{CallInfo, MessagesCallSubType},
	messages_xcm_extension::{
		LocalXcmQueueManager, SenderAndLane, XcmAsPlainPayload, XcmBlobHauler,
		XcmBlobMessageDispatchResult,
	},
	refund_relayer_extension::{RefundSignedExtension, RefundableMessagesLaneId},
};
use codec::{Decode, Encode};
use cumulus_primitives_core::ParaId;
use frame_support::{
	CloneNoBound, DefaultNoBound, EqNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
use pallet_bridge_parachains::RelayBlockNumber;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{DispatchInfoOf, Get, PhantomData, PostDispatchInfoOf, SignedExtension},
	transaction_validity::{TransactionValidity, TransactionValidityError},
	DispatchResult,
};
use xcm::latest::prelude::*;
use xcm_executor::traits::ExportXcm;

/// A `XcmBlobHauler` trait that may be 'tuplified' to support multiple lanes.
pub trait XcmBlobHaulerItem {
	/// Reference to the original `XcmBlobHauler` implementation.
	type Hauler;
	/// A linked route for this [`Self::XcmBlobHauler`].
	type SenderAndLane;

	/// Check if `lane` matches the route and call `on_message_enqueued` to maybe
	/// trigger the congestion mechanism.
	fn try_on_message_enqueued(lane: LaneId, enqueued_messages: MessageNonce);
	/// Check if `lane` matches the route and call `on_messages_delivered` to maybe
	/// trigger the uncongestion mechanism.
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

/// A simple adapter around `XcmBlobHauler` and `SenderAndLane` that implements
/// the `XcmBlobHaulerItem` trait.
pub struct XcmBlobHaulerItemAdapter<H, SL>(PhantomData<(H, SL)>);
impl<H: XcmBlobHauler, SL: Get<SenderAndLane>> XcmBlobHaulerItem
	for XcmBlobHaulerItemAdapter<H, SL>
{
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

/// `OnMessagesDelivered` implementation with proper support for multiple lanes.
/// Our regular `XcmBlobHaulerAdapter` from `bridge-runtime-common` crate ignores
/// the fact that 'congested' and 'uncongested' messages may be different for
/// different sibling parachains, so we are using this trick to support multiple
/// lanes.
///
/// See `on_messages_delivered_from_polkadot_works_as_a_junction` test for more details.
pub struct XcmBlobHaulerAdapter<H>(PhantomData<H>);

impl<H: XcmBlobHaulerItem> OnMessagesDelivered for XcmBlobHaulerAdapter<H> {
	fn on_messages_delivered(lane: LaneId, enqueued_messages: MessageNonce) {
		H::try_on_messages_delivered(lane, enqueued_messages);
	}
}

/// `ExportXcm` implementation with proper support for multiple lanes.
/// Our regular implementation for the `pallet_xcm_bridge_hub::Pallet` ignores
/// the fact that 'congested' and 'uncongested' messages may be different for
/// different sibling parachains, so we are using this trick to support multiple
/// lanes.
///
/// See `to_bridge_hub_polkadot_haul_blob_exporter_works_as_a_junction` test for more details.
pub struct OverBridgeXcmExporter<R, I, H>(PhantomData<(R, I, H)>);

type MessagesPallet<T, I> = pallet_bridge_messages::Pallet<
	T,
	<T as pallet_xcm_bridge_hub::Config<I>>::BridgeMessagesPalletInstance,
>;

impl<R, I, H> ExportXcm for OverBridgeXcmExporter<R, I, H>
where
	R: pallet_xcm_bridge_hub::Config<I>,
	I: 'static,
	H: XcmBlobHaulerItem,
	pallet_xcm_bridge_hub::Pallet<R, I>: ExportXcm<
		Ticket = (
			SenderAndLane,
			<MessagesPallet<R, I> as MessagesBridge<R::OutboundPayload>>::SendMessageArgs,
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
		pallet_xcm_bridge_hub::Pallet::<R, I>::validate(
			network,
			channel,
			universal_source,
			destination,
			message,
		)
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

/// An analogue of `bp_xcm_bridge_hub_router::XcmChannelStatusProvider`, which turns
/// `is_congested` associated function into a method. We cannot use associated functions,
/// because we provide an array of `dyn XcmChannelStatusProvider`.
pub trait XcmChannelStatusProvider {
	/// Returns true if the channel is currently congested.
	fn is_congested(&self) -> bool;
}

/// Simple adapter that implements `XcmChannelStatusProvider` for
/// `cumulus_pallet_xcmp_queue::bridging::OutXcmpChannelStatusProvider`.
pub struct XcmChannelStatusProviderAdapter<P, R>(PhantomData<(P, R)>);

impl<P, R> XcmChannelStatusProviderAdapter<P, R> {
	pub fn new() -> Self {
		XcmChannelStatusProviderAdapter(PhantomData)
	}
}

impl<P: Get<ParaId>, R: cumulus_pallet_xcmp_queue::Config> XcmChannelStatusProvider
	for XcmChannelStatusProviderAdapter<P, R>
{
	fn is_congested(&self) -> bool {
		cumulus_pallet_xcmp_queue::bridging::OutXcmpChannelStatusProvider::<P, R>::is_congested()
	}
}

/// `SourceHeaderChain` implementation with proper support for multiple lanes.
/// Our regular implementation of the `MessageDispatch` for the `XcmBlobMessageDispatch`
/// ignores the fact that there may be several XCMP channels for different lanes and
/// simply returns selected channel state from `is_active`. This implementation
/// moves that check from `MessageDispatch::is_active` to
/// `SourceHeaderChain::verify_messages_proof`.
///
/// **NOTE**: it must be used with the adapted `XcmBlobMessageDispatch` version and
/// the `RefundSignedExtensionAdapter`signed extension.
pub struct SourceHeaderChainAdapter<R, I, B, AR>(PhantomData<(R, I, B, AR)>);

impl<R, I, B, AR> SourceHeaderChain for SourceHeaderChainAdapter<R, I, B, AR>
where
	R: cumulus_pallet_xcmp_queue::Config + pallet_xcm_bridge_hub::Config<I>,
	I: 'static,
	B: MessageBridge,
	AR: Get<sp_std::vec::Vec<(LaneId, Box<dyn XcmChannelStatusProvider>)>>,
{
	type MessagesProof = FromBridgedChainMessagesProof<HashOf<BridgedChain<B>>>;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message>, VerificationError> {
		if !is_outbound_xcmpl_channel_active::<AR>(proof.lane) {
			return Err(VerificationError::Other(
				"XCMP lane with the target sibling chain is inactive",
			))
		}

		bridge_runtime_common::messages::target::SourceHeaderChainAdapter::<B>::verify_messages_proof(proof, messages_count)
	}
}

/// Our regular implementation of the `MessageDispatch` for the `XcmBlobMessageDispatch`
/// ignores the fact that there may be several XCMP channels for different lanes and
/// simply returns selected channel state from `is_active`. This implementation
/// does not check the channel state from `MessageDispatch::is_active`. Instead we
/// perform this check in `XcmBlobMessageDispatch` and `RefundSignedExtensionAdapter`.
///
/// **NOTE**: it must be used with the adapted `XcmBlobMessageDispatch` version and
/// the `RefundSignedExtensionAdapter` signed extension.
pub struct XcmBlobMessageDispatch<T>(PhantomData<T>);

impl<T> MessageDispatch for XcmBlobMessageDispatch<T>
where
	T: MessageDispatch<
		DispatchPayload = XcmAsPlainPayload,
		DispatchLevelResult = XcmBlobMessageDispatchResult,
	>,
{
	type DispatchPayload = XcmAsPlainPayload;
	type DispatchLevelResult = XcmBlobMessageDispatchResult;

	fn is_active() -> bool {
		// we assume the channel is always active here. We actually check the channel state
		// from the `SourceHeaderChainAdapter` and from the `RefundSignedExtensionAdapter`
		true
	}

	fn dispatch_weight(message: &mut DispatchMessage<Self::DispatchPayload>) -> Weight {
		T::dispatch_weight(message)
	}

	fn dispatch(
		message: DispatchMessage<Self::DispatchPayload>,
	) -> MessageDispatchResult<Self::DispatchLevelResult> {
		T::dispatch(message)
	}
}

/// A signed extension that wraps our regular
/// `bridge_runtime_common::refund_relayer_extension::RefundSignedExtensionAdapter` but
/// in addition checks that the outbound XCMP channel, associated with the lane during
/// message delivery transaction, is active.
///
/// **NOTE**: it must be used with the adapted `XcmBlobMessageDispatch` version and
/// the `SourceHeaderChainAdapter` signed extension.
#[derive(
	DefaultNoBound,
	CloneNoBound,
	Decode,
	Encode,
	EqNoBound,
	PartialEqNoBound,
	RuntimeDebugNoBound,
	TypeInfo,
)]
#[scale_info(skip_type_params(AR))]
pub struct RefundSignedExtensionAdapter<R: Default + Clone + PartialEq + sp_std::fmt::Debug, AR>(
	PhantomData<(R, AR)>,
);

impl<R, AR> SignedExtension for RefundSignedExtensionAdapter<R, AR>
where
	R: RefundSignedExtension,
	AR: Get<sp_std::vec::Vec<(LaneId, Box<dyn XcmChannelStatusProvider>)>> + Send + Sync + 'static,
	<R::Runtime as pallet_bridge_grandpa::Config<R::GrandpaInstance>>::BridgedChain:
		Chain<BlockNumber = RelayBlockNumber>,
	<R::Runtime as frame_system::Config>::RuntimeCall: MessagesCallSubType<
		R::Runtime,
		<<R as RefundSignedExtension>::Msgs as RefundableMessagesLaneId>::Instance,
	>,
	bridge_runtime_common::refund_relayer_extension::RefundSignedExtensionAdapter<R>:
		SignedExtension<Call = <R::Runtime as frame_system::Config>::RuntimeCall>,
{
	const IDENTIFIER: &'static str = bridge_runtime_common::refund_relayer_extension::RefundSignedExtensionAdapter::<R>::IDENTIFIER;
	type AccountId = <bridge_runtime_common::refund_relayer_extension::RefundSignedExtensionAdapter<
		R,
	> as SignedExtension>::AccountId;
	type Call = <bridge_runtime_common::refund_relayer_extension::RefundSignedExtensionAdapter::<R> as SignedExtension>::Call;
	type AdditionalSigned = <bridge_runtime_common::refund_relayer_extension::RefundSignedExtensionAdapter::<R> as SignedExtension>::AdditionalSigned;
	type Pre = <bridge_runtime_common::refund_relayer_extension::RefundSignedExtensionAdapter::<R> as SignedExtension>::Pre;

	fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
		bridge_runtime_common::refund_relayer_extension::RefundSignedExtensionAdapter::<R>::default(
		)
		.additional_signed()
	}

	fn validate(
		&self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> TransactionValidity {
		let calls = R::expand_call(call);
		for call in calls {
			if let Some(CallInfo::ReceiveMessagesProof(delivery_info)) = call.call_info_for(
				<<R as RefundSignedExtension>::Msgs as RefundableMessagesLaneId>::Id::get(),
			) {
				let lane_id = delivery_info.base.lane_id;
				if !is_outbound_xcmpl_channel_active::<AR>(lane_id) {
					return sp_runtime::transaction_validity::InvalidTransaction::Stale.into();
				}
			}
		}

		bridge_runtime_common::refund_relayer_extension::RefundSignedExtensionAdapter::<R>::default(
		)
		.validate(who, call, info, len)
	}

	fn pre_dispatch(
		self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		self.validate(who, call, info, len)?;
		bridge_runtime_common::refund_relayer_extension::RefundSignedExtensionAdapter::<R>::default(
		)
		.pre_dispatch(who, call, info, len)
	}

	fn post_dispatch(
		pre: Option<Self::Pre>,
		info: &DispatchInfoOf<Self::Call>,
		post_info: &PostDispatchInfoOf<Self::Call>,
		len: usize,
		result: &DispatchResult,
	) -> Result<(), TransactionValidityError> {
		bridge_runtime_common::refund_relayer_extension::RefundSignedExtensionAdapter::<R>::post_dispatch(pre, info, post_info, len, result)
	}
}

/// Returns true if the local outbound XCMP channel associated with given lane is active.
fn is_outbound_xcmpl_channel_active<AR>(lane: LaneId) -> bool
where
	AR: Get<sp_std::vec::Vec<(LaneId, Box<dyn XcmChannelStatusProvider>)>>,
{
	let provider = AR::get()
		.into_iter()
		.find(|(route_lane, _)| *route_lane == lane)
		.map(|(_, provider)| provider);
	if let Some(provider) = provider {
		provider.is_congested()
	} else {
		false
	}
}
