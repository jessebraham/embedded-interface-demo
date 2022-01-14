use esp_idf_sys::*;
use serde::Serialize;
use strum_macros::FromRepr;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, FromRepr)]
pub enum Model {
    #[serde(rename = "ESP32")]
    Esp32   = 0x1,
    #[serde(rename = "ESP32-C3")]
    Esp32c3 = 0x5,
    #[serde(rename = "ESP32-S2")]
    Esp32s2 = 0x2,
    #[serde(rename = "ESP32-S3")]
    Esp32s3 = 0x9,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, FromRepr)]
pub enum Feature {
    #[serde(rename = "Embedded flash memory")]
    EmbFlash = 0x01,
    #[serde(rename = "2.4GHz WiFi")]
    WifiBgn  = 0x02,
    #[serde(rename = "Bluetooth LE")]
    Ble      = 0x10,
    #[serde(rename = "Bluetooth Classic")]
    Bt       = 0x20,
    #[serde(rename = "Embedded PSRAM")]
    EmbPsram = 0x80,
}

impl Feature {
    pub fn from(flags: usize) -> Vec<Feature> {
        let mut features = vec![];

        // Check each bit in `flags` to see if it is set, and if it corresponds with a
        // feature. We only check the lowest 8 bits, as that's where all of the feature
        // flags live.
        for i in 0..8 {
            let mask = 0x1 << i;
            if let Some(feature) = Feature::from_repr(flags & mask) {
                features.push(feature);
            }
        }

        features
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChipInfo {
    pub model: Option<Model>,
    pub features: Vec<Feature>,
    pub cores: u8,
    pub revision: u8,
}

impl ChipInfo {
    pub fn new() -> Self {
        let mut info = esp_chip_info_t::default();
        unsafe { esp_chip_info(&mut info as *mut esp_chip_info_t) };

        let esp_chip_info_t {
            model,
            features,
            cores,
            revision,
        } = info;

        Self {
            model: Model::from_repr(model as usize),
            features: Feature::from(features as usize),
            cores,
            revision,
        }
    }
}

impl AsRef<ChipInfo> for ChipInfo {
    fn as_ref(&self) -> &Self {
        self
    }
}
