//! Chain specification for Marketplace Chain

use cumulus_primitives_core::ParaId;
use marketplace_runtime::{AccountId, AuraId, Signature, EXISTENTIAL_DEPOSIT};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

pub type ChainSpec =
    sc_service::GenericChainSpec<marketplace_runtime::RuntimeGenesisConfig, Extensions>;

pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    pub relay_chain: String,
    pub para_id: u32,
}

impl Extensions {
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

type AccountPublic = <Signature as Verify>::Signer;

pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
    get_from_seed::<AuraId>(seed)
}

pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn marketplace_session_keys(keys: AuraId) -> marketplace_runtime::SessionKeys {
    marketplace_runtime::SessionKeys { aura: keys }
}

pub fn development_config() -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "MPC".into());
    properties.insert("tokenDecimals".into(), 12.into());
    properties.insert("ss58Format".into(), 42.into());

    ChainSpec::builder(
        marketplace_runtime::WASM_BINARY.expect("WASM binary was not built!"),
        Extensions {
            relay_chain: "rococo-local".to_string(),
            para_id: 2002,
        },
    )
    .with_name("Marketplace Development")
    .with_id("marketplace_dev")
    .with_chain_type(ChainType::Development)
    .with_genesis_config_patch(testnet_genesis(
        vec![(
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_collator_keys_from_seed("Alice"),
        )],
        vec![
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_account_id_from_seed::<sr25519::Public>("Charlie"),
            get_account_id_from_seed::<sr25519::Public>("Dave"),
            get_account_id_from_seed::<sr25519::Public>("Eve"),
            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
        ],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        2002.into(),
    ))
    .build()
}

pub fn local_testnet_config() -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "MPC".into());
    properties.insert("tokenDecimals".into(), 12.into());
    properties.insert("ss58Format".into(), 42.into());

    ChainSpec::builder(
        marketplace_runtime::WASM_BINARY.expect("WASM binary was not built!"),
        Extensions {
            relay_chain: "rococo-local".to_string(),
            para_id: 2002,
        },
    )
    .with_name("Marketplace Local Testnet")
    .with_id("marketplace_local")
    .with_chain_type(ChainType::Local)
    .with_genesis_config_patch(testnet_genesis(
        vec![
            (
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_collator_keys_from_seed("Alice"),
            ),
            (
                get_account_id_from_seed::<sr25519::Public>("Bob"),
                get_collator_keys_from_seed("Bob"),
            ),
        ],
        vec![
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_account_id_from_seed::<sr25519::Public>("Charlie"),
            get_account_id_from_seed::<sr25519::Public>("Dave"),
        ],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        2002.into(),
    ))
    .build()
}

fn testnet_genesis(
    invulnerables: Vec<(AccountId, AuraId)>,
    endowed_accounts: Vec<AccountId>,
    root: AccountId,
    id: ParaId,
) -> serde_json::Value {
    serde_json::json!({
        "balances": {
            "balances": endowed_accounts.iter().cloned().map(|k| (k, 1u64 << 60)).collect::<Vec<_>>(),
        },
        "parachainInfo": {
            "parachainId": id,
        },
        "collatorSelection": {
            "invulnerables": invulnerables.iter().cloned().map(|(acc, _)| acc).collect::<Vec<_>>(),
            "candidacyBond": EXISTENTIAL_DEPOSIT * 16,
        },
        "session": {
            "keys": invulnerables
                .into_iter()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),
                        acc,
                        marketplace_session_keys(aura),
                    )
                })
            .collect::<Vec<_>>(),
        },
        "polkadotXcm": {
            "safeXcmVersion": Some(3),
        },
        "sudo": {
            "key": Some(root),
        },
    })
}
