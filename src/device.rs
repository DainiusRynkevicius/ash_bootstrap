use crate::errors::{DeviceError, QueueError};
use crate::feature_chain::GenericFeatureNode;
use crate::physical_device::PhysicalDevice;
use crate::utils;
use ash::vk;
use ash::vk::{AllocationCallbacks, QueueFlags};
use std::ffi::CString;

pub struct DeviceBuilder<'a> {
    physical_device: PhysicalDevice<'a>,
    device_info: DeviceInfo<'a>,
}

impl<'a> DeviceBuilder<'a> {
    pub fn new(physical_device: PhysicalDevice<'a>) -> Self {
        Self {
            physical_device,
            device_info: Default::default(),
        }
    }
    pub fn build(self) -> Result<Device<'a>, DeviceError> {
        let mut queue_descriptions = self.device_info.queue_descriptions;
        if queue_descriptions.is_empty() {
            for i in 0..self.physical_device.queue_families.len() {
                queue_descriptions.push(CustomQueueDescription {
                    index: i as u32,
                    priorities: vec![1.0],
                })
            }
        }

        let queue_create_infos = queue_descriptions
            .iter()
            .map(|desc| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(desc.index)
                    .queue_priorities(&desc.priorities)
            })
            .collect::<Vec<_>>();

        let mut extensions_to_enable = self
            .physical_device
            .extensions_to_enable
            .iter()
            .map(|name| {
                CString::new(name.clone()).unwrap_or({
                    #[cfg(feature = "logger")]
                    {
                        log::warn!("Malformed extension name: {}", name);
                    }
                    #[cfg(not(feature = "log"))]
                    {
                        println!("Malformed extension name: {}", name);
                    }
                    Default::default()
                })
            })
            .collect::<Vec<_>>();

        if self.physical_device.surface.is_some()
            || self.physical_device.defer_surface_initialization
        {
            extensions_to_enable.push(vk::KHR_SWAPCHAIN_NAME.to_owned());
        }

        let mut final_p_next_chain = self.physical_device.feature_chain.clone();
        let mut device_create_info = vk::DeviceCreateInfo::default();

        let user_defined_phys_dev_features_2 = self
            .device_info
            .p_next_chain
            .iter()
            .any(|s| s.structure_type == vk::StructureType::PHYSICAL_DEVICE_FEATURES_2);

        if user_defined_phys_dev_features_2 && !self.physical_device.feature_chain.nodes.is_empty()
        {
            return Err(DeviceError::VkPhysicalDeviceFeatures2InPNextChainWhileUsingAddRequiredExtensionFeatures);
        }

        let mut local_features =
            vk::PhysicalDeviceFeatures2::default().features(self.physical_device.features);

        if !(user_defined_phys_dev_features_2
            || self.physical_device.instance_version >= vk::API_VERSION_1_1
            || self.physical_device.properties2_ext_enabled)
        {
            device_create_info =
                device_create_info.enabled_features(&self.physical_device.features);
        }

        self.device_info.p_next_chain.into_iter().for_each(|node| {
            final_p_next_chain.add_node(node);
        });

        let extensions_to_enable_ptr = extensions_to_enable
            .iter()
            .map(|x| x.as_ptr())
            .collect::<Vec<_>>();
        unsafe { final_p_next_chain.chain_up_physical_device(&mut local_features) };
        device_create_info = device_create_info
            .push_next(&mut local_features)
            .queue_create_infos(queue_create_infos.as_slice())
            .enabled_extension_names(extensions_to_enable_ptr.as_slice());

        let device = if let Ok(vk_device) = unsafe {
            self.physical_device.instance.create_device(
                self.physical_device.physical_device,
                &device_create_info,
                self.device_info.allocation_callbacks.as_ref(),
            )
        } {
            vk_device
        } else {
            return Err(DeviceError::FailedToCreateDevice);
        };

        Ok(Device {
            device,
            physical_device: self.physical_device.physical_device,
            surface: self.physical_device.surface,
            queue_families: self.physical_device.queue_families,
            allocation_callbacks: self.device_info.allocation_callbacks,
            instance_version: self.physical_device.instance_version,
            instance: self.physical_device.instance,
            entry: self.physical_device.entry.clone(),
        })
    }
    pub fn custom_queue_setup(mut self, queue_descriptions: Vec<CustomQueueDescription>) -> Self {
        self.device_info.queue_descriptions = queue_descriptions;
        self
    }

    pub fn add_allocation_callbacks(mut self, callbacks: ash::vk::AllocationCallbacks<'a>) -> Self {
        self.device_info.allocation_callbacks = Some(callbacks);
        self
    }
}

pub struct CustomQueueDescription {
    index: u32,
    priorities: Vec<f32>,
}

#[derive(Default)]
struct DeviceInfo<'a> {
    flags: vk::DeviceCreateFlags,
    p_next_chain: Vec<GenericFeatureNode<'a>>,
    queue_descriptions: Vec<CustomQueueDescription>,
    allocation_callbacks: Option<AllocationCallbacks<'a>>,
}

pub struct Device<'a> {
    pub device: ash::Device,
    pub physical_device: vk::PhysicalDevice,
    pub surface: Option<vk::SurfaceKHR>,

    pub queue_families: Vec<vk::QueueFamilyProperties>,
    pub allocation_callbacks: Option<vk::AllocationCallbacks<'a>>,

    pub instance_version: u32,

    pub instance: ash::Instance,
    pub entry: ash::Entry,
}

pub enum QueueType {
    Present,
    Graphics,
    Compute,
    Transfer,
}

impl<'a> Device<'a> {
    pub fn get_queue_index(&self, queue_type: QueueType) -> Result<u32, QueueError> {
        match queue_type {
            QueueType::Present => {
                if let Some(surface) = self.surface {
                    utils::get_present_queue_index(
                        &self.entry,
                        &self.instance,
                        self.physical_device,
                        surface,
                        self.queue_families.as_slice(),
                    )
                    .ok_or(QueueError::PresentUnavailable)
                } else {
                    Err(QueueError::PresentUnavailable)
                }
            }
            QueueType::Graphics => {
                utils::get_first_queue_index(self.queue_families.as_slice(), QueueFlags::GRAPHICS)
                    .ok_or(QueueError::GraphicsUnavailable)
            }
            QueueType::Compute => utils::get_separate_queue_index(
                self.queue_families.as_slice(),
                QueueFlags::COMPUTE,
                QueueFlags::TRANSFER,
            )
            .ok_or(QueueError::ComputeUnavailable),
            QueueType::Transfer => utils::get_separate_queue_index(
                self.queue_families.as_slice(),
                QueueFlags::TRANSFER,
                QueueFlags::COMPUTE,
            )
            .ok_or(QueueError::TransferUnavailable),
        }
    }
    pub fn get_dedicated_queue_index(&self, queue_type: QueueType) -> Result<u32, QueueError> {
        match queue_type {
            QueueType::Compute => utils::get_dedicated_queue_index(
                self.queue_families.as_slice(),
                QueueFlags::COMPUTE,
                QueueFlags::TRANSFER,
            )
            .ok_or(QueueError::ComputeUnavailable),
            QueueType::Transfer => utils::get_dedicated_queue_index(
                self.queue_families.as_slice(),
                QueueFlags::TRANSFER,
                QueueFlags::COMPUTE,
            )
            .ok_or(QueueError::TransferUnavailable),
            _ => Err(QueueError::InvalidQueueFamilyIndex),
        }
    }
    pub fn get_queue(&self, queue_type: QueueType) -> Result<vk::Queue, QueueError> {
        let index = self.get_queue_index(queue_type)?;
        Ok(unsafe { self.device.get_device_queue(index, 0) })
    }
    pub fn get_dedicated_queue(&self, queue_type: QueueType) -> Result<vk::Queue, QueueError> {
        let index = self.get_dedicated_queue_index(queue_type)?;
        Ok(unsafe { self.device.get_device_queue(index, 0) })
    }
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - No other operations are performed on `self.device` after this call.
    /// - All resources created using this device have been properly destroyed or released.
    /// - This function is only called once; double destruction leads to undefined behavior.
    pub unsafe fn destroy(self) {
        unsafe {
            self.device
                .destroy_device(self.allocation_callbacks.as_ref())
        };
    }
}
