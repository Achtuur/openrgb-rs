use crate::{DeserFromBuf, ReceivedMessage};

/// Data for OpenRGB plugins.
pub struct PluginData {
    /// Plugin name
    name: String,
    /// Description of plugin
    description: String,
    /// Plugin version
    version: String,
    /// Index of this plugin. This is its id in `plugin_specific` commands.
    index: u32,
    /// Plugin's protocol version.
    plugin_protocol_version: u32,
}

impl DeserFromBuf for PluginData {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> crate::OpenRgbResult<Self> {
        let name = buf.read_value()?;
        let description = buf.read_value()?;
        let version = buf.read_value()?;
        let index = buf.read_value()?;
        let plugin_protocol_version = buf.read_value()?;
        Ok(PluginData {
            name,
            description,
            version,
            index,
            plugin_protocol_version,
        })
    }
}