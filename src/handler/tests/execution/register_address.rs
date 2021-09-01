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
    let my_sighash_signer =
        signer_with_secret("827c39480011a29fa972ed8b671ee5a69edd13e24b5442ee2694514e56d15d88");
    let my_sighash = SigHash::from(&my_sighash_signer);
    let mut tse = ToStateEntryCtx::new(2u64);
    let mut tx_fee = TX_FEE.clone();
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
    let mut request = TpProcessRequest {
        tip: 2,
        ..::core::default::Default::default()
    };
    let mut command = RegisterAddress {
        blockchain: "ethereum".into(),
        address: "myaddress".into(),
        network: "rinkeby".into(),
    };
    let command_guid_ = Guid("some_guid".into());
    let my_sighash_wallet_id_ = WalletId::from(&my_sighash);
    let command_guid_ = Guid("some_guid".into());
    let mut address_proto = crate::protos::Address {
        blockchain: command.blockchain.clone(),
        value: command.address.clone(),
        network: command.network.clone(),
        sighash: my_sighash.to_string(),
    };
    let mut address = AddressId::with_prefix_key(
        crate::handler::constants::ADDR,
        &string!("ethereum", "myaddress", "rinkeby"),
    );
    let mut guid = command_guid_.clone();
    {
        let sig = crate::handler::types::SigHash(my_sighash.clone().to_string());
        ctx.expect_sighash().return_once(move |_| Ok(sig));
    }
    {
        let guid = command_guid_.clone();
        ctx.expect_guid().returning(move |_| guid.clone());
    }
    {
        let address = my_sighash_wallet_id_.clone();
        let ret = Some(tx_fee.clone());
        tx_ctx
            .expect_get_state_entry()
            .withf(move |addr| address.as_str() == addr)
            .return_once(move |_| Ok(wallet_with(ret)));
    }
    expect_get_state_entry(
        &mut tx_ctx,
        address.clone(),
        <Option<crate::protos::Wallet>>::None,
        None,
    );
    expect_set_state_entries(
        &mut tx_ctx,
        vec![
            (
                address.clone().to_string().to_string(),
                address_proto.clone().to_bytes().into(),
            ),
            (
                my_sighash_wallet_id_.clone().to_string(),
                wallet_with(Some(0)).unwrap().into(),
            ),
            make_fee(&guid, &my_sighash, Some(1)),
        ],
    );
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
    let my_sighash_signer =
        signer_with_secret("48b0ae97607427a8550e4da5edc8da0a04617adde25c98a405a0c47114cdf69e");
    let my_sighash = SigHash::from(&my_sighash_signer);
    let other_sighash_signer =
        signer_with_secret("0bf47d913365b3c163897b3a40a03db6c14c2c8637ac732d93552b3ce6dbfabe");
    let other_sighash = SigHash::from(&other_sighash_signer);
    let mut tse = ToStateEntryCtx::new(3u64);
    let mut tx_fee = TX_FEE.clone();
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
    let mut request = TpProcessRequest {
        tip: 2,
        ..::core::default::Default::default()
    };
    let mut command = RegisterAddress {
        blockchain: "ethereum".into(),
        address: "myaddress".into(),
        network: "rinkeby".into(),
    };
    let command_guid_ = Guid("some_guid".into());
    let mut address_proto = crate::protos::Address {
        blockchain: command.blockchain.clone(),
        value: command.address.clone(),
        network: command.network.clone(),
        sighash: my_sighash.clone().to_string(),
    };
    let mut address = AddressId::with_prefix_key(
        crate::handler::constants::ADDR,
        &string!("ethereum", "myaddress", "rinkeby"),
    );
    let my_sighash_wallet_id_ = WalletId::from(&my_sighash);
    let other_sighash_wallet_id_ = WalletId::from(&other_sighash);
    let command_guid_ = Guid("some_guid".into());
    {
        let sig = crate::handler::types::SigHash(my_sighash.clone().to_string());
        ctx.expect_sighash().return_once(move |_| Ok(sig));
    }
    {
        let address = my_sighash_wallet_id_.clone();
        let ret = tx_fee.clone();
        tx_ctx
            .expect_get_state_entry()
            .withf(move |addr| address.as_str() == addr)
            .return_once(move |_| Ok(wallet_with(Option::from(ret))));
    }
    expect_get_state_entry(
        &mut tx_ctx,
        address.clone(),
        Some(address_proto.clone()),
        None,
    );
    execute_failure(
        command,
        &request,
        &tx_ctx,
        &mut ctx,
        "The address has been already registered",
    );
}
