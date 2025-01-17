use comfy_table::presets;
use comfy_table::Table;
use penumbra_asset::asset::Id;
use penumbra_asset::ValueView;
use penumbra_dex::swap::SwapView;
use penumbra_dex::swap_claim::SwapClaimView;
use penumbra_fee::Fee;
use penumbra_keys::AddressView;
use penumbra_num::Amount;
use penumbra_shielded_pool::SpendView;
use penumbra_transaction::view::action_view::OutputView;
use penumbra_transaction::TransactionView;

// Issues identified:
// TODO: FeeView
// TODO: TradingPairView
// Implemented some helper functions which may make more sense as methods on existing Structs

// a helper function to create pretty placeholders for encrypted information
fn format_opaque_bytes(bytes: &[u8]) -> String {
    if bytes.len() < 8 {
        return String::new();
    } else {
        /*
        // TODO: Hm, this can allow the same color for both, should rejig things to avoid this
        // Select foreground and background colors based on the first 8 bytes.
        let fg_color_index = bytes[0] % 8;
        let bg_color_index = bytes[4] % 8;

        // ANSI escape codes for foreground and background colors.
        let fg_color_code = 37; // 30 through 37 are foreground colors
        let bg_color_code = 40; // 40 through 47 are background colors
        */

        // to be more general, perhaps this should be configurable
        // an opaque address needs less space than an opaque memo, etc
        let max_bytes = 32;
        let rem = if bytes.len() > max_bytes {
            bytes[0..max_bytes].to_vec()
        } else {
            bytes.to_vec()
        };

        // Convert the rest of the bytes to hexadecimal.
        let hex_str = hex::encode_upper(rem);
        let opaque_chars: String = hex_str
            .chars()
            .map(|c| {
                match c {
                    '0' => "\u{2595}",
                    '1' => "\u{2581}",
                    '2' => "\u{2582}",
                    '3' => "\u{2583}",
                    '4' => "\u{2584}",
                    '5' => "\u{2585}",
                    '6' => "\u{2586}",
                    '7' => "\u{2587}",
                    '8' => "\u{2588}",
                    '9' => "\u{2589}",
                    'A' => "\u{259A}",
                    'B' => "\u{259B}",
                    'C' => "\u{259C}",
                    'D' => "\u{259D}",
                    'E' => "\u{259E}",
                    'F' => "\u{259F}",
                    _ => "",
                }
                .to_string()
            })
            .collect();

        //format!("\u{001b}[{};{}m{}", fg_color_code, bg_color_code, block_chars)
        format!("{}", opaque_chars)
    }
}

// feels like these functions should be extension traits of their respective structs
// propose moving this to core/keys/src/address/view.rs
fn format_address_view(address_view: &AddressView) -> String {
    match address_view {
        AddressView::Decoded {
            address: _,
            index,
            wallet_id: _,
        } => {
            if !index.is_ephemeral() {
                format!("[account {:?}]", index.account)
            } else {
                format!("[account {:?} (one-time address)]", index.account)
            }
        }
        AddressView::Opaque { address } => {
            // The address being opaque just means we can't see the internal structure,
            // we should render the content so it can be copy-pasted.
            format!("{}", address)
        }
    }
}

// feels like these functions should be extension traits of their respective structs
// propose moving this to core/asset/src/value.rs
fn format_value_view(value_view: &ValueView) -> String {
    match value_view {
        ValueView::KnownAssetId {
            amount,
            metadata: denom,
            ..
        } => {
            let unit = denom.default_unit();
            format!("{}{}", unit.format_value(*amount), unit)
        }
        ValueView::UnknownAssetId { amount, asset_id } => {
            format!("{}{}", amount, asset_id)
        }
    }
}

fn format_fee(fee: &Fee) -> String {
    // TODO: Implement FeeView to show decrypted fee.
    format!("{}", fee.amount())
}

fn format_asset_id(asset_id: &Id) -> String {
    // TODO: Implement TradingPairView to show decrypted .asset_id()
    let input = &asset_id.to_string();
    let truncated = &input[0..10]; //passet1
    let ellipsis = "...";
    let end = &input[(input.len() - 3)..];
    format!("{}{}{}", truncated, ellipsis, end)
}

// When handling ValueViews inside of a Visible variant of an ActionView, handling both cases might be needlessly verbose
// potentially this makes sense as a method on the ValueView enum
// propose moving this to core/asset/src/value.rs
fn value_view_amount(value_view: &ValueView) -> Amount {
    match value_view {
        ValueView::KnownAssetId { amount, .. } | ValueView::UnknownAssetId { amount, .. } => {
            *amount
        }
    }
}

pub trait TransactionViewExt {
    /// Render this transaction view on stdout.
    fn render_terminal(&self);
}

impl TransactionViewExt for TransactionView {
    fn render_terminal(&self) {
        let fee = &self.body_view.transaction_parameters.fee;
        // the denomination should be visible here... does a FeeView exist?
        println!("Fee: {}", format_fee(&fee));

        println!(
            "Expiration Height: {}",
            &self.body_view.transaction_parameters.expiry_height
        );

        if let Some(memo_view) = &self.body_view.memo_view {
            match memo_view {
                penumbra_transaction::MemoView::Visible {
                    plaintext,
                    ciphertext: _,
                } => {
                    println!("Memo Sender: {}", &plaintext.return_address.address());
                    println!("Memo Text: \n{}\n", &plaintext.text);
                }
                penumbra_transaction::MemoView::Opaque { ciphertext } => {
                    println!("Encrypted Memo: \n{}\n", format_opaque_bytes(&ciphertext.0));
                }
            }
        }

        let mut actions_table = Table::new();
        actions_table.load_preset(presets::NOTHING);
        actions_table.set_header(vec!["Tx Action", "Description"]);

        // Iterate over the ActionViews in the TxView & display as appropriate
        for action_view in &self.body_view.action_views {
            let action: String;

            let row = match action_view {
                penumbra_transaction::ActionView::Spend(spend) => {
                    match spend {
                        SpendView::Visible { spend: _, note } => {
                            action = format!(
                                "{} -> {}",
                                format_address_view(&note.address),
                                format_value_view(&note.value)
                            );
                            ["Spend", &action]
                        }
                        SpendView::Opaque { spend } => {
                            let bytes = spend.body.nullifier.to_bytes(); // taken to be a unique value, for aesthetic reasons
                            action = format_opaque_bytes(&bytes);
                            ["Spend", &action]
                        }
                    }
                }
                penumbra_transaction::ActionView::Output(output) => {
                    match output {
                        OutputView::Visible {
                            output: _,
                            note,
                            payload_key: _,
                        } => {
                            action = format!(
                                "{} -> {}",
                                format_value_view(&note.value),
                                format_address_view(&note.address),
                            );
                            ["Output", &action]
                        }
                        OutputView::Opaque { output } => {
                            let bytes = output.body.note_payload.encrypted_note.0; // taken to be a unique value, for aesthetic reasons
                            action = format_opaque_bytes(&bytes);
                            ["Output", &action]
                        }
                    }
                }
                penumbra_transaction::ActionView::Swap(swap) => {
                    // Typical swaps are one asset for another, but we can't know that for sure.
                    match swap {
                        SwapView::Visible {
                            swap: _,
                            swap_plaintext,
                        } => {
                            let (from_asset, from_value, to_asset) = match (
                                swap_plaintext.delta_1_i.value(),
                                swap_plaintext.delta_2_i.value(),
                            ) {
                                (0, v) if v > 0 => (
                                    swap_plaintext.trading_pair.asset_2(),
                                    swap_plaintext.delta_2_i,
                                    swap_plaintext.trading_pair.asset_1(),
                                ),
                                (v, 0) if v > 0 => (
                                    swap_plaintext.trading_pair.asset_1(),
                                    swap_plaintext.delta_1_i,
                                    swap_plaintext.trading_pair.asset_2(),
                                ),
                                // The pathological case (both assets have output values).
                                _ => (
                                    swap_plaintext.trading_pair.asset_1(),
                                    swap_plaintext.delta_1_i,
                                    swap_plaintext.trading_pair.asset_1(),
                                ),
                            };

                            action = format!(
                                "{} {} for {} and paid claim fee {}",
                                from_value,
                                format_asset_id(&from_asset),
                                format_asset_id(&to_asset),
                                format_fee(&swap_plaintext.claim_fee),
                            );

                            ["Swap", &action]
                        }
                        SwapView::Opaque { swap } => {
                            action = format!(
                                "Opaque swap for trading pair: {} <=> {}",
                                format_asset_id(&swap.body.trading_pair.asset_1()),
                                format_asset_id(&swap.body.trading_pair.asset_2()),
                            );
                            ["Swap", &action]
                        }
                    }
                }
                penumbra_transaction::ActionView::SwapClaim(swap_claim) => {
                    match swap_claim {
                        SwapClaimView::Visible {
                            swap_claim,
                            output_1,
                            output_2,
                        } => {
                            // View service can't see SwapClaims: https://github.com/penumbra-zone/penumbra/issues/2547
                            dbg!(swap_claim);
                            let claimed_value = match (
                                value_view_amount(&output_1.value).value(),
                                value_view_amount(&output_2.value).value(),
                            ) {
                                (0, v) if v > 0 => format_value_view(&output_2.value),
                                (v, 0) if v > 0 => format_value_view(&output_1.value),
                                // The pathological case (both assets have output values).
                                _ => format!(
                                    "{} and {}",
                                    format_value_view(&output_1.value),
                                    format_value_view(&output_2.value),
                                ),
                            };

                            action = format!(
                                "Claimed {} with fee {:?}",
                                claimed_value,
                                format_fee(&swap_claim.body.fee),
                            );
                            ["Swap Claim", &action]
                        }
                        SwapClaimView::Opaque { swap_claim } => {
                            let bytes = swap_claim.body.nullifier.to_bytes(); // taken to be a unique value, for aesthetic reasons
                            action = format_opaque_bytes(&bytes);
                            ["Swap Claim", &action]
                        }
                    }
                }
                penumbra_transaction::ActionView::Ics20Withdrawal(withdrawal) => {
                    let unit = withdrawal.denom.best_unit_for(withdrawal.amount);
                    action = format!(
                        "{}{} via {} to {}",
                        unit.format_value(withdrawal.amount),
                        unit,
                        withdrawal.source_channel,
                        withdrawal.destination_chain_address,
                    );
                    ["Ics20 Withdrawal", &action]
                }
                penumbra_transaction::ActionView::PositionOpen(position_open) => {
                    let position = &position_open.position;
                    /* TODO: leaving this around since we may want it to render prices
                    let _unit_pair = DirectedUnitPair {
                        start: unit_1.clone(),
                        end: unit_2.clone(),
                    };
                    */

                    action = format!(
                        "Reserves: ({} {}, {} {}) Fee: {} ID: {}",
                        position.reserves.r1,
                        format_asset_id(&position.phi.pair.asset_1()),
                        position.reserves.r2,
                        format_asset_id(&position.phi.pair.asset_2()),
                        position.phi.component.fee,
                        position.id(),
                    );
                    ["Open Liquidity Position", &action]
                }
                penumbra_transaction::ActionView::PositionClose(_) => {
                    ["Close Liquitity Position", ""]
                }
                penumbra_transaction::ActionView::PositionWithdraw(_) => {
                    ["Withdraw Liquitity Position", ""]
                }
                penumbra_transaction::ActionView::ProposalDepositClaim(proposal_deposit_claim) => {
                    action = format!(
                        "Claim Deposit for Governance Proposal #{}",
                        proposal_deposit_claim.proposal
                    );
                    [&action, ""]
                }
                penumbra_transaction::ActionView::ProposalSubmit(proposal_submit) => {
                    action = format!(
                        "Submit Governance Proposal #{}",
                        proposal_submit.proposal.id
                    );
                    [&action, ""]
                }
                penumbra_transaction::ActionView::ProposalWithdraw(proposal_withdraw) => {
                    action = format!(
                        "Withdraw Governance Proposal #{}",
                        proposal_withdraw.proposal
                    );
                    [&action, ""]
                }
                penumbra_transaction::ActionView::IbcRelay(_) => ["IBC Relay", ""],
                penumbra_transaction::ActionView::DelegatorVote(_) => ["Delegator Vote", ""],
                penumbra_transaction::ActionView::ValidatorDefinition(_) => {
                    ["Upload Validator Definition", ""]
                }
                penumbra_transaction::ActionView::ValidatorVote(_) => ["Validator Vote", ""],
                penumbra_transaction::ActionView::CommunityPoolDeposit(_) => {
                    ["Community Pool Deposit", ""]
                }
                penumbra_transaction::ActionView::CommunityPoolSpend(_) => {
                    ["Community Pool Spend", ""]
                }
                penumbra_transaction::ActionView::CommunityPoolOutput(_) => {
                    ["Community Pool Output", ""]
                }
                penumbra_transaction::ActionView::Delegate(_) => ["Delegation", ""],
                penumbra_transaction::ActionView::Undelegate(_) => ["Undelegation", ""],
                penumbra_transaction::ActionView::UndelegateClaim(_) => ["Undelegation Claim", ""],
            };

            actions_table.add_row(row);
        }

        // Print table of actions and their descriptions
        println!("{actions_table}");
    }
}
