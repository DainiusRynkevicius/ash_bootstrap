//! Headless bootstrap: instance -> physical device -> logical device.
//!
//! No window or surface is involved, so this runs anywhere a Vulkan driver is present.
//!
//! ```sh
//! cargo run --example headless
//! ```

use ash_bootstrap::device::QueueType;
use ash_bootstrap::{DeviceBuilder, InstanceBuilder, PhysicalDeviceSelector};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // You load and own the Vulkan entry point.
    let entry = unsafe { ash::Entry::load()? };

    let instance = InstanceBuilder::new(entry)
        .set_app_name("ash_bootstrap headless example")
        .require_api_version(1, 1, 0)
        .request_validation_layers(true)
        .use_default_debug_messenger()
        .set_headless(true)
        .build()?;

    // No surface in a headless context. The selector is bound to a local because the
    // selected `PhysicalDevice` borrows from it.
    let physical_device = PhysicalDeviceSelector::new(&instance)
        .require_present(false)
        .select()?;

    println!("Selected GPU: {}", physical_device.name);

    let device = DeviceBuilder::new(physical_device).build()?;

    let graphics_queue = device.get_queue(QueueType::Graphics)?;
    println!("Graphics queue handle: {graphics_queue:?}");

    // Tear down in reverse order. `destroy` consumes the wrapper.
    unsafe {
        device.destroy();
        instance.destroy();
    }

    Ok(())
}
