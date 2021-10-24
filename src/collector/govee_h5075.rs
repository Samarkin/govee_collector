use std::collections::HashMap;
use std::convert::TryInto;

#[derive(Debug, Copy, Clone)]
pub struct DeviceData {
    temperature: i32,
    humidity: u16,
    battery: u8,
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum DeviceDataError {
    #[error("unsupported device")]
    UnsupportedDevice,
    #[error("invalid data")]
    InvalidData,
}

const H5075_UPDATE_UUID16: u16 = 0xEC88;

impl DeviceData {
    pub fn decode(manufacturer_data: &HashMap<u16, Vec<u8>>) -> Result<DeviceData, DeviceDataError> {
        let bytes: [u8; 6] = match manufacturer_data.get(&H5075_UPDATE_UUID16) {
            Some(data) => match data.as_slice().try_into() {
                Ok(arr) => arr,
                Err(_) => return Err(DeviceDataError::InvalidData),
            },
            None => return Err(DeviceDataError::UnsupportedDevice),
        };
        // for H5075, temperature/humidity data is transmitted as a 23-bit integer
        let raw_data = ((bytes[1] & 0x7f) as u32) * 0x10000 + (bytes[2] as u32) * 0x100 + (bytes[3] as u32);
        // temperature sign is 1 bit
        let temp_sign = if bytes[1] & 0x80 != 0 { -1 } else { 1 };
        // battery percentage is 8 bits
        let battery = bytes[4];
        // last 3 decimal digits of that 23 bit integer represent humidity (with 1 decimal place)
        let humidity = (raw_data % 1000) as u16;
        // first decimal digits - absolute temperature in ºC (with 1 decimal place)
        let temperature = temp_sign * (raw_data as i32) / 1000;
        Ok(DeviceData { temperature, humidity, battery })
    }

    pub fn temperature_in_c(&self) -> f32 {
        self.temperature as f32 / 10.0
    }

    pub fn temperature_in_f(&self) -> f32 {
        self.temperature as f32 * 0.18 + 32.0
    }

    pub fn humidity(&self) -> f32 {
        self.humidity as f32 / 10.0
    }

    pub fn battery(&self) -> u8 {
        self.battery
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_device_does_not_panic() {
        let data = HashMap::from([
            (0x1234, vec![0x10, 0x20, 0x30]),
            (0x2345, vec![0x10, 0x20, 0x30]),
        ]);
        assert_eq!(DeviceData::decode(&data).err() , Some(DeviceDataError::UnsupportedDevice));
    }

    #[test]
    fn test_invalid_data_does_not_panic() {
        let data = HashMap::from([
            (0x1234, vec![0x10, 0x20, 0x30]),
            (0x2345, vec![0x10, 0x20, 0x30]),
            (H5075_UPDATE_UUID16, vec![0x10, 0x20, 0x30]),
        ]);
        assert_eq!(DeviceData::decode(&data).err(), Some(DeviceDataError::InvalidData));
    }

    #[test]
    fn test_sample_indoor_data_parses_correctly() {
        let data = HashMap::from([
            (0x1234, vec![0x10, 0x20, 0x30]),
            (0x2345, vec![0x10, 0x20, 0x30]),
            (H5075_UPDATE_UUID16, vec![0x00, 0x03, 0x84, 0x7a, 0x39, 0x00]),
        ]);
        let actual = DeviceData::decode(&data).expect("decode failed");
        assert_eq!(actual.temperature_in_c(), 23.0);
        assert_eq!(actual.temperature_in_f(), 73.4);
        assert_eq!(actual.humidity(), 52.2);
        assert_eq!(actual.battery(), 57);
    }

    #[test]
    fn test_sample_outdoor_data_parses_correctly() {
        let data = HashMap::from([
            (0x1234, vec![0x10, 0x20, 0x30]),
            (0x2345, vec![0x10, 0x20, 0x30]),
            (H5075_UPDATE_UUID16, vec![0x00, 0x02, 0xB1, 0xFE, 0x34, 0x00]),
        ]);
        let actual = DeviceData::decode(&data).expect("decode failed");
        assert_eq!(actual.temperature_in_c(), 17.6);
        assert_eq!(actual.temperature_in_f(), 63.68);
        assert_eq!(actual.humidity(), 63.8);
        assert_eq!(actual.battery(), 52);
    }

    #[test]
    fn test_sample_data_zero_c_parses_correctly() {
        let data = HashMap::from([
            (0x1234, vec![0x10, 0x20, 0x30]),
            (0x2345, vec![0x10, 0x20, 0x30]),
            (H5075_UPDATE_UUID16, vec![0x00, 0x00, 0x01, 0x9C, 0x64, 0x00]),
        ]);
        let actual = DeviceData::decode(&data).expect("decode failed");
        assert_eq!(actual.temperature_in_c(), 0.0);
        assert_eq!(actual.temperature_in_f(), 32.0);
        assert_eq!(actual.humidity(), 41.2);
        assert_eq!(actual.battery(), 100);
    }

    #[test]
    fn test_sample_data_below_zero_c_parses_correctly() {
        let data = HashMap::from([
            (0x1234, vec![0x10, 0x20, 0x30]),
            (0x2345, vec![0x10, 0x20, 0x30]),
            (H5075_UPDATE_UUID16, vec![0x00, 0x80, 0xBD, 0x9A, 0x64, 0x00]),
        ]);
        let actual = DeviceData::decode(&data).expect("decode failed");
        assert_eq!(actual.temperature_in_c(), -4.8);
        assert_eq!(actual.temperature_in_f(), 23.36);
        assert_eq!(actual.humidity(), 53.8);
        assert_eq!(actual.battery(), 100);
    }
}
