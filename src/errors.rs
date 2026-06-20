use thiserror::Error;

#[derive(Error, Debug)]
pub enum InstanceError {
    #[error("Vulkan is unavailable on this system.")]
    VulkanUnavailable,
    #[error("Vulkan version is unavailable on this system.")]
    VulkanVersionUnavailable,
    #[error("Vulkan version 1.1 is unavailable on this system.")]
    VulkanVersion1_1Unavailable,
    #[error("Vulkan version 1.2 is unavailable on this system.")]
    VulkanVersion1_2Unavailable,
    #[error("Vulkan version 1.3 is unavailable on this system.")]
    VulkanVersion1_3Unavailable,
    #[error("Vulkan version 1.4 is unavailable on this system.")]
    VulkanVersion1_4Unavailable,
    #[error("Failed to create Vulkan instance.")]
    FailedToCreateInstance,
    #[error("Failed to create Vulkan debug messenger.")]
    FailedToCreateDebugMessenger,
    #[error("Requested Vulkan layers are unavailable on this system.")]
    RequestedLayersUnavailable,
    #[error("Requested Vulkan instance extensions are unavailable on this system.")]
    RequestedExtensionsUnavailable,
    #[error("Requested Vulkan windowing extensions are unavailable on this system.")]
    WindowingExtensionsUnavailable,
    #[error("Provided string contains interior null byte.")]
    InvalidString(#[from] std::ffi::NulError),
    #[error("Failed to enumerate instance properties.")]
    FailedToEnumerate(#[from] ash::vk::Result)
}

#[derive(Error, Debug)]
pub enum PhysicalDeviceError {
    #[error("No Surface was provided")]
    NoSurfaceProvided,
    #[error("Failed to enumerate physical devices")]
    FailedEnumeratePhysicalDevices,
    #[error("No physical devices found")]
    NoPhysicalDevicesFound,
    #[error("No suitable device found")]
    NoSuitableDevice,
}

#[derive(Error, Debug)]
pub enum DeviceError{
    #[error("Failed to create device")]
    FailedToCreateDevice,
    //TODO: name should be better, original one sucks
    #[error("Usage of Add Required Extension Features while having DeviceFeatures2 in PNext chain")]
    VkPhysicalDeviceFeatures2InPNextChainWhileUsingAddRequiredExtensionFeatures,
}

#[derive(Error, Debug)]
pub enum QueueError{
    #[error("Present queue unavailable")]
    PresentUnavailable,
    #[error("Graphics queue unavailable")]
    GraphicsUnavailable,
    #[error("Compute queue unavailable")]
    ComputeUnavailable,
    #[error("Transfer queue unavailable")]
    TransferUnavailable,
    #[error("Queue index out of bounds")]
    QueueIndexOutOfBounds,
    #[error("Invalid queue family index")]
    InvalidQueueFamilyIndex
}

#[derive(Error, Debug)]
pub enum SwapchainError{
    #[error("Surface handle was not provided")]
    SurfaceHandleNotProvided,
    #[error("Failed to query surface support details")]
    FailedToQuerySurfaceSupportDetails,
    #[error("Failed to create swapchain")]
    FailedToCreateSwapchain,
    #[error("Failed to get swapchain images")]
    FailedToGetSwapchainImages,
    #[error("Failed to create swapchain image views")]
    FailedToCreateSwapchainImageViews,
    #[error("Required minimum image count is too low")]
    RequiredMinImageCountTooLow,
    #[error("Required usage is not supported")]
    RequiredUsageNotSupported
}

#[derive(Error, Debug)]
pub enum SurfaceSupportError{
    #[error("Provided surface handle is null")]
    SurfaceHandleNull,
    #[error("Failed to get surface capabilities")]
    FailedToGetSurfaceCapabilities,
    #[error("Failed to enumerate surface formats")]
    FailedToEnumerateSurfaceFormats,
    #[error("Failed to enumerate surface present modes")]
    FailedToEnumeratePresentModes,
    #[error("No suitable surface format found")]
    NoSuitableDesiredFormat
}