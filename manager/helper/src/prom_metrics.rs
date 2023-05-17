use prometheus::TextEncoder;

pub struct PooledMetrics {
    pub encoder: TextEncoder,
    pub buffer:  Vec<u8>,
}

impl Default for PooledMetrics {
    fn default() -> Self {
        Self { encoder: TextEncoder::new(), buffer: vec![] }
    }
}
