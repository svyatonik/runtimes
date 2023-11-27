// Copyright (C) Parity Technologies (UK) Ltd.
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

//! Bridge definitions used for bridging with Polkadot Bulletin Chain.

use crate::{
	weights,
	xcm_config::{UniversalLocation, XcmRouter},
	AccountId, BridgePolkadotBulletinGrandpa, BridgePolkadotBulletinMessages, Runtime,
	RuntimeEvent, RuntimeOrigin,
};
use bp_messages::LaneId;
use bridge_runtime_common::{
	messages,
	messages::{
		source::{FromBridgedChainMessagesDeliveryProof, TargetHeaderChainAdapter},
		target::{FromBridgedChainMessagesProof, SourceHeaderChainAdapter},
		MessageBridge, ThisChainWithMessages, UnderlyingChainProvider,
	},
	messages_xcm_extension::{
		SenderAndLane, XcmAsPlainPayload, XcmBlobHauler, XcmBlobHaulerAdapter,
		XcmBlobMessageDispatch,
	},
	refund_relayer_extension::{
		ActualFeeRefund, RefundableMessagesLane, RefundBridgedGrandpaMessages, RefundSignedExtensionAdapter,
	},
};
use cumulus_primitives_core::ParentThen;
use frame_support::{parameter_types, traits::PalletInfoAccess};
use sp_runtime::{traits::ConstU32, RuntimeDebug};
use xcm::{
	latest::prelude::*,
	prelude::{InteriorMultiLocation, NetworkId},
};
use xcm_builder::{BridgeBlobDispatcher, HaulBlobExporter};

/// Lane identifier, used to connect Kawabunga and Polkadot Bulletin Chain.
pub const XCM_LANE_FOR_KAWABUNGA_TO_POLKADOT_BULLETIN: LaneId = LaneId([0, 0, 0, 1]);

// Parameters, used by both XCM and bridge code.
parameter_types! {
	/// Polkadot Bulletin Network identifier.
	pub PolkadotBulletinGlobalConsensusNetwork: NetworkId = NetworkId::ByGenesis([42u8; 32]); // TODO
	/// Interior location (relative to this runtime) of the with-PolkadotBulletin messages pallet.
	pub BridgePolkadotToPolkadotBulletinMessagesPalletInstance: InteriorMultiLocation = X1(
		PalletInstance(<BridgePolkadotBulletinMessages as PalletInfoAccess>::index() as u8),
	);

	/// Identifier of the sibling Kawabunga parachain.
	pub KawabungaParaId: cumulus_primitives_core::ParaId = 1000.into(); // TODO
	/// A route (XCM location and bridge lane) that the Kawabunga -> Polkadot Bulletin Chain
	/// message is following.
	pub FromKawabungaToPolkadotBulletinRoute: SenderAndLane = SenderAndLane::new(
		ParentThen(X1(Parachain(KawabungaParaId::get().into()))).into(),
		XCM_LANE_FOR_KAWABUNGA_TO_POLKADOT_BULLETIN,
	);

	/// XCM message that is never sent.
	pub NeverSentMessage: Option<Xcm<()>> = None;
}

// Parameters, used by bridge transport code.
parameter_types! {
	/// Number of Polkadot Bulletin Chain headers to keep in the runtime storage.
	///
	/// Note that we are keeping only required header information, not the whole header itself. Roughly, it
	/// is the 2 hours of real time (assuming that every header is submitted).
	pub const RelayChainHeadersToKeep: u32 = 1_200;

	/// Bridge specific chain (network) identifier of the Polkadot Bulletin Chain.
	pub const PolkadotBulletinChainId: bp_runtime::ChainId = bp_runtime::POLKADOT_BULLETIN_CHAIN_ID;

	/// Maximal number of entries in the unrewarded relayers vector at the Polkadot Bridge Hub. It matches the
	/// maximal number of unrewarded relayers that the single confirmation transaction at Polkadot Bulletin Chain
	/// may process.
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: bp_messages::MessageNonce =
		bp_polkadot_bulletin::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	/// Maximal number of unconfirmed messages at the Polkadot Bridge Hub. It matches the maximal number of
	/// uncinfirmed messages that the single confirmation transaction at Polkadot Bulletin Chain may process.
	pub const MaxUnconfirmedMessagesAtInboundLane: bp_messages::MessageNonce =
		bp_polkadot_bulletin::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;

	/// Lane identifier, used to connect Kawabunga and Polkadot Bulletin Chain.
	pub const KawabungaToPolkadotBulletinMessagesLane: bp_messages::LaneId
		= XCM_LANE_FOR_KAWABUNGA_TO_POLKADOT_BULLETIN;
	/// All active lanes that the current bridge supports.
	pub ActiveOutboundLanesToPolkadotBulletin: &'static [bp_messages::LaneId]
		= &[XCM_LANE_FOR_KAWABUNGA_TO_POLKADOT_BULLETIN];

	/// Priority boost that the registered relayer receives for every additional message in the message
	/// delivery transaction.
	///
	/// It is determined semi-automatically - see `FEE_BOOST_PER_MESSAGE` constant to get the
	/// meaning of this value.
	pub PriorityBoostPerMessage: u64 = 1_820_444_444_444;
}

/// Add GRANDPA bridge pallet to track Polkadot Bulletin Chain.
pub type BridgeGrandpaPolkadotBulletinInstance = pallet_bridge_grandpa::Instance2;
impl pallet_bridge_grandpa::Config<BridgeGrandpaPolkadotBulletinInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BridgedChain = bp_polkadot_bulletin::PolkadotBulletin;
	type MaxFreeMandatoryHeadersPerBlock = ConstU32<4>;
	type HeadersToKeep = RelayChainHeadersToKeep;
	// Technically this is incorrect - we have two pallet instances and ideally we shall
	// benchmark every instance separately. But the benchmarking engine has a flaw - it
	// messes with components. E.g. in Kusama maximal validators count is 1024 and in
	// Bulletin chain it is 100. But benchmarking engine runs Bulletin benchmarks using
	// components range, computed for Kusama => it causes an error.
	//
	// In practice, however, GRANDPA pallet works the same way for all bridged chains, so
	// weights are also the same for both bridges.
	type WeightInfo = weights::pallet_bridge_grandpa::WeightInfo<Runtime>;
}

/// Add XCM messages support for exchanging messages with Polkadot Bulletin Chain.
pub type WithPolkadotBulletinMessagesInstance = pallet_bridge_messages::Instance2;
impl pallet_bridge_messages::Config<WithPolkadotBulletinMessagesInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_bridge_messages::WeightInfo<Runtime>; // TODO
	type BridgedChainId = PolkadotBulletinChainId;
	type ActiveOutboundLanes = ActiveOutboundLanesToPolkadotBulletin;
	type MaxUnrewardedRelayerEntriesAtInboundLane = MaxUnrewardedRelayerEntriesAtInboundLane;
	type MaxUnconfirmedMessagesAtInboundLane = MaxUnconfirmedMessagesAtInboundLane;

	type MaximalOutboundPayloadSize = ToPolkadotBulletinMaximalOutboundPayloadSize;
	type OutboundPayload = XcmAsPlainPayload;

	type InboundPayload = XcmAsPlainPayload;
	type InboundRelayer = AccountId;
	type DeliveryPayments = ();

	type TargetHeaderChain = TargetHeaderChainAdapter<WithPolkadotBulletinMessageBridge>;
	type LaneMessageVerifier = ToPolkadotBulletinMessageVerifier;
	// no rewards for delivering messages on that bridge
	type DeliveryConfirmationPayments = ();

	type SourceHeaderChain = SourceHeaderChainAdapter<WithPolkadotBulletinMessageBridge>;
	type MessageDispatch =
		XcmBlobMessageDispatch<FromPolkadotBulletinMessageBlobDispatcher, Self::WeightInfo, ()>;
	// no fees => no congestion/uncongestion messages
	type OnMessagesDelivered = ();
}

/// Proof of messages, coming from Polkadot Bulletin Chain.
pub type FromPolkadotBulletinMessagesProof =
	FromBridgedChainMessagesProof<bp_polkadot_bulletin::Hash>;
/// Messages delivery proof for Polkadot Bridge Hub -> Polkadot Bulletin Chain messages.
pub type ToPolkadotBulletinMessagesDeliveryProof =
	FromBridgedChainMessagesDeliveryProof<bp_polkadot_bulletin::Hash>;

/// Dispatches received XCM messages from Polkadot Bulletin Chain.
type FromPolkadotBulletinMessageBlobDispatcher = BridgeBlobDispatcher<
	XcmRouter,
	UniversalLocation,
	BridgePolkadotToPolkadotBulletinMessagesPalletInstance,
>;

/// Export XCM messages to be relayed to the other side
pub type ToPolkadotBulletinHaulBlobExporter = HaulBlobExporter<
	XcmBlobHaulerAdapter<ToPolkadotBulletinXcmBlobHauler>,
	PolkadotBulletinGlobalConsensusNetwork,
	(),
>;
pub struct ToPolkadotBulletinXcmBlobHauler;
impl XcmBlobHauler for ToPolkadotBulletinXcmBlobHauler {
	type Runtime = Runtime;
	type MessagesInstance = WithPolkadotBulletinMessagesInstance;
	type SenderAndLane = FromKawabungaToPolkadotBulletinRoute;

	type ToSourceChainSender = XcmRouter;
	type CongestedMessage = NeverSentMessage;
	type UncongestedMessage = NeverSentMessage;
}

/// Messaging Bridge configuration for BridgeHubPolkadot -> Polkadot Bulletin Chain.
pub struct WithPolkadotBulletinMessageBridge;
impl MessageBridge for WithPolkadotBulletinMessageBridge {
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bp_bridge_hub_polkadot::WITH_BRIDGE_HUB_POLKADOT_MESSAGES_PALLET_NAME;
	type ThisChain = BridgeHubPolkadot;
	type BridgedChain = PolkadotBulletin;
	type BridgedHeaderChain = BridgePolkadotBulletinGrandpa;
}

/// Message verifier for Polkadot Bulletin messages sent from BridgeHubPolkadot.
pub type ToPolkadotBulletinMessageVerifier =
	messages::source::FromThisChainMessageVerifier<WithPolkadotBulletinMessageBridge>;

/// Maximal outbound payload size of BridgeHubPolkadot -> PolkadotBulletin messages.
pub type ToPolkadotBulletinMaximalOutboundPayloadSize =
	messages::source::FromThisChainMaximalOutboundPayloadSize<WithPolkadotBulletinMessageBridge>;

/// PolkadotBulletin chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct PolkadotBulletin;

impl UnderlyingChainProvider for PolkadotBulletin {
	type Chain = bp_polkadot_bulletin::PolkadotBulletin;
}

impl messages::BridgedChainWithMessages for PolkadotBulletin {}

/// BridgeHubPolkadot chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct BridgeHubPolkadot;

impl UnderlyingChainProvider for BridgeHubPolkadot {
	type Chain = bp_bridge_hub_polkadot::BridgeHubPolkadot;
}

impl ThisChainWithMessages for BridgeHubPolkadot {
	type RuntimeOrigin = RuntimeOrigin;
}

/// Signed extension that refunds relayers that are delivering messages from the Polkadot Bulletin
/// Chain.
pub type RefundPolkadotBulletinMessages = RefundSignedExtensionAdapter<
	RefundBridgedGrandpaMessages<
		Runtime,
		BridgeGrandpaPolkadotBulletinInstance,
		RefundableMessagesLane<
			WithPolkadotBulletinMessagesInstance,
			KawabungaToPolkadotBulletinMessagesLane,
		>,
		ActualFeeRefund<Runtime>,
		PriorityBoostPerMessage,
		StrRefundPolkadotBulletinMessages,
	>,
>;
bp_runtime::generate_static_str_provider!(RefundPolkadotBulletinMessages);

#[cfg(test)]
mod tests {
	use super::*;
	use crate::Balance;
	use bridge_runtime_common::{
		assert_complete_bridge_types,
		integrity::{
			assert_complete_bridge_constants, check_message_lane_weights,
			AssertBridgeMessagesPalletConstants, AssertBridgePalletNames, AssertChainConstants,
			AssertCompleteBridgeConstants,
		},
	};
	use polkadot_runtime_constants as constants;

	/// Every additional message in the message delivery transaction boosts its priority.
	/// So the priority of transaction with `N+1` messages is larger than priority of
	/// transaction with `N` messages by the `PriorityBoostPerMessage`.
	///
	/// Economically, it is an equivalent of adding tip to the transaction with `N` messages.
	/// The `FEE_BOOST_PER_MESSAGE` constant is the value of this tip.
	///
	/// We want this tip to be large enough (delivery transactions with more messages = less
	/// operational costs and a faster bridge), so this value should be significant.
	const FEE_BOOST_PER_MESSAGE: Balance = 2 * constants::currency::UNITS;

	#[test]
	fn ensure_bridge_hub_polkadot_message_lane_weights_are_correct() {
		check_message_lane_weights::<
			bp_bridge_hub_polkadot::BridgeHubPolkadot,
			Runtime,
			WithPolkadotBulletinMessagesInstance,
		>(
			bp_polkadot_bulletin::EXTRA_STORAGE_PROOF_SIZE,
			bp_bridge_hub_polkadot::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX,
			bp_bridge_hub_polkadot::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX,
			true,
		);
	}

	#[test]
	fn ensure_bridge_integrity() {
		assert_complete_bridge_types!(
			runtime: Runtime,
			with_bridged_chain_grandpa_instance: BridgeGrandpaPolkadotBulletinInstance,
			with_bridged_chain_messages_instance: WithPolkadotBulletinMessagesInstance,
			bridge: WithPolkadotBulletinMessageBridge,
			this_chain: bp_polkadot::Polkadot,
			bridged_chain: bp_polkadot_bulletin::PolkadotBulletin,
		);

		assert_complete_bridge_constants::<
			Runtime,
			BridgeGrandpaPolkadotBulletinInstance,
			WithPolkadotBulletinMessagesInstance,
			WithPolkadotBulletinMessageBridge,
		>(AssertCompleteBridgeConstants {
			this_chain_constants: AssertChainConstants {
				block_length: bp_bridge_hub_polkadot::BlockLength::get(),
				block_weights: bp_bridge_hub_polkadot::BlockWeights::get(),
			},
			messages_pallet_constants: AssertBridgeMessagesPalletConstants {
				max_unrewarded_relayers_in_bridged_confirmation_tx:
					bp_polkadot_bulletin::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX,
				max_unconfirmed_messages_in_bridged_confirmation_tx:
					bp_polkadot_bulletin::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX,
				bridged_chain_id: bp_runtime::POLKADOT_BULLETIN_CHAIN_ID,
			},
			pallet_names: AssertBridgePalletNames {
				with_this_chain_messages_pallet_name:
					bp_bridge_hub_polkadot::WITH_BRIDGE_HUB_POLKADOT_MESSAGES_PALLET_NAME,
				with_bridged_chain_grandpa_pallet_name:
					bp_polkadot_bulletin::WITH_POLKADOT_BULLETIN_GRANDPA_PALLET_NAME,
				with_bridged_chain_messages_pallet_name:
					bp_polkadot_bulletin::WITH_POLKADOT_BULLETIN_MESSAGES_PALLET_NAME,
			},
		});

		bridge_runtime_common::priority_calculator::ensure_priority_boost_is_sane::<
			Runtime,
			WithPolkadotBulletinMessagesInstance,
			PriorityBoostPerMessage,
		>(FEE_BOOST_PER_MESSAGE);

		assert_eq!(
			BridgePolkadotToPolkadotBulletinMessagesPalletInstance::get(),
			X1(PalletInstance(
				bp_bridge_hub_polkadot::WITH_BRIDGE_POLKADOT_TO_POLKADOT_BULLETIN_MESSAGES_PALLET_INDEX
			))
		);
	}
}
