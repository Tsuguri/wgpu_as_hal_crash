use ash::vk;
use wgpu::{
    hal::{MemoryFlags, TextureUses},
    TextureDimension,
};

fn find_memory_type(
    requirements: vk::MemoryRequirements,
    mem_properties: vk::PhysicalDeviceMemoryProperties,
    required_properties: vk::MemoryPropertyFlags,
) -> u32 {
    for i in 0..mem_properties.memory_type_count {
        if requirements.memory_type_bits & (1 << i) != 0
            && mem_properties.memory_types[i as usize]
                .property_flags
                .contains(required_properties)
        {
            return i;
        }
    }
    // panic, we can't allocate memory we need
    return 0;
}

fn create_image(
    device: &ash::Device,
    device_mem_properties: vk::PhysicalDeviceMemoryProperties,
    wanted_mem_properties: vk::MemoryPropertyFlags,
    width: u32,
    height: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
) -> (vk::Image, vk::DeviceMemory) {
    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE) // afaik this is about concurent access, not
        // sharing between devices/processes
        .samples(vk::SampleCountFlags::TYPE_1)
        .flags(vk::ImageCreateFlags::empty());

    let image = unsafe { device.create_image(&image_info, None).unwrap() };

    let memory_requirements = unsafe { device.get_image_memory_requirements(image) };
    let memory_type_index = find_memory_type(
        memory_requirements,
        device_mem_properties,
        wanted_mem_properties,
    );

    let allocation_info = vk::MemoryAllocateInfo::default()
        .allocation_size(memory_requirements.size)
        .memory_type_index(memory_type_index);

    let memory = unsafe {
        let mem = device.allocate_memory(&allocation_info, None).unwrap();
        device.bind_image_memory(image, mem, 0).unwrap();
        mem
    };

    (image, memory)
}

pub fn manually_create_texture(
    instance: &wgpu::Instance,
    device: &wgpu::Device,
    size: wgpu::Extent3d,
) -> wgpu::Texture {
    let wgpu_format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let vk_format = vk::Format::R8G8B8A8_UNORM;
    let instance = unsafe {
        instance
            .as_hal::<wgpu::hal::api::Vulkan>()
            .unwrap()
            .shared_instance()
            .raw_instance()
    };
    let (image, memory) = unsafe {
        device
            .as_hal::<wgpu::hal::api::Vulkan, _, _>(|device| {
                let device = device.expect("lol");
                let physical_device = device.raw_physical_device();
                let device = device.raw_device();

                let mem_properties =
                    instance.get_physical_device_memory_properties(physical_device);

                let flags = {
                    use vk::ImageUsageFlags as fl;
                    fl::COLOR_ATTACHMENT | fl::SAMPLED | fl::TRANSFER_DST
                };
                let (image, memory) = create_image(
                    device,
                    mem_properties,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    size.width,
                    size.height,
                    vk_format,
                    vk::ImageTiling::OPTIMAL,
                    flags,
                );
                (image, memory)
            })
            .unwrap()
    };
    let image_texture = unsafe {
        device.create_texture_from_hal::<wgpu::hal::api::Vulkan>(
            device
                .as_hal::<wgpu::hal::api::Vulkan, _, _>(|_device| {
                    use wgpu::hal::vulkan::Device as VulkanDevice;

                    let desc = wgpu::hal::TextureDescriptor {
                        dimension: TextureDimension::D2,
                        label: Some("vulkan allocated texture"),
                        format: wgpu_format,
                        sample_count: 1,
                        mip_level_count: 1,
                        size,
                        usage: TextureUses::COLOR_TARGET
                            | TextureUses::COPY_DST
                            | TextureUses::COPY_SRC
                            | TextureUses::RESOURCE,
                        memory_flags: MemoryFlags::empty(), // no idea what it is tbh
                        view_formats: vec![],
                    };

                    VulkanDevice::texture_from_raw(image, &desc, None)
                })
                .unwrap(),
            &wgpu::TextureDescriptor {
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu_format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                label: Some("vulkan allocated texture"),
                view_formats: &[],
            },
        )
    };
    image_texture
}
