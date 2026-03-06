use clap::Parser;
use serde_json::{json, Value};
use std::process;

#[derive(Parser)]
#[command(name = "divi-add-nodes")]
#[command(version)]
#[command(about = "Fetches reachable nodes from the Divi Network Map and adds them to your running Divi node")]
struct Cli {
    /// RPC host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// RPC port
    #[arg(long, default_value_t = 51473)]
    port: u16,

    /// RPC username
    #[arg(long, short)]
    user: String,

    /// RPC password
    #[arg(long, short)]
    pass: String,

    /// Node map API URL
    #[arg(long, default_value = "https://services.divi.domains/map/api/nodes?limit=10000")]
    api: String,

    /// Show nodes without adding them
    #[arg(long)]
    dry_run: bool,
}

async fn fetch_nodes(client: &reqwest::Client, api_url: &str) -> Result<Vec<String>, String> {
    let resp = client
        .get(api_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch node map: {e}"))?;

    let body: Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {e}"))?;

    let nodes = body["nodes"]
        .as_array()
        .ok_or("API response missing 'nodes' array")?;

    let reachable: Vec<String> = nodes
        .iter()
        .filter(|n| {
            n["user_agent"]
                .as_str()
                .map(|ua| !ua.is_empty())
                .unwrap_or(false)
        })
        .map(|n| {
            let ip = n["ip"].as_str().unwrap_or("0.0.0.0");
            let port = n["port"].as_u64().unwrap_or(51472);
            format!("{ip}:{port}")
        })
        .collect();

    Ok(reachable)
}

async fn rpc_call(
    client: &reqwest::Client,
    url: &str,
    user: &str,
    pass: &str,
    method: &str,
    params: Value,
    id: &str,
) -> Result<Value, String> {
    let body = json!({
        "jsonrpc": "1.0",
        "id": id,
        "method": method,
        "params": params,
    });

    let resp = client
        .post(url)
        .basic_auth(user, Some(pass))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("RPC request failed: {e}"))?;

    let result: Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse RPC response: {e}"))?;

    if let Some(err) = result.get("error") {
        if !err.is_null() {
            return Err(format!("RPC error: {err}"));
        }
    }

    Ok(result)
}

fn timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let secs = now % 60;
    let mins = (now / 60) % 60;
    let hours = (now / 3600) % 24;
    let days = now / 86400;

    // Simple date calculation from days since epoch
    let (year, month, day) = days_to_date(days);

    format!("{year:04}-{month:02}-{day:02} {hours:02}:{mins:02}:{secs:02} UTC")
}

fn days_to_date(days_since_epoch: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days_since_epoch + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1461 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let client = reqwest::Client::new();

    // Fetch reachable nodes
    let nodes = match fetch_nodes(&client, &cli.api).await {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    };

    if nodes.is_empty() {
        eprintln!("Error: no reachable nodes found");
        process::exit(1);
    }

    // Dry-run mode: just list nodes
    if cli.dry_run {
        for node in &nodes {
            println!("{node}");
        }
        println!("Reachable={} (dry run, no nodes added)", nodes.len());
        return;
    }

    // Add each node via RPC
    let rpc_url = format!("http://{}:{}", cli.host, cli.port);
    let mut added = 0u64;
    let mut failed = 0u64;

    for node in &nodes {
        let params = json!([node, "onetry"]);
        match rpc_call(&client, &rpc_url, &cli.user, &cli.pass, "addnode", params, "add").await {
            Ok(_) => added += 1,
            Err(e) => {
                eprintln!("addnode {node}: {e}");
                failed += 1;
            }
        }
    }

    // Get connection count
    let connections = match rpc_call(
        &client,
        &rpc_url,
        &cli.user,
        &cli.pass,
        "getconnectioncount",
        json!([]),
        "count",
    )
    .await
    {
        Ok(v) => v["result"].as_u64().unwrap_or(0),
        Err(e) => {
            eprintln!("getconnectioncount: {e}");
            0
        }
    };

    println!(
        "{} Reachable={} Added={added} Failed={failed} Connections={connections}",
        timestamp(),
        nodes.len(),
    );
}
