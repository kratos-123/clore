

pub mod common;
#[cfg(test)]
mod test {
    use tracing::info;

    #[tokio::test]
    async fn nvidia_test() {
        use nvml_wrapper::Nvml;
        crate::common::setup();

        let nvml = Nvml::init().unwrap();
        info!("{:?}",nvml);
        // Get the first `Device` (GPU) in the system
        let device = nvml.device_by_index(0).unwrap();
        info!("{:?}",device);
        let brand = device.brand().unwrap(); // GeForce on my system
        info!("{:?}",brand);
        let fan_speed = device.fan_speed(0).unwrap(); // Currently 17% on my system
        info!("{:?}",fan_speed);
        let power_limit = device.enforced_power_limit().unwrap(); // 275k milliwatts on my system
        info!("{:?}",power_limit);
        let encoder_util = device.encoder_utilization().unwrap(); // Currently 0 on my system; Not encoding anything
        info!("{:?}",encoder_util);
        let memory_info = device.memory_info().unwrap(); // Currently 1.63/6.37 GB used on my system
        info!("{:?}",memory_info);

        // ... and there's a whole lot more you can do. Most everything in NVML is wrapped and ready to go
    }
}
