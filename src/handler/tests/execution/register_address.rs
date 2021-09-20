use super::*;

#[test]
#[allow(
    unused_variables,
    unused_parens,
    unused_imports,
    unused_mut,
    unused_braces
)]
fn register_address_success() {
    use crate::handler::types::*;
    use std::str::FromStr as _;
    let investor_signer =
        signer_with_secret("827c39480011a29fa972ed8b671ee5a69edd13e24b5442ee2694514e56d15d88");
    let investor = SigHash::from(&investor_signer);
    let mut tse = ToStateEntryCtx::new(2u64);
    let mut tx_fee = TX_FEE.clone();
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
    let mut register_address = RegisterAddress {
        blockchain: "ethereum".into(),
        address: "myaddress".into(),
        network: "rinkeby".into(),
    };
    let (mut address_id, mut address) =
        tse.state_entry_from(register_address.clone(), investor.clone());
    let mut command = register_address.clone();
    let command_guid_ = Guid("some_guid".into());
    let investor_wallet_id_ = WalletId::from(&investor);
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
        let ret = Some(tx_fee.clone());
        tx_ctx
            .expect_get_state_entry()
            .withf(move |addr| address.as_str() == addr)
            .return_once(move |_| Ok(wallet_with(ret)));
    }
    expect_get_state_entry(
        &mut tx_ctx,
        address_id.clone(),
        <Option<crate::protos::Wallet>>::None,
        None,
    );
    expect_set_state_entries(
        &mut tx_ctx,
        vec![
            (
                address_id.clone().to_string().to_string(),
                address.clone().to_bytes().into(),
            ),
            (
                investor_wallet_id_.clone().to_string(),
                wallet_with(Some(0)).unwrap().into(),
            ),
            make_fee(&guid.clone(), &investor.clone(), Some(tse.tip() - 1)),
        ],
    );
    let mut request = TpProcessRequest {
        tip: tse.tip().into(),
        ..Default::default()
    };
    execute_success(command, &request, &tx_ctx, &mut ctx);
}

#[test]
#[allow(
    unused_variables,
    unused_parens,
    unused_imports,
    unused_mut,
    unused_braces
)]
fn register_address_taken() {
    use crate::handler::types::*;
    use std::str::FromStr as _;
    let investor_signer =
        signer_with_secret("48b0ae97607427a8550e4da5edc8da0a04617adde25c98a405a0c47114cdf69e");
    let investor = SigHash::from(&investor_signer);
    let fundraiser_signer =
        signer_with_secret("0bf47d913365b3c163897b3a40a03db6c14c2c8637ac732d93552b3ce6dbfabe");
    let fundraiser = SigHash::from(&fundraiser_signer);
    let mut tse = ToStateEntryCtx::new(3u64);
    let mut tx_fee = TX_FEE.clone();
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
    let mut register_address = RegisterAddress {
        blockchain: "ethereum".into(),
        address: "myaddress".into(),
        network: "rinkeby".into(),
    };
    let (mut address_id, mut address) =
        tse.state_entry_from(register_address.clone(), fundraiser.clone());
    let mut command = register_address.clone();
    let command_guid_ = Guid("some_guid".into());
    let investor_wallet_id_ = WalletId::from(&investor);
    let fundraiser_wallet_id_ = WalletId::from(&fundraiser);
    {
        let sig = crate::handler::types::SigHash(investor.clone().to_string());
        ctx.expect_sighash().return_once(move |_| Ok(sig));
    }
    {
        let address = investor_wallet_id_.clone();
        let ret = tx_fee.clone();
        tx_ctx
            .expect_get_state_entry()
            .withf(move |addr| address.as_str() == addr)
            .return_once(move |_| Ok(wallet_with(Option::from(ret))));
    }
    expect_get_state_entry(&mut tx_ctx, address_id.clone(), Some(address.clone()), None);
    let mut request = TpProcessRequest {
        tip: tse.tip().into(),
        ..Default::default()
    };
    execute_failure(
        command,
        &request,
        &tx_ctx,
        &mut ctx,
        "The address has been already registered",
    );
}
