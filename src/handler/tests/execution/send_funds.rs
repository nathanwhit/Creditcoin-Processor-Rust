use super::*;

#[test]
#[allow(
    unused_variables,
    unused_parens,
    unused_imports,
    unused_mut,
    unused_braces
)]
fn send_funds_success() {
    use crate::handler::types::*;
    use std::str::FromStr as _;
    let my_sighash_signer =
        signer_with_secret("827c39480011a29fa972ed8b671ee5a69edd13e24b5442ee2694514e56d15d88");
    let my_sighash = SigHash::from(&my_sighash_signer);
    let fundraiser_sighash_signer =
        signer_with_secret("48b0ae97607427a8550e4da5edc8da0a04617adde25c98a405a0c47114cdf69e");
    let fundraiser_sighash = SigHash::from(&fundraiser_sighash_signer);
    let mut tse = ToStateEntryCtx::new(3u64);
    let mut tx_fee = TX_FEE.clone();
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
    let mut request = TpProcessRequest {
        tip: 3,
        ..Default::default()
    };
    let mut command = SendFunds {
        amount: 1.into(),
        sighash: fundraiser_sighash.clone().into(),
    };
    let command_guid_ = Guid("some_guid".into());
    let mut amount_needed = command.amount.clone() + tx_fee;
    let my_sighash_wallet_id_ = WalletId::from(&my_sighash);
    let fundraiser_sighash_wallet_id_ = WalletId::from(&fundraiser_sighash);
    let command_guid_ = Guid("some_guid".into());
    let mut guid = command_guid_.clone();
    {
        let sig = crate::handler::types::SigHash(my_sighash.clone().to_string());
        ctx.expect_sighash().return_once(move |_| Ok(sig));
    }
    {
        let guid = guid.clone();
        ctx.expect_guid().returning(move |_| guid.clone());
    }
    {
        let address = my_sighash_wallet_id_.clone();
        let ret = amount_needed.clone();
        tx_ctx
            .expect_get_state_entry()
            .withf(move |addr| address.as_str() == addr)
            .return_once(move |_| Ok(wallet_with(Option::from(ret))));
    }
    {
        let address = fundraiser_sighash_wallet_id_.clone();
        let ret = 0;
        tx_ctx
            .expect_get_state_entry()
            .withf(move |addr| address.as_str() == addr)
            .return_once(move |_| Ok(wallet_with(Option::from(ret))));
    }
    expect_set_state_entries(
        &mut tx_ctx,
        vec![
            (
                my_sighash_wallet_id_.clone().to_string(),
                wallet_with(Some(0)).unwrap().into(),
            ),
            (
                fundraiser_sighash_wallet_id_.clone().to_string(),
                wallet_with(Some(1)).unwrap().into(),
            ),
            (
                AddressId::with_prefix_key(crate::handler::constants::FEE, guid.as_str())
                    .to_string(),
                crate::protos::Fee {
                    sighash: my_sighash.clone().into(),
                    block: 2u64.to_string(),
                }
                .to_bytes()
                .into(),
            ),
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
fn send_funds_cannot_afford_fee() {
    use crate::handler::types::*;
    use std::str::FromStr as _;
    let my_sighash_signer =
        signer_with_secret("0bf47d913365b3c163897b3a40a03db6c14c2c8637ac732d93552b3ce6dbfabe");
    let my_sighash = SigHash::from(&my_sighash_signer);
    let fundraiser_sighash_signer =
        signer_with_secret("544f4fe3edb3e8e44d4a1f0050ce03a729b2da887b644e95ec6bf6a0cfdbf0f4");
    let fundraiser_sighash = SigHash::from(&fundraiser_sighash_signer);
    let mut tse = ToStateEntryCtx::new(2u64);
    let mut tx_fee = TX_FEE.clone();
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
    let mut command = SendFunds {
        amount: 1.into(),
        sighash: fundraiser_sighash.clone().into(),
    };
    let command_guid_ = Guid("some_guid".into());
    let my_sighash_wallet_id_ = WalletId::from(&my_sighash);
    let command_guid_ = Guid("some_guid".into());
    {
        let sig = crate::handler::types::SigHash(my_sighash.clone().to_string());
        ctx.expect_sighash().return_once(move |_| Ok(sig));
    }
    {
        let address = my_sighash_wallet_id_.clone();
        let ret = Some(1);
        tx_ctx
            .expect_get_state_entry()
            .withf(move |addr| address.as_str() == addr)
            .return_once(move |_| Ok(wallet_with(ret)));
    }
    let mut request = TpProcessRequest {
        tip: tse.tip().into(),
        ..Default::default()
    };
    execute_failure(command, &request, &tx_ctx, &mut ctx, "Insufficient funds");
}

#[test]
#[allow(
    unused_variables,
    unused_parens,
    unused_imports,
    unused_mut,
    unused_braces
)]
fn send_funds_cannot_afford_amount() {
    use crate::handler::types::*;
    use std::str::FromStr as _;
    let my_sighash_signer =
        signer_with_secret("f5b200e37586793549de011ef43bfac7cee149feb1f1de9a5f558c75ef46714b");
    let my_sighash = SigHash::from(&my_sighash_signer);
    let fundraiser_sighash_signer =
        signer_with_secret("f5cc8af9d13b3190729fb196a385e5b28663ad538724fdadf7ef25ff20c38b31");
    let fundraiser_sighash = SigHash::from(&fundraiser_sighash_signer);
    let mut tse = ToStateEntryCtx::new(2u64);
    let mut tx_fee = TX_FEE.clone();
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
    let mut command = SendFunds {
        amount: 1.into(),
        sighash: fundraiser_sighash.clone().into(),
    };
    let command_guid_ = Guid("some_guid".into());
    let my_sighash_wallet_id_ = WalletId::from(&my_sighash);
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
    let mut request = TpProcessRequest {
        tip: tse.tip().into(),
        ..Default::default()
    };
    execute_failure(command, &request, &tx_ctx, &mut ctx, "Insufficient funds");
}

#[test]
#[allow(
    unused_variables,
    unused_parens,
    unused_imports,
    unused_mut,
    unused_braces
)]
fn send_funds_to_self() {
    use crate::handler::types::*;
    use std::str::FromStr as _;
    let my_sighash_signer =
        signer_with_secret("9628e0b771d0bc1ecd7975011f18d46dab673de62997297b1f40985f6a166dac");
    let my_sighash = SigHash::from(&my_sighash_signer);
    let mut tse = ToStateEntryCtx::new(1u64);
    let mut tx_fee = TX_FEE.clone();
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
    let mut command = SendFunds {
        amount: 1.into(),
        sighash: my_sighash.clone().into(),
    };
    let command_guid_ = Guid("some_guid".into());
    let command_guid_ = Guid("some_guid".into());
    {
        let sig = crate::handler::types::SigHash(my_sighash.clone().to_string());
        ctx.expect_sighash().return_once(move |_| Ok(sig));
    }
    let mut request = TpProcessRequest {
        tip: tse.tip().into(),
        ..Default::default()
    };
    execute_failure(command, &request, &tx_ctx, &mut ctx, "Invalid destination");
}
