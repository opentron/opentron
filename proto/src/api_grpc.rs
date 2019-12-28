// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]


// interface

pub trait Wallet {
    fn get_account(&self, o: ::grpc::RequestOptions, p: super::Tron::Account) -> ::grpc::SingleResponse<super::Tron::Account>;

    fn get_account_by_id(&self, o: ::grpc::RequestOptions, p: super::Tron::Account) -> ::grpc::SingleResponse<super::Tron::Account>;

    fn create_transaction(&self, o: ::grpc::RequestOptions, p: super::Contract::TransferContract) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn create_transaction2(&self, o: ::grpc::RequestOptions, p: super::Contract::TransferContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn broadcast_transaction(&self, o: ::grpc::RequestOptions, p: super::Tron::Transaction) -> ::grpc::SingleResponse<super::api::Return>;

    fn update_account(&self, o: ::grpc::RequestOptions, p: super::Contract::AccountUpdateContract) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn set_account_id(&self, o: ::grpc::RequestOptions, p: super::Contract::SetAccountIdContract) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn update_account2(&self, o: ::grpc::RequestOptions, p: super::Contract::AccountUpdateContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn vote_witness_account(&self, o: ::grpc::RequestOptions, p: super::Contract::VoteWitnessContract) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn update_setting(&self, o: ::grpc::RequestOptions, p: super::Contract::UpdateSettingContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn update_energy_limit(&self, o: ::grpc::RequestOptions, p: super::Contract::UpdateEnergyLimitContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn vote_witness_account2(&self, o: ::grpc::RequestOptions, p: super::Contract::VoteWitnessContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn create_asset_issue(&self, o: ::grpc::RequestOptions, p: super::Contract::AssetIssueContract) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn create_asset_issue2(&self, o: ::grpc::RequestOptions, p: super::Contract::AssetIssueContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn update_witness(&self, o: ::grpc::RequestOptions, p: super::Contract::WitnessUpdateContract) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn update_witness2(&self, o: ::grpc::RequestOptions, p: super::Contract::WitnessUpdateContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn create_account(&self, o: ::grpc::RequestOptions, p: super::Contract::AccountCreateContract) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn create_account2(&self, o: ::grpc::RequestOptions, p: super::Contract::AccountCreateContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn create_witness(&self, o: ::grpc::RequestOptions, p: super::Contract::WitnessCreateContract) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn create_witness2(&self, o: ::grpc::RequestOptions, p: super::Contract::WitnessCreateContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn transfer_asset(&self, o: ::grpc::RequestOptions, p: super::Contract::TransferAssetContract) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn transfer_asset2(&self, o: ::grpc::RequestOptions, p: super::Contract::TransferAssetContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn participate_asset_issue(&self, o: ::grpc::RequestOptions, p: super::Contract::ParticipateAssetIssueContract) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn participate_asset_issue2(&self, o: ::grpc::RequestOptions, p: super::Contract::ParticipateAssetIssueContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn freeze_balance(&self, o: ::grpc::RequestOptions, p: super::Contract::FreezeBalanceContract) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn freeze_balance2(&self, o: ::grpc::RequestOptions, p: super::Contract::FreezeBalanceContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn unfreeze_balance(&self, o: ::grpc::RequestOptions, p: super::Contract::UnfreezeBalanceContract) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn unfreeze_balance2(&self, o: ::grpc::RequestOptions, p: super::Contract::UnfreezeBalanceContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn unfreeze_asset(&self, o: ::grpc::RequestOptions, p: super::Contract::UnfreezeAssetContract) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn unfreeze_asset2(&self, o: ::grpc::RequestOptions, p: super::Contract::UnfreezeAssetContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn withdraw_balance(&self, o: ::grpc::RequestOptions, p: super::Contract::WithdrawBalanceContract) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn withdraw_balance2(&self, o: ::grpc::RequestOptions, p: super::Contract::WithdrawBalanceContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn update_asset(&self, o: ::grpc::RequestOptions, p: super::Contract::UpdateAssetContract) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn update_asset2(&self, o: ::grpc::RequestOptions, p: super::Contract::UpdateAssetContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn proposal_create(&self, o: ::grpc::RequestOptions, p: super::Contract::ProposalCreateContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn proposal_approve(&self, o: ::grpc::RequestOptions, p: super::Contract::ProposalApproveContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn proposal_delete(&self, o: ::grpc::RequestOptions, p: super::Contract::ProposalDeleteContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn buy_storage(&self, o: ::grpc::RequestOptions, p: super::Contract::BuyStorageContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn buy_storage_bytes(&self, o: ::grpc::RequestOptions, p: super::Contract::BuyStorageBytesContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn sell_storage(&self, o: ::grpc::RequestOptions, p: super::Contract::SellStorageContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn exchange_create(&self, o: ::grpc::RequestOptions, p: super::Contract::ExchangeCreateContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn exchange_inject(&self, o: ::grpc::RequestOptions, p: super::Contract::ExchangeInjectContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn exchange_withdraw(&self, o: ::grpc::RequestOptions, p: super::Contract::ExchangeWithdrawContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn exchange_transaction(&self, o: ::grpc::RequestOptions, p: super::Contract::ExchangeTransactionContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn list_nodes(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::NodeList>;

    fn get_asset_issue_by_account(&self, o: ::grpc::RequestOptions, p: super::Tron::Account) -> ::grpc::SingleResponse<super::api::AssetIssueList>;

    fn get_account_net(&self, o: ::grpc::RequestOptions, p: super::Tron::Account) -> ::grpc::SingleResponse<super::api::AccountNetMessage>;

    fn get_account_resource(&self, o: ::grpc::RequestOptions, p: super::Tron::Account) -> ::grpc::SingleResponse<super::api::AccountResourceMessage>;

    fn get_asset_issue_by_name(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Contract::AssetIssueContract>;

    fn get_asset_issue_list_by_name(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::AssetIssueList>;

    fn get_asset_issue_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Contract::AssetIssueContract>;

    fn get_now_block(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::Tron::Block>;

    fn get_now_block2(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::BlockExtention>;

    fn get_block_by_num(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::Tron::Block>;

    fn get_block_by_num2(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::api::BlockExtention>;

    fn get_transaction_count_by_block_num(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::api::NumberMessage>;

    fn get_block_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::Block>;

    fn get_block_by_limit_next(&self, o: ::grpc::RequestOptions, p: super::api::BlockLimit) -> ::grpc::SingleResponse<super::api::BlockList>;

    fn get_block_by_limit_next2(&self, o: ::grpc::RequestOptions, p: super::api::BlockLimit) -> ::grpc::SingleResponse<super::api::BlockListExtention>;

    fn get_block_by_latest_num(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::api::BlockList>;

    fn get_block_by_latest_num2(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::api::BlockListExtention>;

    fn get_transaction_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn deploy_contract(&self, o: ::grpc::RequestOptions, p: super::Contract::CreateSmartContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn get_contract(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::SmartContract>;

    fn trigger_contract(&self, o: ::grpc::RequestOptions, p: super::Contract::TriggerSmartContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn trigger_constant_contract(&self, o: ::grpc::RequestOptions, p: super::Contract::TriggerSmartContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn clear_contract_abi(&self, o: ::grpc::RequestOptions, p: super::Contract::ClearABIContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn list_witnesses(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::WitnessList>;

    fn get_delegated_resource(&self, o: ::grpc::RequestOptions, p: super::api::DelegatedResourceMessage) -> ::grpc::SingleResponse<super::api::DelegatedResourceList>;

    fn get_delegated_resource_account_index(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::DelegatedResourceAccountIndex>;

    fn list_proposals(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::ProposalList>;

    fn get_paginated_proposal_list(&self, o: ::grpc::RequestOptions, p: super::api::PaginatedMessage) -> ::grpc::SingleResponse<super::api::ProposalList>;

    fn get_proposal_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::Proposal>;

    fn list_exchanges(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::ExchangeList>;

    fn get_paginated_exchange_list(&self, o: ::grpc::RequestOptions, p: super::api::PaginatedMessage) -> ::grpc::SingleResponse<super::api::ExchangeList>;

    fn get_exchange_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::Exchange>;

    fn get_chain_parameters(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::Tron::ChainParameters>;

    fn get_asset_issue_list(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::AssetIssueList>;

    fn get_paginated_asset_issue_list(&self, o: ::grpc::RequestOptions, p: super::api::PaginatedMessage) -> ::grpc::SingleResponse<super::api::AssetIssueList>;

    fn total_transaction(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::NumberMessage>;

    fn get_next_maintenance_time(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::NumberMessage>;

    fn get_transaction_sign(&self, o: ::grpc::RequestOptions, p: super::Tron::TransactionSign) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn get_transaction_sign2(&self, o: ::grpc::RequestOptions, p: super::Tron::TransactionSign) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn create_address(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::BytesMessage>;

    fn easy_transfer_asset(&self, o: ::grpc::RequestOptions, p: super::api::EasyTransferAssetMessage) -> ::grpc::SingleResponse<super::api::EasyTransferResponse>;

    fn easy_transfer_asset_by_private(&self, o: ::grpc::RequestOptions, p: super::api::EasyTransferAssetByPrivateMessage) -> ::grpc::SingleResponse<super::api::EasyTransferResponse>;

    fn easy_transfer(&self, o: ::grpc::RequestOptions, p: super::api::EasyTransferMessage) -> ::grpc::SingleResponse<super::api::EasyTransferResponse>;

    fn easy_transfer_by_private(&self, o: ::grpc::RequestOptions, p: super::api::EasyTransferByPrivateMessage) -> ::grpc::SingleResponse<super::api::EasyTransferResponse>;

    fn generate_address(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::AddressPrKeyPairMessage>;

    fn get_transaction_info_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::TransactionInfo>;

    fn account_permission_update(&self, o: ::grpc::RequestOptions, p: super::Contract::AccountPermissionUpdateContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn add_sign(&self, o: ::grpc::RequestOptions, p: super::Tron::TransactionSign) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn get_transaction_sign_weight(&self, o: ::grpc::RequestOptions, p: super::Tron::Transaction) -> ::grpc::SingleResponse<super::api::TransactionSignWeight>;

    fn get_transaction_approved_list(&self, o: ::grpc::RequestOptions, p: super::Tron::Transaction) -> ::grpc::SingleResponse<super::api::TransactionApprovedList>;

    fn get_node_info(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::Tron::NodeInfo>;

    fn create_shielded_transaction(&self, o: ::grpc::RequestOptions, p: super::api::PrivateParameters) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn get_merkle_tree_voucher_info(&self, o: ::grpc::RequestOptions, p: super::Contract::OutputPointInfo) -> ::grpc::SingleResponse<super::Contract::IncrementalMerkleVoucherInfo>;

    fn scan_note_by_ivk(&self, o: ::grpc::RequestOptions, p: super::api::IvkDecryptParameters) -> ::grpc::SingleResponse<super::api::DecryptNotes>;

    fn scan_and_mark_note_by_ivk(&self, o: ::grpc::RequestOptions, p: super::api::IvkDecryptAndMarkParameters) -> ::grpc::SingleResponse<super::api::DecryptNotesMarked>;

    fn scan_note_by_ovk(&self, o: ::grpc::RequestOptions, p: super::api::OvkDecryptParameters) -> ::grpc::SingleResponse<super::api::DecryptNotes>;

    fn get_spending_key(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::BytesMessage>;

    fn get_expanded_spending_key(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::ExpandedSpendingKeyMessage>;

    fn get_ak_from_ask(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::BytesMessage>;

    fn get_nk_from_nsk(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::BytesMessage>;

    fn get_incoming_viewing_key(&self, o: ::grpc::RequestOptions, p: super::api::ViewingKeyMessage) -> ::grpc::SingleResponse<super::api::IncomingViewingKeyMessage>;

    fn get_diversifier(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::DiversifierMessage>;

    fn get_new_shielded_address(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::ShieldedAddressInfo>;

    fn get_zen_payment_address(&self, o: ::grpc::RequestOptions, p: super::api::IncomingViewingKeyDiversifierMessage) -> ::grpc::SingleResponse<super::api::PaymentAddressMessage>;

    fn get_rcm(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::BytesMessage>;

    fn is_spend(&self, o: ::grpc::RequestOptions, p: super::api::NoteParameters) -> ::grpc::SingleResponse<super::api::SpendResult>;

    fn create_shielded_transaction_without_spend_auth_sig(&self, o: ::grpc::RequestOptions, p: super::api::PrivateParametersWithoutAsk) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn get_shield_transaction_hash(&self, o: ::grpc::RequestOptions, p: super::Tron::Transaction) -> ::grpc::SingleResponse<super::api::BytesMessage>;

    fn create_spend_auth_sig(&self, o: ::grpc::RequestOptions, p: super::api::SpendAuthSigParameters) -> ::grpc::SingleResponse<super::api::BytesMessage>;

    fn create_shield_nullifier(&self, o: ::grpc::RequestOptions, p: super::api::NfParameters) -> ::grpc::SingleResponse<super::api::BytesMessage>;

    fn get_reward_info(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::NumberMessage>;

    fn get_brokerage_info(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::NumberMessage>;

    fn update_brokerage(&self, o: ::grpc::RequestOptions, p: super::Contract::UpdateBrokerageContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;
}

// client

pub struct WalletClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
    method_GetAccount: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Tron::Account, super::Tron::Account>>,
    method_GetAccountById: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Tron::Account, super::Tron::Account>>,
    method_CreateTransaction: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::TransferContract, super::Tron::Transaction>>,
    method_CreateTransaction2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::TransferContract, super::api::TransactionExtention>>,
    method_BroadcastTransaction: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Tron::Transaction, super::api::Return>>,
    method_UpdateAccount: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::AccountUpdateContract, super::Tron::Transaction>>,
    method_SetAccountId: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::SetAccountIdContract, super::Tron::Transaction>>,
    method_UpdateAccount2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::AccountUpdateContract, super::api::TransactionExtention>>,
    method_VoteWitnessAccount: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::VoteWitnessContract, super::Tron::Transaction>>,
    method_UpdateSetting: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::UpdateSettingContract, super::api::TransactionExtention>>,
    method_UpdateEnergyLimit: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::UpdateEnergyLimitContract, super::api::TransactionExtention>>,
    method_VoteWitnessAccount2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::VoteWitnessContract, super::api::TransactionExtention>>,
    method_CreateAssetIssue: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::AssetIssueContract, super::Tron::Transaction>>,
    method_CreateAssetIssue2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::AssetIssueContract, super::api::TransactionExtention>>,
    method_UpdateWitness: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::WitnessUpdateContract, super::Tron::Transaction>>,
    method_UpdateWitness2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::WitnessUpdateContract, super::api::TransactionExtention>>,
    method_CreateAccount: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::AccountCreateContract, super::Tron::Transaction>>,
    method_CreateAccount2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::AccountCreateContract, super::api::TransactionExtention>>,
    method_CreateWitness: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::WitnessCreateContract, super::Tron::Transaction>>,
    method_CreateWitness2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::WitnessCreateContract, super::api::TransactionExtention>>,
    method_TransferAsset: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::TransferAssetContract, super::Tron::Transaction>>,
    method_TransferAsset2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::TransferAssetContract, super::api::TransactionExtention>>,
    method_ParticipateAssetIssue: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::ParticipateAssetIssueContract, super::Tron::Transaction>>,
    method_ParticipateAssetIssue2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::ParticipateAssetIssueContract, super::api::TransactionExtention>>,
    method_FreezeBalance: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::FreezeBalanceContract, super::Tron::Transaction>>,
    method_FreezeBalance2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::FreezeBalanceContract, super::api::TransactionExtention>>,
    method_UnfreezeBalance: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::UnfreezeBalanceContract, super::Tron::Transaction>>,
    method_UnfreezeBalance2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::UnfreezeBalanceContract, super::api::TransactionExtention>>,
    method_UnfreezeAsset: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::UnfreezeAssetContract, super::Tron::Transaction>>,
    method_UnfreezeAsset2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::UnfreezeAssetContract, super::api::TransactionExtention>>,
    method_WithdrawBalance: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::WithdrawBalanceContract, super::Tron::Transaction>>,
    method_WithdrawBalance2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::WithdrawBalanceContract, super::api::TransactionExtention>>,
    method_UpdateAsset: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::UpdateAssetContract, super::Tron::Transaction>>,
    method_UpdateAsset2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::UpdateAssetContract, super::api::TransactionExtention>>,
    method_ProposalCreate: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::ProposalCreateContract, super::api::TransactionExtention>>,
    method_ProposalApprove: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::ProposalApproveContract, super::api::TransactionExtention>>,
    method_ProposalDelete: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::ProposalDeleteContract, super::api::TransactionExtention>>,
    method_BuyStorage: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::BuyStorageContract, super::api::TransactionExtention>>,
    method_BuyStorageBytes: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::BuyStorageBytesContract, super::api::TransactionExtention>>,
    method_SellStorage: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::SellStorageContract, super::api::TransactionExtention>>,
    method_ExchangeCreate: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::ExchangeCreateContract, super::api::TransactionExtention>>,
    method_ExchangeInject: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::ExchangeInjectContract, super::api::TransactionExtention>>,
    method_ExchangeWithdraw: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::ExchangeWithdrawContract, super::api::TransactionExtention>>,
    method_ExchangeTransaction: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::ExchangeTransactionContract, super::api::TransactionExtention>>,
    method_ListNodes: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::NodeList>>,
    method_GetAssetIssueByAccount: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Tron::Account, super::api::AssetIssueList>>,
    method_GetAccountNet: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Tron::Account, super::api::AccountNetMessage>>,
    method_GetAccountResource: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Tron::Account, super::api::AccountResourceMessage>>,
    method_GetAssetIssueByName: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::Contract::AssetIssueContract>>,
    method_GetAssetIssueListByName: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::api::AssetIssueList>>,
    method_GetAssetIssueById: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::Contract::AssetIssueContract>>,
    method_GetNowBlock: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::Tron::Block>>,
    method_GetNowBlock2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::BlockExtention>>,
    method_GetBlockByNum: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::NumberMessage, super::Tron::Block>>,
    method_GetBlockByNum2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::NumberMessage, super::api::BlockExtention>>,
    method_GetTransactionCountByBlockNum: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::NumberMessage, super::api::NumberMessage>>,
    method_GetBlockById: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::Tron::Block>>,
    method_GetBlockByLimitNext: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BlockLimit, super::api::BlockList>>,
    method_GetBlockByLimitNext2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BlockLimit, super::api::BlockListExtention>>,
    method_GetBlockByLatestNum: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::NumberMessage, super::api::BlockList>>,
    method_GetBlockByLatestNum2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::NumberMessage, super::api::BlockListExtention>>,
    method_GetTransactionById: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::Tron::Transaction>>,
    method_DeployContract: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::CreateSmartContract, super::api::TransactionExtention>>,
    method_GetContract: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::Tron::SmartContract>>,
    method_TriggerContract: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::TriggerSmartContract, super::api::TransactionExtention>>,
    method_TriggerConstantContract: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::TriggerSmartContract, super::api::TransactionExtention>>,
    method_ClearContractABI: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::ClearABIContract, super::api::TransactionExtention>>,
    method_ListWitnesses: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::WitnessList>>,
    method_GetDelegatedResource: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::DelegatedResourceMessage, super::api::DelegatedResourceList>>,
    method_GetDelegatedResourceAccountIndex: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::Tron::DelegatedResourceAccountIndex>>,
    method_ListProposals: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::ProposalList>>,
    method_GetPaginatedProposalList: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::PaginatedMessage, super::api::ProposalList>>,
    method_GetProposalById: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::Tron::Proposal>>,
    method_ListExchanges: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::ExchangeList>>,
    method_GetPaginatedExchangeList: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::PaginatedMessage, super::api::ExchangeList>>,
    method_GetExchangeById: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::Tron::Exchange>>,
    method_GetChainParameters: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::Tron::ChainParameters>>,
    method_GetAssetIssueList: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::AssetIssueList>>,
    method_GetPaginatedAssetIssueList: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::PaginatedMessage, super::api::AssetIssueList>>,
    method_TotalTransaction: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::NumberMessage>>,
    method_GetNextMaintenanceTime: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::NumberMessage>>,
    method_GetTransactionSign: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Tron::TransactionSign, super::Tron::Transaction>>,
    method_GetTransactionSign2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Tron::TransactionSign, super::api::TransactionExtention>>,
    method_CreateAddress: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::api::BytesMessage>>,
    method_EasyTransferAsset: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EasyTransferAssetMessage, super::api::EasyTransferResponse>>,
    method_EasyTransferAssetByPrivate: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EasyTransferAssetByPrivateMessage, super::api::EasyTransferResponse>>,
    method_EasyTransfer: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EasyTransferMessage, super::api::EasyTransferResponse>>,
    method_EasyTransferByPrivate: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EasyTransferByPrivateMessage, super::api::EasyTransferResponse>>,
    method_GenerateAddress: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::AddressPrKeyPairMessage>>,
    method_GetTransactionInfoById: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::Tron::TransactionInfo>>,
    method_AccountPermissionUpdate: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::AccountPermissionUpdateContract, super::api::TransactionExtention>>,
    method_AddSign: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Tron::TransactionSign, super::api::TransactionExtention>>,
    method_GetTransactionSignWeight: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Tron::Transaction, super::api::TransactionSignWeight>>,
    method_GetTransactionApprovedList: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Tron::Transaction, super::api::TransactionApprovedList>>,
    method_GetNodeInfo: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::Tron::NodeInfo>>,
    method_CreateShieldedTransaction: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::PrivateParameters, super::api::TransactionExtention>>,
    method_GetMerkleTreeVoucherInfo: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::OutputPointInfo, super::Contract::IncrementalMerkleVoucherInfo>>,
    method_ScanNoteByIvk: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::IvkDecryptParameters, super::api::DecryptNotes>>,
    method_ScanAndMarkNoteByIvk: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::IvkDecryptAndMarkParameters, super::api::DecryptNotesMarked>>,
    method_ScanNoteByOvk: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::OvkDecryptParameters, super::api::DecryptNotes>>,
    method_GetSpendingKey: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::BytesMessage>>,
    method_GetExpandedSpendingKey: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::api::ExpandedSpendingKeyMessage>>,
    method_GetAkFromAsk: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::api::BytesMessage>>,
    method_GetNkFromNsk: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::api::BytesMessage>>,
    method_GetIncomingViewingKey: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::ViewingKeyMessage, super::api::IncomingViewingKeyMessage>>,
    method_GetDiversifier: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::DiversifierMessage>>,
    method_GetNewShieldedAddress: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::ShieldedAddressInfo>>,
    method_GetZenPaymentAddress: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::IncomingViewingKeyDiversifierMessage, super::api::PaymentAddressMessage>>,
    method_GetRcm: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::BytesMessage>>,
    method_IsSpend: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::NoteParameters, super::api::SpendResult>>,
    method_CreateShieldedTransactionWithoutSpendAuthSig: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::PrivateParametersWithoutAsk, super::api::TransactionExtention>>,
    method_GetShieldTransactionHash: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Tron::Transaction, super::api::BytesMessage>>,
    method_CreateSpendAuthSig: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::SpendAuthSigParameters, super::api::BytesMessage>>,
    method_CreateShieldNullifier: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::NfParameters, super::api::BytesMessage>>,
    method_GetRewardInfo: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::api::NumberMessage>>,
    method_GetBrokerageInfo: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::api::NumberMessage>>,
    method_UpdateBrokerage: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::UpdateBrokerageContract, super::api::TransactionExtention>>,
}

impl ::grpc::ClientStub for WalletClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        WalletClient {
            grpc_client: grpc_client,
            method_GetAccount: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetAccount".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetAccountById: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetAccountById".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CreateTransaction: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/CreateTransaction".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CreateTransaction2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/CreateTransaction2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_BroadcastTransaction: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/BroadcastTransaction".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_UpdateAccount: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/UpdateAccount".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_SetAccountId: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/SetAccountId".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_UpdateAccount2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/UpdateAccount2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_VoteWitnessAccount: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/VoteWitnessAccount".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_UpdateSetting: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/UpdateSetting".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_UpdateEnergyLimit: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/UpdateEnergyLimit".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_VoteWitnessAccount2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/VoteWitnessAccount2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CreateAssetIssue: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/CreateAssetIssue".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CreateAssetIssue2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/CreateAssetIssue2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_UpdateWitness: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/UpdateWitness".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_UpdateWitness2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/UpdateWitness2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CreateAccount: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/CreateAccount".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CreateAccount2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/CreateAccount2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CreateWitness: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/CreateWitness".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CreateWitness2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/CreateWitness2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_TransferAsset: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/TransferAsset".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_TransferAsset2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/TransferAsset2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ParticipateAssetIssue: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ParticipateAssetIssue".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ParticipateAssetIssue2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ParticipateAssetIssue2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_FreezeBalance: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/FreezeBalance".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_FreezeBalance2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/FreezeBalance2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_UnfreezeBalance: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/UnfreezeBalance".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_UnfreezeBalance2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/UnfreezeBalance2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_UnfreezeAsset: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/UnfreezeAsset".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_UnfreezeAsset2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/UnfreezeAsset2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_WithdrawBalance: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/WithdrawBalance".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_WithdrawBalance2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/WithdrawBalance2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_UpdateAsset: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/UpdateAsset".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_UpdateAsset2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/UpdateAsset2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ProposalCreate: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ProposalCreate".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ProposalApprove: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ProposalApprove".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ProposalDelete: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ProposalDelete".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_BuyStorage: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/BuyStorage".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_BuyStorageBytes: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/BuyStorageBytes".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_SellStorage: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/SellStorage".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ExchangeCreate: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ExchangeCreate".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ExchangeInject: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ExchangeInject".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ExchangeWithdraw: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ExchangeWithdraw".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ExchangeTransaction: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ExchangeTransaction".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ListNodes: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ListNodes".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetAssetIssueByAccount: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetAssetIssueByAccount".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetAccountNet: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetAccountNet".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetAccountResource: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetAccountResource".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetAssetIssueByName: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetAssetIssueByName".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetAssetIssueListByName: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetAssetIssueListByName".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetAssetIssueById: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetAssetIssueById".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetNowBlock: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetNowBlock".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetNowBlock2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetNowBlock2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetBlockByNum: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetBlockByNum".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetBlockByNum2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetBlockByNum2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetTransactionCountByBlockNum: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetTransactionCountByBlockNum".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetBlockById: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetBlockById".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetBlockByLimitNext: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetBlockByLimitNext".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetBlockByLimitNext2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetBlockByLimitNext2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetBlockByLatestNum: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetBlockByLatestNum".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetBlockByLatestNum2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetBlockByLatestNum2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetTransactionById: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetTransactionById".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_DeployContract: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/DeployContract".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetContract: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetContract".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_TriggerContract: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/TriggerContract".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_TriggerConstantContract: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/TriggerConstantContract".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ClearContractABI: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ClearContractABI".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ListWitnesses: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ListWitnesses".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetDelegatedResource: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetDelegatedResource".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetDelegatedResourceAccountIndex: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetDelegatedResourceAccountIndex".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ListProposals: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ListProposals".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetPaginatedProposalList: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetPaginatedProposalList".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetProposalById: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetProposalById".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ListExchanges: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ListExchanges".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetPaginatedExchangeList: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetPaginatedExchangeList".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetExchangeById: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetExchangeById".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetChainParameters: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetChainParameters".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetAssetIssueList: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetAssetIssueList".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetPaginatedAssetIssueList: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetPaginatedAssetIssueList".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_TotalTransaction: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/TotalTransaction".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetNextMaintenanceTime: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetNextMaintenanceTime".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetTransactionSign: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetTransactionSign".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetTransactionSign2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetTransactionSign2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CreateAddress: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/CreateAddress".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_EasyTransferAsset: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/EasyTransferAsset".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_EasyTransferAssetByPrivate: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/EasyTransferAssetByPrivate".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_EasyTransfer: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/EasyTransfer".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_EasyTransferByPrivate: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/EasyTransferByPrivate".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GenerateAddress: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GenerateAddress".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetTransactionInfoById: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetTransactionInfoById".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_AccountPermissionUpdate: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/AccountPermissionUpdate".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_AddSign: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/AddSign".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetTransactionSignWeight: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetTransactionSignWeight".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetTransactionApprovedList: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetTransactionApprovedList".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetNodeInfo: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetNodeInfo".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CreateShieldedTransaction: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/CreateShieldedTransaction".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetMerkleTreeVoucherInfo: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetMerkleTreeVoucherInfo".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ScanNoteByIvk: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ScanNoteByIvk".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ScanAndMarkNoteByIvk: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ScanAndMarkNoteByIvk".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ScanNoteByOvk: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/ScanNoteByOvk".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetSpendingKey: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetSpendingKey".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetExpandedSpendingKey: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetExpandedSpendingKey".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetAkFromAsk: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetAkFromAsk".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetNkFromNsk: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetNkFromNsk".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetIncomingViewingKey: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetIncomingViewingKey".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetDiversifier: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetDiversifier".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetNewShieldedAddress: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetNewShieldedAddress".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetZenPaymentAddress: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetZenPaymentAddress".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetRcm: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetRcm".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_IsSpend: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/IsSpend".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CreateShieldedTransactionWithoutSpendAuthSig: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/CreateShieldedTransactionWithoutSpendAuthSig".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetShieldTransactionHash: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetShieldTransactionHash".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CreateSpendAuthSig: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/CreateSpendAuthSig".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_CreateShieldNullifier: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/CreateShieldNullifier".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetRewardInfo: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetRewardInfo".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetBrokerageInfo: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/GetBrokerageInfo".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_UpdateBrokerage: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Wallet/UpdateBrokerage".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl Wallet for WalletClient {
    fn get_account(&self, o: ::grpc::RequestOptions, p: super::Tron::Account) -> ::grpc::SingleResponse<super::Tron::Account> {
        self.grpc_client.call_unary(o, p, self.method_GetAccount.clone())
    }

    fn get_account_by_id(&self, o: ::grpc::RequestOptions, p: super::Tron::Account) -> ::grpc::SingleResponse<super::Tron::Account> {
        self.grpc_client.call_unary(o, p, self.method_GetAccountById.clone())
    }

    fn create_transaction(&self, o: ::grpc::RequestOptions, p: super::Contract::TransferContract) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_CreateTransaction.clone())
    }

    fn create_transaction2(&self, o: ::grpc::RequestOptions, p: super::Contract::TransferContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_CreateTransaction2.clone())
    }

    fn broadcast_transaction(&self, o: ::grpc::RequestOptions, p: super::Tron::Transaction) -> ::grpc::SingleResponse<super::api::Return> {
        self.grpc_client.call_unary(o, p, self.method_BroadcastTransaction.clone())
    }

    fn update_account(&self, o: ::grpc::RequestOptions, p: super::Contract::AccountUpdateContract) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_UpdateAccount.clone())
    }

    fn set_account_id(&self, o: ::grpc::RequestOptions, p: super::Contract::SetAccountIdContract) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_SetAccountId.clone())
    }

    fn update_account2(&self, o: ::grpc::RequestOptions, p: super::Contract::AccountUpdateContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_UpdateAccount2.clone())
    }

    fn vote_witness_account(&self, o: ::grpc::RequestOptions, p: super::Contract::VoteWitnessContract) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_VoteWitnessAccount.clone())
    }

    fn update_setting(&self, o: ::grpc::RequestOptions, p: super::Contract::UpdateSettingContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_UpdateSetting.clone())
    }

    fn update_energy_limit(&self, o: ::grpc::RequestOptions, p: super::Contract::UpdateEnergyLimitContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_UpdateEnergyLimit.clone())
    }

    fn vote_witness_account2(&self, o: ::grpc::RequestOptions, p: super::Contract::VoteWitnessContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_VoteWitnessAccount2.clone())
    }

    fn create_asset_issue(&self, o: ::grpc::RequestOptions, p: super::Contract::AssetIssueContract) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_CreateAssetIssue.clone())
    }

    fn create_asset_issue2(&self, o: ::grpc::RequestOptions, p: super::Contract::AssetIssueContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_CreateAssetIssue2.clone())
    }

    fn update_witness(&self, o: ::grpc::RequestOptions, p: super::Contract::WitnessUpdateContract) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_UpdateWitness.clone())
    }

    fn update_witness2(&self, o: ::grpc::RequestOptions, p: super::Contract::WitnessUpdateContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_UpdateWitness2.clone())
    }

    fn create_account(&self, o: ::grpc::RequestOptions, p: super::Contract::AccountCreateContract) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_CreateAccount.clone())
    }

    fn create_account2(&self, o: ::grpc::RequestOptions, p: super::Contract::AccountCreateContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_CreateAccount2.clone())
    }

    fn create_witness(&self, o: ::grpc::RequestOptions, p: super::Contract::WitnessCreateContract) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_CreateWitness.clone())
    }

    fn create_witness2(&self, o: ::grpc::RequestOptions, p: super::Contract::WitnessCreateContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_CreateWitness2.clone())
    }

    fn transfer_asset(&self, o: ::grpc::RequestOptions, p: super::Contract::TransferAssetContract) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_TransferAsset.clone())
    }

    fn transfer_asset2(&self, o: ::grpc::RequestOptions, p: super::Contract::TransferAssetContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_TransferAsset2.clone())
    }

    fn participate_asset_issue(&self, o: ::grpc::RequestOptions, p: super::Contract::ParticipateAssetIssueContract) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_ParticipateAssetIssue.clone())
    }

    fn participate_asset_issue2(&self, o: ::grpc::RequestOptions, p: super::Contract::ParticipateAssetIssueContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_ParticipateAssetIssue2.clone())
    }

    fn freeze_balance(&self, o: ::grpc::RequestOptions, p: super::Contract::FreezeBalanceContract) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_FreezeBalance.clone())
    }

    fn freeze_balance2(&self, o: ::grpc::RequestOptions, p: super::Contract::FreezeBalanceContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_FreezeBalance2.clone())
    }

    fn unfreeze_balance(&self, o: ::grpc::RequestOptions, p: super::Contract::UnfreezeBalanceContract) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_UnfreezeBalance.clone())
    }

    fn unfreeze_balance2(&self, o: ::grpc::RequestOptions, p: super::Contract::UnfreezeBalanceContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_UnfreezeBalance2.clone())
    }

    fn unfreeze_asset(&self, o: ::grpc::RequestOptions, p: super::Contract::UnfreezeAssetContract) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_UnfreezeAsset.clone())
    }

    fn unfreeze_asset2(&self, o: ::grpc::RequestOptions, p: super::Contract::UnfreezeAssetContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_UnfreezeAsset2.clone())
    }

    fn withdraw_balance(&self, o: ::grpc::RequestOptions, p: super::Contract::WithdrawBalanceContract) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_WithdrawBalance.clone())
    }

    fn withdraw_balance2(&self, o: ::grpc::RequestOptions, p: super::Contract::WithdrawBalanceContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_WithdrawBalance2.clone())
    }

    fn update_asset(&self, o: ::grpc::RequestOptions, p: super::Contract::UpdateAssetContract) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_UpdateAsset.clone())
    }

    fn update_asset2(&self, o: ::grpc::RequestOptions, p: super::Contract::UpdateAssetContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_UpdateAsset2.clone())
    }

    fn proposal_create(&self, o: ::grpc::RequestOptions, p: super::Contract::ProposalCreateContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_ProposalCreate.clone())
    }

    fn proposal_approve(&self, o: ::grpc::RequestOptions, p: super::Contract::ProposalApproveContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_ProposalApprove.clone())
    }

    fn proposal_delete(&self, o: ::grpc::RequestOptions, p: super::Contract::ProposalDeleteContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_ProposalDelete.clone())
    }

    fn buy_storage(&self, o: ::grpc::RequestOptions, p: super::Contract::BuyStorageContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_BuyStorage.clone())
    }

    fn buy_storage_bytes(&self, o: ::grpc::RequestOptions, p: super::Contract::BuyStorageBytesContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_BuyStorageBytes.clone())
    }

    fn sell_storage(&self, o: ::grpc::RequestOptions, p: super::Contract::SellStorageContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_SellStorage.clone())
    }

    fn exchange_create(&self, o: ::grpc::RequestOptions, p: super::Contract::ExchangeCreateContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_ExchangeCreate.clone())
    }

    fn exchange_inject(&self, o: ::grpc::RequestOptions, p: super::Contract::ExchangeInjectContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_ExchangeInject.clone())
    }

    fn exchange_withdraw(&self, o: ::grpc::RequestOptions, p: super::Contract::ExchangeWithdrawContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_ExchangeWithdraw.clone())
    }

    fn exchange_transaction(&self, o: ::grpc::RequestOptions, p: super::Contract::ExchangeTransactionContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_ExchangeTransaction.clone())
    }

    fn list_nodes(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::NodeList> {
        self.grpc_client.call_unary(o, p, self.method_ListNodes.clone())
    }

    fn get_asset_issue_by_account(&self, o: ::grpc::RequestOptions, p: super::Tron::Account) -> ::grpc::SingleResponse<super::api::AssetIssueList> {
        self.grpc_client.call_unary(o, p, self.method_GetAssetIssueByAccount.clone())
    }

    fn get_account_net(&self, o: ::grpc::RequestOptions, p: super::Tron::Account) -> ::grpc::SingleResponse<super::api::AccountNetMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetAccountNet.clone())
    }

    fn get_account_resource(&self, o: ::grpc::RequestOptions, p: super::Tron::Account) -> ::grpc::SingleResponse<super::api::AccountResourceMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetAccountResource.clone())
    }

    fn get_asset_issue_by_name(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Contract::AssetIssueContract> {
        self.grpc_client.call_unary(o, p, self.method_GetAssetIssueByName.clone())
    }

    fn get_asset_issue_list_by_name(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::AssetIssueList> {
        self.grpc_client.call_unary(o, p, self.method_GetAssetIssueListByName.clone())
    }

    fn get_asset_issue_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Contract::AssetIssueContract> {
        self.grpc_client.call_unary(o, p, self.method_GetAssetIssueById.clone())
    }

    fn get_now_block(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::Tron::Block> {
        self.grpc_client.call_unary(o, p, self.method_GetNowBlock.clone())
    }

    fn get_now_block2(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::BlockExtention> {
        self.grpc_client.call_unary(o, p, self.method_GetNowBlock2.clone())
    }

    fn get_block_by_num(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::Tron::Block> {
        self.grpc_client.call_unary(o, p, self.method_GetBlockByNum.clone())
    }

    fn get_block_by_num2(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::api::BlockExtention> {
        self.grpc_client.call_unary(o, p, self.method_GetBlockByNum2.clone())
    }

    fn get_transaction_count_by_block_num(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::api::NumberMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetTransactionCountByBlockNum.clone())
    }

    fn get_block_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::Block> {
        self.grpc_client.call_unary(o, p, self.method_GetBlockById.clone())
    }

    fn get_block_by_limit_next(&self, o: ::grpc::RequestOptions, p: super::api::BlockLimit) -> ::grpc::SingleResponse<super::api::BlockList> {
        self.grpc_client.call_unary(o, p, self.method_GetBlockByLimitNext.clone())
    }

    fn get_block_by_limit_next2(&self, o: ::grpc::RequestOptions, p: super::api::BlockLimit) -> ::grpc::SingleResponse<super::api::BlockListExtention> {
        self.grpc_client.call_unary(o, p, self.method_GetBlockByLimitNext2.clone())
    }

    fn get_block_by_latest_num(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::api::BlockList> {
        self.grpc_client.call_unary(o, p, self.method_GetBlockByLatestNum.clone())
    }

    fn get_block_by_latest_num2(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::api::BlockListExtention> {
        self.grpc_client.call_unary(o, p, self.method_GetBlockByLatestNum2.clone())
    }

    fn get_transaction_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_GetTransactionById.clone())
    }

    fn deploy_contract(&self, o: ::grpc::RequestOptions, p: super::Contract::CreateSmartContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_DeployContract.clone())
    }

    fn get_contract(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::SmartContract> {
        self.grpc_client.call_unary(o, p, self.method_GetContract.clone())
    }

    fn trigger_contract(&self, o: ::grpc::RequestOptions, p: super::Contract::TriggerSmartContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_TriggerContract.clone())
    }

    fn trigger_constant_contract(&self, o: ::grpc::RequestOptions, p: super::Contract::TriggerSmartContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_TriggerConstantContract.clone())
    }

    fn clear_contract_abi(&self, o: ::grpc::RequestOptions, p: super::Contract::ClearABIContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_ClearContractABI.clone())
    }

    fn list_witnesses(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::WitnessList> {
        self.grpc_client.call_unary(o, p, self.method_ListWitnesses.clone())
    }

    fn get_delegated_resource(&self, o: ::grpc::RequestOptions, p: super::api::DelegatedResourceMessage) -> ::grpc::SingleResponse<super::api::DelegatedResourceList> {
        self.grpc_client.call_unary(o, p, self.method_GetDelegatedResource.clone())
    }

    fn get_delegated_resource_account_index(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::DelegatedResourceAccountIndex> {
        self.grpc_client.call_unary(o, p, self.method_GetDelegatedResourceAccountIndex.clone())
    }

    fn list_proposals(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::ProposalList> {
        self.grpc_client.call_unary(o, p, self.method_ListProposals.clone())
    }

    fn get_paginated_proposal_list(&self, o: ::grpc::RequestOptions, p: super::api::PaginatedMessage) -> ::grpc::SingleResponse<super::api::ProposalList> {
        self.grpc_client.call_unary(o, p, self.method_GetPaginatedProposalList.clone())
    }

    fn get_proposal_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::Proposal> {
        self.grpc_client.call_unary(o, p, self.method_GetProposalById.clone())
    }

    fn list_exchanges(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::ExchangeList> {
        self.grpc_client.call_unary(o, p, self.method_ListExchanges.clone())
    }

    fn get_paginated_exchange_list(&self, o: ::grpc::RequestOptions, p: super::api::PaginatedMessage) -> ::grpc::SingleResponse<super::api::ExchangeList> {
        self.grpc_client.call_unary(o, p, self.method_GetPaginatedExchangeList.clone())
    }

    fn get_exchange_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::Exchange> {
        self.grpc_client.call_unary(o, p, self.method_GetExchangeById.clone())
    }

    fn get_chain_parameters(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::Tron::ChainParameters> {
        self.grpc_client.call_unary(o, p, self.method_GetChainParameters.clone())
    }

    fn get_asset_issue_list(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::AssetIssueList> {
        self.grpc_client.call_unary(o, p, self.method_GetAssetIssueList.clone())
    }

    fn get_paginated_asset_issue_list(&self, o: ::grpc::RequestOptions, p: super::api::PaginatedMessage) -> ::grpc::SingleResponse<super::api::AssetIssueList> {
        self.grpc_client.call_unary(o, p, self.method_GetPaginatedAssetIssueList.clone())
    }

    fn total_transaction(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::NumberMessage> {
        self.grpc_client.call_unary(o, p, self.method_TotalTransaction.clone())
    }

    fn get_next_maintenance_time(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::NumberMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetNextMaintenanceTime.clone())
    }

    fn get_transaction_sign(&self, o: ::grpc::RequestOptions, p: super::Tron::TransactionSign) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_GetTransactionSign.clone())
    }

    fn get_transaction_sign2(&self, o: ::grpc::RequestOptions, p: super::Tron::TransactionSign) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_GetTransactionSign2.clone())
    }

    fn create_address(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::BytesMessage> {
        self.grpc_client.call_unary(o, p, self.method_CreateAddress.clone())
    }

    fn easy_transfer_asset(&self, o: ::grpc::RequestOptions, p: super::api::EasyTransferAssetMessage) -> ::grpc::SingleResponse<super::api::EasyTransferResponse> {
        self.grpc_client.call_unary(o, p, self.method_EasyTransferAsset.clone())
    }

    fn easy_transfer_asset_by_private(&self, o: ::grpc::RequestOptions, p: super::api::EasyTransferAssetByPrivateMessage) -> ::grpc::SingleResponse<super::api::EasyTransferResponse> {
        self.grpc_client.call_unary(o, p, self.method_EasyTransferAssetByPrivate.clone())
    }

    fn easy_transfer(&self, o: ::grpc::RequestOptions, p: super::api::EasyTransferMessage) -> ::grpc::SingleResponse<super::api::EasyTransferResponse> {
        self.grpc_client.call_unary(o, p, self.method_EasyTransfer.clone())
    }

    fn easy_transfer_by_private(&self, o: ::grpc::RequestOptions, p: super::api::EasyTransferByPrivateMessage) -> ::grpc::SingleResponse<super::api::EasyTransferResponse> {
        self.grpc_client.call_unary(o, p, self.method_EasyTransferByPrivate.clone())
    }

    fn generate_address(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::AddressPrKeyPairMessage> {
        self.grpc_client.call_unary(o, p, self.method_GenerateAddress.clone())
    }

    fn get_transaction_info_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::TransactionInfo> {
        self.grpc_client.call_unary(o, p, self.method_GetTransactionInfoById.clone())
    }

    fn account_permission_update(&self, o: ::grpc::RequestOptions, p: super::Contract::AccountPermissionUpdateContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_AccountPermissionUpdate.clone())
    }

    fn add_sign(&self, o: ::grpc::RequestOptions, p: super::Tron::TransactionSign) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_AddSign.clone())
    }

    fn get_transaction_sign_weight(&self, o: ::grpc::RequestOptions, p: super::Tron::Transaction) -> ::grpc::SingleResponse<super::api::TransactionSignWeight> {
        self.grpc_client.call_unary(o, p, self.method_GetTransactionSignWeight.clone())
    }

    fn get_transaction_approved_list(&self, o: ::grpc::RequestOptions, p: super::Tron::Transaction) -> ::grpc::SingleResponse<super::api::TransactionApprovedList> {
        self.grpc_client.call_unary(o, p, self.method_GetTransactionApprovedList.clone())
    }

    fn get_node_info(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::Tron::NodeInfo> {
        self.grpc_client.call_unary(o, p, self.method_GetNodeInfo.clone())
    }

    fn create_shielded_transaction(&self, o: ::grpc::RequestOptions, p: super::api::PrivateParameters) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_CreateShieldedTransaction.clone())
    }

    fn get_merkle_tree_voucher_info(&self, o: ::grpc::RequestOptions, p: super::Contract::OutputPointInfo) -> ::grpc::SingleResponse<super::Contract::IncrementalMerkleVoucherInfo> {
        self.grpc_client.call_unary(o, p, self.method_GetMerkleTreeVoucherInfo.clone())
    }

    fn scan_note_by_ivk(&self, o: ::grpc::RequestOptions, p: super::api::IvkDecryptParameters) -> ::grpc::SingleResponse<super::api::DecryptNotes> {
        self.grpc_client.call_unary(o, p, self.method_ScanNoteByIvk.clone())
    }

    fn scan_and_mark_note_by_ivk(&self, o: ::grpc::RequestOptions, p: super::api::IvkDecryptAndMarkParameters) -> ::grpc::SingleResponse<super::api::DecryptNotesMarked> {
        self.grpc_client.call_unary(o, p, self.method_ScanAndMarkNoteByIvk.clone())
    }

    fn scan_note_by_ovk(&self, o: ::grpc::RequestOptions, p: super::api::OvkDecryptParameters) -> ::grpc::SingleResponse<super::api::DecryptNotes> {
        self.grpc_client.call_unary(o, p, self.method_ScanNoteByOvk.clone())
    }

    fn get_spending_key(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::BytesMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetSpendingKey.clone())
    }

    fn get_expanded_spending_key(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::ExpandedSpendingKeyMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetExpandedSpendingKey.clone())
    }

    fn get_ak_from_ask(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::BytesMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetAkFromAsk.clone())
    }

    fn get_nk_from_nsk(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::BytesMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetNkFromNsk.clone())
    }

    fn get_incoming_viewing_key(&self, o: ::grpc::RequestOptions, p: super::api::ViewingKeyMessage) -> ::grpc::SingleResponse<super::api::IncomingViewingKeyMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetIncomingViewingKey.clone())
    }

    fn get_diversifier(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::DiversifierMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetDiversifier.clone())
    }

    fn get_new_shielded_address(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::ShieldedAddressInfo> {
        self.grpc_client.call_unary(o, p, self.method_GetNewShieldedAddress.clone())
    }

    fn get_zen_payment_address(&self, o: ::grpc::RequestOptions, p: super::api::IncomingViewingKeyDiversifierMessage) -> ::grpc::SingleResponse<super::api::PaymentAddressMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetZenPaymentAddress.clone())
    }

    fn get_rcm(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::BytesMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetRcm.clone())
    }

    fn is_spend(&self, o: ::grpc::RequestOptions, p: super::api::NoteParameters) -> ::grpc::SingleResponse<super::api::SpendResult> {
        self.grpc_client.call_unary(o, p, self.method_IsSpend.clone())
    }

    fn create_shielded_transaction_without_spend_auth_sig(&self, o: ::grpc::RequestOptions, p: super::api::PrivateParametersWithoutAsk) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_CreateShieldedTransactionWithoutSpendAuthSig.clone())
    }

    fn get_shield_transaction_hash(&self, o: ::grpc::RequestOptions, p: super::Tron::Transaction) -> ::grpc::SingleResponse<super::api::BytesMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetShieldTransactionHash.clone())
    }

    fn create_spend_auth_sig(&self, o: ::grpc::RequestOptions, p: super::api::SpendAuthSigParameters) -> ::grpc::SingleResponse<super::api::BytesMessage> {
        self.grpc_client.call_unary(o, p, self.method_CreateSpendAuthSig.clone())
    }

    fn create_shield_nullifier(&self, o: ::grpc::RequestOptions, p: super::api::NfParameters) -> ::grpc::SingleResponse<super::api::BytesMessage> {
        self.grpc_client.call_unary(o, p, self.method_CreateShieldNullifier.clone())
    }

    fn get_reward_info(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::NumberMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetRewardInfo.clone())
    }

    fn get_brokerage_info(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::NumberMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetBrokerageInfo.clone())
    }

    fn update_brokerage(&self, o: ::grpc::RequestOptions, p: super::Contract::UpdateBrokerageContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_UpdateBrokerage.clone())
    }
}

// server

pub struct WalletServer;


impl WalletServer {
    pub fn new_service_def<H : Wallet + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/protocol.Wallet",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetAccount".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_account(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetAccountById".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_account_by_id(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/CreateTransaction".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.create_transaction(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/CreateTransaction2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.create_transaction2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/BroadcastTransaction".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.broadcast_transaction(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/UpdateAccount".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.update_account(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/SetAccountId".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.set_account_id(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/UpdateAccount2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.update_account2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/VoteWitnessAccount".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.vote_witness_account(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/UpdateSetting".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.update_setting(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/UpdateEnergyLimit".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.update_energy_limit(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/VoteWitnessAccount2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.vote_witness_account2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/CreateAssetIssue".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.create_asset_issue(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/CreateAssetIssue2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.create_asset_issue2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/UpdateWitness".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.update_witness(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/UpdateWitness2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.update_witness2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/CreateAccount".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.create_account(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/CreateAccount2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.create_account2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/CreateWitness".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.create_witness(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/CreateWitness2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.create_witness2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/TransferAsset".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.transfer_asset(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/TransferAsset2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.transfer_asset2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ParticipateAssetIssue".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.participate_asset_issue(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ParticipateAssetIssue2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.participate_asset_issue2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/FreezeBalance".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.freeze_balance(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/FreezeBalance2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.freeze_balance2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/UnfreezeBalance".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.unfreeze_balance(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/UnfreezeBalance2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.unfreeze_balance2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/UnfreezeAsset".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.unfreeze_asset(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/UnfreezeAsset2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.unfreeze_asset2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/WithdrawBalance".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.withdraw_balance(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/WithdrawBalance2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.withdraw_balance2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/UpdateAsset".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.update_asset(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/UpdateAsset2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.update_asset2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ProposalCreate".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.proposal_create(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ProposalApprove".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.proposal_approve(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ProposalDelete".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.proposal_delete(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/BuyStorage".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.buy_storage(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/BuyStorageBytes".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.buy_storage_bytes(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/SellStorage".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.sell_storage(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ExchangeCreate".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.exchange_create(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ExchangeInject".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.exchange_inject(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ExchangeWithdraw".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.exchange_withdraw(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ExchangeTransaction".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.exchange_transaction(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ListNodes".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.list_nodes(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetAssetIssueByAccount".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_asset_issue_by_account(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetAccountNet".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_account_net(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetAccountResource".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_account_resource(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetAssetIssueByName".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_asset_issue_by_name(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetAssetIssueListByName".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_asset_issue_list_by_name(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetAssetIssueById".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_asset_issue_by_id(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetNowBlock".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_now_block(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetNowBlock2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_now_block2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetBlockByNum".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_block_by_num(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetBlockByNum2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_block_by_num2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetTransactionCountByBlockNum".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_transaction_count_by_block_num(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetBlockById".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_block_by_id(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetBlockByLimitNext".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_block_by_limit_next(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetBlockByLimitNext2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_block_by_limit_next2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetBlockByLatestNum".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_block_by_latest_num(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetBlockByLatestNum2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_block_by_latest_num2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetTransactionById".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_transaction_by_id(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/DeployContract".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.deploy_contract(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetContract".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_contract(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/TriggerContract".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.trigger_contract(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/TriggerConstantContract".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.trigger_constant_contract(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ClearContractABI".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.clear_contract_abi(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ListWitnesses".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.list_witnesses(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetDelegatedResource".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_delegated_resource(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetDelegatedResourceAccountIndex".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_delegated_resource_account_index(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ListProposals".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.list_proposals(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetPaginatedProposalList".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_paginated_proposal_list(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetProposalById".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_proposal_by_id(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ListExchanges".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.list_exchanges(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetPaginatedExchangeList".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_paginated_exchange_list(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetExchangeById".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_exchange_by_id(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetChainParameters".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_chain_parameters(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetAssetIssueList".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_asset_issue_list(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetPaginatedAssetIssueList".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_paginated_asset_issue_list(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/TotalTransaction".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.total_transaction(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetNextMaintenanceTime".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_next_maintenance_time(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetTransactionSign".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_transaction_sign(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetTransactionSign2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_transaction_sign2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/CreateAddress".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.create_address(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/EasyTransferAsset".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.easy_transfer_asset(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/EasyTransferAssetByPrivate".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.easy_transfer_asset_by_private(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/EasyTransfer".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.easy_transfer(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/EasyTransferByPrivate".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.easy_transfer_by_private(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GenerateAddress".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.generate_address(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetTransactionInfoById".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_transaction_info_by_id(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/AccountPermissionUpdate".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.account_permission_update(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/AddSign".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.add_sign(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetTransactionSignWeight".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_transaction_sign_weight(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetTransactionApprovedList".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_transaction_approved_list(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetNodeInfo".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_node_info(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/CreateShieldedTransaction".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.create_shielded_transaction(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetMerkleTreeVoucherInfo".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_merkle_tree_voucher_info(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ScanNoteByIvk".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.scan_note_by_ivk(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ScanAndMarkNoteByIvk".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.scan_and_mark_note_by_ivk(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/ScanNoteByOvk".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.scan_note_by_ovk(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetSpendingKey".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_spending_key(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetExpandedSpendingKey".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_expanded_spending_key(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetAkFromAsk".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_ak_from_ask(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetNkFromNsk".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_nk_from_nsk(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetIncomingViewingKey".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_incoming_viewing_key(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetDiversifier".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_diversifier(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetNewShieldedAddress".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_new_shielded_address(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetZenPaymentAddress".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_zen_payment_address(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetRcm".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_rcm(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/IsSpend".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.is_spend(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/CreateShieldedTransactionWithoutSpendAuthSig".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.create_shielded_transaction_without_spend_auth_sig(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetShieldTransactionHash".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_shield_transaction_hash(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/CreateSpendAuthSig".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.create_spend_auth_sig(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/CreateShieldNullifier".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.create_shield_nullifier(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetRewardInfo".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_reward_info(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/GetBrokerageInfo".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_brokerage_info(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Wallet/UpdateBrokerage".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.update_brokerage(o, p))
                    },
                ),
            ],
        )
    }
}

// interface

pub trait WalletSolidity {
    fn get_account(&self, o: ::grpc::RequestOptions, p: super::Tron::Account) -> ::grpc::SingleResponse<super::Tron::Account>;

    fn get_account_by_id(&self, o: ::grpc::RequestOptions, p: super::Tron::Account) -> ::grpc::SingleResponse<super::Tron::Account>;

    fn list_witnesses(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::WitnessList>;

    fn get_asset_issue_list(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::AssetIssueList>;

    fn get_paginated_asset_issue_list(&self, o: ::grpc::RequestOptions, p: super::api::PaginatedMessage) -> ::grpc::SingleResponse<super::api::AssetIssueList>;

    fn get_asset_issue_by_name(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Contract::AssetIssueContract>;

    fn get_asset_issue_list_by_name(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::AssetIssueList>;

    fn get_asset_issue_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Contract::AssetIssueContract>;

    fn get_now_block(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::Tron::Block>;

    fn get_now_block2(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::BlockExtention>;

    fn get_block_by_num(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::Tron::Block>;

    fn get_block_by_num2(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::api::BlockExtention>;

    fn get_transaction_count_by_block_num(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::api::NumberMessage>;

    fn get_delegated_resource(&self, o: ::grpc::RequestOptions, p: super::api::DelegatedResourceMessage) -> ::grpc::SingleResponse<super::api::DelegatedResourceList>;

    fn get_delegated_resource_account_index(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::DelegatedResourceAccountIndex>;

    fn get_exchange_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::Exchange>;

    fn list_exchanges(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::ExchangeList>;

    fn get_transaction_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::Transaction>;

    fn get_transaction_info_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::TransactionInfo>;

    fn generate_address(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::AddressPrKeyPairMessage>;

    fn get_merkle_tree_voucher_info(&self, o: ::grpc::RequestOptions, p: super::Contract::OutputPointInfo) -> ::grpc::SingleResponse<super::Contract::IncrementalMerkleVoucherInfo>;

    fn scan_note_by_ivk(&self, o: ::grpc::RequestOptions, p: super::api::IvkDecryptParameters) -> ::grpc::SingleResponse<super::api::DecryptNotes>;

    fn scan_and_mark_note_by_ivk(&self, o: ::grpc::RequestOptions, p: super::api::IvkDecryptAndMarkParameters) -> ::grpc::SingleResponse<super::api::DecryptNotesMarked>;

    fn scan_note_by_ovk(&self, o: ::grpc::RequestOptions, p: super::api::OvkDecryptParameters) -> ::grpc::SingleResponse<super::api::DecryptNotes>;

    fn is_spend(&self, o: ::grpc::RequestOptions, p: super::api::NoteParameters) -> ::grpc::SingleResponse<super::api::SpendResult>;

    fn trigger_constant_contract(&self, o: ::grpc::RequestOptions, p: super::Contract::TriggerSmartContract) -> ::grpc::SingleResponse<super::api::TransactionExtention>;

    fn get_reward_info(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::NumberMessage>;

    fn get_brokerage_info(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::NumberMessage>;
}

// client

pub struct WalletSolidityClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
    method_GetAccount: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Tron::Account, super::Tron::Account>>,
    method_GetAccountById: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Tron::Account, super::Tron::Account>>,
    method_ListWitnesses: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::WitnessList>>,
    method_GetAssetIssueList: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::AssetIssueList>>,
    method_GetPaginatedAssetIssueList: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::PaginatedMessage, super::api::AssetIssueList>>,
    method_GetAssetIssueByName: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::Contract::AssetIssueContract>>,
    method_GetAssetIssueListByName: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::api::AssetIssueList>>,
    method_GetAssetIssueById: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::Contract::AssetIssueContract>>,
    method_GetNowBlock: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::Tron::Block>>,
    method_GetNowBlock2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::BlockExtention>>,
    method_GetBlockByNum: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::NumberMessage, super::Tron::Block>>,
    method_GetBlockByNum2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::NumberMessage, super::api::BlockExtention>>,
    method_GetTransactionCountByBlockNum: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::NumberMessage, super::api::NumberMessage>>,
    method_GetDelegatedResource: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::DelegatedResourceMessage, super::api::DelegatedResourceList>>,
    method_GetDelegatedResourceAccountIndex: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::Tron::DelegatedResourceAccountIndex>>,
    method_GetExchangeById: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::Tron::Exchange>>,
    method_ListExchanges: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::ExchangeList>>,
    method_GetTransactionById: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::Tron::Transaction>>,
    method_GetTransactionInfoById: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::Tron::TransactionInfo>>,
    method_GenerateAddress: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::AddressPrKeyPairMessage>>,
    method_GetMerkleTreeVoucherInfo: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::OutputPointInfo, super::Contract::IncrementalMerkleVoucherInfo>>,
    method_ScanNoteByIvk: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::IvkDecryptParameters, super::api::DecryptNotes>>,
    method_ScanAndMarkNoteByIvk: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::IvkDecryptAndMarkParameters, super::api::DecryptNotesMarked>>,
    method_ScanNoteByOvk: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::OvkDecryptParameters, super::api::DecryptNotes>>,
    method_IsSpend: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::NoteParameters, super::api::SpendResult>>,
    method_TriggerConstantContract: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::Contract::TriggerSmartContract, super::api::TransactionExtention>>,
    method_GetRewardInfo: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::api::NumberMessage>>,
    method_GetBrokerageInfo: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::BytesMessage, super::api::NumberMessage>>,
}

impl ::grpc::ClientStub for WalletSolidityClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        WalletSolidityClient {
            grpc_client: grpc_client,
            method_GetAccount: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetAccount".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetAccountById: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetAccountById".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ListWitnesses: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/ListWitnesses".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetAssetIssueList: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetAssetIssueList".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetPaginatedAssetIssueList: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetPaginatedAssetIssueList".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetAssetIssueByName: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetAssetIssueByName".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetAssetIssueListByName: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetAssetIssueListByName".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetAssetIssueById: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetAssetIssueById".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetNowBlock: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetNowBlock".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetNowBlock2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetNowBlock2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetBlockByNum: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetBlockByNum".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetBlockByNum2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetBlockByNum2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetTransactionCountByBlockNum: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetTransactionCountByBlockNum".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetDelegatedResource: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetDelegatedResource".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetDelegatedResourceAccountIndex: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetDelegatedResourceAccountIndex".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetExchangeById: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetExchangeById".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ListExchanges: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/ListExchanges".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetTransactionById: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetTransactionById".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetTransactionInfoById: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetTransactionInfoById".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GenerateAddress: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GenerateAddress".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetMerkleTreeVoucherInfo: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetMerkleTreeVoucherInfo".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ScanNoteByIvk: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/ScanNoteByIvk".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ScanAndMarkNoteByIvk: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/ScanAndMarkNoteByIvk".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_ScanNoteByOvk: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/ScanNoteByOvk".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_IsSpend: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/IsSpend".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_TriggerConstantContract: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/TriggerConstantContract".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetRewardInfo: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetRewardInfo".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetBrokerageInfo: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletSolidity/GetBrokerageInfo".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl WalletSolidity for WalletSolidityClient {
    fn get_account(&self, o: ::grpc::RequestOptions, p: super::Tron::Account) -> ::grpc::SingleResponse<super::Tron::Account> {
        self.grpc_client.call_unary(o, p, self.method_GetAccount.clone())
    }

    fn get_account_by_id(&self, o: ::grpc::RequestOptions, p: super::Tron::Account) -> ::grpc::SingleResponse<super::Tron::Account> {
        self.grpc_client.call_unary(o, p, self.method_GetAccountById.clone())
    }

    fn list_witnesses(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::WitnessList> {
        self.grpc_client.call_unary(o, p, self.method_ListWitnesses.clone())
    }

    fn get_asset_issue_list(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::AssetIssueList> {
        self.grpc_client.call_unary(o, p, self.method_GetAssetIssueList.clone())
    }

    fn get_paginated_asset_issue_list(&self, o: ::grpc::RequestOptions, p: super::api::PaginatedMessage) -> ::grpc::SingleResponse<super::api::AssetIssueList> {
        self.grpc_client.call_unary(o, p, self.method_GetPaginatedAssetIssueList.clone())
    }

    fn get_asset_issue_by_name(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Contract::AssetIssueContract> {
        self.grpc_client.call_unary(o, p, self.method_GetAssetIssueByName.clone())
    }

    fn get_asset_issue_list_by_name(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::AssetIssueList> {
        self.grpc_client.call_unary(o, p, self.method_GetAssetIssueListByName.clone())
    }

    fn get_asset_issue_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Contract::AssetIssueContract> {
        self.grpc_client.call_unary(o, p, self.method_GetAssetIssueById.clone())
    }

    fn get_now_block(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::Tron::Block> {
        self.grpc_client.call_unary(o, p, self.method_GetNowBlock.clone())
    }

    fn get_now_block2(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::BlockExtention> {
        self.grpc_client.call_unary(o, p, self.method_GetNowBlock2.clone())
    }

    fn get_block_by_num(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::Tron::Block> {
        self.grpc_client.call_unary(o, p, self.method_GetBlockByNum.clone())
    }

    fn get_block_by_num2(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::api::BlockExtention> {
        self.grpc_client.call_unary(o, p, self.method_GetBlockByNum2.clone())
    }

    fn get_transaction_count_by_block_num(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::api::NumberMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetTransactionCountByBlockNum.clone())
    }

    fn get_delegated_resource(&self, o: ::grpc::RequestOptions, p: super::api::DelegatedResourceMessage) -> ::grpc::SingleResponse<super::api::DelegatedResourceList> {
        self.grpc_client.call_unary(o, p, self.method_GetDelegatedResource.clone())
    }

    fn get_delegated_resource_account_index(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::DelegatedResourceAccountIndex> {
        self.grpc_client.call_unary(o, p, self.method_GetDelegatedResourceAccountIndex.clone())
    }

    fn get_exchange_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::Exchange> {
        self.grpc_client.call_unary(o, p, self.method_GetExchangeById.clone())
    }

    fn list_exchanges(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::ExchangeList> {
        self.grpc_client.call_unary(o, p, self.method_ListExchanges.clone())
    }

    fn get_transaction_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::Transaction> {
        self.grpc_client.call_unary(o, p, self.method_GetTransactionById.clone())
    }

    fn get_transaction_info_by_id(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::Tron::TransactionInfo> {
        self.grpc_client.call_unary(o, p, self.method_GetTransactionInfoById.clone())
    }

    fn generate_address(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::AddressPrKeyPairMessage> {
        self.grpc_client.call_unary(o, p, self.method_GenerateAddress.clone())
    }

    fn get_merkle_tree_voucher_info(&self, o: ::grpc::RequestOptions, p: super::Contract::OutputPointInfo) -> ::grpc::SingleResponse<super::Contract::IncrementalMerkleVoucherInfo> {
        self.grpc_client.call_unary(o, p, self.method_GetMerkleTreeVoucherInfo.clone())
    }

    fn scan_note_by_ivk(&self, o: ::grpc::RequestOptions, p: super::api::IvkDecryptParameters) -> ::grpc::SingleResponse<super::api::DecryptNotes> {
        self.grpc_client.call_unary(o, p, self.method_ScanNoteByIvk.clone())
    }

    fn scan_and_mark_note_by_ivk(&self, o: ::grpc::RequestOptions, p: super::api::IvkDecryptAndMarkParameters) -> ::grpc::SingleResponse<super::api::DecryptNotesMarked> {
        self.grpc_client.call_unary(o, p, self.method_ScanAndMarkNoteByIvk.clone())
    }

    fn scan_note_by_ovk(&self, o: ::grpc::RequestOptions, p: super::api::OvkDecryptParameters) -> ::grpc::SingleResponse<super::api::DecryptNotes> {
        self.grpc_client.call_unary(o, p, self.method_ScanNoteByOvk.clone())
    }

    fn is_spend(&self, o: ::grpc::RequestOptions, p: super::api::NoteParameters) -> ::grpc::SingleResponse<super::api::SpendResult> {
        self.grpc_client.call_unary(o, p, self.method_IsSpend.clone())
    }

    fn trigger_constant_contract(&self, o: ::grpc::RequestOptions, p: super::Contract::TriggerSmartContract) -> ::grpc::SingleResponse<super::api::TransactionExtention> {
        self.grpc_client.call_unary(o, p, self.method_TriggerConstantContract.clone())
    }

    fn get_reward_info(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::NumberMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetRewardInfo.clone())
    }

    fn get_brokerage_info(&self, o: ::grpc::RequestOptions, p: super::api::BytesMessage) -> ::grpc::SingleResponse<super::api::NumberMessage> {
        self.grpc_client.call_unary(o, p, self.method_GetBrokerageInfo.clone())
    }
}

// server

pub struct WalletSolidityServer;


impl WalletSolidityServer {
    pub fn new_service_def<H : WalletSolidity + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/protocol.WalletSolidity",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetAccount".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_account(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetAccountById".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_account_by_id(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/ListWitnesses".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.list_witnesses(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetAssetIssueList".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_asset_issue_list(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetPaginatedAssetIssueList".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_paginated_asset_issue_list(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetAssetIssueByName".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_asset_issue_by_name(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetAssetIssueListByName".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_asset_issue_list_by_name(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetAssetIssueById".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_asset_issue_by_id(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetNowBlock".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_now_block(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetNowBlock2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_now_block2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetBlockByNum".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_block_by_num(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetBlockByNum2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_block_by_num2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetTransactionCountByBlockNum".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_transaction_count_by_block_num(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetDelegatedResource".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_delegated_resource(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetDelegatedResourceAccountIndex".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_delegated_resource_account_index(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetExchangeById".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_exchange_by_id(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/ListExchanges".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.list_exchanges(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetTransactionById".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_transaction_by_id(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetTransactionInfoById".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_transaction_info_by_id(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GenerateAddress".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.generate_address(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetMerkleTreeVoucherInfo".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_merkle_tree_voucher_info(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/ScanNoteByIvk".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.scan_note_by_ivk(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/ScanAndMarkNoteByIvk".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.scan_and_mark_note_by_ivk(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/ScanNoteByOvk".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.scan_note_by_ovk(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/IsSpend".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.is_spend(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/TriggerConstantContract".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.trigger_constant_contract(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetRewardInfo".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_reward_info(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletSolidity/GetBrokerageInfo".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_brokerage_info(o, p))
                    },
                ),
            ],
        )
    }
}

// interface

pub trait WalletExtension {
    fn get_transactions_from_this(&self, o: ::grpc::RequestOptions, p: super::api::AccountPaginated) -> ::grpc::SingleResponse<super::api::TransactionList>;

    fn get_transactions_from_this2(&self, o: ::grpc::RequestOptions, p: super::api::AccountPaginated) -> ::grpc::SingleResponse<super::api::TransactionListExtention>;

    fn get_transactions_to_this(&self, o: ::grpc::RequestOptions, p: super::api::AccountPaginated) -> ::grpc::SingleResponse<super::api::TransactionList>;

    fn get_transactions_to_this2(&self, o: ::grpc::RequestOptions, p: super::api::AccountPaginated) -> ::grpc::SingleResponse<super::api::TransactionListExtention>;
}

// client

pub struct WalletExtensionClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
    method_GetTransactionsFromThis: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::AccountPaginated, super::api::TransactionList>>,
    method_GetTransactionsFromThis2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::AccountPaginated, super::api::TransactionListExtention>>,
    method_GetTransactionsToThis: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::AccountPaginated, super::api::TransactionList>>,
    method_GetTransactionsToThis2: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::AccountPaginated, super::api::TransactionListExtention>>,
}

impl ::grpc::ClientStub for WalletExtensionClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        WalletExtensionClient {
            grpc_client: grpc_client,
            method_GetTransactionsFromThis: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletExtension/GetTransactionsFromThis".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetTransactionsFromThis2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletExtension/GetTransactionsFromThis2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetTransactionsToThis: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletExtension/GetTransactionsToThis".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetTransactionsToThis2: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.WalletExtension/GetTransactionsToThis2".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl WalletExtension for WalletExtensionClient {
    fn get_transactions_from_this(&self, o: ::grpc::RequestOptions, p: super::api::AccountPaginated) -> ::grpc::SingleResponse<super::api::TransactionList> {
        self.grpc_client.call_unary(o, p, self.method_GetTransactionsFromThis.clone())
    }

    fn get_transactions_from_this2(&self, o: ::grpc::RequestOptions, p: super::api::AccountPaginated) -> ::grpc::SingleResponse<super::api::TransactionListExtention> {
        self.grpc_client.call_unary(o, p, self.method_GetTransactionsFromThis2.clone())
    }

    fn get_transactions_to_this(&self, o: ::grpc::RequestOptions, p: super::api::AccountPaginated) -> ::grpc::SingleResponse<super::api::TransactionList> {
        self.grpc_client.call_unary(o, p, self.method_GetTransactionsToThis.clone())
    }

    fn get_transactions_to_this2(&self, o: ::grpc::RequestOptions, p: super::api::AccountPaginated) -> ::grpc::SingleResponse<super::api::TransactionListExtention> {
        self.grpc_client.call_unary(o, p, self.method_GetTransactionsToThis2.clone())
    }
}

// server

pub struct WalletExtensionServer;


impl WalletExtensionServer {
    pub fn new_service_def<H : WalletExtension + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/protocol.WalletExtension",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletExtension/GetTransactionsFromThis".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_transactions_from_this(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletExtension/GetTransactionsFromThis2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_transactions_from_this2(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletExtension/GetTransactionsToThis".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_transactions_to_this(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.WalletExtension/GetTransactionsToThis2".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_transactions_to_this2(o, p))
                    },
                ),
            ],
        )
    }
}

// interface

pub trait Database {
    fn get_block_reference(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::BlockReference>;

    fn get_dynamic_properties(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::Tron::DynamicProperties>;

    fn get_now_block(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::Tron::Block>;

    fn get_block_by_num(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::Tron::Block>;
}

// client

pub struct DatabaseClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
    method_getBlockReference: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::api::BlockReference>>,
    method_GetDynamicProperties: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::Tron::DynamicProperties>>,
    method_GetNowBlock: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::EmptyMessage, super::Tron::Block>>,
    method_GetBlockByNum: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::api::NumberMessage, super::Tron::Block>>,
}

impl ::grpc::ClientStub for DatabaseClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        DatabaseClient {
            grpc_client: grpc_client,
            method_getBlockReference: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Database/getBlockReference".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetDynamicProperties: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Database/GetDynamicProperties".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetNowBlock: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Database/GetNowBlock".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_GetBlockByNum: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/protocol.Database/GetBlockByNum".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl Database for DatabaseClient {
    fn get_block_reference(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::api::BlockReference> {
        self.grpc_client.call_unary(o, p, self.method_getBlockReference.clone())
    }

    fn get_dynamic_properties(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::Tron::DynamicProperties> {
        self.grpc_client.call_unary(o, p, self.method_GetDynamicProperties.clone())
    }

    fn get_now_block(&self, o: ::grpc::RequestOptions, p: super::api::EmptyMessage) -> ::grpc::SingleResponse<super::Tron::Block> {
        self.grpc_client.call_unary(o, p, self.method_GetNowBlock.clone())
    }

    fn get_block_by_num(&self, o: ::grpc::RequestOptions, p: super::api::NumberMessage) -> ::grpc::SingleResponse<super::Tron::Block> {
        self.grpc_client.call_unary(o, p, self.method_GetBlockByNum.clone())
    }
}

// server

pub struct DatabaseServer;


impl DatabaseServer {
    pub fn new_service_def<H : Database + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/protocol.Database",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Database/getBlockReference".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_block_reference(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Database/GetDynamicProperties".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_dynamic_properties(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Database/GetNowBlock".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_now_block(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/protocol.Database/GetBlockByNum".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_block_by_num(o, p))
                    },
                ),
            ],
        )
    }
}

// interface

pub trait Network {
}

// client

pub struct NetworkClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
}

impl ::grpc::ClientStub for NetworkClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        NetworkClient {
            grpc_client: grpc_client,
        }
    }
}

impl Network for NetworkClient {
}

// server

pub struct NetworkServer;


impl NetworkServer {
    pub fn new_service_def<H : Network + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/protocol.Network",
            vec![
            ],
        )
    }
}
