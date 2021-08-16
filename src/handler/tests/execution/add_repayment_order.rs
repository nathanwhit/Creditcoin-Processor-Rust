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
    fn wallet_with(balance: Option<impl Into<Integer> + Clone>) -> Option<Vec<u8>> {
        balance.map(|b| {
            let wallet = crate::protos::Wallet {
                amount: b.into().to_string(),
            };
            let mut buf = Vec::with_capacity(wallet.encoded_len());
            wallet.encode(&mut buf).unwrap();
            buf
        })
    }
    let investor_signer =
        signer_with_secret("a2b3847f44537a3f287ac1cc8defe4fb1da3da554409f31522f715b9dec731ab");
    let investor = SigHash::from(&investor_signer);
    let fundraiser_signer =
        signer_with_secret("75956bc5d5e28ee8139485d76b45098d8eef8f5f3d1086a553ed50c67d2f390d");
    let fundraiser = SigHash::from(&fundraiser_signer);
    let collector_signer =
        signer_with_secret("441646304d6903821c7ba7eee745ddfa74c0ab33db76bf4d97f9304c973a5539");
    let collector = SigHash::from(&collector_signer);
    let mut tx_fee = TX_FEE.clone();
    let mut request = TpProcessRequest {
        tip: 13,
        ..::core::default::Default::default()
    };
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
    let mut investor_address_id = address_id_for("investoraddress");
    let mut fundraiser_address_id = address_id_for("fundraiseraddress");
    let mut collector_address_id = address_id_for("collectoraddress");
    let mut register_address_investor = register_address_for("investoraddress");
    let mut register_address_fundraiser = register_address_for("fundraiseraddress");
    let mut register_address_collector = register_address_for("collectoraddress");
    let mut investor_address = address_for("investoraddress", &investor.clone());
    let mut fundraiser_address = address_for("fundraiseraddress", &fundraiser.clone());
    let mut collector_address = address_for("collectoraddress", &collector.clone());
    let mut add_ask_order = AddAskOrder {
        address_id: investor_address_id.clone().into(),
        amount_str: "1000".into(),
        interest: "100".into(),
        maturity: "10".into(),
        fee_str: "1".into(),
        expiration: 10000.into(),
    };
    let mut add_ask_order_guid = Guid::from(make_nonce());
    let mut ask_order_id =
        Address::with_prefix_key(ASK_ORDER.clone(), add_ask_order_guid.clone().as_str());
    let mut ask_order = crate::protos::AskOrder {
        blockchain: investor_address.clone().blockchain.clone(),
        address: add_ask_order.clone().address_id.clone(),
        amount: add_ask_order.clone().amount_str.clone(),
        interest: add_ask_order.clone().interest.clone(),
        maturity: add_ask_order.clone().maturity.clone(),
        fee: add_ask_order.clone().fee_str.clone(),
        expiration: add_ask_order.clone().expiration.into(),
        block: (request.tip - 6).to_string(),
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
    let mut add_bid_order_guid = Guid::from(make_nonce());
    let mut bid_order_id =
        Address::with_prefix_key(BID_ORDER.clone(), add_bid_order_guid.clone().as_str());
    let mut bid_order = crate::protos::BidOrder {
        blockchain: fundraiser_address.clone().blockchain.clone(),
        address: fundraiser_address_id.clone().into(),
        amount: add_bid_order.clone().amount_str.clone(),
        interest: add_bid_order.clone().interest.clone(),
        maturity: add_bid_order.clone().maturity.clone(),
        fee: add_bid_order.clone().fee_str.clone(),
        expiration: add_bid_order.clone().expiration.into(),
        block: (request.tip - 5).to_string(),
        sighash: fundraiser.clone().into(),
    };
    let mut add_offer = AddOffer {
        ask_order_id: ask_order_id.clone().into(),
        bid_order_id: bid_order_id.clone().into(),
        expiration: 10000.into(),
    };
    let mut add_offer_guid = Guid::from(make_nonce());
    let mut offer_id =
        Address::with_prefix_key(OFFER.clone(), &string!(&ask_order_id, &bid_order_id));
    let mut offer = crate::protos::Offer {
        blockchain: investor_address.clone().blockchain.clone(),
        ask_order: ask_order_id.clone().into(),
        bid_order: bid_order_id.clone().into(),
        expiration: add_offer.clone().expiration.into(),
        block: (request.tip - 4).to_string(),
        sighash: investor.clone().to_string(),
    };
    let mut add_deal_order = AddDealOrder {
        offer_id: offer_id.clone().into(),
        expiration: 10000.into(),
    };
    let mut deal_order_id = Address::with_prefix_key(DEAL_ORDER.clone(), &offer_id.clone());
    let mut deal_order = crate::protos::DealOrder {
        blockchain: offer.clone().blockchain,
        src_address: ask_order.clone().address,
        dst_address: bid_order.clone().address,
        amount: bid_order.clone().amount,
        interest: bid_order.clone().interest,
        maturity: bid_order.clone().maturity,
        fee: bid_order.clone().fee,
        expiration: add_ask_order.clone().expiration.into(),
        sighash: fundraiser.clone().to_string(),
        block: (request.tip - 3).to_string(),
        ..::core::default::Default::default()
    };
    let mut register_transfer = RegisterTransfer {
        gain: 0.into(),
        order_id: deal_order_id.clone().into(),
        blockchain_tx_id: String::from("blockchaintxid"),
    };
    let mut transfer_id = Address::with_prefix_key(
        TRANSFER.clone(),
        &string!(
            &investor_address.blockchain,
            &register_transfer.blockchain_tx_id,
            &investor_address.network
        ),
    );
    let mut transfer = crate::protos::Transfer {
        blockchain: investor_address.clone().blockchain.clone(),
        dst_address: fundraiser_address_id.clone().to_string(),
        src_address: investor_address_id.clone().to_string(),
        order: register_transfer.clone().order_id.clone(),
        amount: deal_order.clone().amount,
        tx: register_transfer.clone().blockchain_tx_id.clone(),
        sighash: investor.clone().to_string(),
        block: (request.tip - 2).to_string(),
        processed: false,
    };
    let mut complete_deal_order = CompleteDealOrder {
        deal_order_id: deal_order_id.clone().into(),
        transfer_id: transfer_id.clone().into(),
    };
    let mut updated_deal_order = crate::protos::DealOrder {
        loan_transfer: transfer_id.clone().into(),
        block: (request.tip - 1).to_string(),
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
    let mut repayment_order = crate::protos::RepaymentOrder {
        blockchain: collector_address.clone().blockchain,
        src_address: collector_address_id.clone().into(),
        dst_address: deal_order.clone().src_address,
        amount: add_repayment_order.clone().amount_str,
        expiration: add_repayment_order.clone().expiration.into(),
        block: (request.tip - 1).to_string(),
        deal: add_repayment_order.clone().deal_order_id,
        sighash: collector.clone().into(),
        ..::core::default::Default::default()
    };
    let mut command = add_repayment_order.clone();
    let command_guid_ = Guid("some_guid".into());
    let mut repayment_order_id =
        Address::with_prefix_key(REPAYMENT_ORDER.clone(), &command_guid_.clone());
    let investor_wallet_id_ = WalletId::from(&investor);
    let fundraiser_wallet_id_ = WalletId::from(&fundraiser);
    let collector_wallet_id_ = WalletId::from(&collector);
    let add_ask_order_guid_ = Guid("some_guid".into());
    let add_bid_order_guid_ = Guid("some_guid".into());
    {
        let sig = crate::handler::types::SigHash(collector.clone().to_string());
        ctx.expect_sighash().return_once(move |_| Ok(sig));
    }
    {
        let guid = command_guid_.clone().clone();
        ctx.expect_guid().returning(move |_| guid.clone());
    }
    {
        let address = collector_wallet_id_.clone().clone();
        let ret = tx_fee.clone().clone();
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
                Some(request.tip - 1),
            ),
        ],
    );
    execute_success(command, &request, &tx_ctx, &mut ctx);
}
