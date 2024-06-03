use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tokio::time::{sleep, Duration};

#[derive(Deserialize)]
struct IpResponse {
    ip: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DnsRecord {
    r#type: String,
    name: String,
    content: String,
    ttl: u32,
    proxied: bool,
}

#[derive(Deserialize)]
struct CloudflareResponse {
    success: bool,
    errors: Vec<String>,
    messages: Vec<String>,
    result: Option<DnsRecord>,
}

async fn get_current_ip(client: &Client) -> Result<String, reqwest::Error> {
    let response = client
        .get("https://api.ipify.org?format=json")
        .send()
        .await?
        .json::<IpResponse>()
        .await?;

    Ok(response.ip)
}

async fn update_dns_record(
    client: &Client,
    zone_id: &str,
    record_id: &str,
    dns_record: &DnsRecord,
    api_key: &str,
) -> Result<(), reqwest::Error> {
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
        zone_id, record_id
    );

    let response = client
        .put(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(dns_record)
        .send()
        .await?
        .json::<CloudflareResponse>()
        .await?;

    if response.success {
        println!("DNS record updated successfully.");
    } else {
        eprintln!("Failed to update DNS record: {:?}", response.errors);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();

    let api_key = env::var("CLOUDFLARE_API_KEY").expect("CLOUDFLARE_API_KEY not set");
    let zone_id = env::var("CLOUDFLARE_ZONE_ID").expect("CLOUDFLARE_ZONE_ID not set");
    let record_id = env::var("CLOUDFLARE_RECORD_ID").expect("CLOUDFLARE_RECORD_ID not set");
    let domain_name = env::var("DOMAIN_NAME").expect("DOMAIN_NAME not set");

    let client = Client::new();

    loop {
        match get_current_ip(&client).await {
            Ok(ip) => {
                let dns_record = DnsRecord {
                    r#type: "A".to_string(),
                    name: domain_name.clone(),
                    content: ip.clone(),
                    ttl: 300,
                    proxied: true,
                };

                println!("{:#?}", dns_record);

                if let Err(e) =
                    update_dns_record(&client, &zone_id, &record_id, &dns_record, &api_key).await
                {
                    eprintln!("Error updating DNS record: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Error fetching current IP: {}", e);
            }
        }

        sleep(Duration::from_secs(300)).await;
    }
}
