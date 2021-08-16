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
        signer_with_secret("c250b76ec20c4b510978117d443784c523dd25ea34cc6e38bec3240f4df7ecd5");
    let my_sighash = SigHash::from(&my_sighash_signer);
    let fundraiser_sighash_signer =
        signer_with_secret("50dd5bb3cf7baf745f8b9a9a4d0737abfb46db83dc35c42b30947fb92703dcdf");
    let fundraiser_sighash = SigHash::from(&fundraiser_sighash_signer);
    let mut tx_fee = TX_FEE.clone();
    let mut request = TpProcessRequest {
        tip: 3,
        ..Default::default()
    };
    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();
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
        let guid = guid.clone().clone();
        ctx.expect_guid().returning(move |_| guid.clone());
    }
    {
        let address = my_sighash_wallet_id_.clone().clone();
        let ret = amount_needed.clone().clone();
        tx_ctx
            .expect_get_state_entry()
            .withf(move |addr| address.as_str() == addr)
            .return_once(move |_| Ok(wallet_with(Option::from(ret))));
    }
    {
        let address = fundraiser_sighash_wallet_id_.clone().clone();
        let ret = 0.clone();
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
                Address::with_prefix_key(crate::handler::constants::FEE, guid.as_str()).to_string(),
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
        signer_with_secret("0f840553c290c0edee8c6203767d39685e7b7b9abd51de6f322b03ef2e56babf");
    let my_sighash = SigHash::from(&my_sighash_signer);
    let fundraiser_sighash_signer =
        signer_with_secret("7bcbeecf06719c532498caf170ab9081284bd91fcacad775c850ccffa67a1dac");
    let fundraiser_sighash = SigHash::from(&fundraiser_sighash_signer);
    let mut tx_fee = TX_FEE.clone();
    let mut request = TpProcessRequest::default();
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
        let address = my_sighash_wallet_id_.clone().clone();
        let ret = Some(1).clone();
        tx_ctx
            .expect_get_state_entry()
            .withf(move |addr| address.as_str() == addr)
            .return_once(move |_| Ok(wallet_with(ret)));
    }
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
        signer_with_secret("8fb7a9ee534e27b004424d6073b83aa91c032860f2e150d05eb64e7ec4354bd0");
    let my_sighash = SigHash::from(&my_sighash_signer);
    let fundraiser_sighash_signer =
        signer_with_secret("d226d9f3133f374d866860e65e604f9b5fea737631239e1a716339cf0858e7f5");
    let fundraiser_sighash = SigHash::from(&fundraiser_sighash_signer);
    let mut tx_fee = TX_FEE.clone();
    let mut request = TpProcessRequest::default();
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
        let address = my_sighash_wallet_id_.clone().clone();
        let ret = tx_fee.clone().clone();
        tx_ctx
            .expect_get_state_entry()
            .withf(move |addr| address.as_str() == addr)
            .return_once(move |_| Ok(wallet_with(Option::from(ret))));
    }
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
        signer_with_secret("1f033c7d7833fe5433ae7f8ad3f940e8c4baa2a44292730315f42a8c1d6164cf");
    let my_sighash = SigHash::from(&my_sighash_signer);
    let mut tx_fee = TX_FEE.clone();
    let mut request = TpProcessRequest::default();
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
    execute_failure(command, &request, &tx_ctx, &mut ctx, "Invalid destination");
}
