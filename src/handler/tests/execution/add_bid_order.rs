use super::*;

#[test]
#[allow(
    unused_variables,
    unused_parens,
    unused_imports,
    unused_mut,
    unused_braces
)]
fn add_bid_order_success() {
    use crate::handler::types::*;
    use std::str::FromStr as _;
    let fundraiser_signer =
        signer_with_secret("827c39480011a29fa972ed8b671ee5a69edd13e24b5442ee2694514e56d15d88");
    let fundraiser = SigHash::from(&fundraiser_signer);
    let mut tse = ToStateEntryCtx::new(3u64);
    let mut tx_fee = TX_FEE.clone();
    let mut request = TpProcessRequest {
        tip: 3,
        ..::core::default::Default::default()
    };
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
    let mut address_id = address_id_for("fundraiseraddress");
    let mut command = AddBidOrder {
        address_id: address_id.clone().into(),
        amount_str: "1000".into(),
        interest: "100".into(),
        maturity: "10".into(),
        fee_str: "1".into(),
        expiration: 10000.into(),
    };
    let command_guid_ = Guid("some_guid".into());
    let mut bid_order_id =
        AddressId::with_prefix_key(BID_ORDER.clone(), command_guid_.clone().as_str());
    let fundraiser_wallet_id_ = WalletId::from(&fundraiser);
    let mut address_proto = address_for("fundraiseraddress", &fundraiser.clone());
    let mut bid_order = crate::protos::BidOrder {
        blockchain: address_proto.blockchain.clone(),
        address: command.address_id.clone(),
        amount: command.amount_str.clone(),
        interest: command.interest.clone(),
        maturity: command.maturity.clone(),
        fee: command.fee_str.clone(),
        expiration: command.expiration.clone().into(),
        block: (request.tip - 1).to_string(),
        sighash: fundraiser.clone().into(),
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
        let guid = command_guid_.clone();
        ctx.expect_guid().returning(move |_| guid.clone());
    }
    {
        let address = fundraiser_wallet_id_.clone();
        let ret = tx_fee.clone();
        tx_ctx
            .expect_get_state_entry()
            .withf(move |addr| address.as_str() == addr)
            .return_once(move |_| Ok(wallet_with(Option::from(ret))));
    }
    expect_get_state_entry(
        &mut tx_ctx,
        bid_order_id.clone(),
        <Option<crate::protos::Wallet>>::None,
        None,
    );
    expect_get_state_entry(
        &mut tx_ctx,
        address_id.clone(),
        Some(address_proto.clone()),
        None,
    );
    expect_set_state_entries(
        &mut tx_ctx,
        vec![
            (
                fundraiser_wallet_id_.clone().to_string(),
                wallet_with(Some(0)).unwrap().into(),
            ),
            (
                bid_order_id.clone().to_string().to_string(),
                bid_order.clone().to_bytes().into(),
            ),
            make_fee(&command_guid_.clone(), &fundraiser.clone(), Some(2)),
        ],
    );
    execute_success(command, &request, &tx_ctx, &mut ctx);
}
