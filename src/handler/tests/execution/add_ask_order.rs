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
        signer_with_secret("debc4dbf50ec93a211d1da5f276e2d218155da973149b307de88abb099fd4d50");
    let investor = SigHash::from(&investor_signer);
    let mut tx_fee = TX_FEE.clone();
    let mut request = TpProcessRequest {
        tip: 3,
        ..::core::default::Default::default()
    };
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
    let mut address_id = address_id_for("investoraddress");
    let mut command = AddAskOrder {
        address_id: address_id.clone().into(),
        amount_str: "1000".into(),
        interest: "100".into(),
        maturity: "10".into(),
        fee_str: "1".into(),
        expiration: 10000.into(),
    };
    let command_guid_ = Guid("some_guid".into());
    let mut ask_order_id =
        Address::with_prefix_key(ASK_ORDER.clone(), command_guid_.clone().as_str());
    let investor_wallet_id_ = WalletId::from(&investor);
    let mut address_proto = address_for("investoraddress", &investor.clone());
    let mut ask_order = crate::protos::AskOrder {
        blockchain: address_proto.clone().blockchain.clone(),
        address: command.clone().address_id.clone(),
        amount: command.clone().amount_str.clone(),
        interest: command.clone().interest.clone(),
        maturity: command.clone().maturity.clone(),
        fee: command.clone().fee_str.clone(),
        expiration: command.clone().expiration.into(),
        block: (request.tip - 1).to_string(),
        sighash: investor.clone().into(),
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
        ask_order_id.clone(),
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
    execute_success(command, &request, &tx_ctx, &mut ctx);
}
