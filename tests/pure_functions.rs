//! Tests for the GPU-free selection / utility logic.
//!
//! These exercise the pure functions in `ash_bootstrap::utils` — no Vulkan driver or
//! device is required, so they run in any CI.

use ash::vk;
use ash_bootstrap::utils::{
    AsBool, SupportsFeatures, check_device_extension_support, find_best_surface_format,
    find_desired_surface_format, find_extent, find_present_mode, get_dedicated_queue_index,
    get_first_queue_index, get_separate_queue_index,
};

fn fmt(format: vk::Format, color_space: vk::ColorSpaceKHR) -> vk::SurfaceFormatKHR {
    vk::SurfaceFormatKHR::default()
        .format(format)
        .color_space(color_space)
}

fn family(flags: vk::QueueFlags) -> vk::QueueFamilyProperties {
    vk::QueueFamilyProperties::default()
        .queue_flags(flags)
        .queue_count(1)
}

#[test]
fn desired_surface_format_respects_priority() {
    let available = vec![
        fmt(
            vk::Format::B8G8R8A8_UNORM,
            vk::ColorSpaceKHR::SRGB_NONLINEAR,
        ),
        fmt(vk::Format::B8G8R8A8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR),
    ];
    // Both are available; the first entry in `desired` wins.
    let desired = vec![
        fmt(vk::Format::B8G8R8A8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR),
        fmt(
            vk::Format::B8G8R8A8_UNORM,
            vk::ColorSpaceKHR::SRGB_NONLINEAR,
        ),
    ];
    let chosen = find_desired_surface_format(&available, &desired).unwrap();
    assert_eq!(chosen.format, vk::Format::B8G8R8A8_SRGB);
}

#[test]
fn desired_surface_format_skips_unavailable_first_choice() {
    let available = vec![fmt(
        vk::Format::B8G8R8A8_SRGB,
        vk::ColorSpaceKHR::SRGB_NONLINEAR,
    )];
    let desired = vec![
        // Highest preference, but not available.
        fmt(vk::Format::R8G8B8A8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR),
        // Next preference, available.
        fmt(vk::Format::B8G8R8A8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR),
    ];
    let chosen = find_desired_surface_format(&available, &desired).unwrap();
    assert_eq!(chosen.format, vk::Format::B8G8R8A8_SRGB);
}

#[test]
fn desired_surface_format_errors_when_none_match() {
    let available = vec![fmt(
        vk::Format::B8G8R8A8_UNORM,
        vk::ColorSpaceKHR::SRGB_NONLINEAR,
    )];
    let desired = vec![fmt(
        vk::Format::R8G8B8A8_SRGB,
        vk::ColorSpaceKHR::SRGB_NONLINEAR,
    )];
    assert!(find_desired_surface_format(&available, &desired).is_err());
}

#[test]
fn best_surface_format_falls_back_to_first_available() {
    let available = vec![
        fmt(
            vk::Format::B8G8R8A8_UNORM,
            vk::ColorSpaceKHR::SRGB_NONLINEAR,
        ),
        fmt(
            vk::Format::R8G8B8A8_UNORM,
            vk::ColorSpaceKHR::SRGB_NONLINEAR,
        ),
    ];
    // None of the desired formats are available -> fall back to available[0].
    let desired = vec![fmt(
        vk::Format::R8G8B8A8_SRGB,
        vk::ColorSpaceKHR::SRGB_NONLINEAR,
    )];
    let chosen = find_best_surface_format(&available, &desired);
    assert_eq!(chosen.format, vk::Format::B8G8R8A8_UNORM);
}

#[test]
fn present_mode_prefers_desired_then_fifo() {
    let available = vec![vk::PresentModeKHR::FIFO, vk::PresentModeKHR::IMMEDIATE];

    let desired = vec![vk::PresentModeKHR::MAILBOX, vk::PresentModeKHR::IMMEDIATE];
    assert_eq!(
        find_present_mode(&available, &desired),
        vk::PresentModeKHR::IMMEDIATE
    );

    // Nothing desired is available -> guaranteed FIFO fallback.
    let desired_unavailable = vec![vk::PresentModeKHR::MAILBOX];
    assert_eq!(
        find_present_mode(&available, &desired_unavailable),
        vk::PresentModeKHR::FIFO
    );
}

#[test]
fn extent_uses_current_when_fixed() {
    let caps = vk::SurfaceCapabilitiesKHR::default()
        .current_extent(vk::Extent2D {
            width: 800,
            height: 600,
        })
        .min_image_extent(vk::Extent2D {
            width: 1,
            height: 1,
        })
        .max_image_extent(vk::Extent2D {
            width: 4096,
            height: 4096,
        });
    let e = find_extent(&caps, 1280, 720);
    assert_eq!((e.width, e.height), (800, 600));
}

#[test]
fn extent_clamps_desired_when_flexible() {
    let caps = vk::SurfaceCapabilitiesKHR::default()
        .current_extent(vk::Extent2D {
            width: u32::MAX,
            height: u32::MAX,
        })
        .min_image_extent(vk::Extent2D {
            width: 640,
            height: 480,
        })
        .max_image_extent(vk::Extent2D {
            width: 1920,
            height: 1080,
        });

    let within = find_extent(&caps, 1280, 720);
    assert_eq!((within.width, within.height), (1280, 720));

    let too_small = find_extent(&caps, 100, 100);
    assert_eq!((too_small.width, too_small.height), (640, 480));

    let too_big = find_extent(&caps, 5000, 5000);
    assert_eq!((too_big.width, too_big.height), (1920, 1080));
}

#[test]
fn queue_index_selection() {
    use vk::QueueFlags as Q;
    let families = vec![
        family(Q::GRAPHICS | Q::COMPUTE | Q::TRANSFER), // 0: universal
        family(Q::COMPUTE | Q::TRANSFER),               // 1: async compute (+transfer)
        family(Q::TRANSFER),                            // 2: dedicated transfer
        family(Q::COMPUTE),                             // 3: dedicated compute
    ];

    assert_eq!(get_first_queue_index(&families, Q::GRAPHICS), Some(0));

    // Dedicated = has desired, no graphics, and none of the undesired flags.
    assert_eq!(
        get_dedicated_queue_index(&families, Q::COMPUTE, Q::TRANSFER),
        Some(3)
    );
    assert_eq!(
        get_dedicated_queue_index(&families, Q::TRANSFER, Q::COMPUTE),
        Some(2)
    );

    // Separate = first non-graphics with desired, preferring one without the undesired flag.
    assert_eq!(
        get_separate_queue_index(&families, Q::COMPUTE, Q::TRANSFER),
        Some(3)
    );
}

#[test]
fn separate_queue_falls_back_when_only_overlapping_exists() {
    use vk::QueueFlags as Q;
    let families = vec![
        family(Q::GRAPHICS | Q::COMPUTE | Q::TRANSFER), // 0
        family(Q::COMPUTE | Q::TRANSFER),               // 1: overlapping, used as fallback
    ];
    assert_eq!(
        get_separate_queue_index(&families, Q::COMPUTE, Q::TRANSFER),
        Some(1)
    );
}

#[test]
fn extension_support_filters_present() {
    let available = vec!["VK_KHR_swapchain".to_string(), "VK_EXT_foo".to_string()];
    let required = vec!["VK_KHR_swapchain".to_string(), "VK_KHR_missing".to_string()];
    let supported = check_device_extension_support(&available, &required);
    assert_eq!(supported, vec!["VK_KHR_swapchain".to_string()]);
}

#[test]
fn supports_features_checks_subset() {
    let required = vk::PhysicalDeviceFeatures::default().geometry_shader(true);

    let with = vk::PhysicalDeviceFeatures::default()
        .geometry_shader(true)
        .tessellation_shader(true);
    let without = vk::PhysicalDeviceFeatures::default();

    // `required.supports(candidate)` is true iff the candidate has every feature required.
    assert!(required.supports(&with));
    assert!(!required.supports(&without));
}

#[test]
fn as_bool_round_trips() {
    assert!(!vk::FALSE.as_bool());
    assert!(vk::TRUE.as_bool());
}
