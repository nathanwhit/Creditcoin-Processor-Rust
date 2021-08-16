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
    let my_sighash_signer =
        signer_with_secret("3c133fe90c6dead21bc5a2a7e16461fbdfd394c50c89222811ebd8d989c42a29");
    let my_sighash = SigHash::from(&my_sighash_signer);
    let fundraiser_sighash_signer =
        signer_with_secret("481269c7a99268d1b74d971499709b4158bb166cfa525a56e3b2f7be409a62eb");
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
    let my_sighash_signer =
        signer_with_secret("6b8e3f47a58f3ea32aa0dc96406d26a76b4075b066f0a95f29c233f7bfcdf147");
    let my_sighash = SigHash::from(&my_sighash_signer);
    let fundraiser_sighash_signer =
        signer_with_secret("8152255372b2da609bebf5fa66d1a7bdea017e4776d971f6732dc38add5f7ef7");
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
    let my_sighash_signer =
        signer_with_secret("97dd769194342f371aab634251f1cd8ed9c72350f32168200559148fbee3ba89");
    let my_sighash = SigHash::from(&my_sighash_signer);
    let fundraiser_sighash_signer =
        signer_with_secret("9b8b66c38ba3d7c58454caf9559bedea3273d4af34d4533bb279d27eeff79e6a");
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
    let my_sighash_signer =
        signer_with_secret("848f01a2dd2f43cd2d0a7aa0c47e9ff76a01b81bcc7c4e7ec14d2aa7d19e7edf");
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
