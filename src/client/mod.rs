//! Wrapper around the OpenRGB client to make it friendlier to use.

mod controller;
mod zone;
mod command;
mod group;
mod segment;

pub use {controller::*, zone::*};

use tokio::net::ToSocketAddrs;

use crate::{
    client::group::ControllerGroup, data::DeviceType, error::OpenRgbResult, protocol::{data::ModeData, OpenRgbProtocol, DEFAULT_ADDR}, Color, OpenRgbError
};

pub struct OpenRgbClientWrapper {
    proto: OpenRgbProtocol,
}

impl OpenRgbClientWrapper {
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
        Ok(Self { proto: client })
    }
}

impl OpenRgbClientWrapper {
    pub async fn get_all_controllers(&self) -> OpenRgbResult<ControllerGroup> {
        let count = self.proto.get_controller_count().await? as usize;
        let mut controllers = Vec::with_capacity(count as usize);
        for id in 0..count {
            let controller = self.get_controller(id).await?;
            controllers.push(controller);
        }
        Ok(ControllerGroup::new(controllers))
    }

    pub async fn get_controllers_of_type(&self, device_type: DeviceType) -> OpenRgbResult<ControllerGroup> {
        let group = self.get_all_controllers().await?;
        group
        .split_per_type()
        .remove(&device_type)
        .ok_or(OpenRgbError::CommandError(format!(
            "No controllers of type {device_type:?} found"
        )))
    }

    pub async fn get_controller(&self, i: usize) -> OpenRgbResult<Controller> {
        let c_data = self.proto.get_controller(i as u32).await?;
        Ok(Controller::new(i, self.proto.clone(), c_data))
    }
}

// delegation if it would exist
impl OpenRgbClientWrapper {
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

    pub async fn save_mode(&self, controller_id: u32, mode: ModeData) -> OpenRgbResult<()> {
        self.proto.save_mode(controller_id, &mode).await
    }

    pub async fn update_zone_leds(
        &self,
        controller_id: u32,
        zone_id: u32,
        colors: &[Color],
    ) -> OpenRgbResult<()> {
        self.proto
            .update_zone_leds(controller_id, zone_id, colors)
            .await
    }
}
