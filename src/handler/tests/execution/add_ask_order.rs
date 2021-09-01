use super::*;

#[test]
#[allow(
    unused_variables,
    unused_parens,
    unused_imports,
    unused_mut,
    unused_braces
)]
fn add_ask_order_success() {
    use crate::handler::types::*;
    use std::str::FromStr as _;
    let investor_signer =
        signer_with_secret("827c39480011a29fa972ed8b671ee5a69edd13e24b5442ee2694514e56d15d88");
    let investor = SigHash::from(&investor_signer);
    let mut tse = ToStateEntryCtx::new(2u64);
    let mut tx_fee = TX_FEE.clone();
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
    let mut register_address = register_address_for("investoraddress");
    let mut register_address_guid = Guid::from(make_nonce());
    let (mut address_id, mut address) =
        tse.state_entry_from(register_address.clone(), investor.clone());
    let mut add_ask_order = AddAskOrder {
        address_id: address_id.clone().into(),
        amount_str: "1000".into(),
        interest: "100".into(),
        maturity: "10".into(),
        fee_str: "1".into(),
        expiration: 10000.into(),
    };
    let mut add_ask_order_guid = Guid::from(make_nonce());
    let (mut ask_order_id, mut ask_order) = tse.state_entry_from(
        add_ask_order.clone(),
        AddAskOrderArgs {
            guid: add_ask_order_guid.clone(),
            sighash: investor.clone(),
            address: address.clone().clone(),
        },
    );
    let mut command_guid_ = add_ask_order_guid.clone();
    let mut command = add_ask_order.clone();
    let investor_wallet_id_ = WalletId::from(&investor);
    {
        let sig = crate::handler::types::SigHash(investor.clone().to_string());
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
        let address = investor_wallet_id_.clone();
        let ret = tx_fee.clone();
        tx_ctx
            .expect_get_state_entry()
            .withf(move |addr| address.as_str() == addr)
            .return_once(move |_| Ok(wallet_with(Option::from(ret))));
    }
    expect_get_state_entry(
        &mut tx_ctx,
        ask_order_id.clone(),
        <Option<crate::protos::Wallet>>::None,
        None,
    );
    expect_get_state_entry(&mut tx_ctx, address_id.clone(), Some(address.clone()), None);
    expect_set_state_entries(
        &mut tx_ctx,
        vec![
            (
                investor_wallet_id_.clone().to_string(),
                wallet_with(Some(0)).unwrap().into(),
            ),
            (
                ask_order_id.clone().to_string().to_string(),
                ask_order.clone().to_bytes().into(),
            ),
            make_fee(&command_guid_.clone(), &investor.clone(), Some(2)),
        ],
    );
    let mut request = TpProcessRequest {
        tip: tse.tip().into(),
        ..Default::default()
    };
    execute_success(command, &request, &tx_ctx, &mut ctx);
}
