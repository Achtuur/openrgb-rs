//! Wrapper around the OpenRGB client to make it friendlier to use.
use std::{collections::HashMap, sync::Arc};

use tokio::{net::{TcpStream, ToSocketAddrs}, sync::Mutex};

use crate::{protocol::data::{Controller, DeviceType}, error::OpenRgbResult, protocol::{OpenRgbProtocol, OpenRgbStream, DEFAULT_ADDR}, OpenRgbError};

pub struct OpenRgbClientWrapper<S: OpenRgbStream> {
    // todo: make this not public
    pub client: OpenRgbProtocol<S>,
    controller_cache: HashMap<u32, Controller>,
}


impl OpenRgbClientWrapper<TcpStream> {
    /// Connect to default OpenRGB server.
    ///
    /// Use [OpenRGB::connect_to] to connect to a specific server.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use openrgb::OpenRGB;
    /// # use std::error::Error;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// let client = OpenRGB::connect().await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect() -> OpenRgbResult<Self> {
        Self::connect_to(DEFAULT_ADDR).await
    }

    /// Connect to OpenRGB server at given coordinates.
    ///
    /// Use [OpenRGB::connect] to connect to default server.
    ///
    /// # Arguments
    /// * `addr` - A socket address (eg: a `(host, port)` tuple)
    ///
    /// # Example
    /// ```no_run
    /// # use openrgb::OpenRGB;
    /// # use std::error::Error;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// let client = OpenRGB::connect_to(("localhost", 6742)).await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_to(addr: impl ToSocketAddrs + std::fmt::Debug + Copy) -> OpenRgbResult<Self> {
        let client = OpenRgbProtocol::connect_to(addr).await?;
        Ok(Self {
            client,
            controller_cache: HashMap::new(),
        })
    }
}

impl<S: OpenRgbStream> OpenRgbClientWrapper<S> {
    pub async fn get_all_controllers(&mut self) -> OpenRgbResult<impl Iterator<Item =&Controller>> {
        let count = self.client.get_controller_count().await?;
        for i in 0..count {
            if self.controller_cache.contains_key(&i) {
                continue;
            }
            let controller = self.client.get_controller(i).await?;
            self.controller_cache.insert(i, controller);
        }
        Ok(self.controller_cache.values())
    }

    pub async fn get_controller(&mut self, i: u32) -> OpenRgbResult<&Controller> {
        if self.controller_cache.contains_key(&i) {
            return Ok(self.controller_cache.get(&i).unwrap());
        }
        let controller = self.client.get_controller(i).await?;
        self.controller_cache.insert(i, controller);
        return Ok(self.controller_cache.get(&i).unwrap());
    }

    pub async fn get_controller_by_name(&mut self, name: impl AsRef<str>) -> Option<&Controller> {
        self.get_all_controllers()
        .await
        .ok()?
        .find(|controller| controller.name == name.as_ref())
    }

    pub async fn get_controller_by_device_type(&mut self, device_type: &DeviceType) -> Option<&Controller> {
        self.get_all_controllers()
        .await
        .ok()?
        .find(|controller| controller.device_type == *device_type)
    }
}

// delegation if it would exist
impl<S: OpenRgbStream> OpenRgbClientWrapper<S> {
    pub fn get_protocol_version(&mut self) -> u32 {
        self.client.get_protocol_version()
    }

    pub async fn set_name(&mut self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.client.set_name(name).await
    }

    pub async fn get_controller_count(&mut self) -> OpenRgbResult<u32> {
        self.client.get_controller_count().await
    }
}