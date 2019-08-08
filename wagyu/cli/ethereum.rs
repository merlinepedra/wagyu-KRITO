use crate::cli::{flag, option, subcommand, types::*, CLI, CLIError};
use crate::ethereum::{
    EthereumAddress, EthereumDerivationPath, EthereumMnemonic,
    EthereumPrivateKey, EthereumPublicKey, EthereumExtendedPrivateKey,
    EthereumExtendedPublicKey, wordlist::*,
};
use crate::model::{ExtendedPrivateKey, ExtendedPublicKey, MnemonicExtended, PrivateKey, PublicKey};

use clap::ArgMatches;
use rand::rngs::StdRng;
use rand_core::SeedableRng;
use serde::Serialize;
use std::{fmt, fmt::Display, marker::PhantomData, str::FromStr};

/// Represents custom options for a Ethereum wallet
#[derive(Serialize, Clone, Debug)]
pub struct EthereumOptions {
    pub wallet_values: Option<WalletValues>,
    pub hd_values: Option<HdValues>,
    pub count: usize,
    pub json: bool,
}

/// Represents values to derive standard wallets
#[derive(Serialize, Clone, Debug)]
pub struct WalletValues {
    pub private_key: Option<String>,
    pub public_key: Option<String>,
    pub address: Option<String>,
}

/// Represents values to derive HD wallets
#[derive(Serialize, Clone, Debug, Default)]
pub struct HdValues {
    pub account: Option<String>,
    pub change: Option<String>,
    pub extended_private_key: Option<String>,
    pub extended_public_key: Option<String>,
    pub index: Option<String>,
    pub language: Option<String>,
    pub mnemonic: Option<String>,
    pub password: Option<String>,
    pub path: Option<String>,
    pub word_count: Option<u8>,
}

/// Represents a generic wallet to output
#[derive(Serialize, Debug, Default)]
struct EthereumWallet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mnemonic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_private_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_public_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    pub address: String,
}

#[cfg_attr(tarpaulin, skip)]
impl Display for EthereumWallet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = [
            match &self.path {
                Some(path) => format!("      Path                 {}\n", path),
                _ => "".to_owned(),
            },
            match &self.password {
                Some(password) => format!("      Password             {}\n", password),
                _ => "".to_owned(),
            },
            match &self.mnemonic {
                Some(mnemonic) => format!("      Mnemonic             {}\n", mnemonic),
                _ => "".to_owned(),
            },
            match &self.extended_private_key {
                Some(extended_private_key) => format!("      Extended Private Key {}\n", extended_private_key),
                _ => "".to_owned(),
            },
            match &self.extended_public_key {
                Some(extended_public_key) => format!("      Extended Public Key  {}\n", extended_public_key),
                _ => "".to_owned(),
            },
            match &self.private_key {
                Some(private_key) => format!("      Private Key          {}\n", private_key),
                _ => "".to_owned(),
            },
            match &self.public_key {
                Some(public_key) => format!("      Public Key           {}\n", public_key),
                _ => "".to_owned(),
            },
            format!("      Address              {}\n", self.address),
        ]
        .concat();

        // Removes final new line character
        let output = output[..output.len() - 1].to_owned();
        write!(f, "\n{}", output)
    }
}

pub struct EthereumCLI;

impl CLI for EthereumCLI {
    type Options = EthereumOptions;

    const NAME: NameType = "ethereum";
    const ABOUT: AboutType = "Generates a Ethereum wallet (include -h for more options)";
    const FLAGS: &'static [FlagType] = &[flag::JSON];
    const OPTIONS: &'static [OptionType] = &[option::COUNT];
    const SUBCOMMANDS: &'static [SubCommandType] = &[
        subcommand::HD_ETHEREUM,
        subcommand::IMPORT_ETHEREUM,
        subcommand::IMPORT_HD_ETHEREUM,
    ];

    /// Handle all CLI arguments and flags for Ethereum
    #[cfg_attr(tarpaulin, skip)]
    fn parse(arguments: &ArgMatches) ->  Result<Self::Options, CLIError> {
        let mut options = EthereumOptions {
            wallet_values: None,
            hd_values: None,
            count: clap::value_t!(arguments.value_of("count"), usize).unwrap_or_else(|_e| 1),
            json: arguments.is_present("json"),
        };

        match arguments.subcommand() {
            ("hd", Some(hd_matches)) => {
                let language = hd_matches.value_of("language").map(|s| s.to_string());
                let password = hd_matches.value_of("password").map(|s| s.to_string());
                let path = hd_matches.value_of("derivation").map(|s| s.to_string());
                let word_count = hd_matches.value_of("word count").map(|s| s.parse::<u8>().unwrap());

                options.count = clap::value_t!(hd_matches.value_of("count"), usize).unwrap_or(options.count);
                options.json |= hd_matches.is_present("json");
                options.hd_values = Some(HdValues {
                    language,
                    mnemonic: None,
                    password,
                    path,
                    word_count,
                    ..Default::default()
                });
            }
            ("import", Some(import_matches)) => {
                let address = import_matches.value_of("address").map(|s| s.to_string());
                let public_key = import_matches.value_of("public key").map(|s| s.to_string());
                let private_key = import_matches.value_of("private key").map(|s| s.to_string());

                options.json |= import_matches.is_present("json");
                options.wallet_values = Some(WalletValues { address, public_key, private_key });
            }
            ("import-hd", Some(import_hd_matches)) => {
                let account = import_hd_matches.value_of("account").map(|i| i.to_string());
                let change = import_hd_matches.value_of("change").map(|i| i.to_string());
                let extended_private_key = import_hd_matches.value_of("extended private").map(|s| s.to_string());
                let extended_public_key = import_hd_matches.value_of("extended public").map(|s| s.to_string());
                let index = import_hd_matches.value_of("index").map(|i| i.to_string());
                let mnemonic = import_hd_matches.value_of("mnemonic").map(|s| s.to_string());
                let password = import_hd_matches.value_of("password").map(|s| s.to_string());
                let path = import_hd_matches.value_of("derivation").map(|s| s.to_string());

                options.json |= import_hd_matches.is_present("json");
                options.hd_values = Some(HdValues {
                    account,
                    change,
                    extended_private_key,
                    extended_public_key,
                    index,
                    mnemonic,
                    password,
                    path,
                    ..Default::default()
                });
            }
            _ => {}
        };

        Ok(options)
    }

    /// Generate the Ethereum wallet and print the relevant fields
    #[cfg_attr(tarpaulin, skip)]
    fn print(options: Self::Options) -> Result<(), CLIError> {
        for _ in 0..options.count {
            let wallet = match (options.wallet_values.to_owned(), options.hd_values.to_owned()) {
                (None, None) => {
                    let private_key = EthereumPrivateKey::new(&mut StdRng::from_entropy())?;
                    let public_key = private_key.to_public_key();
                    let address = public_key.to_address(&PhantomData)?;

                    EthereumWallet {
                        private_key: Some(private_key.to_string()),
                        public_key: Some(public_key.to_string()),
                        address: address.to_string(),
                        ..Default::default()
                    }
                }
                (Some(wallet_values), None) => {
                    match (
                        wallet_values.private_key.as_ref(),
                        wallet_values.public_key.as_ref(),
                        wallet_values.address.as_ref(),
                    ) {
                        (Some(private_key), None, None) => match EthereumPrivateKey::from_str(&private_key) {
                            Ok(private_key) => {
                                let public_key = private_key.to_public_key();
                                let address = public_key.to_address(&PhantomData)?;

                                EthereumWallet {
                                    private_key: Some(private_key.to_string()),
                                    public_key: Some(public_key.to_string()),
                                    address: address.to_string(),
                                    ..Default::default()
                                }
                            }
                            Err(_) => {
                                let private_key = EthereumPrivateKey::from_str(&private_key)?;
                                let public_key = private_key.to_public_key();
                                let address = public_key.to_address(&PhantomData)?;

                                EthereumWallet {
                                    private_key: Some(private_key.to_string()),
                                    public_key: Some(public_key.to_string()),
                                    address: address.to_string(),
                                    ..Default::default()
                                }
                            }
                        },
                        (None, Some(public_key), None) => {
                            let public_key = EthereumPublicKey::from_str(&public_key)?;
                            let address = public_key.to_address(&PhantomData)?;

                            EthereumWallet { public_key: Some(public_key.to_string()), address: address.to_string(), ..Default::default() }
                        }
                        (None, None, Some(address)) => match EthereumAddress::from_str(&address) {
                            Ok(address) => EthereumWallet { address: address.to_string(), ..Default::default() },
                            Err(error) => return Err(CLIError::AddressError(error)),
                        },
                        _ => unreachable!(),
                    }
                }
                (None, Some(hd_values)) => {

                    fn process_mnemonic<EW: EthereumWordlist>(mnemonic: Option<String>, word_count: u8, password: &Option<&str>)
                                                                                 -> Result<(String, EthereumExtendedPrivateKey), CLIError> {
                        let mnemonic = match mnemonic {
                            Some(mnemonic) => EthereumMnemonic::<EW>::from_phrase(&mnemonic)?,
                            None => EthereumMnemonic::<EW>::new(word_count, &mut StdRng::from_entropy())?,
                        };

                        Ok((mnemonic.to_string(), mnemonic.to_extended_private_key(*password)?))
                    }

                    const DEFAULT_WORD_COUNT: u8 = 12;

                    let index = hd_values.index.unwrap_or("0".to_string());
                    let path: String = match hd_values.path.as_ref().map(String::as_str) {
                        Some("ethereum") => format!("m/44'/60'/0'/{}", index),
                        Some("keepkey") => format!("m/44'/60'/{}'/0", index),
                        Some("ledger-legacy") => format!("m/44'/60'/0'/{}", index),
                        Some("ledger-live") => format!("m/44'/60'/{}'/0/0", index),
                        Some("trezor") => format!("m/44'/60'/0'/{}", index),
                        Some(custom_path) => custom_path.to_string(),
                        None => format!("m/44'/60'/0'/{}", index), // Default - ethereum
                    };

                    let mut final_path = Some(path.to_string());

                    let word_count = match hd_values.word_count {
                        Some(word_count) => word_count,
                        None => DEFAULT_WORD_COUNT,
                    };

                    let password = hd_values.password.as_ref().map(String::as_str);
                    let (mnemonic, extended_private_key, extended_public_key) = match (
                        hd_values.mnemonic,
                        hd_values.extended_private_key,
                        hd_values.extended_public_key,
                    ) {
                        (None, None, None) => {
                            let (mnemonic, master_extended_private_key)
                                = match hd_values.language.as_ref().map(String::as_str) {
                                Some("chinese_simplified") => process_mnemonic::<ChineseSimplified>(None, word_count, &password)?,
                                Some("chinese_traditional") => process_mnemonic::<ChineseTraditional>(None, word_count, &password)?,
                                Some("english") => process_mnemonic::<English>(None, word_count, &password)?,
                                Some("french") => process_mnemonic::<French>(None, word_count, &password)?,
                                Some("italian") => process_mnemonic::<Italian>(None, word_count, &password)?,
                                Some("japanese") => process_mnemonic::<Japanese>(None, word_count, &password)?,
                                Some("korean") => process_mnemonic::<Korean>(None, word_count, &password)?,
                                Some("spanish") => process_mnemonic::<Spanish>(None, word_count, &password)?,
                                _ => process_mnemonic::<English>(None, word_count, &password)?, // Default language - English
                            };

                            let extended_private_key = master_extended_private_key
                                .derive(&EthereumDerivationPath::from_str(&path)?)?;
                            let extended_public_key = extended_private_key.to_extended_public_key();

                            (Some(mnemonic), Some(extended_private_key), extended_public_key)
                        }
                        (Some(mnemonic), None, None) => {
                            let (mnemonic, master_extended_private_key) =
                                process_mnemonic::<ChineseSimplified>(Some(mnemonic.to_owned()), word_count, &password)
                                    .or(process_mnemonic::<ChineseTraditional>(Some(mnemonic.to_owned()), word_count, &password))
                                    .or(process_mnemonic::<English>(Some(mnemonic.to_owned()), word_count, &password))
                                    .or(process_mnemonic::<French>(Some(mnemonic.to_owned()), word_count, &password))
                                    .or(process_mnemonic::<Italian>(Some(mnemonic.to_owned()), word_count, &password))
                                    .or(process_mnemonic::<Japanese>(Some(mnemonic.to_owned()), word_count, &password))
                                    .or(process_mnemonic::<Korean>(Some(mnemonic.to_owned()), word_count, &password))
                                    .or(process_mnemonic::<Spanish>(Some(mnemonic.to_owned()), word_count, &password))?;

                            let extended_private_key = master_extended_private_key
                                .derive(&EthereumDerivationPath::from_str(&path)?)?;
                            let extended_public_key = extended_private_key.to_extended_public_key();

                            (Some(mnemonic.to_string()), Some(extended_private_key), extended_public_key)
                        }
                        (None, Some(extended_private_key), None) => {
                            let mut extended_private_key = EthereumExtendedPrivateKey::from_str(&extended_private_key)?;

                            match hd_values.path {
                                Some(_) => extended_private_key = extended_private_key.derive(&EthereumDerivationPath::from_str(&path)?)?,
                                None => final_path = None,
                            };

                            let extended_public_key = extended_private_key.to_extended_public_key();

                            (None, Some(extended_private_key), extended_public_key)
                        }
                        (None, None, Some(extended_public_key)) => {
                            let mut extended_public_key = EthereumExtendedPublicKey::from_str(&extended_public_key)?;

                            match hd_values.path {
                                Some(_) => extended_public_key = extended_public_key.derive(&EthereumDerivationPath::from_str(&path)?)?,
                                None => final_path = None,
                            };

                            (None, None, extended_public_key)
                        }
                        _ => unreachable!(),
                    };

                    let private_key = extended_private_key.as_ref().map(|key| key.to_private_key().to_string());
                    let public_key = extended_public_key.to_public_key();
                    let address = public_key.to_address(&PhantomData)?;

                    EthereumWallet {
                        path: final_path,
                        password: hd_values.password,
                        mnemonic: mnemonic.map(|key| key.to_string()),
                        extended_private_key: extended_private_key.map(|key| key.to_string()),
                        extended_public_key: Some(extended_public_key.to_string()),
                        private_key,
                        public_key: Some(public_key.to_string()),
                        address: address.to_string(),
                        ..Default::default()
                    }
                }
                _ => unreachable!(),
            };

            match options.json {
                true => println!("{}\n", serde_json::to_string_pretty(&wallet)?),
                false => println!("{}\n", wallet),
            };
        }

        Ok(())
    }
}