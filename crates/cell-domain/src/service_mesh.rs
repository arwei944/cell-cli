use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshConfig {
    pub name: String,
    pub namespace: String,
    pub version: String,
    pub labels: HashMap<String, String>,
}

impl Default for MeshConfig {
    fn default() -> Self {
        let mut labels = HashMap::new();
        labels.insert("app.kubernetes.io/managed-by".into(), "cell-architecture".into());
        Self {
            name: "default".into(),
            namespace: "default".into(),
            version: "v1".into(),
            labels,
        }
    }
}

impl MeshConfig {
    pub fn new(name: impl Into<String>, namespace: impl Into<String>) -> Self {
        let mut labels = HashMap::new();
        labels.insert("app.kubernetes.io/managed-by".into(), "cell-architecture".into());
        Self {
            name: name.into(),
            namespace: namespace.into(),
            version: "v1".into(),
            labels,
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDestination {
    pub host: String,
    pub subset: Option<String>,
    pub weight: u32,
}

impl RouteDestination {
    pub fn new(host: impl Into<String>, weight: u32) -> Self {
        Self { host: host.into(), subset: None, weight }
    }

    pub fn with_subset(mut self, subset: impl Into<String>) -> Self {
        self.subset = Some(subset.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderMatch {
    pub name: String,
    pub exact: Option<String>,
    pub prefix: Option<String>,
}

impl HeaderMatch {
    pub fn exact(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self { name: name.into(), exact: Some(value.into()), prefix: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteMatch {
    PathPrefix(String),
    PathExact(String),
    Headers(Vec<HeaderMatch>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRoute {
    pub name: String,
    pub match_rules: Vec<RouteMatch>,
    pub destinations: Vec<RouteDestination>,
    pub timeout_seconds: Option<u32>,
    pub retries: Option<RetryPolicy>,
}

impl HttpRoute {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), match_rules: vec![], destinations: vec![], timeout_seconds: None, retries: None }
    }

    pub fn with_path_prefix(mut self, path: impl Into<String>) -> Self {
        self.match_rules.push(RouteMatch::PathPrefix(path.into()));
        self
    }

    pub fn with_headers(mut self, headers: Vec<HeaderMatch>) -> Self {
        self.match_rules.push(RouteMatch::Headers(headers));
        self
    }

    pub fn add_destination(mut self, dest: RouteDestination) -> Self {
        self.destinations.push(dest);
        self
    }

    pub fn with_timeout(mut self, seconds: u32) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }

    pub fn with_retries(mut self, retries: RetryPolicy) -> Self {
        self.retries = Some(retries);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualService {
    pub name: String,
    pub namespace: String,
    pub hosts: Vec<String>,
    pub gateways: Vec<String>,
    pub http_routes: Vec<HttpRoute>,
    pub labels: HashMap<String, String>,
}

impl VirtualService {
    pub fn new(name: impl Into<String>, namespace: impl Into<String>) -> Self {
        let mut labels = HashMap::new();
        labels.insert("app.kubernetes.io/managed-by".into(), "cell-architecture".into());
        Self { name: name.into(), namespace: namespace.into(), hosts: vec![], gateways: vec![], http_routes: vec![], labels }
    }

    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.hosts.push(host.into());
        self
    }

    pub fn with_gateway(mut self, gateway: impl Into<String>) -> Self {
        self.gateways.push(gateway.into());
        self
    }

    pub fn add_http_route(mut self, route: HttpRoute) -> Self {
        self.http_routes.push(route);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subset {
    pub name: String,
    pub labels: HashMap<String, String>,
    pub traffic_policy: Option<TrafficPolicy>,
}

impl Subset {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), labels: HashMap::new(), traffic_policy: None }
    }

    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub attempts: u32,
    pub per_try_timeout_seconds: u32,
    pub retry_on: Vec<String>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self { attempts: 3, per_try_timeout_seconds: 5, retry_on: vec!["5xx".into(), "connect-failure".into()] }
    }
}

impl RetryPolicy {
    pub fn new(attempts: u32, per_try_timeout_seconds: u32) -> Self {
        Self { attempts, per_try_timeout_seconds, retry_on: vec!["5xx".into()] }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreaker {
    pub max_connections: u32,
    pub consecutive_errors: u32,
    pub sleep_window_seconds: u32,
    pub half_open_max_requests: u32,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self { max_connections: 100, consecutive_errors: 5, sleep_window_seconds: 30, half_open_max_requests: 1 }
    }
}

impl CircuitBreaker {
    pub fn new(consecutive_errors: u32, sleep_window_seconds: u32) -> Self {
        Self { max_connections: 100, consecutive_errors, sleep_window_seconds, half_open_max_requests: 1 }
    }

    pub fn with_half_open_max_requests(mut self, max: u32) -> Self {
        self.half_open_max_requests = max;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_unit: u32,
    pub unit: RateLimitUnit,
    pub burst: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RateLimitUnit {
    Second,
    Minute,
    Hour,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self { requests_per_unit: 100, unit: RateLimitUnit::Second, burst: 50 }
    }
}

impl RateLimit {
    pub fn new(requests_per_unit: u32, unit: RateLimitUnit) -> Self {
        Self { requests_per_unit, unit, burst: 0 }
    }

    pub fn with_burst(mut self, burst: u32) -> Self {
        self.burst = burst;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficPolicy {
    pub timeout_seconds: Option<u32>,
    pub retries: Option<RetryPolicy>,
    pub circuit_breaker: Option<CircuitBreaker>,
    pub rate_limit: Option<RateLimit>,
    pub load_balancer: Option<LoadBalancerType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoadBalancerType {
    RoundRobin,
    LeastConn,
    Random,
}

impl Default for TrafficPolicy {
    fn default() -> Self {
        Self { timeout_seconds: Some(30), retries: Some(RetryPolicy::default()), circuit_breaker: None, rate_limit: None, load_balancer: Some(LoadBalancerType::RoundRobin) }
    }
}

impl TrafficPolicy {
    pub fn new() -> Self { Self::default() }
    pub fn with_timeout(mut self, seconds: u32) -> Self { self.timeout_seconds = Some(seconds); self }
    pub fn with_retries(mut self, retries: RetryPolicy) -> Self { self.retries = Some(retries); self }
    pub fn with_circuit_breaker(mut self, cb: CircuitBreaker) -> Self { self.circuit_breaker = Some(cb); self }
    pub fn with_rate_limit(mut self, rl: RateLimit) -> Self { self.rate_limit = Some(rl); self }
    pub fn with_load_balancer(mut self, lb: LoadBalancerType) -> Self { self.load_balancer = Some(lb); self }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationRule {
    pub name: String,
    pub namespace: String,
    pub host: String,
    pub subsets: Vec<Subset>,
    pub traffic_policy: Option<TrafficPolicy>,
    pub labels: HashMap<String, String>,
}

impl DestinationRule {
    pub fn new(name: impl Into<String>, namespace: impl Into<String>, host: impl Into<String>) -> Self {
        let mut labels = HashMap::new();
        labels.insert("app.kubernetes.io/managed-by".into(), "cell-architecture".into());
        Self { name: name.into(), namespace: namespace.into(), host: host.into(), subsets: vec![], traffic_policy: None, labels }
    }

    pub fn add_subset(mut self, subset: Subset) -> Self { self.subsets.push(subset); self }
    pub fn with_traffic_policy(mut self, policy: TrafficPolicy) -> Self { self.traffic_policy = Some(policy); self }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayPort {
    pub number: u16,
    pub name: String,
    pub protocol: String,
}

impl GatewayPort {
    pub fn http(port: u16) -> Self { Self { number: port, name: "http".into(), protocol: "HTTP".into() } }
    pub fn https(port: u16) -> Self { Self { number: port, name: "https".into(), protocol: "HTTPS".into() } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gateway {
    pub name: String,
    pub namespace: String,
    pub selector: HashMap<String, String>,
    pub servers: Vec<GatewayServer>,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayServer {
    pub port: GatewayPort,
    pub hosts: Vec<String>,
}

impl Gateway {
    pub fn new(name: impl Into<String>, namespace: impl Into<String>) -> Self {
        let mut labels = HashMap::new();
        labels.insert("app.kubernetes.io/managed-by".into(), "cell-architecture".into());
        let mut selector = HashMap::new();
        selector.insert("istio".into(), "ingressgateway".into());
        Self { name: name.into(), namespace: namespace.into(), selector, servers: vec![], labels }
    }

    pub fn add_server(mut self, port: GatewayPort, hosts: Vec<String>) -> Self {
        self.servers.push(GatewayServer { port, hosts });
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sidecar {
    pub name: String,
    pub namespace: String,
    pub workload_selector: Option<HashMap<String, String>>,
    pub labels: HashMap<String, String>,
}

impl Sidecar {
    pub fn new(name: impl Into<String>, namespace: impl Into<String>) -> Self {
        let mut labels = HashMap::new();
        labels.insert("app.kubernetes.io/managed-by".into(), "cell-architecture".into());
        Self { name: name.into(), namespace: namespace.into(), workload_selector: None, labels }
    }

    pub fn with_workload_selector(mut self, labels: HashMap<String, String>) -> Self {
        self.workload_selector = Some(labels);
        self
    }
}

pub struct MeshGenerator;

impl MeshGenerator {
    pub fn new() -> Self { Self }

    fn fmt_labels(&self, labels: &HashMap<String, String>, indent: usize) -> String {
        let pad = " ".repeat(indent);
        labels.iter().map(|(k, v)| format!("{pad}{k}: \"{v}\"")).collect::<Vec<_>>().join("\n")
    }

    pub fn generate_virtualservice_yaml(&self, vs: &VirtualService) -> String {
        let labels = self.fmt_labels(&vs.labels, 2);
        let hosts = vs.hosts.iter().map(|h| format!("    - \"{h}\"")).collect::<Vec<_>>().join("\n");
        let gateways = if vs.gateways.is_empty() { String::new() } else {
            let gw = vs.gateways.iter().map(|g| format!("    - \"{g}\"")).collect::<Vec<_>>().join("\n");
            format!("  gateways:\n{gw}\n")
        };
        let http = self.gen_http_routes(&vs.http_routes);
        format!(
r#"apiVersion: networking.istio.io/v1alpha3
kind: VirtualService
metadata:
  name: "{name}"
  namespace: "{ns}"
  labels:
{labels}
spec:
  hosts:
{hosts}
{gw}{http}"#,
            name = vs.name, ns = vs.namespace, labels = labels, hosts = hosts, gw = gateways
        )
    }

    fn gen_http_routes(&self, routes: &[HttpRoute]) -> String {
        if routes.is_empty() { return String::new(); }
        let routes_str: Vec<String> = routes.iter().map(|r| self.gen_single_route(r)).collect();
        format!("  http:\n{}", routes_str.join("\n"))
    }

    fn gen_single_route(&self, route: &HttpRoute) -> String {
        let m = self.gen_match_rules(&route.match_rules);
        let d = self.gen_destinations(&route.destinations);
        let t = route.timeout_seconds.map(|s| format!("    timeout: {s}s\n")).unwrap_or_default();
        format!("{m}{d}{t}")
    }

    fn gen_match_rules(&self, rules: &[RouteMatch]) -> String {
        if rules.is_empty() { return String::new(); }
        let matches: Vec<String> = rules.iter().map(|rule| {
            let i = "    ";
            match rule {
                RouteMatch::PathPrefix(p) => format!("{i}  - uri:\n{i}      prefix: \"{p}\""),
                RouteMatch::PathExact(p) => format!("{i}  - uri:\n{i}      exact: \"{p}\""),
                RouteMatch::Headers(headers) => {
                    let hl: Vec<String> = headers.iter().map(|h| {
                        if let Some(ref e) = h.exact {
                            format!("{i}      headers:\n{i}        - name: {}\n{i}          exact: \"{}\"", h.name, e)
                        } else { String::new() }
                    }).collect();
                    format!("{i}  -{}", hl.join("\n"))
                }
            }
        }).collect();
        format!("    match:\n{}\n", matches.join("\n"))
    }

    fn gen_destinations(&self, dests: &[RouteDestination]) -> String {
        dests.iter().map(|d| {
            let subset_str = d.subset.as_ref().map(|s| format!("        subset: \"{s}\"\n")).unwrap_or_default();
            format!("    - route:\n      - destination:\n          host: \"{}\"\n{subset_str}        weight: {}", d.host, d.weight)
        }).collect::<Vec<_>>().join("\n")
    }

    pub fn generate_destinationrule_yaml(&self, dr: &DestinationRule) -> String {
        let labels = self.fmt_labels(&dr.labels, 2);
        let subsets = self.gen_subsets(&dr.subsets);
        let tp = dr.traffic_policy.as_ref().map(|p| self.gen_traffic_policy(p, 2)).unwrap_or_default();
        format!(
r#"apiVersion: networking.istio.io/v1alpha3
kind: DestinationRule
metadata:
  name: "{name}"
  namespace: "{ns}"
  labels:
{labels}
spec:
  host: "{host}"
{tp}{subsets}"#,
            name = dr.name, ns = dr.namespace, labels = labels, host = dr.host, tp = tp
        )
    }

    fn gen_subsets(&self, subsets: &[Subset]) -> String {
        if subsets.is_empty() { return String::new(); }
        let ss: Vec<String> = subsets.iter().map(|s| {
            let l = self.fmt_labels(&s.labels, 4);
            format!("  - name: \"{}\"\n    labels:\n{l}\n", s.name)
        }).collect();
        format!("  subsets:\n{}\n", ss.join("\n"))
    }

    fn gen_traffic_policy(&self, tp: &TrafficPolicy, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let mut parts = vec![];
        if let Some(to) = tp.timeout_seconds {
            parts.push(format!("{pad}connectionPool:\n{pad}  tcp:\n{pad}    connectTimeout: {to}s"));
        }
        if let Some(ref cb) = tp.circuit_breaker {
            parts.push(format!("{pad}outlierDetection:\n{pad}  consecutive5xxErrors: {}\n{pad}  baseEjectionTime: {}s", cb.consecutive_errors, cb.sleep_window_seconds));
        }
        if let Some(ref lb) = tp.load_balancer {
            let s = match lb { LoadBalancerType::RoundRobin => "ROUND_ROBIN", LoadBalancerType::LeastConn => "LEAST_CONN", LoadBalancerType::Random => "RANDOM" };
            parts.push(format!("{pad}loadBalancer:\n{pad}  simple: {s}"));
        }
        if parts.is_empty() { String::new() } else { format!("{pad}trafficPolicy:\n{}\n", parts.join("\n")) }
    }

    pub fn generate_gateway_yaml(&self, gw: &Gateway) -> String {
        let labels = self.fmt_labels(&gw.labels, 2);
        let sel = self.fmt_labels(&gw.selector, 4);
        let svrs = self.gen_gateway_servers(&gw.servers);
        format!(
r#"apiVersion: networking.istio.io/v1alpha3
kind: Gateway
metadata:
  name: "{name}"
  namespace: "{ns}"
  labels:
{labels}
spec:
  selector:
{sel}
{svrs}"#,
            name = gw.name, ns = gw.namespace, labels = labels, sel = sel, svrs = svrs
        )
    }

    fn gen_gateway_servers(&self, servers: &[GatewayServer]) -> String {
        if servers.is_empty() { return String::new(); }
        let svrs: Vec<String> = servers.iter().map(|s| {
            let hosts = s.hosts.iter().map(|h| format!("      - \"{h}\"")).collect::<Vec<_>>().join("\n");
            format!("  - port:\n      number: {}\n      name: \"{}\"\n      protocol: \"{}\"\n    hosts:\n{}", s.port.number, s.port.name, s.port.protocol, hosts)
        }).collect();
        format!("  servers:\n{}\n", svrs.join("\n"))
    }

    pub fn generate_sidecar_yaml(&self, sc: &Sidecar) -> String {
        let labels = self.fmt_labels(&sc.labels, 2);
        let sel = sc.workload_selector.as_ref().map(|s| format!("  workloadSelector:\n    labels:\n{}\n", self.fmt_labels(s, 6))).unwrap_or_default();
        format!(
r#"apiVersion: networking.istio.io/v1alpha3
kind: Sidecar
metadata:
  name: "{name}"
  namespace: "{ns}"
  labels:
{labels}
spec:
{sel}  egress:
    - hosts:
        - "*/*"
"#,
            name = sc.name, ns = sc.namespace, labels = labels, sel = sel
        )
    }

    pub fn generate_weighted_route(&self, svc: impl Into<String>, ns: impl Into<String>, versions: &[(String, u32)]) -> VirtualService {
        let name = format!("{}-weighted", svc.into());
        let ns = ns.into();
        let host = format!("{}.{}.svc.cluster.local", name.split('-').next().unwrap_or(&name), ns);
        let vs = VirtualService::new(name, ns).with_host(host.clone());
        let mut route = HttpRoute::new("weighted-routing");
        for (v, w) in versions { route = route.add_destination(RouteDestination::new(host.clone(), *w).with_subset(v.clone())); }
        vs.add_http_route(route)
    }

    pub fn generate_header_route(&self, svc: impl Into<String>, ns: impl Into<String>, hdr: impl Into<String>, val: impl Into<String>, canary: impl Into<String>, stable: impl Into<String>) -> VirtualService {
        let name = format!("{}-header-routing", svc.into());
        let ns = ns.into();
        let (hdr, val, canary, stable) = (hdr.into(), val.into(), canary.into(), stable.into());
        let host = format!("{}.{}.svc.cluster.local", name.split('-').next().unwrap_or(&name), ns);
        
        VirtualService::new(name, ns).with_host(host.clone())
            .add_http_route(HttpRoute::new("canary-route").with_headers(vec![HeaderMatch::exact(hdr, val)]).add_destination(RouteDestination::new(host.clone(), 100).with_subset(canary)))
            .add_http_route(HttpRoute::new("stable-route").add_destination(RouteDestination::new(host, 100).with_subset(stable)))
    }

    pub fn generate_path_route(&self, svc: impl Into<String>, ns: impl Into<String>, paths: &[(String, String)]) -> VirtualService {
        let name = format!("{}-path-routing", svc.into());
        let ns = ns.into();
        let host = format!("{}.{}.svc.cluster.local", name.split('-').next().unwrap_or(&name), ns);
        let mut vs = VirtualService::new(name, ns).with_host(host.clone());
        for (p, v) in paths {
            let r = HttpRoute::new(format!("path-{}", p.replace('/', "-").trim_matches('-'))).with_path_prefix(p.clone()).add_destination(RouteDestination::new(host.clone(), 100).with_subset(v.clone()));
            vs = vs.add_http_route(r);
        }
        vs
    }

    pub fn generate_canary_release(&self, svc: impl Into<String>, ns: impl Into<String>, stable: impl Into<String>, canary: impl Into<String>, weight: u32) -> VirtualService {
        let (name, ns, stable, canary) = (svc.into(), ns.into(), stable.into(), canary.into());
        let host = format!("{name}.{ns}.svc.cluster.local");
        VirtualService::new(format!("{name}-canary"), ns).with_host(host.clone())
            .add_http_route(HttpRoute::new("canary-release")
                .add_destination(RouteDestination::new(host.clone(), 100 - weight).with_subset(stable))
                .add_destination(RouteDestination::new(host, weight).with_subset(canary)))
    }

    pub fn generate_complete_config(&self, svc: impl Into<String>, ns: impl Into<String>, versions: &[String], gw_name: Option<String>) -> CompleteMeshConfig {
        let (name, ns) = (svc.into(), ns.into());
        let host = format!("{name}.{ns}.svc.cluster.local");
        let subsets: Vec<Subset> = versions.iter().map(|v| Subset::new(v.clone()).with_label("version", v)).collect();
        let dr = subsets.into_iter().fold(
            DestinationRule::new(format!("{name}-dr"), ns.clone(), host.clone())
                .with_traffic_policy(TrafficPolicy::new().with_timeout(30).with_retries(RetryPolicy::default()).with_circuit_breaker(CircuitBreaker::default())),
            DestinationRule::add_subset
        );
        let mut vs = VirtualService::new(format!("{name}-vs"), ns.clone()).with_host(host.clone());
        if let Some(ref gw) = gw_name { vs = vs.with_gateway(gw.clone()); }
        let weights: Vec<(String, u32)> = if versions.len() == 1 {
            vec![(versions[0].clone(), 100)]
        } else {
            let w = 100 / versions.len() as u32;
            versions.iter().enumerate().map(|(i, v)| (v.clone(), if i == versions.len() - 1 { 100 - w * (versions.len() as u32 - 1) } else { w })).collect()
        };
        let route = weights.iter().fold(HttpRoute::new("primary"), |acc, (v, w)| acc.add_destination(RouteDestination::new(host.clone(), *w).with_subset(v.clone())));
        vs = vs.add_http_route(route);
        let gw = gw_name.map(|g| Gateway::new(g, ns.clone()).add_server(GatewayPort::http(80), vec!["*".into()]));
        let sidecar = Sidecar::new(format!("{name}-sidecar"), ns);
        CompleteMeshConfig { virtual_service: vs, destination_rule: dr, gateway: gw, sidecar }
    }
}

impl Default for MeshGenerator { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone)]
pub struct CompleteMeshConfig {
    pub virtual_service: VirtualService,
    pub destination_rule: DestinationRule,
    pub gateway: Option<Gateway>,
    pub sidecar: Sidecar,
}

impl CompleteMeshConfig {
    pub fn to_yaml(&self) -> String {
        let mut parts = vec![];
        let generator = MeshGenerator::new();
        parts.push(generator.generate_virtualservice_yaml(&self.virtual_service));
        parts.push("---".into());
        parts.push(generator.generate_destinationrule_yaml(&self.destination_rule));
        if let Some(ref gw) = self.gateway { parts.push("---".into()); parts.push(generator.generate_gateway_yaml(gw)); }
        parts.push("---".into());
        parts.push(generator.generate_sidecar_yaml(&self.sidecar));
        parts.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtualservice_generation() {
        let generator = MeshGenerator::new();
        let vs = VirtualService::new("test-vs", "default")
            .with_host("test-service.default.svc.cluster.local")
            .with_gateway("test-gateway")
            .add_http_route(HttpRoute::new("primary").with_path_prefix("/api")
                .add_destination(RouteDestination::new("test-service.default.svc.cluster.local", 100).with_subset("v1")));
        let yaml = generator.generate_virtualservice_yaml(&vs);
        assert!(yaml.contains("kind: VirtualService"));
        assert!(yaml.contains("name: \"test-vs\""));
        assert!(yaml.contains("test-gateway"));
    }

    #[test]
    fn test_destinationrule_generation() {
        let generator = MeshGenerator::new();
        let dr = DestinationRule::new("test-dr", "default", "test-service")
            .add_subset(Subset::new("v1").with_label("version", "v1"))
            .with_traffic_policy(TrafficPolicy::new().with_timeout(15));
        let yaml = generator.generate_destinationrule_yaml(&dr);
        assert!(yaml.contains("kind: DestinationRule"));
        assert!(yaml.contains("host: \"test-service\""));
        assert!(yaml.contains("v1"));
    }

    #[test]
    fn test_gateway_generation() {
        let generator = MeshGenerator::new();
        let gw = Gateway::new("test-gw", "default").add_server(GatewayPort::http(80), vec!["*.example.com".into()]);
        let yaml = generator.generate_gateway_yaml(&gw);
        assert!(yaml.contains("kind: Gateway"));
        assert!(yaml.contains("number: 80"));
        assert!(yaml.contains("*.example.com"));
    }

    #[test]
    fn test_weighted_routing() {
        let generator = MeshGenerator::new();
        let vs = generator.generate_weighted_route("myapp", "prod", &[("v1".into(), 80), ("v2".into(), 20)]);
        assert_eq!(vs.http_routes[0].destinations.len(), 2);
        assert_eq!(vs.http_routes[0].destinations[0].weight, 80);
        assert_eq!(vs.http_routes[0].destinations[1].weight, 20);
    }

    #[test]
    fn test_header_routing() {
        let generator = MeshGenerator::new();
        let vs = generator.generate_header_route("myapp", "prod", "x-type", "beta", "v2", "v1");
        assert_eq!(vs.http_routes.len(), 2);
        if let RouteMatch::Headers(ref h) = vs.http_routes[0].match_rules[0] {
            assert_eq!(h[0].name, "x-type");
            assert_eq!(h[0].exact.as_deref(), Some("beta"));
        }
    }

    #[test]
    fn test_timeout_configuration() {
        let tp = TrafficPolicy::new().with_timeout(45);
        assert_eq!(tp.timeout_seconds, Some(45));
        let route = HttpRoute::new("test").with_timeout(30);
        assert_eq!(route.timeout_seconds, Some(30));
    }

    #[test]
    fn test_retry_configuration() {
        let r = RetryPolicy::new(5, 10);
        assert_eq!(r.attempts, 5);
        assert_eq!(r.per_try_timeout_seconds, 10);
        assert_eq!(RetryPolicy::default().attempts, 3);
    }

    #[test]
    fn test_circuit_breaker_configuration() {
        let cb = CircuitBreaker::new(10, 60).with_half_open_max_requests(5);
        assert_eq!(cb.consecutive_errors, 10);
        assert_eq!(cb.sleep_window_seconds, 60);
        assert_eq!(cb.half_open_max_requests, 5);
        assert_eq!(CircuitBreaker::default().consecutive_errors, 5);
    }

    #[test]
    fn test_canary_release() {
        let generator = MeshGenerator::new();
        let vs = generator.generate_canary_release("orders", "staging", "v1", "v2", 10);
        assert_eq!(vs.http_routes[0].destinations[0].weight, 90);
        assert_eq!(vs.http_routes[0].destinations[1].weight, 10);
        assert_eq!(vs.http_routes[0].destinations[0].subset.as_deref(), Some("v1"));
    }

    #[test]
    fn test_complete_config_generation() {
        let generator = MeshGenerator::new();
        let cfg = generator.generate_complete_config("payments", "prod", &["v1".into(), "v2".into()], Some("payments-gw".into()));
        let yaml = cfg.to_yaml();
        assert!(yaml.contains("VirtualService"));
        assert!(yaml.contains("DestinationRule"));
        assert!(yaml.contains("Gateway"));
        assert!(yaml.contains("Sidecar"));
        assert!(yaml.contains("---"));
    }

    #[test]
    fn test_path_routing() {
        let generator = MeshGenerator::new();
        let vs = generator.generate_path_route("api", "default", &[("/v1/".into(), "v1".into()), ("/v2/".into(), "v2".into())]);
        assert_eq!(vs.http_routes.len(), 2);
        if let RouteMatch::PathPrefix(ref p) = vs.http_routes[0].match_rules[0] {
            assert_eq!(p, "/v1/");
        }
    }

    #[test]
    fn test_rate_limit_config() {
        let rl = RateLimit::new(1000, RateLimitUnit::Minute).with_burst(200);
        assert_eq!(rl.requests_per_unit, 1000);
        assert_eq!(rl.unit, RateLimitUnit::Minute);
        assert_eq!(rl.burst, 200);
        assert_eq!(RateLimit::default().unit, RateLimitUnit::Second);
    }

    #[test]
    fn test_sidecar_generation() {
        let generator = MeshGenerator::new();
        let mut sel = HashMap::new();
        sel.insert("app".into(), "myapp".into());
        let sc = Sidecar::new("myapp-sc", "default").with_workload_selector(sel);
        let yaml = generator.generate_sidecar_yaml(&sc);
        assert!(yaml.contains("kind: Sidecar"));
        assert!(yaml.contains("workloadSelector"));
    }

    #[test]
    fn test_mesh_config_defaults() {
        let c = MeshConfig::default();
        assert_eq!(c.name, "default");
        assert!(c.labels.contains_key("app.kubernetes.io/managed-by"));
    }

    #[test]
    fn test_load_balancer_types() {
        let tp = TrafficPolicy::new().with_load_balancer(LoadBalancerType::LeastConn);
        assert_eq!(tp.load_balancer, Some(LoadBalancerType::LeastConn));
    }
}
