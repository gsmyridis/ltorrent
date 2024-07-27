/// Represents the configuration settings for the application.
pub struct Configuration {
    peer_id: String,
    port: u16,
}

impl Configuration {
    /// Returns the peer ID, which is a 20-byte array.
    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    /// Returns the port number.
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            peer_id: "00112233445566778899".to_string(),
            port: 6881,
        }
    }
}