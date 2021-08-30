use super::*;

#[test]
#[allow(
    unused_variables,
    unused_parens,
    unused_imports,
    unused_mut,
    unused_braces
)]
fn add_deal_order_success() {
    use crate::handler::types::*;
    use std::str::FromStr as _;
    let investor_signer =
        signer_with_secret("827c39480011a29fa972ed8b671ee5a69edd13e24b5442ee2694514e56d15d88");
    let investor = SigHash::from(&investor_signer);
    let fundraiser_signer =
        signer_with_secret("48b0ae97607427a8550e4da5edc8da0a04617adde25c98a405a0c47114cdf69e");
    let fundraiser = SigHash::from(&fundraiser_signer);
    let mut tse = ToStateEntryCtx::new(4u64);
    let mut tx_fee = TX_FEE.clone();
    let mut request = TpProcessRequest {
        tip: 8,
        ..::core::default::Default::default()
    };
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
    let mut investor_address_id = address_id_for("investoraddress");
    let mut fundraiser_address_id = address_id_for("fundraiseraddress");
    let mut add_ask_order_guid = Guid::random();
    let mut add_bid_order_guid = Guid::random();
    let mut add_offer_guid = Guid::random();
    let mut ask_order_id =
        AddressId::with_prefix_key(ASK_ORDER.clone(), add_ask_order_guid.clone().as_str());
    let mut bid_order_id =
        AddressId::with_prefix_key(BID_ORDER.clone(), add_bid_order_guid.clone().as_str());
    let mut offer_id =
        AddressId::with_prefix_key(OFFER.clone(), &string!(&ask_order_id, &bid_order_id));
    let mut command = AddDealOrder {
        offer_id: offer_id.clone().into(),
        expiration: 10000.into(),
    };
    let command_guid_ = Guid("some_guid".into());
    let mut investor_address_proto = address_for("investoraddress", &investor.clone());
    let mut fundraiser_address_proto = address_for("fundraiseraddress", &fundraiser.clone());
    let mut add_ask_order = AddAskOrder {
        address_id: investor_address_id.clone().into(),
        amount_str: "1000".into(),
        interest: "100".into(),
        maturity: "10".into(),
        fee_str: "1".into(),
        expiration: 10000.into(),
    };
    let mut ask_order = crate::protos::AskOrder {
        blockchain: investor_address_proto.blockchain.clone(),
        address: add_ask_order.address_id.clone(),
        amount: add_ask_order.amount_str.clone(),
        interest: add_ask_order.interest.clone(),
        maturity: add_ask_order.maturity.clone(),
        fee: add_ask_order.fee_str.clone(),
        expiration: add_ask_order.expiration.clone().into(),
        block: (request.tip - 4).to_string(),
        sighash: investor.clone().into(),
    };
    let mut add_bid_order = AddBidOrder {
        address_id: fundraiser_address_id.clone().into(),
        amount_str: "1000".into(),
        interest: "100".into(),
        maturity: "10".into(),
        fee_str: "1".into(),
        expiration: 10000.into(),
    };
    let mut bid_order = crate::protos::BidOrder {
        blockchain: fundraiser_address_proto.blockchain.clone(),
        address: fundraiser_address_id.clone().into(),
        amount: add_bid_order.amount_str.clone(),
        interest: add_bid_order.interest.clone(),
        maturity: add_bid_order.maturity.clone(),
        fee: add_bid_order.fee_str.clone(),
        expiration: add_bid_order.expiration.clone().into(),
        block: (request.tip - 3).to_string(),
        sighash: fundraiser.clone().into(),
    };
    let mut add_offer = AddOffer {
        ask_order_id: ask_order_id.clone().into(),
        bid_order_id: bid_order_id.clone().into(),
        expiration: 10000.into(),
    };
    let mut offer = crate::protos::Offer {
        blockchain: investor_address_proto.blockchain.clone(),
        ask_order: ask_order_id.clone().into(),
        bid_order: bid_order_id.clone().into(),
        expiration: command.expiration.clone().into(),
        block: (request.tip - 2).to_string(),
        sighash: investor.clone().to_string(),
    };
    let investor_wallet_id_ = WalletId::from(&investor);
    let fundraiser_wallet_id_ = WalletId::from(&fundraiser);
    let add_ask_order_guid_ = Guid("some_guid".into());
    let add_bid_order_guid_ = Guid("some_guid".into());
    let mut deal_order_id =
        AddressId::with_prefix_key(DEAL_ORDER.clone(), &command.offer_id.clone());
    let mut deal_order = crate::protos::DealOrder {
        blockchain: offer.blockchain.clone(),
        src_address: ask_order.address.clone(),
        dst_address: bid_order.address.clone(),
        amount: bid_order.amount.clone(),
        interest: bid_order.interest.clone(),
        maturity: bid_order.maturity.clone(),
        fee: bid_order.fee.clone(),
        expiration: command.expiration.clone().into(),
        sighash: fundraiser.clone().to_string(),
        block: (request.tip - 1).to_string(),
        ..::core::default::Default::default()
    };
    {
        let sig = crate::handler::types::SigHash(fundraiser.clone().to_string());
        ctx.expect_sighash().return_once(move |_| Ok(sig));
    }
    {
        let guid = command_guid_.clone();
        ctx.expect_guid().returning(move |_| guid.clone());
    }
    {
        let address = fundraiser_wallet_id_.clone();
        let ret = tx_fee.clone() + 1;
        tx_ctx
            .expect_get_state_entry()
            .withf(move |addr| address.as_str() == addr)
            .return_once(move |_| Ok(wallet_with(Option::from(ret))));
    }
    expect_get_state_entry(
        &mut tx_ctx,
        deal_order_id.clone(),
        <Option<crate::protos::Wallet>>::None,
        None,
    );
    expect_get_state_entry(&mut tx_ctx, offer_id.clone(), Some(offer.clone()), None);
    expect_get_state_entry(
        &mut tx_ctx,
        ask_order_id.clone(),
        Some(ask_order.clone()),
        None,
    );
    expect_get_state_entry(
        &mut tx_ctx,
        bid_order_id.clone(),
        Some(bid_order.clone()),
        None,
    );
    expect_set_state_entries(
        &mut tx_ctx,
        vec![
            (
                deal_order_id.clone().to_string(),
                deal_order.clone().to_bytes().into(),
            ),
            (
                fundraiser_wallet_id_.clone().to_string(),
                wallet_with(Some(0)).unwrap().into(),
            ),
            make_fee(
                &command_guid_.clone(),
                &fundraiser.clone(),
                Some(request.tip - 1),
            ),
        ],
    );
    expect_delete_state_entries(
        &mut tx_ctx,
        vec![
            offer.ask_order.clone().to_string(),
            offer.bid_order.clone().to_string(),
            offer_id.clone().to_string(),
        ],
    );
    execute_success(command, &request, &tx_ctx, &mut ctx);
}
