use std::fmt::Debug;
use std::net::Ipv4Addr;
use std::sync::Arc;

use log::debug;
use tokio::net::ToSocketAddrs;
use tokio::sync::Mutex;

use super::data::{Color, ControllerData, ModeData, RawString, SegmentData};
use crate::protocol::{OpenRgbStream, PacketId};
use crate::{OpenRgbError, OpenRgbResult};

use super::{ProtocolTcpStream, Writable, WritableStream};

/// Default protocol version used by [OpenRGB] client.
pub static DEFAULT_PROTOCOL: u32 = 4;

/// Default address used by [OpenRGB::connect].
pub static DEFAULT_ADDR: (Ipv4Addr, u16) = (Ipv4Addr::LOCALHOST, 6742);

pub(crate) const MAGIC: [u8; 4] = *b"ORGB";

/// OpenRGB client.
///
/// This struct makes sure the protocol_id and the stream are in sync.
pub struct OpenRgbProtocol<S: OpenRgbStream> {
    protocol_id: u32,
    stream: Arc<Mutex<S>>,
}

impl<S: OpenRgbStream> Clone for OpenRgbProtocol<S> {
    fn clone(&self) -> Self {
        Self {
            protocol_id: self.protocol_id,
            stream: self.stream.clone(),
        }
    }
}

impl OpenRgbProtocol<ProtocolTcpStream> {
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
    pub async fn connect_to(addr: impl ToSocketAddrs + Debug + Copy) -> OpenRgbResult<Self> {
        debug!("Connecting to OpenRGB server at {:?}...", addr);
        let stream = ProtocolTcpStream::connect(addr).await.map_err(|source| {
            OpenRgbError::ConnectionError {
                addr: format!("{:?}", addr),
                source,
            }
        })?;
        Self::new(stream).await
    }

    /// Connects a new client using the same address as this client.
    pub(crate) async fn connect_clone(&self) -> OpenRgbResult<Self> {
        let addr = self.stream.lock().await.peer_addr()?;
        Self::connect_to(addr).await
    }
}

impl<S: OpenRgbStream> OpenRgbProtocol<S> {
    /// Build a new client from given stream.
    ///
    /// This constructor expects a connected, ready to use stream.
    pub async fn new(mut stream: S) -> OpenRgbResult<Self> {
        let req_protocol = stream
            .request(0, PacketId::RequestProtocolVersion, &DEFAULT_PROTOCOL)
            .await?;
        let protocol = DEFAULT_PROTOCOL.min(req_protocol);

        debug!(
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

    /// Set client name.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_set_client_name) for more information.
    pub async fn set_name(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.stream
            .lock()
            .await
            .write_packet(0, PacketId::SetClientName, &RawString(&name.into()))
            .await
    }

    /// Get number of controllers.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_controller_count) for more information.
    pub async fn get_controller_count(&self) -> OpenRgbResult<u32> {
        self.stream
            .lock()
            .await
            .request(0, PacketId::RequestControllerCount, &())
            .await
    }

    /// Get controller data. This also caches the obtained controller.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_controller_data) for more information.
    pub async fn get_controller(&mut self, controller_id: u32) -> OpenRgbResult<ControllerData> {
        let mut c: ControllerData = self
            .stream
            .lock()
            .await
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
    pub async fn resize_zone(&self, zone_id: i32, new_size: i32) -> OpenRgbResult<()> {
        self.stream
            .lock()
            .await
            .write_packet(0, PacketId::RGBControllerResizeZone, &(zone_id, new_size))
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
        self.stream
            .lock()
            .await
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
        // todo: optimise this maybe so that we don't allocate a Vec
        // this would mean that we write the data byte last
        let size = colors.size() + size_of::<u32>(); // count the data_size field too
        self.stream
            .lock()
            .await
            .write_packet(
                controller_id,
                PacketId::RGBControllerUpdateLeds,
                &(size, colors),
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
        // size of data packet needs to count the data_size field too
        let data_size = size_of::<u32>() + zone_id.size() + colors.size();
        self.stream
            .lock()
            .await
            .write_packet(
                controller_id,
                PacketId::RGBControllerUpdateZoneLeds,
                &(data_size, zone_id, colors),
            )
            .await
    }

    pub async fn add_segment(
        &self,
        controller_id: u32,
        zone_id: u32,
        segment: &SegmentData,
    ) -> OpenRgbResult<()> {
        self.check_protocol_version(5, "Add Segment")?;
        let data_size = size_of::<u32>() + zone_id.size() + segment.size();
        self.stream
            .lock()
            .await
            .write_packet(
                controller_id,
                PacketId::RGBControllerAddSegment,
                &(data_size, zone_id, segment),
            )
            .await
    }

    /// Get profiles.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_profile_list) for more information.
    pub async fn get_profiles(&self) -> OpenRgbResult<Vec<String>> {
        self.check_protocol_version(2, "Get profiles")?;
        self.stream
            .lock()
            .await
            .request::<_, (u32, Vec<String>)>(0, PacketId::RequestProfileList, &())
            .await
            .map(|(_size, profiles)| profiles)
    }

    /// Load a profile.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_load_profile) for more information.
    pub async fn load_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.check_protocol_version(2, "Load profiles")?;
        self.stream
            .lock()
            .await
            .write_packet(0, PacketId::RequestLoadProfile, &RawString(&name.into()))
            .await
    }

    /// Save a profile.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_save_profile) for more information.
    pub async fn save_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.check_protocol_version(2, "Save profiles")?;
        self.stream
            .lock()
            .await
            .write_packet(0, PacketId::RequestSaveProfile, &name.into())
            .await
    }

    /// Delete a profile.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_delete_profile) for more information.
    pub async fn delete_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.check_protocol_version(2, "Delete profiles")?;
        self.stream
            .lock()
            .await
            .write_packet(0, PacketId::RequestDeleteProfile, &name.into())
            .await
    }

    /// Set custom mode.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_setcustommode) for more information.
    pub async fn set_custom_mode(&self, controller_id: u32) -> OpenRgbResult<()> {
        self.stream
            .lock()
            .await
            .write_packet(controller_id, PacketId::RGBControllerSetCustomMode, &())
            .await
    }

    /// Update a mode. This sets it to the current mode.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_updatemode) for more information.
    pub async fn update_mode(&self, controller_id: u32, mode: &ModeData) -> OpenRgbResult<()> {
        // count the data_size field too
        let size = size_of::<u32>() + mode.index.size() + mode.size();
        self.stream
            .lock()
            .await
            .write_packet(
                controller_id,
                PacketId::RGBControllerUpdateMode,
                &(size, mode.index, mode),
            )
            .await
    }

    /// Save a mode.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_savemode) for more information.
    pub async fn save_mode(&self, controller_id: u32, mode: ModeData) -> OpenRgbResult<()> {
        self.check_protocol_version(3, "Save mode")?;
        self.stream
            .lock()
            .await
            .write_packet(controller_id, PacketId::RGBControllerSaveMode, &mode)
            .await
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

    use crate::protocol::tests::{OpenRGBMockBuilder, setup};

    #[tokio::test]
    async fn test_negotiate_protocol_version_3() -> Result<(), Box<dyn Error>> {
        setup()?;

        let client = Builder::new().negotiate_protocol(3).to_client().await?;

        assert_eq!(client.get_protocol_version(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_negotiate_protocol_version_2() -> Result<(), Box<dyn Error>> {
        setup()?;

        let client = Builder::new().negotiate_protocol(2).to_client().await?;

        assert_eq!(client.get_protocol_version(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_set_name() -> Result<(), Box<dyn Error>> {
        setup()?;

        let client = Builder::new()
            .negotiate_default_protocol()
            .write(b"ORGB") // magic
            .write(&0_u32.to_le_bytes()) // device id
            .write(&50_u32.to_le_bytes()) // packet id
            .write(&5_u32.to_le_bytes()) // data size
            .write(b"test\0") // name
            .to_client()
            .await?;

        client.set_name("test").await?;

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_controller_count() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client()
            .await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_controller() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client()
            .await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_zone_leds() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client()
            .await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_resize_zone() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client()
            .await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_save_profile() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client()
            .await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_leds() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client()
            .await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete_profile() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client()
            .await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_load_profile() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client()
            .await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_led() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client()
            .await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_profiles() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client()
            .await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_mode() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client()
            .await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_set_custom_mode() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client()
            .await?;

        todo!("test not implemented")
    }

    #[tokio::test]
    #[ignore]
    async fn test_save_mode() -> Result<(), Box<dyn Error>> {
        setup()?;

        let _client = Builder::new()
            .negotiate_default_protocol()
            .to_client()
            .await?;

        todo!("test not implemented")
    }
}
