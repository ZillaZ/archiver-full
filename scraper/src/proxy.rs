pub struct ProxyManager {
    proxies: Vec<String>,
    current: usize,
}

impl ProxyManager {
    pub fn new(proxies: Vec<String>) -> Self {
        Self { proxies, current: 0 }
    }

    pub fn next(&mut self) -> Option<String> {
        if self.proxies.is_empty() {
            return None;
        }
        let proxy = self.proxies[self.current].clone();
        self.current = (self.current + 1) % self.proxies.len();
        Some(proxy)
    }

    pub fn mark_bad(&mut self) {
        if !self.proxies.is_empty() {
            let bad = self.proxies.remove(self.current.saturating_sub(1) % self.proxies.len().max(1));
            tracing::warn!("removed bad proxy: {bad}");
        }
    }

    pub fn add(&mut self, proxy: String) {
        self.proxies.push(proxy);
    }
}