use std::{error::Error, mem, usize};
use serde::{Deserialize, Serialize};
use serde_json;

pub struct DeviceConfig {
    temperature: f32,
    humidity: f32
}

#[derive(Clone, Default)]
pub struct HardDevice {
    id: u32,
    name: String,
    temperature: f32,
    humidity: f32,
    target_temperature: Option<f32>,
    target_humidity: Option<f32>
}

#[derive(Serialize, Deserialize)]
struct MongoStructure {
    temperature: f32,
    humidity: f32
}


pub trait Device {
    fn get_name(&self) -> String;
    fn as_json(&self) -> String;
    fn set_config(&mut self, config: &String);
    fn target_as_bytes(&self) -> Vec<u8>;
}

impl Device for HardDevice {

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn as_json(&self) -> String {
        format!("{{\"id\":{}, \"name\":\"{}\", \"temperature\":{}, \"humidity\":{}}}", self.id, self.name, self.temperature, self.humidity)
    }

    fn set_config(&mut self, config: &String) {
        let mongo_config: MongoStructure = serde_json::from_str(config).unwrap();
        self.target_humidity = Some(mongo_config.humidity);
        self.target_temperature = Some(mongo_config.temperature);
    }

    fn target_as_bytes(&self) -> Vec<u8> {
        let mut buf_vec = Vec::new();
        buf_vec.extend_from_slice(&self.id.to_le_bytes());
        buf_vec.extend_from_slice(&self.target_humidity.unwrap().to_le_bytes());
        buf_vec.extend_from_slice(&self.target_temperature.unwrap().to_le_bytes());
        buf_vec
    }
}

impl HardDevice {
    pub fn factory(buf: &[u8], len: usize) -> Result<Vec<HardDevice>, Box<dyn Error>> {
        println!("In factory");
        if len < 2 {
            return Err("Not enough data to parse header".into());
        }
        println!("Parse devices");
        let mut devices = Vec::new();
        let informer_id: u32 = buf[0] as u32;
        let devices_count = buf[1];
        println!("Received {} devices", devices_count);
        if len < 2 + devices_count as usize * 8 {
            return Err("Not enough data to parse devices".into());
        }

        for i in 0..devices_count {
            let device_data = &buf[2 + i as usize * 8..2 + (i + 1) as usize * 8];
            let temperature = f32::from_le_bytes(device_data[0..4].try_into().unwrap());
            let humidity = f32::from_le_bytes(device_data[4..8].try_into().unwrap());
            let device_id = (informer_id * 100 + (i as u32) + 1);
            devices.push(
                HardDevice {
                    id: device_id,
                    name: format!("Device {}", device_id),
                    temperature: temperature,
                    humidity: humidity,
                    ..Default::default()
                }
            );

        }
        Ok(devices)
    }

    pub fn get_temperature(&self) -> f32 {
        self.temperature
    }

    pub fn get_humidity(&self) -> f32 {
        self.humidity
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_id_str(&self) -> String {
        format!("{}", self.id)
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_json() {
        let humidity = 12.3;
        let id = 456;
        let name = String::from("789");
        let temperature = 10.11;
        let device = HardDevice{
            humidity,
            id,
            name,
            temperature,
            ..Default::default()
        };
        assert_eq!(device.as_json(), format!("{{\"id\":{id}, \"name\":\"789\", \"temperature\":{temperature}, \"humidity\":{humidity}}}"))
    }

    #[test]
    fn target_as_bytes() {
        let id = 456;
        let target_temerature = 12.3;
        let target_humidity = 45.6;
        let device = HardDevice{
            id: id,
            name: String::from("just-name"),
            temperature: 0.0,
            humidity: 0.0,
            target_humidity: Some(target_humidity),
            target_temperature: Some(target_temerature),
        };
        let targets = device.target_as_bytes();
        let id_bytes = targets[0..4].try_into().unwrap();
        let humidity_bytes = targets[4..8].try_into().unwrap();
        let temperature_bytes = targets[8..12].try_into().unwrap();
        assert_eq!(u32::from_le_bytes(id_bytes), id);
        assert_eq!(f32::from_le_bytes(temperature_bytes), target_temerature);
        assert_eq!(f32::from_le_bytes(humidity_bytes), target_humidity);
    }

    #[test]
    fn set_config_from_mongo() {
        let id = 123;
        let target_temerature = 12.3;
        let target_humidity = 45.6;

        let mut device = HardDevice{
            id: id,
            name: String::from("just-name"),
            temperature: 0.0,
            humidity: 0.0,
            ..Default::default()
        };
        
        let mongo_config: MongoStructure = MongoStructure{
            temperature: target_temerature,
            humidity: target_humidity
        };

        let config = serde_json::to_string(&mongo_config).unwrap();
        device.set_config(&config);
        let targets = device.target_as_bytes();
        let id_bytes = targets[0..4].try_into().unwrap();
        let humidity_bytes = targets[4..8].try_into().unwrap();
        let temperature_bytes = targets[8..12].try_into().unwrap();
        assert_eq!(u32::from_le_bytes(id_bytes), id);
        assert_eq!(f32::from_le_bytes(temperature_bytes), target_temerature);
        assert_eq!(f32::from_le_bytes(humidity_bytes), target_humidity);

    }

    #[test]
    fn test_single_init() {
        let devices = HardDevice::factory(&[1, 1, 0, 0, 0, 0, 0, 0, 0, 0], 10).unwrap();
        assert_eq!(devices.len(), 1);
    }
    #[test]
    fn test_single_init_big_buff() {
        let mut buf = [0; 1024];
        buf[1] = 1;
        let devices = HardDevice::factory(&buf, 1024).unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].get_id(), 1);
    }
    #[test]
    fn test_double_init_big_buff_2() {
        let mut buf = [0; 1024];
        buf[1] = 2;
        let devices = HardDevice::factory(&buf, 1024).unwrap();
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].get_id(), 1);
        assert_eq!(devices[1].get_id(), 2);
    }
    #[test]
    fn test_id_gen() {
        let mut buf = [0; 1024];
        buf[1] = 2;
        buf[0] = 12;
        let devices = HardDevice::factory(&buf, 1024).unwrap();
        assert_eq!(devices[0].get_id(), 1201);
        assert_eq!(devices[1].get_id(), 1202);
    }

    #[test]
    fn test_telemetry_calculation() {
        let mut buf = [0; 1024];
        buf[0] = 7; //id
        buf[1] = 2; //devices count

        //temperature
        buf[2] = f32::to_le_bytes(0.123)[0];
        buf[3] = f32::to_le_bytes(0.123)[1];
        buf[4] = f32::to_le_bytes(0.123)[2];
        buf[5] = f32::to_le_bytes(0.123)[3];

        //humidity
        buf[6] = f32::to_le_bytes(0.456)[0];
        buf[7] = f32::to_le_bytes(0.456)[1];
        buf[8] = f32::to_le_bytes(0.456)[2];
        buf[9] = f32::to_le_bytes(0.456)[3];

        let devices = HardDevice::factory(&buf, 1024).unwrap();
        assert_eq!(devices[0].get_temperature(), 0.123);
        assert_eq!(devices[0].get_humidity(), 0.456);
        assert_eq!(devices[1].get_temperature(), 0.0);
        assert_eq!(devices[1].get_humidity(), 0.0);
    }
    #[test]
    fn test_not_full_package() {
        let buf = [1,2,3,4];
        let devices = HardDevice::factory(&buf, buf.len());
        assert!(devices.is_err());
        assert_eq!(devices.err().unwrap().to_string(), "Not enough data to parse devices");
    }
    #[test]
    fn test_not_enough_data_for_headers() {
        let buf = [1];
        let devices = HardDevice::factory(&buf, buf.len());
        assert!(devices.is_err());
        assert_eq!(devices.err().unwrap().to_string(), "Not enough data to parse header");
    }
}

/*

fn process_income_data(data: &[u8], metrics: &Metrics, connection_string: &str, db_name: &str) -> Result<Settings, String> {
    println!("Received {} bytes", data.len());
    if data.len() < 1 {
        println!("No data received");
        return Err("No data received".to_owned());
    }
    let device_id: u8 = data[0];
    let devices_count = data[1];
    println!("Receieved data about {} devices", devices_count);
    for i in 0..devices_count {
        let device_data = &data[2 + i as usize * 8..2 + (i + 1) as usize * 8];
        let temperature = f32::from_le_bytes(device_data[0..4].try_into().unwrap());
        let humidity = f32::from_le_bytes(device_data[4..8].try_into().unwrap());
        metrics.temperature.with_label_values(&[&device_id.to_string()]).set(temperature as f64);
        metrics.humidity.with_label_values(&[&device_id.to_string()]).set(humidity as f64);
        info!("Device {}: temperature: {}, humidity: {}", i, temperature, humidity);
    }
    debug!("Receive data for device {}", device_id);
    debug!("Connection string: {}", connection_string);
    let settings = tokio::runtime::Runtime::new().unwrap().block_on(get_programm_for_device(device_id, connection_string, db_name));
    debug!("DB name: {}", db_name);

    return Ok(settings);
}
*/