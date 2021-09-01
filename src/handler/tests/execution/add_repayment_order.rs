use super::*;

#[test]
#[allow(
    unused_variables,
    unused_parens,
    unused_imports,
    unused_mut,
    unused_braces
)]
fn add_repayment_order_success() {
    use crate::handler::types::*;
    use std::str::FromStr as _;
    let investor_signer =
        signer_with_secret("827c39480011a29fa972ed8b671ee5a69edd13e24b5442ee2694514e56d15d88");
    let investor = SigHash::from(&investor_signer);
    let fundraiser_signer =
        signer_with_secret("48b0ae97607427a8550e4da5edc8da0a04617adde25c98a405a0c47114cdf69e");
    let fundraiser = SigHash::from(&fundraiser_signer);
    let collector_signer =
        signer_with_secret("0bf47d913365b3c163897b3a40a03db6c14c2c8637ac732d93552b3ce6dbfabe");
    let collector = SigHash::from(&collector_signer);
    let mut tse = ToStateEntryCtx::new(4u64);
    let mut tx_fee = TX_FEE.clone();
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
    let mut register_address_investor = register_address_for("investoraddress");
    let mut register_address_fundraiser = register_address_for("fundraiseraddress");
    let mut register_address_collector = register_address_for("collectoraddress");
    let (mut investor_address_id, mut investor_address) =
        tse.state_entry_from(register_address_investor.clone(), investor.clone());
    let (mut fundraiser_address_id, mut fundraiser_address) =
        tse.state_entry_from(register_address_fundraiser.clone(), fundraiser.clone());
    let (mut collector_address_id, mut collector_address) =
        tse.state_entry_from(register_address_collector.clone(), collector.clone());
    let mut add_ask_order = AddAskOrder {
        address_id: investor_address_id.clone().into(),
        amount_str: "1000".into(),
        interest: "100".into(),
        maturity: "10".into(),
        fee_str: "1".into(),
        expiration: 10000.into(),
    };
    let mut add_ask_order_guid = Guid::random();
    let (mut ask_order_id, mut ask_order) = tse.state_entry_from(
        add_ask_order.clone(),
        AddAskOrderArgs {
            guid: add_ask_order_guid.clone(),
            address: investor_address.clone(),
            sighash: investor.clone(),
        },
    );
    let mut add_bid_order = AddBidOrder {
        address_id: fundraiser_address_id.clone().into(),
        amount_str: "1000".into(),
        interest: "100".into(),
        maturity: "10".into(),
        fee_str: "1".into(),
        expiration: 10000.into(),
    };
    let mut add_bid_order_guid = Guid::random();
    let (mut bid_order_id, mut bid_order) = tse.state_entry_from(
        add_bid_order.clone(),
        AddBidOrderArgs {
            guid: add_bid_order_guid.clone(),
            address: fundraiser_address.clone(),
            sighash: fundraiser.clone(),
        },
    );
    let mut add_offer = AddOffer {
        ask_order_id: ask_order_id.clone().into(),
        bid_order_id: bid_order_id.clone().into(),
        expiration: 10000.into(),
    };
    let mut add_offer_guid = Guid::random();
    let (mut offer_id, mut offer) = tse.state_entry_from(
        add_offer.clone(),
        AddOfferArgs {
            src_address: investor_address.clone(),
            sighash: investor.clone(),
        },
    );
    let mut add_deal_order_guid = Guid::random();
    let mut add_deal_order = AddDealOrder {
        offer_id: offer_id.clone().into(),
        expiration: 10000.into(),
    };
    let (mut deal_order_id, mut deal_order) = tse.state_entry_from(
        add_deal_order.clone(),
        AddDealOrderArgs {
            bid_order: bid_order.clone().clone(),
            ask_order: ask_order.clone().clone(),
            offer: offer.clone().clone(),
            sighash: fundraiser.clone(),
        },
    );
    let mut register_transfer = RegisterTransfer {
        gain: 0.into(),
        order_id: deal_order_id.clone().into(),
        blockchain_tx_id: String::from("blockchaintxid"),
    };
    let (mut transfer_id, mut transfer) = tse.state_entry_from(
        register_transfer.clone(),
        RegisterTransferArgs {
            kind: TransferKind::DealOrder(deal_order.clone()),
            src_address: investor_address.clone(),
            src_sighash: investor.clone(),
        },
    );
    let mut complete_deal_order = CompleteDealOrder {
        deal_order_id: deal_order_id.clone().into(),
        transfer_id: transfer_id.clone().into(),
    };
    tse.inc_tip();
    let mut updated_deal_order = crate::protos::DealOrder {
        loan_transfer: transfer_id.clone().into(),
        block: tse.tip().to_string(),
        ..deal_order.clone()
    };
    let mut updated_transfer = crate::protos::Transfer {
        processed: true,
        ..transfer.clone()
    };
    let mut add_repayment_order = AddRepaymentOrder {
        deal_order_id: deal_order_id.clone().into(),
        address_id: collector_address_id.clone().into(),
        amount_str: String::from("100"),
        expiration: 10000.into(),
    };
    let mut add_repayment_order_guid = Guid::from(make_nonce());
    let (mut repayment_order_id, mut repayment_order) = tse.state_entry_from(
        add_repayment_order.clone(),
        AddRepaymentOrderArgs {
            guid: add_repayment_order_guid.clone(),
            src_address: collector_address.clone(),
            deal_order: deal_order.clone().clone(),
            sighash: collector.clone(),
        },
    );
    let mut command_guid_ = add_repayment_order_guid.clone();
    let mut command = add_repayment_order.clone();
    let investor_wallet_id_ = WalletId::from(&investor);
    let fundraiser_wallet_id_ = WalletId::from(&fundraiser);
    let collector_wallet_id_ = WalletId::from(&collector);
    {
        let sig = crate::handler::types::SigHash(collector.clone().to_string());
        ctx.expect_sighash().return_once(move |_| Ok(sig));
    }
    {
        let guid = command_guid_.clone();
        ctx.expect_guid().returning(move |_| guid.clone());
    }
    {
        let address = collector_wallet_id_.clone();
        let ret = tx_fee.clone();
        tx_ctx
            .expect_get_state_entry()
            .withf(move |addr| address.as_str() == addr)
            .return_once(move |_| Ok(wallet_with(Option::from(ret))));
    }
    expect_get_state_entry(
        &mut tx_ctx,
        deal_order_id.clone(),
        Some(updated_deal_order.clone()),
        None,
    );
    expect_get_state_entry(
        &mut tx_ctx,
        repayment_order_id.clone(),
        <Option<crate::protos::Wallet>>::None,
        None,
    );
    expect_get_state_entry(
        &mut tx_ctx,
        investor_address_id.clone(),
        Some(investor_address.clone()),
        None,
    );
    expect_get_state_entry(
        &mut tx_ctx,
        collector_address_id.clone(),
        Some(collector_address.clone()),
        None,
    );
    expect_set_state_entries(
        &mut tx_ctx,
        vec![
            (
                repayment_order_id.clone().to_string(),
                repayment_order.clone().to_bytes().into(),
            ),
            (
                collector_wallet_id_.clone().to_string(),
                wallet_with(Some(0)).unwrap().into(),
            ),
            make_fee(
                &command_guid_.clone(),
                &collector.clone(),
                Some(tse.tip() - 1),
            ),
        ],
    );
    let mut request = TpProcessRequest {
        tip: tse.tip().into(),
        ..Default::default()
    };
    execute_success(command, &request, &tx_ctx, &mut ctx);
}
