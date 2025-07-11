use std::fmt::Debug;
use std::net::Ipv4Addr;
use std::sync::Arc;

use tokio::net::ToSocketAddrs;
use tokio::sync::Mutex;

use super::data::{Color, ControllerData, ModeData, RawString, SegmentData};
use crate::{OpenRgbError, OpenRgbResult, PluginData};

/// Default protocol version used by [OpenRGB] client.
pub const DEFAULT_PROTOCOL: u32 = 5;

/// Default address used by [OpenRGB::connect].
pub const DEFAULT_ADDR: (Ipv4Addr, u16) = (Ipv4Addr::LOCALHOST, 6742);

/// Device ID to use when no specific device is targeted.
const NO_DEVICE_ID: u32 = 0;

pub mod data;
mod packet;
mod stream;
mod serialize;
mod deserialize;

pub(crate) use {
    deserialize::*,
    serialize::*,
    packet::*,
    stream::*,
};

/// OpenRGB client.
///
/// This struct makes sure the protocol_id and the stream are in sync.
///
/// Todo: reintroduce a generic `stream` type to support sync/async streams.
#[derive(Clone)]
pub(crate) struct OpenRgbProtocol {
    protocol_id: u32,
    stream: Arc<Mutex<ProtocolStream>>,
}

impl OpenRgbProtocol {
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
    pub async fn connect_to(addr: impl ToSocketAddrs + Debug + Copy, protocol_version: u32) -> OpenRgbResult<Self> {
        tracing::debug!("Connecting to OpenRGB server at {:?}...", addr);
        let stream = ProtocolStream::connect(addr, protocol_version).await.map_err(|source| {
            OpenRgbError::ConnectionError {
                addr: format!("{addr:?}"),
                source,
            }
        })?;
        Self::new(stream).await
    }
}

impl OpenRgbProtocol {
    /// Build a new client from given stream.
    ///
    /// This constructor expects a connected, ready to use stream.
    pub async fn new(mut stream: ProtocolStream) -> OpenRgbResult<Self> {
        let req_protocol = stream
            .request(NO_DEVICE_ID, PacketId::RequestProtocolVersion, &DEFAULT_PROTOCOL)
            .await?;
        let protocol = DEFAULT_PROTOCOL.min(req_protocol);

        tracing::debug!(
            "Connected to OpenRGB server using protocol version {:?}",
            protocol
        );
        stream.set_protocol_version(protocol);

        Ok(Self {
            protocol_id: protocol,
            stream: Arc::new(Mutex::new(stream)),
        })
    }

    /// Get protocol version negotiated with server.
    ///
    /// This is the lowest between this client maximum supported version ([DEFAULT_PROTOCOL]) and server version.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#protocol-versions) for more information.
    pub fn get_protocol_version(&self) -> u32 {
        self.protocol_id
    }

    /// Helper method to write a packet to the server.
    async fn write_packet<T: SerToBuf>(&self, device_id: u32, packet_id: PacketId, data: &T) -> OpenRgbResult<()> {
        self.stream.lock().await.write_packet(device_id, packet_id, data).await
    }

    /// Helper method to write a packet to the server and parse the response.
    async fn request<I: SerToBuf, O: DeserFromBuf>(
        &self,
        device_id: u32,
        packet_id: PacketId,
        data: &I,
    ) -> OpenRgbResult<O> {
        self.stream.lock().await.request(device_id, packet_id, data).await
    }

    /// Set client name.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_set_client_name) for more information.
    pub async fn set_name(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self
            .write_packet(NO_DEVICE_ID, PacketId::SetClientName, &RawString(&name.into()))
            .await
    }

    /// Get number of controllers.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_controller_count) for more information.
    pub async fn get_controller_count(&self) -> OpenRgbResult<u32> {
        self
            .request(NO_DEVICE_ID, PacketId::RequestControllerCount, &())
            .await
    }

    /// Get controller data. This also caches the obtained controller.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_controller_data) for more information.
    pub async fn get_controller(&self, controller_id: u32) -> OpenRgbResult<ControllerData> {
        let mut c: ControllerData = self
            .request(
                controller_id,
                PacketId::RequestControllerData,
                &self.protocol_id,
            )
            .await?;
        c.id = controller_id;
        Ok(c)
    }

    /// Resize a controller zone.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_resizezone) for more information.
    pub async fn resize_zone(&self, controller_id: u32, zone_id: u32, new_size: u32) -> OpenRgbResult<()> {
        self
            .write_packet(controller_id, PacketId::RGBControllerResizeZone, &(zone_id, new_size))
            .await
    }

    /// Update a single LED.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_updatesingleled) for more information.
    pub async fn update_led(
        &self,
        controller_id: u32,
        led_id: i32,
        color: &Color,
    ) -> OpenRgbResult<()> {
        self
            .write_packet(
                controller_id,
                PacketId::RGBControllerUpdateSingleLed,
                &(led_id, color),
            )
            .await
    }

    /// Update LEDs.
    ///
    /// Structure:
    /// - `u32` - data size
    /// - `u16` - color counts
    /// - `[u32]` - colors
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_updateleds) for more information.
    pub async fn update_leds(&self, controller_id: u32, colors: &[Color]) -> OpenRgbResult<()> {
        let packet = OpenRgbPacket::new(colors);
        self
            .write_packet(
                controller_id,
                PacketId::RGBControllerUpdateLeds,
                &packet,
            )
            .await
    }

    /// Update a zone LEDs.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_updatezoneleds) for more information.
    pub async fn update_zone_leds(
        &self,
        controller_id: u32,
        zone_id: u32,
        colors: &[Color],
    ) -> OpenRgbResult<()> {
        let packet = OpenRgbPacket::new((zone_id, colors));
        self
            .write_packet(
                controller_id,
                PacketId::RGBControllerUpdateZoneLeds,
                &packet,
            )
            .await
    }

    /// Update a mode. This sets it to the current mode.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_updatemode) for more information.
    pub async fn update_mode(&self, controller_id: u32, mode: &ModeData) -> OpenRgbResult<()> {
        let packet = OpenRgbPacket::new((mode.index, mode));
        self
            .write_packet(
                controller_id,
                PacketId::RGBControllerUpdateMode,
                &packet,
            )
            .await
    }

    /// Set custom mode.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_setcustommode) for more information.
    pub async fn set_custom_mode(&self, controller_id: u32) -> OpenRgbResult<()> {
        unimplemented!("Not implemented as per recommendation from OpenRGB devs (https://discord.com/channels/699861463375937578/709998213310054490/1372954035581096158)");
        // self
        //     .write_packet(controller_id, PacketId::RGBControllerSetCustomMode, &())
        //     .await
    }

    /// Get profiles.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_profile_list) for more information.
    pub async fn get_profiles(&self) -> OpenRgbResult<Vec<String>> {
        self.check_protocol_version(2, "Get profiles")?;
        self
            .request::<_, (u32, Vec<String>)>(0, PacketId::RequestProfileList, &())
            .await
            .map(|(_size, profiles)| profiles)
    }

    /// Load a profile.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_load_profile) for more information.
    pub async fn load_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.check_protocol_version(2, "Load profiles")?;
        self
            .write_packet(0, PacketId::RequestLoadProfile, &RawString(&name.into()))
            .await
    }

    /// Save a profile.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_save_profile) for more information.
    pub async fn save_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.check_protocol_version(2, "Save profiles")?;
        self
            .write_packet(0, PacketId::RequestSaveProfile, &name.into())
            .await
    }

    /// Delete a profile.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_delete_profile) for more information.
    pub async fn delete_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.check_protocol_version(2, "Delete profiles")?;
        self
            .write_packet(0, PacketId::RequestDeleteProfile, &name.into())
            .await
    }

    /// Save a mode.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_savemode) for more information.
    pub async fn save_mode(&self, controller_id: u32, mode: &ModeData) -> OpenRgbResult<()> {
        self.check_protocol_version(3, "Save mode")?;
        let packet = OpenRgbPacket::new((mode.index, mode));
        self
            .write_packet(controller_id, PacketId::RGBControllerSaveMode, &packet)
            .await
    }

    /// Returns a a list of names of installed plugins.
    pub async fn get_plugins(&self) -> OpenRgbResult<Vec<PluginData>> {
        self.check_protocol_version(4, "Request Plugin List")?;
        // response contains length of data in the packet
        let resp: (u32, Vec<_>) = self.request(NO_DEVICE_ID, PacketId::RequestPluginList, &()).await?;
        Ok(resp.1)
    }

    /// Performs a plugin specific command. Depends on the plugin what this does.
    ///
    /// In this case, the `pkt_dev_idx` (`controller_id`) is used as the Plugin ID.
    pub async fn plugin_specific_command<I, O>(&self, plugin_id: u32, data: &I) -> OpenRgbResult<O> 
    where I: SerToBuf, O: DeserFromBuf
    {
        self.check_protocol_version(4, "Plugin Specific Command")?;
        self.request(plugin_id, PacketId::PluginSpecific, &data).await
    }

    pub async fn add_segment(
        &self,
        controller_id: u32,
        zone_id: u32,
        segment: &SegmentData,
    ) -> OpenRgbResult<()> {
        // segments are version 4, segments commands are version 5
        self.check_protocol_version(5, "Add Segment")?;
        let packet = OpenRgbPacket::new((zone_id, segment));
        self
            .write_packet(
                controller_id,
                PacketId::RGBControllerAddSegment,
                &packet,
            )
            .await
    }

    pub async fn clear_segments(&self, controller_id: u32) -> OpenRgbResult<()> {
        self.check_protocol_version(5, "Clear segment")?;
        self
            .write_packet(controller_id, PacketId::RgbControllerClearSegments, &())
            .await
    }

    /// Request a device rescan.
    pub async fn rescan_devices(&self) -> OpenRgbResult<()> {
        self.check_protocol_version(5, "Rescan devices")?;
        self.write_packet(NO_DEVICE_ID, PacketId::RequestDeviceRescan, &()).await
    }

    fn check_protocol_version(&self, min: u32, msg: &str) -> OpenRgbResult<()> {
        if self.protocol_id < min {
            return Err(OpenRgbError::UnsupportedOperation {
                operation: msg.to_owned(),
                current_protocol_version: self.protocol_id,
                min_protocol_version: min,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use tokio_test::io::Builder;
    use tracing_test::traced_test;

    use crate::{
        // protocol::tests::{setup, OpenRGBMockBuilder},
        Color, OpenRgbProtocol, OpenRgbResult, DEFAULT_ADDR, DEFAULT_PROTOCOL};

    // #[tokio::test]
    // async fn test_negotiate_protocol_version_3() -> Result<(), Box<dyn Error>> {
    //     setup()?;

    //     let client = Builder::new().negotiate_protocol(3).to_client().await?;

    //     assert_eq!(client.get_protocol_version(), 3);

    //     Ok(())
    // }

    // #[tokio::test]
    // async fn test_negotiate_protocol_version_2() -> Result<(), Box<dyn Error>> {
    //     setup()?;

    //     let client = Builder::new().negotiate_protocol(2).to_client().await?;

    //     assert_eq!(client.get_protocol_version(), 2);

    //     Ok(())
    // }

    // #[tokio::test]
    // async fn test_set_name() -> Result<(), Box<dyn Error>> {
    //     setup()?;

    //     let client = Builder::new()
    //         .negotiate_default_protocol()
    //         .write(b"ORGB") // magic
    //         .write(&0_u32.to_le_bytes()) // device id
    //         .write(&50_u32.to_le_bytes()) // packet id
    //         .write(&5_u32.to_le_bytes()) // data size
    //         .write(b"test\0") // name
    //         .to_client()
    //         .await?;

    //     client.set_name("test").await?;

    //     Ok(())
    // }

    // #[tokio::test]
    // #[ignore]
    // async fn test_get_controller_count() -> Result<(), Box<dyn Error>> {
    //     setup()?;

    //     let _client = Builder::new()
    //         .negotiate_default_protocol()
    //         .to_client()
    //         .await?;

    //     todo!("test not implemented")
    // }

    // #[tokio::test]
    // #[ignore]
    // async fn test_get_controller() -> Result<(), Box<dyn Error>> {
    //     setup()?;

    //     let _client = Builder::new()
    //         .negotiate_default_protocol()
    //         .to_client()
    //         .await?;

    //     todo!("test not implemented")
    // }

    // #[tokio::test]
    // #[ignore]
    // async fn test_update_zone_leds() -> Result<(), Box<dyn Error>> {
    //     setup()?;

    //     let _client = Builder::new()
    //         .negotiate_default_protocol()
    //         .to_client()
    //         .await?;

    //     todo!("test not implemented")
    // }

    // #[tokio::test]
    // #[ignore]
    // async fn test_resize_zone() -> Result<(), Box<dyn Error>> {
    //     setup()?;

    //     let _client = Builder::new()
    //         .negotiate_default_protocol()
    //         .to_client()
    //         .await?;

    //     todo!("test not implemented")
    // }

    // #[tokio::test]
    // #[ignore]
    // async fn test_save_profile() -> Result<(), Box<dyn Error>> {
    //     setup()?;

    //     let _client = Builder::new()
    //         .negotiate_default_protocol()
    //         .to_client()
    //         .await?;

    //     todo!("test not implemented")
    // }

    // #[tokio::test]
    // #[ignore]
    // async fn test_update_leds() -> Result<(), Box<dyn Error>> {
    //     setup()?;

    //     let _client = Builder::new()
    //         .negotiate_default_protocol()
    //         .to_client()
    //         .await?;

    //     todo!("test not implemented")
    // }

    // #[tokio::test]
    // #[ignore]
    // async fn test_delete_profile() -> Result<(), Box<dyn Error>> {
    //     setup()?;

    //     let _client = Builder::new()
    //         .negotiate_default_protocol()
    //         .to_client()
    //         .await?;

    //     todo!("test not implemented")
    // }

    // #[tokio::test]
    // #[ignore]
    // async fn test_load_profile() -> Result<(), Box<dyn Error>> {
    //     setup()?;

    //     let _client = Builder::new()
    //         .negotiate_default_protocol()
    //         .to_client()
    //         .await?;

    //     todo!("test not implemented")
    // }

    // #[tokio::test]
    // #[ignore]
    // async fn test_update_led() -> Result<(), Box<dyn Error>> {
    //     setup()?;

    //     let _client = Builder::new()
    //         .negotiate_default_protocol()
    //         .to_client()
    //         .await?;

    //     todo!("test not implemented")
    // }

    // #[tokio::test]
    // #[ignore]
    // async fn test_get_profiles() -> Result<(), Box<dyn Error>> {
    //     setup()?;

    //     let _client = Builder::new()
    //         .negotiate_default_protocol()
    //         .to_client()
    //         .await?;

    //     todo!("test not implemented")
    // }

    // #[tokio::test]
    // #[ignore]
    // async fn test_update_mode() -> Result<(), Box<dyn Error>> {
    //     setup()?;

    //     let _client = Builder::new()
    //         .negotiate_default_protocol()
    //         .to_client()
    //         .await?;

    //     todo!("test not implemented")
    // }

    // #[tokio::test]
    // #[ignore]
    // async fn test_set_custom_mode() -> Result<(), Box<dyn Error>> {
    //     setup()?;

    //     let _client = Builder::new()
    //         .negotiate_default_protocol()
    //         .to_client()
    //         .await?;

    //     todo!("test not implemented")
    // }

    // #[tokio::test]
    // #[ignore]
    // async fn test_save_mode() -> Result<(), Box<dyn Error>> {
    //     setup()?;

    //     let _client = Builder::new()
    //         .negotiate_default_protocol()
    //         .to_client()
    //         .await?;

    //     todo!("test not implemented")
    // }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_connect_real() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_update_led_real() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        client.update_led(5, 1, &Color::new(255, 0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_update_leds_real() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        client.update_leds(1, &[Color::new(255, 0, 0); 20]).await?;
        // client.update_led(4, 0, &Color::new(255, 0, 0)).await?;
        Ok(())
    }
}
