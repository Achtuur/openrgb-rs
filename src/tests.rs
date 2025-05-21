use std::error::Error;
use std::sync::Once;

use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use tokio_test::io::{Builder, Mock};

use crate::protocol::{ReadableStream, OpenRgbStream, WritableStream};
use crate::{OpenRgbClient, OpenRgbError, DEFAULT_PROTOCOL};

impl ReadableStream for Mock {}

impl WritableStream for Mock {}

impl OpenRgbStream for Mock {}

static INIT_ONCE: Once = Once::new();

pub fn setup() -> Result<(), Box<dyn Error>> {
    INIT_ONCE.call_once(|| {
        CombinedLogger::init(vec![TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::default(),
            ColorChoice::Auto,
        )])
        .expect("failed initializing logger")
    });

    Ok(())
}

pub trait OpenRGBMockBuilder<S: OpenRgbStream> {
    async fn to_client(&mut self) -> Result<OpenRgbClient<S>, OpenRgbError>;
    fn negotiate_default_protocol(&mut self) -> &mut Self;
    fn negotiate_protocol(&mut self) -> &mut Self;
}

impl OpenRGBMockBuilder<Mock> for Builder {
    async fn to_client(&mut self) -> Result<OpenRgbClient<Mock>, OpenRgbError> {
        OpenRgbClient::new(self.build()).await
    }

    fn negotiate_default_protocol(&mut self) -> &mut Self {
        self.negotiate_protocol(DEFAULT_PROTOCOL)
    }

    fn negotiate_protocol(&mut self) -> &mut Self {
        self
            // request protocol version request
            .write(b"ORGB") // magic
            .write(&0_u32.to_le_bytes()) // device id
            .write(&40_u32.to_le_bytes()) // packet id
            .write(&4_u32.to_le_bytes()) // data size
            .write(&DEFAULT_PROTOCOL.to_le_bytes()) // protocol version
            // request protocol version response
            .read(b"ORGB") // magic
            .read(&0_u32.to_le_bytes()) // device id
            .read(&40_u32.to_le_bytes()) // packet id
            .read(&4_u32.to_le_bytes()) // data size
            .read(&protocol.to_le_bytes()) // protocol version
    }
}
