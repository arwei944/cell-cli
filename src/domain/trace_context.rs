use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpanId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TraceContext {
    pub trace_id: TraceId,
    pub parent_span_id: Option<SpanId>,
    pub span_id: SpanId,
    pub trace_flags: u8,
    pub trace_state: HashMap<String, String>,
    pub version: u8,
}

impl TraceContext {
    pub fn new() -> Self {
        Self {
            trace_id: TraceId(Self::generate_trace_id()),
            parent_span_id: None,
            span_id: SpanId(Self::generate_span_id()),
            trace_flags: 0x01,
            trace_state: HashMap::new(),
            version: 0,
        }
    }

    pub fn new_child(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            parent_span_id: Some(self.span_id.clone()),
            span_id: SpanId(Self::generate_span_id()),
            trace_flags: self.trace_flags,
            trace_state: self.trace_state.clone(),
            version: self.version,
        }
    }

    pub fn from_traceparent(header: &str) -> Result<Self, TraceContextError> {
        let parts: Vec<&str> = header.trim().split('-').collect();
        if parts.len() < 4 {
            return Err(TraceContextError::InvalidFormat(
                "traceparent must have 4 parts".to_string(),
            ));
        }

        let version = u8::from_str_radix(parts[0], 16)
            .map_err(|e| TraceContextError::InvalidFormat(format!("version: {}", e)))?;

        if parts[1].len() != 32 {
            return Err(TraceContextError::InvalidFormat(
                "trace-id must be 32 hex chars".to_string(),
            ));
        }
        let trace_id = TraceId(parts[1].to_string());

        if parts[2].len() != 16 {
            return Err(TraceContextError::InvalidFormat(
                "span-id must be 16 hex chars".to_string(),
            ));
        }
        let span_id = SpanId(parts[2].to_string());

        let trace_flags = u8::from_str_radix(parts[3], 16)
            .map_err(|e| TraceContextError::InvalidFormat(format!("trace-flags: {}", e)))?;

        Ok(Self {
            trace_id,
            parent_span_id: None,
            span_id,
            trace_flags,
            trace_state: HashMap::new(),
            version,
        })
    }

    pub fn to_traceparent(&self) -> String {
        format!(
            "{:02x}-{}-{}-{:02x}",
            self.version, self.trace_id.0, self.span_id.0, self.trace_flags
        )
    }

    pub fn from_tracestate(header: &str) -> Result<HashMap<String, String>, TraceContextError> {
        let mut state = HashMap::new();

        for pair in header.trim().split(',') {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }

            let parts: Vec<&str> = pair.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(TraceContextError::InvalidFormat(
                    "tracestate pair must have key=value".to_string(),
                ));
            }

            state.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
        }

        Ok(state)
    }

    pub fn to_tracestate(&self) -> String {
        let pairs: Vec<String> = self
            .trace_state
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        pairs.join(",")
    }

    pub fn is_sampled(&self) -> bool {
        (self.trace_flags & 0x01) != 0
    }

    pub fn set_sampled(&mut self, sampled: bool) {
        if sampled {
            self.trace_flags |= 0x01;
        } else {
            self.trace_flags &= 0xFE;
        }
    }

    pub fn add_tracestate_entry(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.trace_state.insert(key.into(), value.into());
    }

    fn generate_trace_id() -> String {
        let mut bytes = [0u8; 16];
        Self::fill_random_bytes(&mut bytes);
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    fn generate_span_id() -> String {
        let mut bytes = [0u8; 8];
        Self::fill_random_bytes(&mut bytes);
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    fn fill_random_bytes(bytes: &mut [u8]) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let mut seed = nanos;
        for byte in bytes.iter_mut() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            *byte = (seed >> 56) as u8;
        }
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TraceContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TraceContext(trace_id={}, span_id={}, sampled={})",
            self.trace_id.0, self.span_id.0, self.is_sampled())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraceContextError {
    InvalidFormat(String),
    InvalidTraceId,
    InvalidSpanId,
}

impl fmt::Display for TraceContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TraceContextError::InvalidFormat(msg) => write!(f, "Invalid trace context format: {}", msg),
            TraceContextError::InvalidTraceId => write!(f, "Invalid trace ID"),
            TraceContextError::InvalidSpanId => write!(f, "Invalid span ID"),
        }
    }
}

impl std::error::Error for TraceContextError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
    pub name: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub status: SpanStatus,
    pub attributes: HashMap<String, String>,
    pub events: Vec<SpanEvent>,
    pub kind: SpanKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpanStatus {
    Unset,
    Ok,
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpanKind {
    Internal,
    Server,
    Client,
    Producer,
    Consumer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp: u64,
    pub attributes: HashMap<String, String>,
}

impl Span {
    pub fn new(context: &TraceContext, name: impl Into<String>) -> Self {
        Self {
            trace_id: context.trace_id.clone(),
            span_id: context.span_id.clone(),
            parent_span_id: context.parent_span_id.clone(),
            name: name.into(),
            start_time: Self::current_time_millis(),
            end_time: None,
            status: SpanStatus::Unset,
            attributes: HashMap::new(),
            events: Vec::new(),
            kind: SpanKind::Internal,
        }
    }

    pub fn end(&mut self) {
        self.end_time = Some(Self::current_time_millis());
        if self.status == SpanStatus::Unset {
            self.status = SpanStatus::Ok;
        }
    }

    pub fn set_status_ok(&mut self) {
        self.status = SpanStatus::Ok;
    }

    pub fn set_status_error(&mut self, message: impl Into<String>) {
        self.status = SpanStatus::Error { message: message.into() };
    }

    pub fn add_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    pub fn add_event(&mut self, name: impl Into<String>) {
        self.events.push(SpanEvent {
            name: name.into(),
            timestamp: Self::current_time_millis(),
            attributes: HashMap::new(),
        });
    }

    pub fn duration_millis(&self) -> Option<u64> {
        self.end_time.map(|end| end - self.start_time)
    }

    fn current_time_millis() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

pub struct TracePropagator;

impl TracePropagator {
    pub fn extract_from_headers(headers: &HashMap<String, String>) -> Result<TraceContext, TraceContextError> {
        let traceparent = headers.get("traceparent")
            .or_else(|| headers.get("Traceparent"))
            .ok_or_else(|| TraceContextError::InvalidFormat("missing traceparent header".to_string()))?;

        let mut context = TraceContext::from_traceparent(traceparent)?;

        if let Some(tracestate) = headers.get("tracestate").or_else(|| headers.get("Tracestate")) {
            context.trace_state = TraceContext::from_tracestate(tracestate)?;
        }

        Ok(context)
    }

    pub fn inject_into_headers(context: &TraceContext, headers: &mut HashMap<String, String>) {
        headers.insert("traceparent".to_string(), context.to_traceparent());
        let tracestate = context.to_tracestate();
        if !tracestate.is_empty() {
            headers.insert("tracestate".to_string(), tracestate);
        }
    }

    pub fn extract_from_meta(meta: &[(&str, &str)]) -> Result<TraceContext, TraceContextError> {
        let mut headers = HashMap::new();
        for (k, v) in meta {
            headers.insert(k.to_string(), v.to_string());
        }
        Self::extract_from_headers(&headers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_context_new() {
        let ctx = TraceContext::new();
        assert_eq!(ctx.trace_id.0.len(), 32);
        assert_eq!(ctx.span_id.0.len(), 16);
        assert!(ctx.is_sampled());
        assert!(ctx.parent_span_id.is_none());
    }

    #[test]
    fn test_new_child() {
        let parent = TraceContext::new();
        let child = parent.new_child();

        assert_eq!(parent.trace_id.0, child.trace_id.0);
        assert_eq!(child.parent_span_id.as_ref().unwrap().0, parent.span_id.0);
        assert_ne!(parent.span_id.0, child.span_id.0);
    }

    #[test]
    fn test_traceparent_roundtrip() {
        let ctx = TraceContext::new();
        let header = ctx.to_traceparent();
        let parsed = TraceContext::from_traceparent(&header).unwrap();

        assert_eq!(ctx.trace_id.0, parsed.trace_id.0);
        assert_eq!(ctx.span_id.0, parsed.span_id.0);
        assert_eq!(ctx.trace_flags, parsed.trace_flags);
        assert_eq!(ctx.version, parsed.version);
    }

    #[test]
    fn test_traceparent_known_value() {
        let header = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        let ctx = TraceContext::from_traceparent(header).unwrap();

        assert_eq!(ctx.version, 0);
        assert_eq!(ctx.trace_id.0, "0af7651916cd43dd8448eb211c80319c");
        assert_eq!(ctx.span_id.0, "b7ad6b7169203331");
        assert!(ctx.is_sampled());
    }

    #[test]
    fn test_traceparent_invalid_short() {
        let result = TraceContext::from_traceparent("00-abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_traceparent_invalid_trace_id_length() {
        let result = TraceContext::from_traceparent("00-1234-b7ad6b7169203331-01");
        assert!(result.is_err());
    }

    #[test]
    fn test_traceparent_invalid_span_id_length() {
        let result = TraceContext::from_traceparent("00-0af7651916cd43dd8448eb211c80319c-1234-01");
        assert!(result.is_err());
    }

    #[test]
    fn test_tracestate_parse() {
        let state = TraceContext::from_tracestate("vendor1=value1,vendor2=value2").unwrap();
        assert_eq!(state.len(), 2);
        assert_eq!(state.get("vendor1").unwrap(), "value1");
        assert_eq!(state.get("vendor2").unwrap(), "value2");
    }

    #[test]
    fn test_tracestate_roundtrip() {
        let mut ctx = TraceContext::new();
        ctx.add_tracestate_entry("vendor1", "value1");
        ctx.add_tracestate_entry("vendor2", "value2");

        let state_str = ctx.to_tracestate();
        let parsed = TraceContext::from_tracestate(&state_str).unwrap();

        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_set_sampled() {
        let mut ctx = TraceContext::new();
        assert!(ctx.is_sampled());

        ctx.set_sampled(false);
        assert!(!ctx.is_sampled());

        ctx.set_sampled(true);
        assert!(ctx.is_sampled());
    }

    #[test]
    fn test_span_lifecycle() {
        let ctx = TraceContext::new();
        let mut span = Span::new(&ctx, "test-span");

        assert_eq!(span.name, "test-span");
        assert_eq!(span.status, SpanStatus::Unset);
        assert!(span.end_time.is_none());

        span.add_attribute("key1", "value1");
        span.add_event("something-happened");

        span.end();
        assert!(span.end_time.is_some());
        assert_eq!(span.status, SpanStatus::Ok);
        assert!(span.duration_millis().is_some());
    }

    #[test]
    fn test_span_error_status() {
        let ctx = TraceContext::new();
        let mut span = Span::new(&ctx, "test-span");
        span.set_status_error("something went wrong");
        span.end();

        if let SpanStatus::Error { message } = &span.status {
            assert_eq!(message, "something went wrong");
        } else {
            panic!("Expected error status");
        }
    }

    #[test]
    fn test_propagator_roundtrip() {
        let ctx = TraceContext::new();
        let mut headers = HashMap::new();

        TracePropagator::inject_into_headers(&ctx, &mut headers);
        assert!(headers.contains_key("traceparent"));

        let extracted = TracePropagator::extract_from_headers(&headers).unwrap();
        assert_eq!(ctx.trace_id.0, extracted.trace_id.0);
        assert_eq!(ctx.span_id.0, extracted.span_id.0);
    }

    #[test]
    fn test_trace_context_display() {
        let ctx = TraceContext::new();
        let display = format!("{}", ctx);
        assert!(display.contains("TraceContext"));
        assert!(display.contains("trace_id="));
    }
}
