use crate::core::{interface::IfMngr, module::Module};
use crate::create_imc_interface;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::net::IpAddr;

create_imc_interface! {
    pub interface NetCtrlIf {
        fn get_ip() -> Option<IpAddr>;
    }
}

pub struct NetCtrl {}

impl NetCtrl {
    pub async fn build(_if_mngr: IfMngr) -> Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl Module for NetCtrl {
    async fn run(&mut self, if_mngr: IfMngr) -> Result<()> {
        let interface = NetCtrlIfBackend::builder();
        let interface = interface.on_get_ip(move || async move {
            let iface_name = "wlan0";
            let addrs = if_addrs::get_if_addrs().ok()?;
            addrs
                .into_iter()
                .find(|a| {
                    a.name == iface_name && a.addr.ip().is_ipv4() && !a.addr.ip().is_loopback()
                })
                .map(|a| a.addr.ip())
        });

        let mut interface = interface
            .build()
            .context("Failed to build net controller interface")?;

        let interface_front = interface.get_if();
        if_mngr
            .reg("net_ctrl", move || interface_front.clone())
            .await
            .context("Failed to reg net controller interface")?;

        loop {
            let event = interface.poll_event().await;
            interface.handle_event(event).await;
        }
    }
}
