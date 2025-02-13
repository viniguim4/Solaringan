# Solaringan

A high-performance Solana trading bot built in Rust, designed to monitor and analyze transactions on multiple DEXes including Raydium and PUMPFUN.

<p align="center">
  <img src="solaringan.png" alt="Solaringan" width="32%">
</p>


## Features

- Real-time transaction monitoring using Yellowstone gRPC
- Parallel monitoring of multiple DEX protocols:
  - Raydium DEX
  - PUMPFUN DEX
- Automatic reconnection handling
- Configurable wallet tracking
- Detailed transaction logging
- High-performance Rust implementation

## Setup

1. Install Rust and Cargo if you haven't already:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Configure your settings in `config/default.json`:
   - Set target wallet addresses to track
   - Configure gRPC endpoint
   - Set DEX program IDs

3. Build the project:
```bash
cargo build --release
```

4. Run the bot:
```bash
cargo run --release
```

## Configuration

Edit `config/default.json` to customize:

- gRPC endpoint settings
- Target wallet addresses
- DEX configurations:
  - Raydium program ID
  - PUMPFUN program ID
- Monitoring parameters

## Architecture

The project uses:
- Yellowstone gRPC for efficient blockchain monitoring
- Tokio for async runtime and task management
- Futures for stream processing
- Custom event loops for resilient connections
- Structured logging with timestamps

## Security Notes

- Monitor logs regularly for system health
- Keep your config file secure
- Use trusted RPC endpoints
- Regularly update dependencies

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
