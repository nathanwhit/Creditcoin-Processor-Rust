use super::*;

#[test]
#[allow(
    unused_variables,
    unused_parens,
    unused_imports,
    unused_mut,
    unused_braces
)]
fn add_offer_success() {
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
        signer_with_secret("11c5bcfc9b716aeedfd1f91d6ac118181a867f973828295e5d1ae5174a94355f");
    let investor = SigHash::from(&investor_signer);
    let fundraiser_signer =
        signer_with_secret("f32cc029fb7430631ab83bea178e683e14dc0a70713f6c6cad83585bce76134c");
    let fundraiser = SigHash::from(&fundraiser_signer);
    let mut tx_fee = TX_FEE.clone();
    let mut request = TpProcessRequest {
        tip: 7,
        ..::core::default::Default::default()
    };
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
    let mut investor_address_id = address_id_for("investoraddress");
    let mut fundraiser_address_id = address_id_for("fundraiseraddress");
    let mut add_ask_order_guid = Guid::from(make_nonce());
    let mut add_bid_order_guid = Guid::from(make_nonce());
    let mut ask_order_id =
        Address::with_prefix_key(ASK_ORDER.clone(), add_ask_order_guid.clone().as_str());
    let mut bid_order_id =
        Address::with_prefix_key(BID_ORDER.clone(), add_bid_order_guid.clone().as_str());
    let mut command = AddOffer {
        ask_order_id: ask_order_id.clone().into(),
        bid_order_id: bid_order_id.clone().into(),
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
        blockchain: investor_address_proto.clone().blockchain.clone(),
        address: add_ask_order.clone().address_id.clone(),
        amount: add_ask_order.clone().amount_str.clone(),
        interest: add_ask_order.clone().interest.clone(),
        maturity: add_ask_order.clone().maturity.clone(),
        fee: add_ask_order.clone().fee_str.clone(),
        expiration: add_ask_order.clone().expiration.into(),
        block: (request.tip - 1).to_string(),
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
        blockchain: fundraiser_address_proto.clone().blockchain.clone(),
        address: fundraiser_address_id.clone().into(),
        amount: add_bid_order.clone().amount_str.clone(),
        interest: add_bid_order.clone().interest.clone(),
        maturity: add_bid_order.clone().maturity.clone(),
        fee: add_bid_order.clone().fee_str.clone(),
        expiration: add_bid_order.clone().expiration.into(),
        block: (request.tip - 1).to_string(),
        sighash: fundraiser.clone().into(),
    };
    let investor_wallet_id_ = WalletId::from(&investor);
    let fundraiser_wallet_id_ = WalletId::from(&fundraiser);
    let add_ask_order_guid_ = Guid("some_guid".into());
    let add_bid_order_guid_ = Guid("some_guid".into());
    let mut offer_address_id = Address::with_prefix_key(
        OFFER.clone(),
        &string!(&command.ask_order_id, &command.bid_order_id),
    );
    let mut offer = crate::protos::Offer {
        blockchain: investor_address_proto.clone().blockchain.clone(),
        ask_order: command.clone().ask_order_id.clone(),
        bid_order: command.clone().bid_order_id.clone(),
        expiration: command.clone().expiration.into(),
        block: (request.tip - 1).to_string(),
        sighash: investor.clone().to_string(),
    };
    {
        let sig = crate::handler::types::SigHash(investor.clone().to_string());
        ctx.expect_sighash().return_once(move |_| Ok(sig));
    }
    {
        let guid = command_guid_.clone().clone();
        ctx.expect_guid().returning(move |_| guid.clone());
    }
    {
        let address = investor_wallet_id_.clone().clone();
        let ret = tx_fee.clone().clone();
        tx_ctx
            .expect_get_state_entry()
            .withf(move |addr| address.as_str() == addr)
            .return_once(move |_| Ok(wallet_with(Option::from(ret))));
    }
    expect_get_state_entry(
        &mut tx_ctx,
        offer_address_id.clone(),
        <Option<crate::protos::Wallet>>::None,
        None,
    );
    expect_get_state_entry(
        &mut tx_ctx,
        command.clone().ask_order_id,
        Some(ask_order.clone()),
        None,
    );
    expect_get_state_entry(
        &mut tx_ctx,
        command.clone().bid_order_id,
        Some(bid_order.clone()),
        None,
    );
    expect_get_state_entry(
        &mut tx_ctx,
        investor_address_id.clone(),
        Some(investor_address_proto.clone()),
        None,
    );
    expect_get_state_entry(
        &mut tx_ctx,
        fundraiser_address_id.clone(),
        Some(fundraiser_address_proto.clone()),
        None,
    );
    expect_set_state_entries(
        &mut tx_ctx,
        vec![
            (
                offer_address_id.clone().to_string(),
                offer.clone().to_bytes().into(),
            ),
            (
                investor_wallet_id_.clone().to_string(),
                wallet_with(Some(0)).unwrap().into(),
            ),
            make_fee(
                &command_guid_.clone(),
                &investor.clone(),
                Some(request.tip - 1),
            ),
        ],
    );
    execute_success(command, &request, &tx_ctx, &mut ctx);
}
