use crate::core::{interface::IfMngr, module::Module};
use anyhow::{Result, bail};

pub mod display;
pub mod init;
pub mod keyboard;
pub mod power_ctrl;
pub mod i2c;
pub mod net_ctrl;
pub mod meteo_sensor;
pub mod system;
pub mod desktop;
pub mod apps;

macro_rules! create_modules_builder {
    ($($name: literal => $type: ty,)*) => {
        pub async fn build_module(name: impl AsRef<str>, if_mngr: IfMngr) -> Result<Box<dyn Module>> {
            match name.as_ref() {
                $($name => Ok(Box::new(<$type>::build(if_mngr).await?) as Box<dyn Module>),)*
                name => bail!("Unknown module: {name}")
            }
        }
    };
}

create_modules_builder! {
    "Display"     => display::Display,
    "Init"        => init::Init,
    "Keyboard"    => keyboard::Keyboard,
    "PowerCtrl"   => power_ctrl::PowerCtrl,
    "I2C"         => i2c::I2C,
    "NetCtrl"     => net_ctrl::NetCtrl,
    "MeteoSensor" => meteo_sensor::MeteoSensor,
    "System"      => system::System,
    "Desktop"     => desktop::Desktop,
    // apps
    "CommonTest"  => apps::common_test::CommonTest,
    "ExitTest"  => apps::exit_test::ExitTest,
    "Barometer"  => apps::barometer::Barometer,

}
