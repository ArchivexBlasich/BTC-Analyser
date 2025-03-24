# btcAnalyser

`btcAnalyser` is a Rust-based CLI tool for analyzing recent Bitcoin transactions. It fetches data from blockchain explorers and provides insights into unconfirmed transactions and specific addresses.

## Features

- Retrieve the **latest unconfirmed transactions**
- Inspect **specific Bitcoin addresses**
- Display transaction details including **inputs, outputs, fees, and timestamps**
- Fast and efficient, thanks to **async Rust**
- **Color-coded** terminal output for better readability

## Installation

### Prerequisites
Ensure you have Rust installed. If not, install it using:
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Clone and Build
```sh
git clone https://github.com/ArchivexBlasich/BTC-Analyser.git
cd BTC-Analyser.git
cargo build --release
```

## Usage

### Get the Latest Unconfirmed Transactions
```sh
./target/release/btcAnalyser -e unconfirmed_transactions -n 10  # Fetch the latest 10 unconfirmed transactions
```

### Inspect a Specific Transaction
```sh
./btcAnalyser -e inspect -i 136937e5a742645ce873f079f8668aefdc2d06b8172e903d031a8bfb48969450
```

### Inspect a Specific Bitcoin Address
```sh
./btcAnalyser -e address -a bc1q9jk7j04lcjzdns6wjegeju78mxq27reg5e4wkycrs407yd0r07psvs8x7u
```

## Example Output
```
🔍 Fetching the latest 5 unconfirmed transactions...
./target/release/btcAnalyser -e unconfirmed_transactions -n 5
+------------------------------------------------------------------+-----------------+-------------------+-------+
| Hash                                                             | Bitcoin         | Amount(USD)       | Time  |
+------------------------------------------------------------------+-----------------+-------------------+-------+
| 889562cc39a509d71f07d4396ca69459bd40fc1bd83ed5222bb0973ff4c531e3 | 0.00000546 BTC  | $00.47940         | 14:07 |
+------------------------------------------------------------------+-----------------+-------------------+-------+
| 636357d9076721641759c5f186f57fa6b9ed95b6f3b570a6e2a9461e221aa38d | 0.00037342 BTC  | $320.78691        | 14:07 |
+------------------------------------------------------------------+-----------------+-------------------+-------+
| d486bde2769ebbc24a13d9834882906f24f1bbe8df23f37e7ff0fd7120761284 | 0.00000546 BTC  | $00.47940         | 14:07 |
+------------------------------------------------------------------+-----------------+-------------------+-------+
| a8803a8c11aff4416ccc8be2ed8b2819834d49185c95b7738a09b6f70070484e | 0.00146642 BTC  | $1280.75417       | 14:07 |
+------------------------------------------------------------------+-----------------+-------------------+-------+
| 4b5d679670c5c6e2489c63e862db40f5d3609b699bc71193fd473507e2b2ad8f | 11.88049037 BTC | $1,043,1270.25132 | 14:07 |
+------------------------------------------------------------------+-----------------+-------------------+-------+

+--------------+-------------------+
| Total Amount | $1,043,2890.75119 |
+--------------+-------------------+
```

## Dependencies
This project uses the following Rust crates:
- `reqwest` for HTTP requests
- `serde_json` for parsing JSON responses
- `chrono` for handling timestamps
- `ansi_term` for colorized output
- `clap` for CLI argument parsing


## Contributing
Contributions are welcome! Feel free to open an issue or submit a pull request.

## License
This project is licensed under the MIT License.

---

🚀 **Made with Rust by [Fabricio Blasich]**

