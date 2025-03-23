use ansi_term::Color::{self, Red};
use chrono::{NaiveTime};
use clap::Parser;
use cli_table::{print_stdout, Style, Table, WithTitle};
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
}

#[derive(Debug, Table)]
struct UndefinedTransaction {
    #[table(title = "Hash", color = "cli_table::Color::Yellow")]
    hash: String,

    #[table(title = "Bitcoin", color = "cli_table::Color::Yellow")]
    amount_bitcoin: f64,

    #[table(title = "Amount(USD)", color = "cli_table::Color::Yellow")]
    amount_usd: f64,

    #[table(title = "Time", color = "cli_table::Color::Yellow")]
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
    let _inspect_transaction_url = "https://www.blockchain.com/explorer/transactions/btc/";
    let _inspect_address_url = "https://www.blockchain.com/explorer/addresses/btc/";
    let bitcoin_price_url = "https://api.blockchain.info/stats";

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
            print_stdout(undefined_transaction_vec.with_title().foreground_color(Some(cli_table::Color::Yellow)))?;

            // Show the total amount of money that was transfer
            let mut total = 0.0;
            for value in &undefined_transaction_vec {
                total += value.amount_usd;
            }
            println!("\n{}{}", Color::Yellow.paint("Total amount: $"), Color::Yellow.paint(total.round().to_string()));

            // exit the program
            std::process::exit(0);
        }
        Some("inspect") => {
            println!("Inspecting a transaction hash...");
        }
        Some("address") => {
            println!("Inspecting a transaction address...");
        }
        _ => {
            help_panel();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn help_panel() {
    println!("{}", Color::Red.paint("[!] Usage:  ./btcAnalyser -e [-n]"));
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


    // Here we serialize the undefined_transaction JSON
    // to get hash, value (amount of satoshis) and generate an vector of undefined transactions
    let parsed: Value = serde_json::from_str(&undefined_transaction_json.as_str())?;
    let mut undefined_transaction_vec = Vec::new();

    
    if let Some(txs) = parsed["txs"].as_array() {
        for (i, tx) in txs.iter().enumerate() {

            if i == number_outputs {
                break;
            }

            // get the hash transaction
            let hash = tx["hash"].as_str().unwrap();

            // Calculate the amoun of bitcoin send in the transaction
            let mut amount_bitcoin = 0;
            if let Some(outs) = tx["out"].as_array() {
                for out in outs {
                    if let Some(value) = out["value"].as_i64() {
                        amount_bitcoin += value;
                    }
                }
            }
            let amount_bitcoin = amount_bitcoin as f64 / SATOSHIS_PER_BTC as f64;
            let amount_usd = amount_bitcoin * bitcoin_price;


            // make a undefined_transaction and push it to the vector
            undefined_transaction_vec.push(UndefinedTransaction::new(hash.to_string(), amount_bitcoin, amount_usd, timestamp));
        }
    }

    Ok(undefined_transaction_vec)
}




