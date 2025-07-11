use std::{collections::HashMap, ops::Index};

use crate::{client::command::UpdateLedCommandGroup, data::DeviceType, Color, Controller, OpenRgbError, OpenRgbResult};


pub trait ControllerIndex {
    fn index(self, group: &ControllerGroup) -> OpenRgbResult<&Controller>;
}

impl ControllerIndex for usize {
    fn index(self, group: &ControllerGroup) -> OpenRgbResult<&Controller> {
        group.controllers.get(self)
        .ok_or(OpenRgbError::CommandError(format!(
            "Controller with index {self} not found"
        )))
    }
}

impl ControllerIndex for &Controller {
    fn index(self, group: &ControllerGroup) -> OpenRgbResult<&Controller> {
        group.controllers.get(self.id())
        .ok_or(OpenRgbError::CommandError(format!(
            "Controller {} not found", self.name()
        )))
    }
}

impl ControllerIndex for Controller {
    fn index(self, group: &ControllerGroup) -> OpenRgbResult<&Controller> {
        (&self).index(group)
    }
}


#[derive(Debug)]
pub struct ControllerGroup {
    controllers: Vec<Controller>,
}

impl ControllerGroup {
    pub fn new(controllers: Vec<Controller>) -> Self {
        Self { controllers }
    }

    fn empty() -> Self {
        Self {
            controllers: Vec::new(),
        }
    }

    pub fn controllers(&self) -> &[Controller] {
        &self.controllers
    }

    pub fn split_per_type(self) -> HashMap<DeviceType, ControllerGroup> {
        self.controllers
        .into_iter()
        .fold(HashMap::new(), |mut acc, controller| {
            let entry = acc
            .entry(controller.data().device_type)
            .or_insert_with(ControllerGroup::empty);
            entry.controllers.push(controller);
            acc
        })
    }

    pub fn get_controller<I>(&self, idx: I) -> OpenRgbResult<&Controller>
    where I: ControllerIndex
    {
        idx.index(self)
    }

    pub fn cmd(&self) -> UpdateLedCommandGroup {
        UpdateLedCommandGroup::new(self)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Controller> {
        self.controllers.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = Controller> {
        self.controllers.into_iter()
    }
}

impl IntoIterator for ControllerGroup {
    type Item = Controller;
    type IntoIter = <Vec<Controller> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.controllers.into_iter()
    }
}

impl<'a> IntoIterator for &'a ControllerGroup {
    type Item = &'a Controller;
    type IntoIter = <&'a Vec<Controller> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.controllers.iter()
    }
}

#[cfg(test)]
mod tests {
use crate::OpenRgbClientWrapper;

use super::*;

    #[tokio::test]
    async fn test_group() -> OpenRgbResult<()> {
        let mut client = OpenRgbClientWrapper::connect().await?;
        let group = client.get_all_controllers().await?;
        let mut cmd = group.cmd();
        for i in 0..6 {
            cmd.add_update_controller_leds(i, vec![Color::new(255, 0, 255); 10])?;
        }
        cmd.execute().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_per_type() -> OpenRgbResult<()> {
        let mut client = OpenRgbClientWrapper::connect().await?;
        let group = client.get_all_controllers().await?;
        let split = group.split_per_type();
        println!("split: {0:?}", split);
        for (device_type, controllers) in split {
            println!("Device type: {device_type:?}");
            for controller in controllers.controllers() {
                println!("  Controller: {} ({})", controller.name(), controller.id());
            }
        }
        Ok(())
    }
}