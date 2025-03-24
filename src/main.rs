use ansi_term::Color::{self, Red};
use chrono::NaiveTime;
use clap::Parser;
use cli_table::{print_stdout, Cell, Style, Table};
use num_format::{Locale, ToFormattedString};
use serde::Deserialize;
use serde_json::Value;

/// btcAnalyser - A CLI tool to analyze recent Bitcoin transactions.
///
/// A Bitcoin CLI tool to view the last 'n' unconfirmed transactions,
/// inspect transactions by hash, and inspect transactions addresses.
///
///
#[derive(Parser)]
#[command(version, about, long_about)]
struct Cli {
    #[arg(short = 'e', long)]
    exploration_mode: Option<String>,

    #[arg(short = 'n', long)]
    number_outputs: Option<usize>,

    #[arg(short = 'i', long)]
    inspect_transcation: Option<String>,

    #[arg(short = 'a', long)]
    inspect_address: Option<String>,
}


#[derive(Debug)]
struct UndefinedTransaction {
    hash: String,
    amount_bitcoin: f64,
    amount_usd: f64,
    time: NaiveTime,
}

impl UndefinedTransaction {
    fn new(hash: String, amount_bitcoin: f64, amount_usd: f64, time: NaiveTime) -> UndefinedTransaction {
        UndefinedTransaction {
            hash,
            amount_bitcoin,
            amount_usd,
            time,
        }
    }
}


#[derive(Debug, Deserialize)]
struct Transaction {
    inputs: Vec<Input>,
    out: Vec<Output>,
    hash: String,
}

#[derive(Debug, Deserialize)]
struct Input {
    prev_out: PrevOut,
}

#[derive(Debug, Deserialize)]
struct PrevOut {
    addr: String,
    value: u64,
}

#[derive(Debug, Deserialize)]
struct Output {
    value: u64,
    addr: String,
}

#[derive(Debug, Deserialize)]
struct BitcoinAddress {
    n_tx: u64, // Number of transaction that this address made
    total_received: u64,
    total_sent: u64,
    final_balance: u64,
}

const SATOSHIS_PER_BTC: u32 = 100_000_000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // CTRL-C Handlig
    ctrlc::set_handler(|| {
        println!("{}", Red.paint("\n[!] Exiting...\n"));
        std::process::exit(1);
    })
    .expect("Error setting Ctrl-C handler");

    // Handling Command line arguments
    let cli = Cli::parse();

    // Global variables, that contain the URL to makes request to the API

    let unconfirmed_transactions_url =
        "https://blockchain.info/unconfirmed-transactions?format=json";
    let inspect_transaction_url = "https://blockchain.info/rawtx/";
    let inspect_address_url = "https://blockchain.info/rawaddr/";
    let bitcoin_price_url = "https://api.blockchain.info/stats" ;
    

    match cli.exploration_mode.as_deref() {
            // We check if the user specified a number of outputs, by default is 100
            Some("unconfirmed_transactions") => {
                let number_outputs = match cli.number_outputs {
                    Some(number_outputs) => number_outputs,
                    None => 100,
                };

            // Call the function unconfirmed_transactions to get the number_output od unconfirmed transaction in a vector
            let undefined_transaction_vec =
                unconfirmed_transactions(unconfirmed_transactions_url, bitcoin_price_url, number_outputs).await?;

            // Print the table
            let table: Vec<_> = undefined_transaction_vec.iter().map(|undefined_transaction| {
                vec![
                    Color::Yellow.paint(undefined_transaction.hash.clone()).cell(), 
                    Color::Yellow.paint(format!("{} BTC", undefined_transaction.amount_bitcoin)).cell(),
                    Color::Yellow.paint(format!("${}{:.5}", (undefined_transaction.amount_usd.trunc() as u64).to_formatted_string(&Locale::en), {undefined_transaction.amount_usd.fract()})).cell(),
                    Color::Yellow.paint(format!("{}", undefined_transaction.time.format("%H:%M"))).cell(),
                ]
            }).collect();

            let undefined_transaction_table = cli_table::Table::table(table)
                .title(vec![
                    "Hash".cell().bold(true),
                    "Bitcoin".cell().bold(true),
                    "Amount(USD)".cell().bold(true),
                    "Time".cell().bold(true),
                ])
                .foreground_color(Some(cli_table::Color::Yellow));

            print_stdout(undefined_transaction_table)?;
            println!();

            // Show the total amount of money that was transfer
            let total:f64 = undefined_transaction_vec.iter().map(|ut| ut.amount_usd).sum();
            let table = vec![
                vec![
                    Color::Purple.paint("Total Amount").cell(), 
                    Color::Purple.paint(format!("${}{:.5}", (total.trunc() as u64).to_formatted_string(&Locale::en), {total.fract()})).cell(),
                ]
            ].table();

            print_stdout(table.foreground_color(Some(cli_table::Color::Magenta)))?;

            // exit the program
            std::process::exit(0);
        }
        Some("inspect") => {
            if let None = cli.inspect_transcation {
                println!("{}", Color::Cyan.paint("Provide a transaction hash (i.e -i 136937e5a742645ce873f079f8668aefdc2d06b8172e903d031a8bfb48969450)\n"));
                help_panel();
                std::process::exit(1);
            }

            match inspect_transcation(&cli.inspect_transcation.unwrap(), inspect_transaction_url).await {
                Ok(transaction) => {
                    // Extract values from inputs[].prev_out.value
                    let total_inputs: Vec<&PrevOut> = transaction.inputs.iter().map(|input| &input.prev_out).collect();
                    let total_input: u64 = total_inputs.iter().map(|prev_output| prev_output.value).sum();
                    let total_input: f64 = total_input as f64 / SATOSHIS_PER_BTC as f64;
                    
                    // Extract values from out[].value
                    let total_outputs: Vec<&Output> = transaction.out.iter().collect();
                    let total_output: u64 = total_outputs.iter().map(|output| output.value).sum();
                    let total_output:f64 = total_output as f64 / SATOSHIS_PER_BTC as f64;

                    // Show Total inputs table
                    let totals_table = vec![
                        vec![
                            Color::Yellow.paint(format!("{} BTC",total_input.to_string())).cell(), 
                            Color::Yellow.paint(format!("{} BTC", total_output.to_string())).cell()
                        ]
                    ]
                    .table()
                    .title(vec![
                        "Total Input".cell().bold(true),
                        "Total Output".cell().bold(true),
                    ]);

                    print_stdout(totals_table.foreground_color(Some(cli_table::Color::Yellow)))?;


                    // Show Address inputs and its Value table
                    let table: Vec<_> = total_inputs.iter().map(|prev_out| {
                        let btc = prev_out.value as f64 / SATOSHIS_PER_BTC as f64;
                        vec![
                            Color::Green.paint(prev_out.addr.clone()).cell(), 
                            Color::Green.paint(format!("{} BTC", btc.to_string())).cell()
                        ]
                    }).collect();

                    let table_inputs = cli_table::Table::table(table)
                        .title(vec![
                            "Address (input)".cell().bold(true),
                            "Value".cell().bold(true),
                        ])
                        .foreground_color(Some(cli_table::Color::Green));

                    print_stdout(table_inputs)?;
                    println!();

                    // Show Address Outputs and its Value table
                    let table: Vec<_> = total_outputs.iter().map(|output| {
                        let btc = output.value as f64 / SATOSHIS_PER_BTC as f64;
                        vec![
                            Color::Green.paint(output.addr.clone()).cell(), 
                            Color::Green.paint(format!("{} BTC", btc.to_string())).cell()
                        ]
                    }).collect();

                    let table_outputs = cli_table::Table::table(table)
                        .title(vec![
                            "Address (output)".cell().bold(true),
                            "Value".cell().bold(true),
                        ])
                        .foreground_color(Some(cli_table::Color::Green));

                    print_stdout(table_outputs)?;
                },
                Err(_) => println!("{}",Color::Red.paint("[!] There is not transaction with the hash received")),
            };
        }
        Some("address") => {
            if let None = cli.inspect_address {
                println!("{}", Color::Cyan.paint("Provide a Bitcoin Addres (i.e -a bc1q9jk7j04lcjzdns6wjegeju78mxq27reg5e4wkycrs407yd0r07psvs8x7u)\n"));
                help_panel();
                std::process::exit(1);
            }

            match inspect_address(&cli.inspect_address.unwrap(), inspect_address_url).await {
                Ok(bitcoin_address) => {
                    let bitcoin_price_query =  reqwest::get(bitcoin_price_url)
                            .await
                            .unwrap()
                            .text()
                            .await
                            .unwrap();

                    let parsed: Value = serde_json::from_str(&bitcoin_price_query)?;
                    let bitcoin_price = parsed["market_price_usd"].as_f64().unwrap();

                    let bitcoin_address_table =  {
                        let total_received_btc = bitcoin_address.total_received as f64 / SATOSHIS_PER_BTC as f64;
                        let total_received_usd = total_received_btc * bitcoin_price;
                        let total_sent_btc = bitcoin_address.total_sent as f64 / SATOSHIS_PER_BTC as f64;
                        let total_sent_usd = total_sent_btc * bitcoin_price;
                        let final_balance_btc = bitcoin_address.final_balance as f64 / SATOSHIS_PER_BTC as f64;
                        let final_balance_usd = final_balance_btc * bitcoin_price;
                        vec![
                            vec![
                                Color::Cyan.paint(bitcoin_address.n_tx.to_string()).cell(),
                                Color::Cyan.paint(format!("{} BTC", total_received_btc)).cell(),
                                Color::Cyan.paint(format!("{} BTC", total_sent_btc)).cell(),
                                Color::Cyan.paint(format!("{} BTC", final_balance_btc)).cell(),
                            ],
                            vec![
                                " ".cell(),
                                Color::Cyan.paint(format!("${}{:.5}", (total_received_usd.trunc() as u64).to_formatted_string(&Locale::en), {total_received_usd.fract()})).cell(),
                                Color::Cyan.paint(format!("${}{:.5}", (total_sent_usd.trunc() as u64).to_formatted_string(&Locale::en), {total_sent_usd.fract()})).cell(),
                                Color::Cyan.paint(format!("${}{:.3}", (final_balance_usd.trunc() as u64).to_formatted_string(&Locale::en), {final_balance_usd.fract()})).cell(),
                            ],
                        ]
                        .table()
                        .title(vec![
                            "Transactions Made".cell().bold(true),
                            "Total Amount Received".cell().bold(true),
                            "Total Amount Sent".cell().bold(true),
                            "Total Balance in the Account".cell().bold(true),
                        ])
                        .foreground_color(Some(cli_table::Color::Cyan))
                    };

                    print_stdout(bitcoin_address_table)?;
                    println!();
                },
                Err(_) => println!("{}",Color::Red.paint("[!] There is not transaction with the hash received")),
            }
        }
        _ => {
            help_panel();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn help_panel() {
    println!("{}", Color::Red.paint("[!] Usage:  ./btcAnalyser -e [-n] [-i] [-a]"));
    println!("{}", Color::Red.paint("---------------------------------------------------------------------------------------------------"));
    println!("\n\t{}", Color::Yellow.paint("[-e] Exploration Mode"));
    println!(
        "\t\t{}\t{}",
        Color::Purple.paint("unconfirmed_transactions:"),
        Color::Yellow.paint("List unconfirmed transactions.")
    );
    println!(
        "\t\t{}\t\t\t{}",
        Color::Purple.paint("inspect:"),
        Color::Yellow.paint("Inspect a transaction hash.")
    );
    println!(
        "\t\t{}\t\t\t{}",
        Color::Purple.paint("address:"),
        Color::Yellow.paint("Inspect a transaction address.")
    );
    println!("\n\t{}", Color::Yellow.paint("[-n] Limit the number of outputs"));
    println!(
        "\t\t{}\t{}",
        Color::Purple.paint("Example:"),
        Color::Yellow.paint("./btcAnalyser -e unconfirmed_transactions -n 10")
    );
    println!("\n\t{}", Color::Yellow.paint("[-i] Provide the transaction hash"));
    println!(
        "\t\t{}\t{}",
        Color::Purple.paint("Example:"),
        Color::Yellow.paint("./btcAnalyser -e inspect -i 136937e5a742645ce873f079f8668aefdc2d06b8172e903d031a8bfb48969450")
    );
    println!("\n\t{}", Color::Yellow.paint("[-a] Provide the Bitcoin Address"));
    println!(
        "\t\t{}\t{}",
        Color::Purple.paint("Example:"),
        Color::Yellow.paint("./btcAnalyser -e address -a bc1q9jk7j04lcjzdns6wjegeju78mxq27reg5e4wkycrs407yd0r07psvs8x7u")
    );
    println!();
}

async fn unconfirmed_transactions(
    unconfirmed_transactions_url: &str,
    bitcoin_price_url: &str,
    number_outputs: usize
) -> Result<Vec<UndefinedTransaction>, Box<dyn std::error::Error>> {
    // We make a http request to the APIs
    let undefined_transaction_query = async {
        reqwest::get(unconfirmed_transactions_url)
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    };

    let bitcoin_price_query = async {
        reqwest::get(bitcoin_price_url)
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    };


    // We get the response from the API, an we get a JSON string
    let (undefined_transaction_json, bitcoin_price_json) =
        tokio::join!(undefined_transaction_query, bitcoin_price_query);


    // Here we serialize the bitcoin_price JSON
    // to get bitcoin_price 
    let parsed: Value = serde_json::from_str(&bitcoin_price_json)?;
    let bitcoin_price = parsed["market_price_usd"].as_f64().unwrap();
    let timestamp = chrono::offset::Local::now().time();


    // Parse the JSON string into a `Value`
    let parsed: Value = serde_json::from_str(&undefined_transaction_json)?;

    // Extract transactions array
    let empty_vec:Vec<Value> = Vec::new();
    let transactions = parsed["txs"].as_array().unwrap_or(&empty_vec);

    // Limit number of transactions
    let transactions = transactions.iter().take(number_outputs);


    let undefined_transaction_vec: Vec<UndefinedTransaction> = transactions
        .map(|tx| {
            let hash = tx["hash"].as_str().unwrap_or("").to_string();

            // Sum all `value` fields in the `out` array
            let amount_satoshis: i64 = tx["out"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|out| out["value"].as_i64())
                .sum();

            // Convert to Bitcoin and calculate USD value
            let amount_bitcoin = amount_satoshis as f64 / SATOSHIS_PER_BTC as f64;
            let amount_usd = amount_bitcoin * bitcoin_price;

            UndefinedTransaction::new(hash, amount_bitcoin, amount_usd, timestamp)
        })
        .collect();

    Ok(undefined_transaction_vec)
}


async fn inspect_transcation(
    transaction_hash: &str,
    inspect_transaction_url: &str,
) -> Result<Transaction, Box<dyn std::error::Error>> {
    let inspect_transaction_url = format!("{inspect_transaction_url}{transaction_hash}");

    let inspect_transaction_query = async {
        let response = reqwest::get(&inspect_transaction_url).await?;
        
        if !response.status().is_success() {
            return Err(Box::<dyn std::error::Error>::from("There is no transaction with the provided hash"));
        }

        response.text().await.map_err(|e| e.into())
    };

    let inspect_transaction_json = tokio::join!(inspect_transaction_query);

    

    // Serialize the JSON
    let transaction: Transaction = serde_json::from_str(&inspect_transaction_json.0?)?;

    Ok(transaction)
}

async fn inspect_address(
    bitcoin_address: &str,
    inspect_address_url: &str,
) -> Result<BitcoinAddress, Box<dyn std::error::Error>> {
    let inspect_address_url = format!("{inspect_address_url}{bitcoin_address}");

    let inspect_address_query = async {
        let response = reqwest::get(&inspect_address_url).await?;
        
        if !response.status().is_success() {
            return Err(Box::<dyn std::error::Error>::from("There is no transaction with the provided hash"));
        }

        response.text().await.map_err(|e| e.into())
    };

    let inspect_addres_json = tokio::join!(inspect_address_query);

    // Serialize the JSON
    let bitcoin_address: BitcoinAddress = serde_json::from_str(&inspect_addres_json.0?)?;


    Ok(bitcoin_address)
}


