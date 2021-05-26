use cpal::traits::{DeviceTrait, HostTrait};
use log::info;

pub mod sample_converter;

pub fn enumerate_devices() -> Result<(), anyhow::Error> {
    info!("Supported hosts:\n  {:?}", cpal::ALL_HOSTS);
    let available_hosts = cpal::available_hosts();
    info!("Available hosts:\n  {:?}", available_hosts);

    for host_id in available_hosts {
        info!("{}", host_id.name());
        let host = cpal::host_from_id(host_id)?;

        let default_in = host.default_input_device().map(|e| e.name().unwrap());
        let default_out = host.default_output_device().map(|e| e.name().unwrap());
        info!("  Default Input Device:\n    {:?}", default_in);
        info!("  Default Output Device:\n    {:?}", default_out);

        let devices = host.devices()?;
        info!("  Devices: ");
        for (device_index, device) in devices.enumerate() {
            info!("  {}. \"{}\"", device_index + 1, device.name()?);

            // Input configs
            if let Ok(conf) = device.default_input_config() {
                info!("    Default input stream config:\n      {:?}", conf);
            }
            let input_configs = match device.supported_input_configs() {
                Ok(f) => f.collect(),
                Err(e) => {
                    info!("    Error getting supported input configs: {:?}", e);
                    Vec::new()
                }
            };
            if !input_configs.is_empty() {
                info!("    All supported input stream configs:");
                for (config_index, config) in input_configs.into_iter().enumerate() {
                    info!(
                        "      {}.{}. {:?}",
                        device_index + 1,
                        config_index + 1,
                        config
                    );
                }
            }

            // Output configs
            if let Ok(conf) = device.default_output_config() {
                info!("    Default output stream config:\n      {:?}", conf);
            }
            let output_configs = match device.supported_output_configs() {
                Ok(f) => f.collect(),
                Err(e) => {
                    info!("    Error getting supported output configs: {:?}", e);
                    Vec::new()
                }
            };
            if !output_configs.is_empty() {
                info!("    All supported output stream configs:");
                for (config_index, config) in output_configs.into_iter().enumerate() {
                    info!(
                        "      {}.{}. {:?}",
                        device_index + 1,
                        config_index + 1,
                        config
                    );
                }
            }
        }
    }

    Ok(())
}
