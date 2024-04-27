use crate::device::Device;

use crate::external::abstract_external::ExternalDatabase;
use std::{collections::HashMap, time};

struct CacheElement {
    pub val: String,
    pub last_update: time::SystemTime,
}

pub struct Datacache {
    external_channel: Box<dyn ExternalDatabase>,
    cache: HashMap<String, CacheElement>,
    experity_time: u64,
    capacity: usize,
}

impl Datacache {
    pub fn new(external_channel: Box<dyn ExternalDatabase>) -> Datacache {
        return Datacache {
            external_channel: external_channel,
            cache: HashMap::new(),
            experity_time: 60 * 60,
            capacity: 2000,
        };
    }

    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity;
    }

    pub fn set_experity_time(&mut self, experity_time: u64) {
        self.experity_time = experity_time;
    }

    fn has_capacity(&mut self) -> bool {
        return self.cache.len() < self.capacity;
    }

    fn cleanup(&mut self) {
        let expired_keys: Vec<String> = self
            .cache
            .iter()
            .filter(|(k, v)| v.last_update.elapsed().unwrap().as_secs() >= self.experity_time)
            .map(|(k, v)| k.clone())
            .collect();
        expired_keys.iter().for_each(|k| {
            self.cache.remove(k);
        })
    }

    pub async fn set_config(&mut self, device: &mut dyn Device) -> Result<(), std::io::Error> {
        let name = device.get_name();
        match self.cache.get(&name) {
            Some(device_raw_config) => {
                if device_raw_config.last_update.elapsed().unwrap().as_secs() < self.experity_time {
                    device.set_config(&device_raw_config.val);
                    return Ok(());
                }
            }
            None => {}
        }
        let config = self.external_channel.get_device_config(&name).await?;
        device.set_config(&config);
        if !self.has_capacity() {
            self.cleanup();
        }
        if self.has_capacity() {
            self.cache.insert(
                name,
                CacheElement {
                    val: config,
                    last_update: time::SystemTime::now(),
                },
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use async_std::task::block_on;
    use async_trait::async_trait;
    use futures::TryFutureExt;
    use std::{borrow::BorrowMut, collections::HashMap, sync::{Arc, Mutex}};

    use crate::{device::Device, external::abstract_external::ExternalDatabase};

    use super::Datacache;

    struct MockExternalDatabaseInstance {
        pub data: HashMap<String, String>,
        pub counter: u16,
    }

    struct MockExternalDatabase {
        instance: Mutex<Arc<MockExternalDatabaseInstance>>,
    }


    struct MockDevice {
        pub name: String,
        pub config: String
    }

    impl Device for MockDevice {
        fn get_name(&self) -> String {
            self.name.clone()
        }

        fn as_json(&self) -> String {
            String::new()
        }

        fn set_config(&mut self, config: &String) {
            self.config = config.clone();
        }
        
    }

    #[async_trait]
    impl ExternalDatabase for MockExternalDatabase {
        async fn get_device_config(&mut self, key: &String) -> Result<String, std::io::Error> {
            self.instance.lock().unwrap().counter+=1;
            match self.instance.lock().unwrap().data.get(key) {
                Some(data) => Ok(data.clone()),
                None => Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Can't find this key",
                )),
            }
        }
    }

    #[test]
    fn test_get_data() {
        let external_database_instance = Arc::new(MockExternalDatabaseInstance {
            data: HashMap::new(),
            counter: 0,
        });

        let external_database = MockExternalDatabase{instance: Mutex::new(external_database_instance.clone())};

        let cache = Datacache::new(Box::new(external_database));
        let mut device: Box<dyn Device> = Box::new(MockDevice {name: "Device1".to_owned(), config: "".to_owned()});
        external_database_instance.data.insert("Device1".to_owned(), "config1".to_owned());
        block_on(cache.set_config(&mut *device));
        assert_eq!(external_database_instance.counter, 1);
    }
}
