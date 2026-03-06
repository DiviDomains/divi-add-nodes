#!/bin/bash
# =============================================================================
# Divi Node Connector
# Fetches reachable nodes from the Divi Network Map and adds them to your
# running Divi node to improve connectivity and staking performance.
#
# Usage:
#   ./add-nodes.sh                      # auto-detect divi-cli
#   ./add-nodes.sh --cli /path/to/divi-cli
#   ./add-nodes.sh --docker <container>  # run divi-cli inside a container
#
# Cron (every hour):
#   0 * * * * /path/to/add-nodes.sh --docker divi_node >> /var/log/add-nodes.log 2>&1
# =============================================================================

set -euo pipefail

API_URL="https://services.divi.domains/map/api/nodes?limit=10000"
DEFAULT_PORT=51472

# ---------------------------------------------------------------------------
# Parse arguments
# ---------------------------------------------------------------------------
DOCKER_CONTAINER=""
DIVI_CLI=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --docker)
            DOCKER_CONTAINER="$2"
            shift 2
            ;;
        --cli)
            DIVI_CLI="$2"
            shift 2
            ;;
        --api)
            API_URL="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [--cli /path/to/divi-cli] [--docker container] [--api url]"
            echo ""
            echo "Options:"
            echo "  --cli PATH       Path to divi-cli binary (auto-detected if omitted)"
            echo "  --docker NAME    Run divi-cli inside this Docker container"
            echo "  --api URL        Override the node map API URL"
            echo ""
            echo "Examples:"
            echo "  $0                                    # local divi-cli, auto-detect"
            echo "  $0 --cli /usr/local/bin/divi-cli      # specific binary"
            echo "  $0 --docker my_divi_container         # Docker mode"
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

# ---------------------------------------------------------------------------
# Build the CLI command
# ---------------------------------------------------------------------------
run_cli() {
    if [ -n "$DOCKER_CONTAINER" ]; then
        docker exec "$DOCKER_CONTAINER" $DIVI_CLI "$@" 2>/dev/null
    else
        $DIVI_CLI "$@" 2>/dev/null
    fi
}

# Auto-detect divi-cli
if [ -z "$DIVI_CLI" ]; then
    if [ -n "$DOCKER_CONTAINER" ]; then
        # Try common paths inside container
        for path in /usr/local/bin/divi-cli /usr/bin/divi-cli /root/divi-3.0.0/bin/divi-cli /root/divi/bin/divi-cli; do
            if docker exec "$DOCKER_CONTAINER" test -f "$path" 2>/dev/null; then
                DIVI_CLI="$path"
                break
            fi
        done
    else
        DIVI_CLI=$(which divi-cli 2>/dev/null || true)
    fi
fi

if [ -z "$DIVI_CLI" ]; then
    echo "$(date -u '+%Y-%m-%d %H:%M:%S UTC') ERROR: divi-cli not found. Use --cli or --docker." >&2
    exit 1
fi

# Verify divi-cli works
if ! run_cli getconnectioncount > /dev/null 2>&1; then
    echo "$(date -u '+%Y-%m-%d %H:%M:%S UTC') ERROR: divi-cli not responding. Is divid running?" >&2
    exit 1
fi

# ---------------------------------------------------------------------------
# Fetch reachable nodes from the API
# ---------------------------------------------------------------------------
NODES=$(curl -s --max-time 30 "$API_URL" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    for n in data.get('nodes', []):
        if n.get('user_agent') and n.get('ip'):
            print(f\"{n['ip']}:{n.get('port', $DEFAULT_PORT)}\")
except Exception:
    pass
" 2>/dev/null)

if [ -z "$NODES" ]; then
    echo "$(date -u '+%Y-%m-%d %H:%M:%S UTC') ERROR: No reachable nodes fetched from API" >&2
    exit 1
fi

# ---------------------------------------------------------------------------
# Add nodes
# ---------------------------------------------------------------------------
TOTAL=$(echo "$NODES" | wc -l | tr -d ' ')
ADDED=0
FAILED=0

while IFS= read -r NODE; do
    [ -z "$NODE" ] && continue
    if run_cli addnode "$NODE" onetry; then
        ADDED=$((ADDED + 1))
    else
        FAILED=$((FAILED + 1))
    fi
done <<< "$NODES"

CONN=$(run_cli getconnectioncount 2>/dev/null || echo "?")
echo "$(date -u '+%Y-%m-%d %H:%M:%S UTC') Reachable=$TOTAL Added=$ADDED Failed=$FAILED Connections=$CONN"
