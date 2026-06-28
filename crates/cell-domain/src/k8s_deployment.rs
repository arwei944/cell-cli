use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sDeploymentConfig {
    pub name: String,
    pub namespace: String,
    pub image: String,
    pub replicas: u32,
    pub port: u16,
    pub service_port: u16,
    pub service_type: ServiceType,
    pub resources: ResourceRequirements,
    pub health_check: HealthCheckConfig,
    pub hpa: HpaConfig,
    pub env_vars: Vec<EnvVar>,
    pub labels: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum ServiceType {
    #[default]
    ClusterIP,
    NodePort,
    LoadBalancer,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_request: String,
    pub memory_request: String,
    pub cpu_limit: String,
    pub memory_limit: String,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_request: "100m".to_string(),
            memory_request: "128Mi".to_string(),
            cpu_limit: "500m".to_string(),
            memory_limit: "512Mi".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub startup_probe: ProbeConfig,
    pub liveness_probe: ProbeConfig,
    pub readiness_probe: ProbeConfig,
    pub health_path: String,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            startup_probe: ProbeConfig {
                enabled: true,
                initial_delay_seconds: 0,
                period_seconds: 10,
                timeout_seconds: 5,
                failure_threshold: 30,
                success_threshold: 1,
            },
            liveness_probe: ProbeConfig {
                enabled: true,
                initial_delay_seconds: 15,
                period_seconds: 20,
                timeout_seconds: 5,
                failure_threshold: 3,
                success_threshold: 1,
            },
            readiness_probe: ProbeConfig {
                enabled: true,
                initial_delay_seconds: 5,
                period_seconds: 10,
                timeout_seconds: 5,
                failure_threshold: 3,
                success_threshold: 1,
            },
            health_path: "/health".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeConfig {
    pub enabled: bool,
    pub initial_delay_seconds: u32,
    pub period_seconds: u32,
    pub timeout_seconds: u32,
    pub failure_threshold: u32,
    pub success_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HpaConfig {
    pub enabled: bool,
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_utilization: u32,
    pub target_memory_utilization: u32,
}

impl Default for HpaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_replicas: 2,
            max_replicas: 10,
            target_cpu_utilization: 70,
            target_memory_utilization: 80,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: Option<String>,
    pub value_from: Option<EnvVarSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarSource {
    pub config_map_name: Option<String>,
    pub config_map_key: Option<String>,
    pub secret_name: Option<String>,
    pub secret_key: Option<String>,
}

impl K8sDeploymentConfig {
    pub fn new(name: impl Into<String>, image: impl Into<String>) -> Self {
        let name = name.into();
        let mut labels = std::collections::HashMap::new();
        labels.insert("app".to_string(), name.clone());
        labels.insert("app.kubernetes.io/name".to_string(), name.clone());
        labels.insert("app.kubernetes.io/part-of".to_string(), "cell-architecture".to_string());

        Self {
            name,
            namespace: "default".to_string(),
            image: image.into(),
            replicas: 2,
            port: 8080,
            service_port: 80,
            service_type: ServiceType::default(),
            resources: ResourceRequirements::default(),
            health_check: HealthCheckConfig::default(),
            hpa: HpaConfig::default(),
            env_vars: Vec::new(),
            labels,
        }
    }

    pub fn generate_deployment_yaml(&self) -> String {
        let labels_str = self.labels.iter()
            .map(|(k, v)| format!("    {k}: \"{v}\""))
            .collect::<Vec<_>>()
            .join("\n");

        let selector_labels = self.labels.iter()
            .map(|(k, v)| format!("      {k}: \"{v}\""))
            .collect::<Vec<_>>()
            .join("\n");

        let template_labels = self.labels.iter()
            .map(|(k, v)| format!("      {k}: \"{v}\""))
            .collect::<Vec<_>>()
            .join("\n");

        let probes = self.generate_probes_yaml();

        let env_section = if self.env_vars.is_empty() {
            String::new()
        } else {
            let env_vars: Vec<String> = self.env_vars.iter()
                .map(|env| {
                    if let Some(ref value) = env.value {
                        format!("        - name: {}\n          value: \"{}\"", env.name, value)
                    } else {
                        format!("        - name: {}", env.name)
                    }
                })
                .collect();
            format!("        env:\n{}\n", env_vars.join("\n"))
        };

        format!(
r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: "{name}"
  namespace: "{namespace}"
  labels:
{labels}
spec:
  replicas: {replicas}
  selector:
    matchLabels:
{selector_labels}
  template:
    metadata:
      labels:
{template_labels}
    spec:
      containers:
        - name: "{name}"
          image: "{image}"
          ports:
            - containerPort: {port}
              name: http
              protocol: TCP
          resources:
            requests:
              cpu: "{cpu_request}"
              memory: "{memory_request}"
            limits:
              cpu: "{cpu_limit}"
              memory: "{memory_limit}"
{env_section}{probes}
"#,
            name = self.name,
            namespace = self.namespace,
            labels = labels_str,
            replicas = self.replicas,
            selector_labels = selector_labels,
            template_labels = template_labels,
            image = self.image,
            port = self.port,
            cpu_request = self.resources.cpu_request,
            memory_request = self.resources.memory_request,
            cpu_limit = self.resources.cpu_limit,
            memory_limit = self.resources.memory_limit,
            env_section = env_section,
            probes = probes,
        )
    }

    fn generate_probes_yaml(&self) -> String {
        if !self.health_check.enabled {
            return String::new();
        }

        let mut probes = Vec::new();

        if self.health_check.startup_probe.enabled {
            probes.push(format!(
r#"          startupProbe:
            httpGet:
              path: "{path}"
              port: {port}
            initialDelaySeconds: {init_delay}
            periodSeconds: {period}
            timeoutSeconds: {timeout}
            failureThreshold: {failure}
            successThreshold: {success}"#,
                path = self.health_check.health_path,
                port = self.port,
                init_delay = self.health_check.startup_probe.initial_delay_seconds,
                period = self.health_check.startup_probe.period_seconds,
                timeout = self.health_check.startup_probe.timeout_seconds,
                failure = self.health_check.startup_probe.failure_threshold,
                success = self.health_check.startup_probe.success_threshold,
            ));
        }

        if self.health_check.liveness_probe.enabled {
            probes.push(format!(
r#"          livenessProbe:
            httpGet:
              path: "{path}"
              port: {port}
            initialDelaySeconds: {init_delay}
            periodSeconds: {period}
            timeoutSeconds: {timeout}
            failureThreshold: {failure}
            successThreshold: {success}"#,
                path = self.health_check.health_path,
                port = self.port,
                init_delay = self.health_check.liveness_probe.initial_delay_seconds,
                period = self.health_check.liveness_probe.period_seconds,
                timeout = self.health_check.liveness_probe.timeout_seconds,
                failure = self.health_check.liveness_probe.failure_threshold,
                success = self.health_check.liveness_probe.success_threshold,
            ));
        }

        if self.health_check.readiness_probe.enabled {
            probes.push(format!(
r#"          readinessProbe:
            httpGet:
              path: "{path}"
              port: {port}
            initialDelaySeconds: {init_delay}
            periodSeconds: {period}
            timeoutSeconds: {timeout}
            failureThreshold: {failure}
            successThreshold: {success}"#,
                path = self.health_check.health_path,
                port = self.port,
                init_delay = self.health_check.readiness_probe.initial_delay_seconds,
                period = self.health_check.readiness_probe.period_seconds,
                timeout = self.health_check.readiness_probe.timeout_seconds,
                failure = self.health_check.readiness_probe.failure_threshold,
                success = self.health_check.readiness_probe.success_threshold,
            ));
        }

        if probes.is_empty() {
            String::new()
        } else {
            probes.join("\n")
        }
    }

    pub fn generate_service_yaml(&self) -> String {
        let labels_str = self.labels.iter()
            .map(|(k, v)| format!("  {k}: \"{v}\""))
            .collect::<Vec<_>>()
            .join("\n");

        let selector_labels = self.labels.iter()
            .map(|(k, v)| format!("    {k}: \"{v}\""))
            .collect::<Vec<_>>()
            .join("\n");

        let service_type_str = match self.service_type {
            ServiceType::ClusterIP => "ClusterIP",
            ServiceType::NodePort => "NodePort",
            ServiceType::LoadBalancer => "LoadBalancer",
        };

        format!(
r#"apiVersion: v1
kind: Service
metadata:
  name: "{name}"
  namespace: "{namespace}"
  labels:
{labels}
spec:
  type: {service_type}
  selector:
{selector_labels}
  ports:
    - name: http
      port: {service_port}
      targetPort: {port}
      protocol: TCP
"#,
            name = self.name,
            namespace = self.namespace,
            labels = labels_str,
            service_type = service_type_str,
            selector_labels = selector_labels,
            service_port = self.service_port,
            port = self.port,
        )
    }

    pub fn generate_hpa_yaml(&self) -> String {
        if !self.hpa.enabled {
            return String::new();
        }

        format!(
r#"apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: "{name}"
  namespace: "{namespace}"
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: "{name}"
  minReplicas: {min_replicas}
  maxReplicas: {max_replicas}
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: {cpu_target}
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: {mem_target}
"#,
            name = self.name,
            namespace = self.namespace,
            min_replicas = self.hpa.min_replicas,
            max_replicas = self.hpa.max_replicas,
            cpu_target = self.hpa.target_cpu_utilization,
            mem_target = self.hpa.target_memory_utilization,
        )
    }

    pub fn generate_all_yaml(&self) -> String {
        let mut parts = Vec::new();

        parts.push(self.generate_deployment_yaml());
        parts.push("---".to_string());
        parts.push(self.generate_service_yaml());

        if self.hpa.enabled {
            parts.push("---".to_string());
            parts.push(self.generate_hpa_yaml());
        }

        parts.join("\n")
    }

    pub fn generate_health_handler_rs(&self) -> String {
        r#"use axum::{Json, http::StatusCode};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

pub async fn health_check() -> (StatusCode, Json<HealthResponse>) {
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0,
    };
    (StatusCode::OK, Json(response))
}

pub fn health_router() -> axum::Router {
    axum::Router::new()
        .route("/health", axum::routing::get(health_check))
}
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_config_default() {
        let config = K8sDeploymentConfig::new("test-service", "test/image:latest");
        assert_eq!(config.name, "test-service");
        assert_eq!(config.image, "test/image:latest");
        assert_eq!(config.replicas, 2);
        assert_eq!(config.port, 8080);
        assert!(config.health_check.enabled);
    }

    #[test]
    fn test_generate_deployment_yaml() {
        let config = K8sDeploymentConfig::new("test-service", "test/image:v1");
        let yaml = config.generate_deployment_yaml();

        assert!(yaml.contains("apiVersion: apps/v1"));
        assert!(yaml.contains("kind: Deployment"));
        assert!(yaml.contains("name: \"test-service\""));
        assert!(yaml.contains("image: \"test/image:v1\""));
        assert!(yaml.contains("replicas: 2"));
        assert!(yaml.contains("containerPort: 8080"));
    }

    #[test]
    fn test_generate_service_yaml() {
        let config = K8sDeploymentConfig::new("test-service", "test/image:v1");
        let yaml = config.generate_service_yaml();

        assert!(yaml.contains("apiVersion: v1"));
        assert!(yaml.contains("kind: Service"));
        assert!(yaml.contains("type: ClusterIP"));
        assert!(yaml.contains("port: 80"));
        assert!(yaml.contains("targetPort: 8080"));
    }

    #[test]
    fn test_generate_hpa_yaml() {
        let config = K8sDeploymentConfig::new("test-service", "test/image:v1");
        let yaml = config.generate_hpa_yaml();

        assert!(yaml.contains("HorizontalPodAutoscaler"));
        assert!(yaml.contains("minReplicas: 2"));
        assert!(yaml.contains("maxReplicas: 10"));
        assert!(yaml.contains("averageUtilization: 70"));
    }

    #[test]
    fn test_hpa_disabled() {
        let mut config = K8sDeploymentConfig::new("test-service", "test/image:v1");
        config.hpa.enabled = false;
        let yaml = config.generate_hpa_yaml();
        assert!(yaml.is_empty());
    }

    #[test]
    fn test_health_check_disabled() {
        let mut config = K8sDeploymentConfig::new("test-service", "test/image:v1");
        config.health_check.enabled = false;
        let yaml = config.generate_deployment_yaml();
        assert!(!yaml.contains("startupProbe"));
        assert!(!yaml.contains("livenessProbe"));
        assert!(!yaml.contains("readinessProbe"));
    }

    #[test]
    fn test_generate_all_yaml() {
        let config = K8sDeploymentConfig::new("test-service", "test/image:v1");
        let yaml = config.generate_all_yaml();

        assert!(yaml.contains("Deployment"));
        assert!(yaml.contains("Service"));
        assert!(yaml.contains("HorizontalPodAutoscaler"));
        assert!(yaml.contains("---"));
    }

    #[test]
    fn test_loadbalancer_service() {
        let mut config = K8sDeploymentConfig::new("test-service", "test/image:v1");
        config.service_type = ServiceType::LoadBalancer;
        let yaml = config.generate_service_yaml();
        assert!(yaml.contains("type: LoadBalancer"));
    }

    #[test]
    fn test_health_handler_rs() {
        let config = K8sDeploymentConfig::new("test-service", "test/image:v1");
        let code = config.generate_health_handler_rs();

        assert!(code.contains("health_check"));
        assert!(code.contains("HealthResponse"));
        assert!(code.contains("/health"));
    }

    #[test]
    fn test_resource_defaults() {
        let resources = ResourceRequirements::default();
        assert_eq!(resources.cpu_request, "100m");
        assert_eq!(resources.memory_limit, "512Mi");
    }

    #[test]
    fn test_service_type_default() {
        assert_eq!(ServiceType::default(), ServiceType::ClusterIP);
    }

    #[test]
    fn test_env_vars_in_deployment() {
        let mut config = K8sDeploymentConfig::new("test-service", "test/image:v1");
        config.env_vars.push(EnvVar {
            name: "RUST_LOG".to_string(),
            value: Some("info".to_string()),
            value_from: None,
        });

        let yaml = config.generate_deployment_yaml();
        assert!(yaml.contains("RUST_LOG"));
        assert!(yaml.contains("info"));
    }
}
