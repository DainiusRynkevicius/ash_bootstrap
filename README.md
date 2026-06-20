# ash_bootstrap

A Rust port of [vk-bootstrap](https://github.com/charles-lunarg/vk-bootstrap) for the
[`ash`](https://github.com/ash-rs/ash) Vulkan wrapper. It handles instance creation,
physical device selection, logical device creation and swapchain building.

The API maps almost 1:1 onto vk-bootstrap, so its
[usage guide](https://github.com/charles-lunarg/vk-bootstrap/blob/main/docs/getting_started.md)
is the best reference - the builder names and methods are the same, just in Rust style
(`set_app_name`, `require_api_version`, `select`, ...).

## Quick start

```rust,ignore
use ash_bootstrap::{InstanceBuilder, PhysicalDeviceSelector, DeviceBuilder, SwapchainBuilder};

// You load and own the Vulkan entry point.
let entry = unsafe { ash::Entry::load()? };

let instance = InstanceBuilder::new(entry.clone())
    .set_app_name("my app")
    .require_api_version(1, 3, 0)
    .request_validation_layers(true)
    .use_default_debug_messenger()
    .build()?;

// Create `surface` with your windowing layer (e.g. `ash-window`).

let selector = PhysicalDeviceSelector::new_with_surface(&instance, Some(surface));
let physical_device = selector.select()?;

let device = DeviceBuilder::new(physical_device).build()?;

let swapchain = SwapchainBuilder::new(&device)
    .set_desired_extent(1280, 720)
    .build()?;
```

## Examples

- `cargo run --example headless` — instance, physical device and logical device, no window.
- `cargo run --example swapchain` — full windowed setup with a surface and swapchain via
  `winit` + `ash-window`.

## Features

- `portability_enumeration` (default) — enables `VK_KHR_portability_enumeration`.
- `logger` — routes the default debug callback through the `log` crate instead of stdout.

## License

[MIT](LICENSE)