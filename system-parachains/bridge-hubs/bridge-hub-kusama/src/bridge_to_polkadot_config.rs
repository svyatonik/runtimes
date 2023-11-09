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

//! Bridge definitions used for bridging with Polkadot Bridge Hub.

use crate::{
	xcm_config::{UniversalLocation, XcmRouter},
	AccountId, Balance, Balances, BlockNumber, BridgePolkadotMessages, Runtime, RuntimeEvent,
	RuntimeOrigin,
};
use bp_messages::LaneId;
use bp_parachains::SingleParaStoredHeaderDataBuilder;
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
		ActualFeeRefund, RefundBridgedParachainMessages, RefundSignedExtensionAdapter,
		RefundableMessagesLane, RefundableParachain,
	},
};
use codec::Encode;
use cumulus_primitives_core::ParentThen;
use frame_support::{parameter_types, traits::PalletInfoAccess};
use kusama_runtime_constants as constants;
use sp_runtime::{traits::ConstU32, RuntimeDebug};
use xcm::{
	latest::prelude::*,
	prelude::{InteriorMultiLocation, NetworkId},
};
use xcm_builder::{BridgeBlobDispatcher, HaulBlobExporter};

/// Lane identifier, used to connect Kusama Asset Hub and Polkadot Asset Hub.
pub const XCM_LANE_FOR_ASSET_HUB_KUSAMA_TO_ASSET_HUB_POLKADOT: LaneId = LaneId([0, 0, 0, 1]);

// Parameters that may be changed by the governance.
parameter_types! {
	/// Reward that is paid (by the Kusama Asset Hub) to relayers for delivering a single
	/// Kusama -> Polkadot bridge message.
	///
	/// This payment is tracked by the `pallet_bridge_relayers` pallet at the Kusama
	/// Bridge Hub.
	pub storage DeliveryRewardInBalance: Balance = constants::currency::UNITS / 10_000;

	/// Registered relayer stake.
	///
	/// Any relayer may reserve this amount on his account and get a priority boost for his
	/// message delivery transactions. In exchange, he risks losing his stake if he would
	/// submit an invalid transaction. The set of such (registered) relayers is tracked
	/// by the `pallet_bridge_relayers` pallet at the Kusama Bridge Hub.
	pub storage RequiredStakeForStakeAndSlash: Balance = 100 * constants::currency::UNITS;
}

// Parameters, used by both XCM and bridge code.
parameter_types! {
	/// Polkadot Network identifier.
	pub PolkadotGlobalConsensusNetwork: NetworkId = NetworkId::Polkadot;
	/// Interior location (relative to this runtime) of the with-Polkadot messages pallet.
	pub BridgeKusamaToPolkadotMessagesPalletInstance: InteriorMultiLocation = X1(
		PalletInstance(<BridgePolkadotMessages as PalletInfoAccess>::index() as u8),
	);

	/// Identifier of the sibling Kusama Asset Hub parachain.
	pub AssetHubKusamaParaId: cumulus_primitives_core::ParaId = 1000.into(); // TODO: bp_asset_hub_kusama::ASSET_HUB_KUSAMA_PARACHAIN_ID.into();
	/// A route (XCM location and bridge lane) that the Kusama Asset Hub -> Polkadot Asset Hub
	/// message is following.
	pub FromAssetHubKusamaToAssetHubPolkadotRoute: SenderAndLane = SenderAndLane::new(
		ParentThen(X1(Parachain(AssetHubKusamaParaId::get().into()))).into(),
		XCM_LANE_FOR_ASSET_HUB_KUSAMA_TO_ASSET_HUB_POLKADOT,
	);

	/// Message that is sent to the sibling Kusama Asset Hub when the with-Polkadot bridge becomes congested.
	pub CongestedMessage: Xcm<()> = build_congestion_message(true).into();
	/// Message that is sent to the sibling Kusama Asset Hub when the with-Polkadot bridge becomes uncongested.
	pub UncongestedMessage: Xcm<()> = build_congestion_message(false).into();
}

// Parameters, used by bridge transport code.
parameter_types! {
	/// Number of Polkadot headers to keep in the runtime storage.
	///
	/// Note that we are keeping only required header information, not the whole header itself. Roughly, it
	/// is the 2 hours of real time (assuming that every header is submitted).
	pub const RelayChainHeadersToKeep: u32 = 1_200;
	/// Number of Polkadot Bridge Hub headers to keep in the runtime storage.
	///
	/// Note that we are keeping only required header information, not the whole header itself. Roughly, it
	/// is the 2 hours of real time (assuming that every header is submitted).
	pub const ParachainHeadsToKeep: u32 = 600;
	/// Maximal size of Polkadot Bridge Hub header **part** that we are storing in the runtime storage.
	pub const MaxParaHeadDataSize: u32 = bp_polkadot::MAX_NESTED_PARACHAIN_HEAD_DATA_SIZE;

	/// Bridge specific chain (network) identifier of the Polkadot Bridge Hub.
	pub const BridgeHubPolkadotChainId: bp_runtime::ChainId = bp_runtime::BRIDGE_HUB_POLKADOT_CHAIN_ID;
	/// Name of the `paras` pallet at Polkadot that tracks all parachain heads.
	pub const ParachainPalletNameAtPolkadot: &'static str = bp_polkadot::PARAS_PALLET_NAME;

	/// Maximal number of entries in the unrewarded relayers vector at the Kusama Bridge Hub. It matches the
	/// maximal number of unrewarded relayers that the single confirmation transaction at Polkadot Bridge
	/// Hub may process.
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: bp_messages::MessageNonce =
		bp_bridge_hub_polkadot::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	/// Maximal number of unconfirmed messages at the Kusama Bridge Hub. It matches the maximal number of
	/// uncinfirmed messages that the single confirmation transaction at Polkadot Bridge Hub may process.
	pub const MaxUnconfirmedMessagesAtInboundLane: bp_messages::MessageNonce =
		bp_bridge_hub_polkadot::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;

	/// Lane identifier, used to connect Kusama Asset Hub and Polkadot Asset Hub.
	pub const AssetHubKusamaToAssetHubPolkadotMessagesLane: bp_messages::LaneId
		= XCM_LANE_FOR_ASSET_HUB_KUSAMA_TO_ASSET_HUB_POLKADOT;
	/// All active lanes that the current bridge supports.
	pub ActiveOutboundLanesToBridgeHubPolkadot: &'static [bp_messages::LaneId]
		= &[XCM_LANE_FOR_ASSET_HUB_KUSAMA_TO_ASSET_HUB_POLKADOT];

	/// Reserve identifier, used by the `pallet_bridge_relayers` to hold funds of registered relayer.
	pub const RelayerStakeReserveId: [u8; 8] = *b"brdgrlrs";
	/// Minimal period of relayer registration. Roughly, it is the 1 hour of real time.
	pub const RelayerStakeLease: u32 = 300;
	/// Priority boost that the registered relayer receives for every additional message in the message
	/// delivery transaction.
	///
	/// It is determined semi-automatically - see `FEE_BOOST_PER_MESSAGE` constant to get the
	/// meaning of this value
	pub PriorityBoostPerMessage: u64 = 182_044_444_444_444;
}

/// Add GRANDPA bridge pallet to track Polkadot relay chain.
pub type BridgeGrandpaPolkadotInstance = pallet_bridge_grandpa::Instance1;
impl pallet_bridge_grandpa::Config<BridgeGrandpaPolkadotInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type BridgedChain = bp_polkadot::Polkadot;
	type MaxFreeMandatoryHeadersPerBlock = ConstU32<4>;
	type HeadersToKeep = RelayChainHeadersToKeep;
	type WeightInfo = (); // TODO: update me
}

/// Add parachain bridge pallet to track Polkadot BridgeHub parachain.
pub type BridgeParachainPolkadotInstance = pallet_bridge_parachains::Instance1;
impl pallet_bridge_parachains::Config<BridgeParachainPolkadotInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = (); // TODO
	type BridgesGrandpaPalletInstance = BridgeGrandpaPolkadotInstance;
	type ParasPalletName = ParachainPalletNameAtPolkadot;
	type ParaStoredHeaderDataBuilder =
		SingleParaStoredHeaderDataBuilder<bp_bridge_hub_polkadot::BridgeHubPolkadot>;
	type HeadsToKeep = ParachainHeadsToKeep;
	type MaxParaHeadDataSize = MaxParaHeadDataSize;
}

/// Allows collect and claim rewards for relayers.
impl pallet_bridge_relayers::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Reward = Balance;
	type PaymentProcedure =
		bp_relayers::PayRewardFromAccount<pallet_balances::Pallet<Runtime>, AccountId>;
	type StakeAndSlash = pallet_bridge_relayers::StakeAndSlashNamed<
		AccountId,
		BlockNumber,
		Balances,
		RelayerStakeReserveId,
		RequiredStakeForStakeAndSlash,
		RelayerStakeLease,
	>;
	type WeightInfo = (); // TODO
}

/// Add XCM messages support for exchanging messages with BridgeHubPolkadot.
pub type WithBridgeHubPolkadotMessagesInstance = pallet_bridge_messages::Instance1;
impl pallet_bridge_messages::Config<WithBridgeHubPolkadotMessagesInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = (); // TODO
	type BridgedChainId = BridgeHubPolkadotChainId;
	type ActiveOutboundLanes = ActiveOutboundLanesToBridgeHubPolkadot;
	type MaxUnrewardedRelayerEntriesAtInboundLane = MaxUnrewardedRelayerEntriesAtInboundLane;
	type MaxUnconfirmedMessagesAtInboundLane = MaxUnconfirmedMessagesAtInboundLane;

	type MaximalOutboundPayloadSize = ToBridgeHubPolkadotMaximalOutboundPayloadSize;
	type OutboundPayload = XcmAsPlainPayload;

	type InboundPayload = XcmAsPlainPayload;
	type InboundRelayer = AccountId;
	type DeliveryPayments = ();

	type TargetHeaderChain = TargetHeaderChainAdapter<WithBridgeHubPolkadotMessageBridge>;
	type LaneMessageVerifier = ToBridgeHubPolkadotMessageVerifier;
	type DeliveryConfirmationPayments = pallet_bridge_relayers::DeliveryConfirmationPaymentsAdapter<
		Runtime,
		WithBridgeHubPolkadotMessagesInstance,
		DeliveryRewardInBalance,
	>;

	type SourceHeaderChain = SourceHeaderChainAdapter<WithBridgeHubPolkadotMessageBridge>;
	type MessageDispatch = XcmBlobMessageDispatch<
		FromPolkadotMessageBlobDispatcher,
		Self::WeightInfo,
		cumulus_pallet_xcmp_queue::bridging::OutXcmpChannelStatusProvider<
			AssetHubKusamaParaId,
			Runtime,
		>,
	>;
	type OnMessagesDelivered = OnMessagesDeliveredFromPolkadot;
}

fn build_congestion_message<Call>(is_congested: bool) -> sp_std::vec::Vec<Instruction<Call>> {
	sp_std::vec![
		UnpaidExecution { weight_limit: Unlimited, check_origin: None },
		Transact {
			origin_kind: OriginKind::Xcm,
			require_weight_at_most:
				bp_asset_hub_kusama::XcmBridgeHubRouterTransactCallMaxWeight::get(),
			call: bp_asset_hub_kusama::Call::ToPolkadotXcmRouter(
				bp_asset_hub_kusama::XcmBridgeHubRouterCall::report_bridge_status {
					bridge_id: Default::default(),
					is_congested,
				}
			)
			.encode()
			.into(),
		}
	]
}

/// Proof of messages, coming from Polkadot.
pub type FromPolkadotBridgeHubMessagesProof =
	FromBridgedChainMessagesProof<bp_bridge_hub_polkadot::Hash>;
/// Messages delivery proof for Kusama Bridge Hub -> Polkadot Bridge Hub messages.
pub type ToPolkadotBridgeHubMessagesDeliveryProof =
	FromBridgedChainMessagesDeliveryProof<bp_bridge_hub_polkadot::Hash>;

/// Dispatches received XCM messages from Polkadot BridgeHub.
type FromPolkadotMessageBlobDispatcher = BridgeBlobDispatcher<
	XcmRouter,
	UniversalLocation,
	BridgeKusamaToPolkadotMessagesPalletInstance,
>;

/// Export XCM messages to be relayed to the other side
pub type ToBridgeHubPolkadotHaulBlobExporter = HaulBlobExporter<
	XcmBlobHaulerAdapter<ToBridgeHubPolkadotXcmBlobHauler>,
	PolkadotGlobalConsensusNetwork,
	(),
>;
pub struct ToBridgeHubPolkadotXcmBlobHauler;
impl XcmBlobHauler for ToBridgeHubPolkadotXcmBlobHauler {
	type Runtime = Runtime;
	type MessagesInstance = WithBridgeHubPolkadotMessagesInstance;
	type SenderAndLane = FromAssetHubKusamaToAssetHubPolkadotRoute;

	type ToSourceChainSender = XcmRouter;
	type CongestedMessage = CongestedMessage;
	type UncongestedMessage = UncongestedMessage;
}

/// On messages delivered callback.
type OnMessagesDeliveredFromPolkadot = XcmBlobHaulerAdapter<ToBridgeHubPolkadotXcmBlobHauler>;

/// Messaging Bridge configuration for BridgeHubKusama -> BridgeHubPolkadot
pub struct WithBridgeHubPolkadotMessageBridge;
impl MessageBridge for WithBridgeHubPolkadotMessageBridge {
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bp_bridge_hub_kusama::WITH_BRIDGE_HUB_KUSAMA_MESSAGES_PALLET_NAME;
	type ThisChain = BridgeHubKusama;
	type BridgedChain = BridgeHubPolkadot;
	type BridgedHeaderChain = pallet_bridge_parachains::ParachainHeaders<
		Runtime,
		BridgeParachainPolkadotInstance,
		bp_bridge_hub_polkadot::BridgeHubPolkadot,
	>;
}

/// Message verifier for BridgeHubPolkadot messages sent from BridgeHubKusama
pub type ToBridgeHubPolkadotMessageVerifier =
	messages::source::FromThisChainMessageVerifier<WithBridgeHubPolkadotMessageBridge>;

/// Maximal outbound payload size of BridgeHubKusama -> BridgeHubPolkadot messages.
pub type ToBridgeHubPolkadotMaximalOutboundPayloadSize =
	messages::source::FromThisChainMaximalOutboundPayloadSize<WithBridgeHubPolkadotMessageBridge>;

/// BridgeHubPolkadot chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct BridgeHubPolkadot;

impl UnderlyingChainProvider for BridgeHubPolkadot {
	type Chain = bp_bridge_hub_polkadot::BridgeHubPolkadot;
}

impl messages::BridgedChainWithMessages for BridgeHubPolkadot {}

/// BridgeHubKusama chain from message lane point of view.
#[derive(RuntimeDebug, Clone, Copy)]
pub struct BridgeHubKusama;

impl UnderlyingChainProvider for BridgeHubKusama {
	type Chain = bp_bridge_hub_kusama::BridgeHubKusama;
}

impl ThisChainWithMessages for BridgeHubKusama {
	type RuntimeOrigin = RuntimeOrigin;
}

/// Signed extension that refunds relayers that are delivering messages from the Polkadot parachain.
pub type RefundBridgeHubPolkadotMessages = RefundSignedExtensionAdapter<
	RefundBridgedParachainMessages<
		Runtime,
		RefundableParachain<
			BridgeParachainPolkadotInstance,
			bp_bridge_hub_polkadot::BridgeHubPolkadot,
		>,
		RefundableMessagesLane<
			WithBridgeHubPolkadotMessagesInstance,
			AssetHubKusamaToAssetHubPolkadotMessagesLane,
		>,
		ActualFeeRefund<Runtime>,
		PriorityBoostPerMessage,
		StrRefundBridgeHubPolkadotMessages,
	>,
>;
bp_runtime::generate_static_str_provider!(RefundBridgeHubPolkadotMessages);

#[cfg(test)]
mod tests {
	use super::*;
	use bridge_runtime_common::{
		assert_complete_bridge_types,
		integrity::{
			assert_complete_bridge_constants, check_message_lane_weights,
			AssertBridgeMessagesPalletConstants, AssertBridgePalletNames, AssertChainConstants,
			AssertCompleteBridgeConstants,
		},
	};

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
	fn ensure_bridge_hub_kusama_message_lane_weights_are_correct() {
		check_message_lane_weights::<
			bp_bridge_hub_kusama::BridgeHubKusama,
			Runtime,
			WithBridgeHubPolkadotMessagesInstance,
		>(
			bp_bridge_hub_polkadot::EXTRA_STORAGE_PROOF_SIZE,
			bp_bridge_hub_kusama::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX,
			bp_bridge_hub_kusama::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX,
			true,
		);
	}

	#[test]
	fn ensure_bridge_integrity() {
		assert_complete_bridge_types!(
			runtime: Runtime,
			with_bridged_chain_grandpa_instance: BridgeGrandpaPolkadotInstance,
			with_bridged_chain_messages_instance: WithBridgeHubPolkadotMessagesInstance,
			bridge: WithBridgeHubPolkadotMessageBridge,
			this_chain: bp_kusama::Kusama,
			bridged_chain: bp_polkadot::Polkadot,
		);

		assert_complete_bridge_constants::<
			Runtime,
			BridgeGrandpaPolkadotInstance,
			WithBridgeHubPolkadotMessagesInstance,
			WithBridgeHubPolkadotMessageBridge,
		>(AssertCompleteBridgeConstants {
			this_chain_constants: AssertChainConstants {
				block_length: bp_bridge_hub_kusama::BlockLength::get(),
				block_weights: bp_bridge_hub_kusama::BlockWeights::get(),
			},
			messages_pallet_constants: AssertBridgeMessagesPalletConstants {
				max_unrewarded_relayers_in_bridged_confirmation_tx:
					bp_bridge_hub_polkadot::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX,
				max_unconfirmed_messages_in_bridged_confirmation_tx:
					bp_bridge_hub_polkadot::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX,
				bridged_chain_id: bp_runtime::BRIDGE_HUB_POLKADOT_CHAIN_ID,
			},
			pallet_names: AssertBridgePalletNames {
				with_this_chain_messages_pallet_name:
					bp_bridge_hub_kusama::WITH_BRIDGE_HUB_KUSAMA_MESSAGES_PALLET_NAME,
				with_bridged_chain_grandpa_pallet_name:
					bp_polkadot::WITH_POLKADOT_GRANDPA_PALLET_NAME,
				with_bridged_chain_messages_pallet_name:
					bp_bridge_hub_polkadot::WITH_BRIDGE_HUB_POLKADOT_MESSAGES_PALLET_NAME,
			},
		});

		bridge_runtime_common::priority_calculator::ensure_priority_boost_is_sane::<
			Runtime,
			WithBridgeHubPolkadotMessagesInstance,
			PriorityBoostPerMessage,
		>(FEE_BOOST_PER_MESSAGE);

		assert_eq!(
			BridgeKusamaToPolkadotMessagesPalletInstance::get(),
			X1(PalletInstance(
				bridge_hub_kusama_runtime_constants::WITH_BRIDGE_KUSAMA_TO_POLKADOT_MESSAGES_PALLET_INDEX
			))
		);
	}
}
