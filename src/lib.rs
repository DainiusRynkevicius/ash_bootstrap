//! `ash_bootstrap` — a Rust port of [vk-bootstrap] for the [`ash`] Vulkan wrapper.
//!
//! It removes the boilerplate from Vulkan initialization: instance creation,
//! physical-device selection, logical-device creation, and swapchain building.
//!
//!
//! # Example
//!
//! ```ignore
//! use ash_bootstrap::{InstanceBuilder, PhysicalDeviceSelector, DeviceBuilder, SwapchainBuilder};
//!
//! // You load the Vulkan entry point.
//! let entry = unsafe { ash::Entry::load()? };
//!
//! // 1. Instance
//! let instance = InstanceBuilder::new(entry.clone())
//!     .set_app_name("my app")
//!     .require_api_version(1, 3, 0)
//!     .request_validation_layers(true)
//!     .use_default_debug_messenger()
//!     .build()?;
//!
//! // 2. Surface — created by your windowing layer (e.g. `ash-window`).
//! let surface = create_surface(&entry, &instance.instance, &window)?;
//!
//! // 3. Physical device
//! let physical_device = PhysicalDeviceSelector::new_with_surface(&instance, Some(surface))
//!     .set_minimum_version(1, 3)
//!     .select()?;
//!
//! // 4. Logical device
//! let device = DeviceBuilder::new(physical_device).build()?;
//!
//! // 5. Swapchain
//! let swapchain = SwapchainBuilder::new(&device)
//!     .set_desired_extent(1280, 720)
//!     .build()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! [vk-bootstrap]: https://github.com/charles-lunarg/vk-bootstrap
//! [`ash`]: https://docs.rs/ash

pub mod device;
pub mod errors;
mod feature_chain;
pub mod instance;
pub mod physical_device;
pub mod swapchain;
pub mod system_info;
pub mod utils;

pub use device::{Device, DeviceBuilder};
pub use errors::*;
pub use instance::{Instance, InstanceBuilder};
pub use physical_device::{PhysicalDevice, PhysicalDeviceSelector};
pub use swapchain::{Swapchain, SwapchainBuilder};
pub use system_info::SystemInfo;
