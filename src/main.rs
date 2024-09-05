async fn run() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::DX12,
        ..Default::default()
    });

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                label: None,
            },
            None,
        )
        .await
        .unwrap();

    let is_vulkan = unsafe {
        device
            .as_hal::<wgpu::hal::api::Vulkan, _, _>(|device| device.is_some())
            .unwrap()
    };

    println!(
        "We are running on {}",
        if is_vulkan { "vulkan" } else { "not_vulkan" }
    );
}

fn main() {
    pollster::block_on(run());
}
