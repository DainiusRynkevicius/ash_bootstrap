use crate::device::{Device, QueueType};
use crate::errors::SwapchainError;
use crate::feature_chain::{GenericFeatureChain, GenericFeatureNode};
use crate::utils;
use ash::vk;
use std::ffi::c_void;
use std::ptr::null_mut;
use ash::vk::Handle;

pub struct Swapchain<'a> {
    device: ash::Device,
    instance: ash::Instance,
    pub swapchain: vk::SwapchainKHR,
    image_count: u32,
    image_format: vk::Format,
    color_space: vk::ColorSpaceKHR,
    image_usage_flags: vk::ImageUsageFlags,
    extent: vk::Extent2D,

    requested_min_image_count: u32,
    present_mode: vk::PresentModeKHR,
    instance_version: u32,
    allocation_callbacks: Option<&'a vk::AllocationCallbacks<'a>>,
}

impl Swapchain<'_> {
    pub fn get_images(&self) -> Result<Vec<vk::Image>, SwapchainError> {
        unsafe {
            let khr_device = ash::khr::swapchain::Device::new(&self.instance, &self.device);
            khr_device
                .get_swapchain_images(self.swapchain)
                .ok()
                .ok_or(SwapchainError::FailedToGetSwapchainImages)
        }
    }

    pub unsafe fn get_image_views(
        &self,
        p_next: Option<*mut c_void>,
    ) -> Result<Vec<vk::ImageView>, SwapchainError> {
        let swapchain_images = self.get_images()?;

        let contains_image_view_usage = if let Some(mut p_next) = p_next {
            let mut found = false;
            while !p_next.is_null() {
                unsafe {
                    let base = *(p_next as *const vk::BaseInStructure);
                    if base.s_type
                        == vk::StructureType::IMAGE_VIEW_CREATE_INFO
                    {
                        found = true;
                        break;
                    }else{
                       p_next = base.p_next as *mut c_void;
                    }
                }
            }
            found
        } else {
            false
        };

        let mut desired_flags = vk::ImageViewUsageCreateInfo {
            p_next: p_next.unwrap_or(null_mut()),
            usage: self.image_usage_flags,
            ..Default::default()
        };

        let views = swapchain_images
            .into_iter()
            .map(|image| {
                let create_info = vk::ImageViewCreateInfo {
                    p_next: if self.instance_version >= vk::API_VERSION_1_1
                        && !contains_image_view_usage
                    {
                        &mut desired_flags as *mut _ as *mut c_void
                    } else {
                        p_next.unwrap_or(null_mut())
                    },
                    ..Default::default()
                }
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(self.image_format)
                .components(
                    vk::ComponentMapping::default()
                        .r(vk::ComponentSwizzle::IDENTITY)
                        .g(vk::ComponentSwizzle::IDENTITY)
                        .b(vk::ComponentSwizzle::IDENTITY)
                        .a(vk::ComponentSwizzle::IDENTITY),
                )
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                );

                unsafe {
                    self.device
                        .create_image_view(&create_info, self.allocation_callbacks)
                        .map_err(|_| SwapchainError::FailedToCreateSwapchainImageViews)
                }
            })
            .collect::<Result<Vec<vk::ImageView>, SwapchainError>>()?;

        Ok(views)
    }
    pub fn destroy_image_views(&self, image_views: &[vk::ImageView]) {
        image_views.iter().for_each(|view| unsafe {
            self.device
                .destroy_image_view(*view, self.allocation_callbacks)
        });
    }

    /// # Safety
    ///
    /// This function is unsafe because it destroys the Vulkan swapchain without validating
    /// whether it is still in use or has dependent resources (e.g., image views or framebuffers).
    ///
    /// The caller must ensure:
    /// - All resources created from `self.swapchain` (e.g., image views, framebuffers) have been destroyed before calling this.
    /// - No commands are currently executing that use the swapchain.
    /// - The `self.device` and `self.instance` are still valid and not destroyed.
    /// - This function is only called once per swapchain; calling it multiple times results in undefined behavior.
    pub unsafe fn destroy(&self) {
        if !self.swapchain.is_null() {
            let khr_device = ash::khr::swapchain::Device::new(&self.instance, &self.device);
            unsafe { khr_device.destroy_swapchain(self.swapchain, self.allocation_callbacks) };
        }
    }

    pub fn image_format(&self) -> vk::Format {self.image_format}
    pub fn color_space(&self) -> vk::ColorSpaceKHR {self.color_space}
    pub fn extent(&self) -> vk::Extent2D {self.extent}
    pub fn present_mode(&self) -> vk::PresentModeKHR {self.present_mode}
    pub fn image_count(&self) -> u32 {self.image_count}
    pub fn image_usage_flags(&self) -> vk::ImageUsageFlags {self.image_usage_flags}
}

struct SwapchainInfo<'a> {
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    instance: ash::Instance,
    p_next_chain: GenericFeatureChain<'a>,
    create_flags: vk::SwapchainCreateFlagsKHR,
    surface: Option<vk::SurfaceKHR>,
    desired_formats: Vec<vk::SurfaceFormatKHR>,
    instance_version: u32,
    desired_width: u32,
    desired_height: u32,
    array_layer_count: u32,
    min_image_count: u32,
    required_min_image_count: u32,
    image_usage_flags: vk::ImageUsageFlags,
    graphics_queue_index: u32,
    present_queue_index: u32,
    pre_transform: vk::SurfaceTransformFlagsKHR,
    composite_alpha: vk::CompositeAlphaFlagsKHR,
    desired_present_modes: Vec<vk::PresentModeKHR>,
    clipped: bool,
    old_swapchain: Option<vk::SwapchainKHR>,
    allocation_callbacks: Option<&'a vk::AllocationCallbacks<'a>>,
}

impl SwapchainInfo<'_> {
    /// Returns an empty swapchain info
    fn new(
        device: ash::Device,
        physical_device: vk::PhysicalDevice,
        instance: ash::Instance,
    ) -> Self {
        Self {
            physical_device,
            device,
            instance,
            p_next_chain: Default::default(),
            create_flags: Default::default(),
            surface: None,
            desired_formats: vec![],
            instance_version: vk::API_VERSION_1_0,
            desired_width: 256,
            desired_height: 256,
            array_layer_count: 1,
            min_image_count: 0,
            required_min_image_count: 0,
            image_usage_flags: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            graphics_queue_index: 0,
            present_queue_index: 0,
            pre_transform: Default::default(),
            composite_alpha: if cfg!(target_os = "android") {
                vk::CompositeAlphaFlagsKHR::INHERIT
            } else {
                vk::CompositeAlphaFlagsKHR::OPAQUE
            },
            desired_present_modes: vec![],
            clipped: true,
            old_swapchain: None,
            allocation_callbacks: None,
        }
    }
}

pub struct SwapchainBuilder<'a> {
    info: SwapchainInfo<'a>,
    entry: ash::Entry,
}

impl<'a> SwapchainBuilder<'a> {
    pub fn new(device: &Device) -> Self {
        let info = SwapchainInfo {
            surface: device.surface,
            instance_version: device.instance_version,
            present_queue_index: device
                .get_queue_index(QueueType::Present)
                .expect("Invalid present queue index."),
            graphics_queue_index: device
                .get_queue_index(QueueType::Graphics)
                .expect("Invalid graphics queue index."),
            ..SwapchainInfo::new(
                device.device.clone(),
                device.physical_device,
                device.instance.clone(),
            )
        };
        Self { info, entry: device.entry.clone() }
    }

    pub fn build(self) -> Result<Swapchain<'a>, SwapchainError> {
        if self.info.surface.is_none() {
            return Err(SwapchainError::SurfaceHandleNotProvided);
        }

        let mut desired_formats = self.info.desired_formats;
        if desired_formats.is_empty() {
            Self::add_desired_formats(&mut desired_formats);
        }
        let mut desired_present_modes = self.info.desired_present_modes;
        if desired_present_modes.is_empty() {
            Self::add_desired_present_modes(&mut desired_present_modes);
        }

        let surface_support = utils::query_surface_support_details(
            &self.entry,
            self.info.instance.clone(),
            self.info.physical_device,
            self.info.surface.unwrap(),
        )
        .map_err(|_| SwapchainError::FailedToQuerySurfaceSupportDetails)?;

        let mut image_count = self.info.min_image_count;
        if self.info.required_min_image_count >= 1 {
            if self.info.required_min_image_count < surface_support.capabilities.min_image_count {
                return Err(SwapchainError::RequiredMinImageCountTooLow);
            }
            image_count = self.info.required_min_image_count;
        } else if self.info.min_image_count == 0 {
            image_count = surface_support.capabilities.min_image_count + 1;
        } else {
            image_count = self.info.min_image_count;
            if image_count < surface_support.capabilities.min_image_count {
                image_count = surface_support.capabilities.min_image_count;
            }
        }
        if surface_support.capabilities.max_image_count > 0
            && image_count > surface_support.capabilities.max_image_count
        {
            image_count = surface_support.capabilities.max_image_count;
        }

        let surface_format = utils::find_best_surface_format(
            surface_support.formats.as_slice(),
            desired_formats.as_slice(),
        );

        let extent = utils::find_extent(
            &surface_support.capabilities,
            self.info.desired_width,
            self.info.desired_height,
        );

        let image_array_layers =
            if surface_support.capabilities.max_image_array_layers < self.info.array_layer_count {
                surface_support.capabilities.max_image_array_layers
            } else if self.info.array_layer_count == 0 {
                1
            } else {
                self.info.array_layer_count
            };

        let queue_family_indices = &[
            self.info.graphics_queue_index,
            self.info.present_queue_index,
        ];

        let present_mode = utils::find_present_mode(
            surface_support.present_modes.as_slice(),
            desired_present_modes.as_slice(),
        );

        let is_unextended_present_mode = |present_mode: vk::PresentModeKHR| {
            (present_mode == vk::PresentModeKHR::IMMEDIATE)
                || (present_mode == vk::PresentModeKHR::FIFO)
                || (present_mode == vk::PresentModeKHR::MAILBOX)
                || (present_mode == vk::PresentModeKHR::FIFO_RELAXED)
        };

        if is_unextended_present_mode(present_mode)
            && (self.info.image_usage_flags & surface_support.capabilities.supported_usage_flags)
                != self.info.image_usage_flags
        {
            return Err(SwapchainError::RequiredUsageNotSupported);
        }

        let pre_transform = if self.info.pre_transform == vk::SurfaceTransformFlagsKHR::from_raw(0)
        {
            surface_support.capabilities.current_transform
        } else {
            self.info.pre_transform
        };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .flags(self.info.create_flags)
            .surface(self.info.surface.unwrap())
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(image_array_layers)
            .image_usage(self.info.image_usage_flags);
        let mut swapchain_create_info =
            if self.info.graphics_queue_index != self.info.present_queue_index {
                swapchain_create_info
                    .image_sharing_mode(vk::SharingMode::CONCURRENT)
                    .queue_family_indices(queue_family_indices.as_slice())
            } else {
                swapchain_create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            }
            .pre_transform(pre_transform)
            .composite_alpha(self.info.composite_alpha)
            .present_mode(present_mode)
            .clipped(self.info.clipped)
            .old_swapchain(self.info.old_swapchain.unwrap_or(vk::SwapchainKHR::null()));
        let mut chain = self.info.p_next_chain.clone();
        unsafe { chain.chain_up_swapchain(&mut swapchain_create_info) };

        let khr_device = ash::khr::swapchain::Device::new(&self.info.instance, &self.info.device);
        let vk_swapchain = unsafe {
            khr_device
                .create_swapchain(&swapchain_create_info, self.info.allocation_callbacks)
                .map_err(|_| SwapchainError::FailedToCreateSwapchain)?
        };

        let mut swapchain = Swapchain {
            device: self.info.device,
            instance: self.info.instance,
            swapchain: vk_swapchain,
            image_count: 0,
            image_format: surface_format.format,
            color_space: surface_format.color_space,
            image_usage_flags: self.info.image_usage_flags,
            extent,
            requested_min_image_count: image_count,
            present_mode,
            instance_version: self.info.instance_version,
            allocation_callbacks: self.info.allocation_callbacks,
        };

        let images = swapchain.get_images()?;
        swapchain.image_count = images.len() as u32;

        Ok(swapchain)
    }
    pub fn set_old_vk_swapchain(mut self, old_swapchain: vk::SwapchainKHR) -> Self {
        self.info.old_swapchain = Some(old_swapchain);
        self
    }
    pub fn set_old_swapchain(mut self, old_swapchain: &Swapchain) -> Self {
        self.info.old_swapchain = Some(old_swapchain.swapchain);
        self
    }

    pub fn set_desired_extent(mut self, width: u32, height: u32) -> Self {
        self.info.desired_width = width;
        self.info.desired_height = height;
        self
    }

    pub fn set_desired_format(mut self, format: vk::SurfaceFormatKHR) -> Self {
        self.info.desired_formats.insert(0, format);
        self
    }
    pub fn add_fallback_format(mut self, format: vk::SurfaceFormatKHR) -> Self {
        self.info.desired_formats.push(format);
        self
    }
    pub fn use_default_format_selection(mut self) -> Self {
        self.info.desired_formats.clear();
        Self::add_desired_formats(&mut self.info.desired_formats);
        self
    }

    pub fn set_desired_present_mode(mut self, mode: vk::PresentModeKHR) -> Self {
        self.info.desired_present_modes.insert(0, mode);
        self
    }
    pub fn add_fallback_present_mode(mut self, mode: vk::PresentModeKHR) -> Self {
        self.info.desired_present_modes.push(mode);
        self
    }
    pub fn use_default_present_mode_selection(mut self) -> Self {
        self.info.desired_present_modes.clear();
        Self::add_desired_present_modes(&mut self.info.desired_present_modes);
        self
    }

    pub fn set_image_usage_flags(mut self, flags: vk::ImageUsageFlags) -> Self {
        self.info.image_usage_flags = flags;
        self
    }
    pub fn add_image_usage_flags(mut self, flags: vk::ImageUsageFlags) -> Self {
        self.info.image_usage_flags |= flags;
        self
    }
    pub fn use_default_image_usage_flags(mut self) -> Self {
        self.info.image_usage_flags = vk::ImageUsageFlags::COLOR_ATTACHMENT;
        self
    }
    pub fn set_image_array_layer_count(mut self, count: u32) -> Self {
        self.info.array_layer_count = count;
        self
    }

    pub fn set_desired_min_image_count(mut self, count: u32) -> Self {
        self.info.min_image_count = count;
        self
    }
    pub fn set_required_min_image_count(mut self, count: u32) -> Self {
        self.info.required_min_image_count = count;
        self
    }

    pub fn set_clipped(mut self, clipped: bool) -> Self {
        self.info.clipped = clipped;
        self
    }

    pub fn set_create_flags(mut self, flags: vk::SwapchainCreateFlagsKHR) -> Self {
        self.info.create_flags = flags;
        self
    }
    pub fn set_pre_transform_flags(mut self, flags: vk::SurfaceTransformFlagsKHR) -> Self {
        self.info.pre_transform = flags;
        self
    }
    pub fn set_composite_alpha_flags(mut self, flags: vk::CompositeAlphaFlagsKHR) -> Self {
        self.info.composite_alpha = flags;
        self
    }
    pub fn add_p_next(mut self, next: impl vk::ExtendsSwapchainCreateInfoKHR) -> Self {
        self.info
            .p_next_chain
            .add_node(GenericFeatureNode::from_swapchain_feature(next));
        self
    }
    pub fn set_allocation_callbacks(mut self, callbacks: &'a vk::AllocationCallbacks<'a>) -> Self {
        self.info.allocation_callbacks = Some(callbacks);
        self
    }

    fn add_desired_formats(formats: &mut Vec<vk::SurfaceFormatKHR>) {
        formats.push(
            vk::SurfaceFormatKHR::default()
                .format(vk::Format::B8G8R8A8_SRGB)
                .color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR),
        );
        formats.push(
            vk::SurfaceFormatKHR::default()
                .format(vk::Format::R8G8B8A8_SRGB)
                .color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR),
        );
    }
    fn add_desired_present_modes(modes: &mut Vec<vk::PresentModeKHR>) {
        modes.push(vk::PresentModeKHR::MAILBOX);
        modes.push(vk::PresentModeKHR::FIFO);
    }
}
