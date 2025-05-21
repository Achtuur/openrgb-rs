//! Wrapper around the OpenRGB client to make it friendlier to use.
use std::collections::HashMap;

use tokio::net::{TcpStream, ToSocketAddrs};

use crate::{
    error::OpenRgbResult,
    protocol::{
        DEFAULT_ADDR, OpenRgbProtocol, OpenRgbStream,
        data::{Controller, DeviceType, Mode},
    },
};

pub struct OpenRgbClientWrapper<S: OpenRgbStream> {
    // todo: make this not public
    pub proto: OpenRgbProtocol<S>,
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
    pub async fn connect_to(
        addr: impl ToSocketAddrs + std::fmt::Debug + Copy,
    ) -> OpenRgbResult<Self> {
        let client = OpenRgbProtocol::connect_to(addr).await?;
        Ok(Self {
            proto: client,
            controller_cache: HashMap::new(),
        })
    }
}

impl<S: OpenRgbStream> OpenRgbClientWrapper<S> {
    pub async fn get_all_controllers(
        &mut self,
    ) -> OpenRgbResult<impl Iterator<Item = &Controller>> {
        let count = self.proto.get_controller_count().await?;
        for i in 0..count {
            if self.controller_cache.contains_key(&i) {
                continue;
            }
            let controller = self.proto.get_controller(i).await?;
            self.controller_cache.insert(i, controller);
        }
        Ok(self.controller_cache.values())
    }

    pub async fn get_controller(&mut self, i: u32) -> OpenRgbResult<&Controller> {
        if self.controller_cache.contains_key(&i) {
            return Ok(self.controller_cache.get(&i).unwrap());
        }
        let controller = self.proto.get_controller(i).await?;
        self.controller_cache.insert(i, controller);
        Ok(self.controller_cache.get(&i).unwrap())
    }

    pub async fn get_controller_by_name(&mut self, name: impl AsRef<str>) -> Option<&Controller> {
        self.get_all_controllers()
            .await
            .ok()?
            .find(|controller| controller.name == name.as_ref())
    }

    pub async fn get_controller_by_device_type(
        &mut self,
        device_type: &DeviceType,
    ) -> Option<&Controller> {
        self.get_all_controllers()
            .await
            .ok()?
            .find(|controller| controller.device_type == *device_type)
    }
}

// delegation if it would exist
impl<S: OpenRgbStream> OpenRgbClientWrapper<S> {
    pub fn get_protocol_version(&mut self) -> u32 {
        self.proto.get_protocol_version()
    }

    pub async fn set_name(&mut self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.proto.set_name(name).await
    }

    pub async fn get_profiles(&self) -> OpenRgbResult<Vec<String>> {
        self.proto.get_profiles().await
    }

    pub async fn save_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.proto.save_profile(name).await
    }

    pub async fn load_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.proto.load_profile(name).await
    }

    pub async fn delete_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.proto.delete_profile(name).await
    }

    pub async fn get_controller_count(&mut self) -> OpenRgbResult<u32> {
        self.proto.get_controller_count().await
    }

    pub async fn save_mode(&self, controller_id: u32, mode: Mode) -> OpenRgbResult<()> {
        self.proto.save_mode(controller_id, mode).await
    }
}
