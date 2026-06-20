//! Full windowed bootstrap: instance -> surface -> physical device -> device -> swapchain.
//!
//! Uses `winit` for the window and `ash-window` to create the Vulkan surface. The example
//! performs the full setup, prints the resulting swapchain details, tears everything down
//! and exits — it does not render a frame.
//!
//! ```sh
//! cargo run --example swapchain
//! ```

use ash_bootstrap::device::QueueType;
use ash_bootstrap::utils;
use ash_bootstrap::{DeviceBuilder, InstanceBuilder, PhysicalDeviceSelector, SwapchainBuilder};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
    window: Option<Window>,
    done: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.done {
            return;
        }

        let window = event_loop
            .create_window(
                Window::default_attributes().with_title("ash_bootstrap swapchain example"),
            )
            .expect("failed to create window");

        if let Err(e) = bootstrap(&window) {
            eprintln!("bootstrap failed: {e}");
        }

        self.window = Some(window);
        self.done = true;

        // This example only demonstrates setup, so exit once it is complete.
        event_loop.exit();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let WindowEvent::CloseRequested = event {
            event_loop.exit();
        }
    }
}

fn bootstrap(window: &Window) -> Result<(), Box<dyn std::error::Error>> {
    // You load and own the Vulkan entry point; clone it for surface creation.
    let entry = unsafe { ash::Entry::load()? };

    let instance = InstanceBuilder::new(entry.clone())
        .set_app_name("ash_bootstrap swapchain example")
        .require_api_version(1, 1, 0)
        .request_validation_layers(true)
        .use_default_debug_messenger()
        .build()?;

    // Create the surface from the winit window via ash-window.
    let surface = unsafe {
        ash_window::create_surface(
            &entry,
            &instance.instance,
            window.display_handle()?.as_raw(),
            window.window_handle()?.as_raw(),
            None,
        )?
    };

    // The selector is bound to a local because the selected `PhysicalDevice` borrows from it.
    let selector =
        PhysicalDeviceSelector::new_with_surface(&instance, Some(surface)).require_present(true);
    let physical_device = selector.select()?;
    println!("Selected GPU: {}", physical_device.name);

    let device = DeviceBuilder::new(physical_device).build()?;
    let _present_queue = device.get_queue(QueueType::Present)?;

    let size = window.inner_size();
    let swapchain = SwapchainBuilder::new(&device)
        .set_desired_extent(size.width, size.height)
        .build()?;

    println!(
        "Swapchain created: {} images, format {:?}, extent {}x{}",
        swapchain.image_count(),
        swapchain.image_format(),
        swapchain.extent().width,
        swapchain.extent().height,
    );

    // Tear down in reverse order of creation.
    unsafe {
        swapchain.destroy();
        utils::destroy_surface(&entry, &instance, surface);
        device.destroy();
        instance.destroy();
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}
