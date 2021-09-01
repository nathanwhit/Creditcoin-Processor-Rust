use super::*;

#[test]
#[allow(
    unused_variables,
    unused_parens,
    unused_imports,
    unused_mut,
    unused_braces
)]
fn register_transfer_success() {
    use crate::handler::types::*;
    use std::str::FromStr as _;
    let investor_signer =
        signer_with_secret("827c39480011a29fa972ed8b671ee5a69edd13e24b5442ee2694514e56d15d88");
    let investor = SigHash::from(&investor_signer);
    let fundraiser_signer =
        signer_with_secret("48b0ae97607427a8550e4da5edc8da0a04617adde25c98a405a0c47114cdf69e");
    let fundraiser = SigHash::from(&fundraiser_signer);
    let mut tse = ToStateEntryCtx::new(3u64);
    let mut tx_fee = TX_FEE.clone();
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
    let mut request = TpProcessRequest {
        tip: 9,
        ..::core::default::Default::default()
    };
    let mut investor_address_id = address_id_for("myaddress");
    let mut fundraiser_address_id = address_id_for("otheraddress");
    let mut bid_order_guid = Guid::random();
    let mut ask_order_guid = Guid::random();
    let mut offer_guid = Guid::random();
    let mut ask_order_id = AddressId::with_prefix_key(ASK_ORDER, ask_order_guid.as_str());
    let mut bid_order_id = AddressId::with_prefix_key(BID_ORDER, bid_order_guid.as_str());
    let mut offer_id = AddressId::with_prefix_key(
        OFFER,
        &string!(ask_order_id.as_str(), bid_order_id.as_str()),
    );
    let mut deal_order_guid = Guid::random();
    let mut deal_order_id = AddressId::with_prefix_key(DEAL_ORDER, offer_id.as_str());
    let mut command = RegisterTransfer {
        gain: 1.into(),
        order_id: deal_order_id.clone().into(),
        blockchain_tx_id: String::from("blockchaintxid"),
    };
    let command_guid_ = Guid("some_guid".into());
    let investor_wallet_id_ = WalletId::from(&investor);
    let fundraiser_wallet_id_ = WalletId::from(&fundraiser);
    let mut deal_order = crate::protos::DealOrder {
        blockchain: String::from("ethereum"),
        dst_address: investor_address_id.clone().into(),
        src_address: fundraiser_address_id.clone().into(),
        amount: String::from("1"),
        sighash: investor.clone().to_string(),
        ..::core::default::Default::default()
    };
    let mut investor_address = crate::protos::Address {
        blockchain: String::from("ethereum"),
        value: String::from("myaddress"),
        network: String::from("rinkeby"),
        sighash: investor.clone().to_string(),
    };
    let mut fundraiser_address = crate::protos::Address {
        blockchain: String::from("ethereum"),
        value: String::from("otheraddress"),
        network: String::from("rinkeby"),
        sighash: fundraiser.clone().to_string(),
    };
    let mut transfer_id = AddressId::with_prefix_key(
        TRANSFER,
        &string!(
            &investor_address.blockchain,
            &command.blockchain_tx_id,
            &investor_address.network
        ),
    );
    let mut transfer = crate::protos::Transfer {
        blockchain: investor_address.blockchain.clone(),
        dst_address: fundraiser_address_id.clone().to_string(),
        src_address: investor_address_id.clone().to_string(),
        order: command.order_id.clone(),
        amount: (command.gain.clone() + 1).to_string(),
        tx: command.blockchain_tx_id.clone(),
        sighash: investor.clone().to_string(),
        block: 8.to_string(),
        processed: false,
    };
    let mut guid = command_guid_.clone();
    {
        let sig = crate::handler::types::SigHash(investor.clone().to_string());
        ctx.expect_sighash().return_once(move |_| Ok(sig));
    }
    {
        let guid = command_guid_.clone();
        ctx.expect_guid().returning(move |_| guid.clone());
    }
    {
        let address = investor_wallet_id_.clone();
        let ret = tx_fee.clone();
        tx_ctx
            .expect_get_state_entry()
            .withf(move |addr| address.as_str() == addr)
            .return_once(move |_| Ok(wallet_with(Option::from(ret))));
    }
    expect_get_state_entry(
        &mut tx_ctx,
        deal_order_id.clone(),
        Some(deal_order.clone()),
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
        fundraiser_address_id.clone(),
        Some(fundraiser_address.clone()),
        None,
    );
    expect_get_state_entry(
        &mut tx_ctx,
        transfer_id.clone(),
        <Option<crate::protos::Wallet>>::None,
        None,
    );
    {
        let ret = Ok(());
        ctx.expect_verify().return_once(move |_| ret);
    }
    expect_set_state_entries(
        &mut tx_ctx,
        vec![
            (
                transfer_id.clone().to_string().to_string(),
                transfer.clone().to_bytes().into(),
            ),
            (
                investor_wallet_id_.clone().to_string(),
                wallet_with(Some(0)).unwrap().into(),
            ),
            make_fee(&guid, &investor, Some(8)),
        ],
    );
    execute_success(command, &request, &tx_ctx, &mut ctx);
}
