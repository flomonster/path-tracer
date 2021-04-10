use serde::Deserialize;

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Default for Resolution {
    fn default() -> Self {
        Resolution {
            width: 1920,
            height: 1080,
        }
    }
}
