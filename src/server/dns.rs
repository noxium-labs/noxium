use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;
use trust_dns_server::authority::{Authority, ZoneType};
use trust_dns_server::proto::dns::{DnsResponse, Message, RecordType};
use trust_dns_server::proto::xfer::{DnsRequest, DnsResponse as DnsResponseTrait};
use trust_dns_server::server::{ServerFuture, ResponseHandler, RequestHandler};
use trust_dns_server::server::response::Response;
use trust_dns_client::client::Client;
use trust_dns_client::proto::dns::DnsRequest as ClientDnsRequest;
use trust_dns_client::proto::dns::DnsResponse as ClientDnsResponse;
use std::sync::{Arc, Mutex};
use log::{info, error};

/// DNS Server struct that contains zone data, cache, and upstream servers.
#[derive(Debug)]
struct DnsServer {
    zone: Authority,
    cache: Arc<Mutex<Cache>>,
    upstream_servers: Vec<SocketAddr>,
}

/// In-memory cache for DNS responses.
#[derive(Debug, Default)]
struct Cache {
    entries: std::collections::HashMap<String, DnsResponse>,
}

impl DnsServer {
    /// Creates a new `DnsServer` with the given zone and upstream servers.
    fn new(zone: Authority, upstream_servers: Vec<SocketAddr>) -> Self {
        Self {
            zone,
            cache: Arc::new(Mutex::new(Cache::default())),
            upstream_servers,
        }
    }

    /// Forwards DNS queries to upstream DNS servers if not found in the local zone.
    async fn forward_query(&self, query: &Message) -> Result<DnsResponse, Box<dyn std::error::Error>> {
        info!("Forwarding query to upstream servers");

        // Iterate through upstream servers and try to get a response
        for server in &self.upstream_servers {
            // Create and connect a UDP socket to the upstream server
            let client = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
            client.connect(*server).await?;

            // Send the DNS request to the upstream server
            let request = ClientDnsRequest::new(query.clone());
            client.send(&request.to_bytes()).await?;

            // Receive the response from the upstream server
            let mut buf = [0; 512];
            let _ = client.recv(&mut buf).await?;
            let response_msg = ClientDnsResponse::from_bytes(&buf)?;
            return Ok(response_msg);
        }

        Err("No response from upstream servers".into())
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let address = "127.0.0.1:53".parse::<SocketAddr>()?;
    let socket = UdpSocket::bind(&address).await?;

    let zone = create_zone();
    let upstream_servers = vec!["8.8.8.8:53".parse().unwrap()]; // Example upstream server
    let server = DnsServer::new(zone, upstream_servers);

    let mut dns_server = ServerFuture::new();
    dns_server.register_handler(Box::new(server));

    info!("DNS server listening on {}", address);

    dns_server.serve_with_socket(socket).await
}

impl RequestHandler for DnsServer {
    type Response = DnsResponse;

    /// Handles DNS requests, checking the cache and forwarding to upstream servers if necessary.
    async fn handle_request(
        &self,
        request: DnsRequest,
        handler: &ResponseHandler,
    ) -> Result<Self::Response, Box<dyn std::error::Error>> {
        let message = request.message().clone();
        info!("Received DNS request: {:?}", message);

        // Check cache for a response
        if let Some(cached_response) = self.cache.lock().unwrap().entries.get(&message.to_string()) {
            info!("Cache hit for query: {:?}", message);
            handler.send_response(cached_response.clone()).await?;
            return Ok(cached_response.clone());
        }

        // Process the query
        let response = if self.zone.contains(&message) {
            self.handle_query(message)?
        } else {
            self.forward_query(&message).await?
        };

        // Cache the response
        self.cache.lock().unwrap().entries.insert(message.to_string(), response.clone());
        handler.send_response(response).await?;
        Ok(response)
    }
}

impl DnsServer {
    /// Handles DNS queries for different record types and constructs responses.
    fn handle_query(&self, message: Message) -> Result<DnsResponse, Box<dyn std::error::Error>> {
        let mut response = message.response();
        let mut message = response.message();
        
        for query in message.queries() {
            let name = query.name();
            let record_type = query.query_type();

            match record_type {
                RecordType::A => {
                    let ip = Ipv4Addr::new(127, 0, 0, 1);
                    let record = trust_dns_proto::rr::RData::A(ip);
                    response.add_answer(name.clone(), 3600, record);
                    info!("Added A record for {}: {:?}", name, ip);
                }
                RecordType::AAAA => {
                    let ip = trust_dns_proto::rr::RData::AAAA(
                        trust_dns_proto::rr::rdata::AAAA::new(0, 0, 0, 0, 0, 0, 0, 1),
                    );
                    response.add_answer(name.clone(), 3600, ip);
                    info!("Added AAAA record for {}: {:?}", name, ip);
                }
                RecordType::CNAME => {
                    let cname = trust_dns_proto::rr::RData::CNAME(name.clone());
                    response.add_answer(name.clone(), 3600, cname);
                    info!("Added CNAME record for {}: {:?}", name, cname);
                }
                RecordType::MX => {
                    let mx = trust_dns_proto::rr::RData::MX(10, "mail.example.com.".to_string());
                    response.add_answer(name.clone(), 3600, mx);
                    info!("Added MX record for {}: {:?}", name, mx);
                }
                RecordType::TXT => {
                    let txt = trust_dns_proto::rr::RData::TXT(vec!["v=spf1 include:_spf.example.com ~all".to_string()]);
                    response.add_answer(name.clone(), 3600, txt);
                    info!("Added TXT record for {}: {:?}", name, txt);
                }
                RecordType::PTR => {
                    let ptr = trust_dns_proto::rr::RData::PTR("example.com.".to_string());
                    response.add_answer(name.clone(), 3600, ptr);
                    info!("Added PTR record for {}: {:?}", name, ptr);
                }
                RecordType::SRV => {
                    let srv = trust_dns_proto::rr::RData::SRV(
                        10, 5, 5060, "sip.example.com.".to_string()
                    );
                    response.add_answer(name.clone(), 3600, srv);
                    info!("Added SRV record for {}: {:?}", name, srv);
                }
                _ => {
                    // Log unsupported record types
                    info!("Received unsupported record type: {:?}", record_type);
                }
            }
        }

        Ok(response)
    }
}

/// Creates a sample DNS zone with example records.
fn create_zone() -> Authority {
    let zone_name = "example.com.".to_string();
    let mut authority = Authority::new(zone_name, ZoneType::Master);

    // Insert example records into the zone
    authority.insert_record(
        "example.com.".to_string(),
        RecordType::A,
        3600,
        Ipv4Addr::new(127, 0, 0, 1).into(),
    );

    authority.insert_record(
        "example.com.".to_string(),
        RecordType::AAAA,
        3600,
        trust_dns_proto::rr::rdata::AAAA::new(0, 0, 0, 0, 0, 0, 0, 1).into(),
    );

    authority
}