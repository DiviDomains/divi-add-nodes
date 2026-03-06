# Divi Node Connector

Standalone binary that fetches reachable nodes from the [Divi Network Map](https://services.divi.domains/map/) and adds them to your running Divi node via RPC. Zero runtime dependencies.

## Download

Pre-built binaries for all platforms on the [Releases](https://github.com/DiviDomains/divi-add-nodes/releases) page:

| Platform | Binary |
|----------|--------|
| Linux x64 | `divi-add-nodes-x86_64-unknown-linux-gnu` |
| Linux ARM64 | `divi-add-nodes-aarch64-unknown-linux-gnu` |
| macOS Intel | `divi-add-nodes-x86_64-apple-darwin` |
| macOS Apple Silicon | `divi-add-nodes-aarch64-apple-darwin` |
| Windows x64 | `divi-add-nodes-x86_64-pc-windows-msvc.exe` |

## Usage

```bash
divi-add-nodes --user <rpc-user> --pass <rpc-pass> [OPTIONS]
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--user` | (required) | RPC username |
| `--pass` | (required) | RPC password |
| `--host` | `127.0.0.1` | RPC host |
| `--port` | `51473` | RPC port |
| `--api` | services.divi.domains | Node map API URL |
| `--dry-run` | | List nodes without adding them |

### Examples

```bash
# Add nodes to local Divi node
./divi-add-nodes --user divirpc --pass yourpassword

# Dry run - see which nodes would be added
./divi-add-nodes --user divirpc --pass yourpassword --dry-run

# Custom RPC host/port
./divi-add-nodes --user divirpc --pass yourpassword --host 192.168.1.100 --port 51473
```

## Cron Setup

Run hourly to maintain good connectivity:

```bash
# Add to crontab (crontab -e)
0 * * * * /usr/local/bin/divi-add-nodes --user divirpc --pass yourpassword >> /var/log/divi-add-nodes.log 2>&1
```

## How It Works

1. Fetches all known nodes from the Divi Network Map API
2. Filters to reachable nodes (those with a user agent from the last crawl)
3. Calls `addnode` RPC for each node
4. Reports summary with connection count

## Example Output

```
2026-03-06 20:44:50 UTC Reachable=79 Added=79 Failed=0 Connections=188
```

## Build from Source

```bash
cargo build --release
```

## License

MIT
