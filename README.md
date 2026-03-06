# Divi Node Connector

Automatically fetches reachable nodes from the [Divi Network Map](https://services.divi.domains/map/) and adds them to your running Divi node to improve connectivity and staking performance.

## Requirements

- `curl`
- `python3`
- `divi-cli` (or a Docker container running `divid`)

## Usage

```bash
# Local divi-cli (auto-detected from PATH)
./add-nodes.sh

# Specify divi-cli path
./add-nodes.sh --cli /usr/local/bin/divi-cli

# Docker mode - run divi-cli inside a container
./add-nodes.sh --docker my_divi_container

# Custom API endpoint
./add-nodes.sh --api https://your-map-instance.com/api/nodes?limit=10000
```

## Cron Setup (Recommended)

Run every hour to keep your node well-connected:

```bash
# Edit crontab
crontab -e

# Add this line:
0 * * * * /path/to/add-nodes.sh --docker my_divi_container >> /var/log/divi-add-nodes.log 2>&1
```

## How It Works

1. Queries the Divi Network Map API for all known nodes
2. Filters to only **reachable** nodes (those that responded during the last crawl)
3. Runs `divi-cli addnode <ip:port> onetry` for each one
4. Logs a summary line with counts and current connection total

## Example Output

```
2026-03-06 20:44:50 UTC Reachable=79 Added=79 Failed=0 Connections=188
```

## License

MIT
