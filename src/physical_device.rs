use crate::errors::PhysicalDeviceError;
use crate::feature_chain::{GenericFeatureChain, GenericFeatureNode};
use crate::instance::Instance;
use crate::utils;
use crate::utils::SupportsFeatures;
use ash::{vk, Entry};
use std::cmp::PartialEq;
use std::ffi::CStr;

pub struct PhysicalDeviceSelector<'a> {
    criteria: SelectionCriteria<'a>,
    instance: InstanceInfo,
    entry: ash::Entry,
}

impl PhysicalDeviceSelector<'_> {
    pub fn new(instance: &Instance) -> Self {
        Self::new_with_surface(instance, None)
    }

    pub fn new_with_surface(instance: &Instance, surface: Option<vk::SurfaceKHR>) -> Self {
        Self {
            criteria: SelectionCriteria {
                require_present: !instance.headless,
                required_version: instance.api_version,
                ..Default::default()
            },
            instance: InstanceInfo {
                instance: instance.instance.clone(),
                surface,
                version: instance.instance_version,
                headless: instance.headless,
                properties2_ext_enabled: instance.properties2_ext_enabled,
            },
            entry: instance.entry.clone(),
        }
    }

    pub fn select(&self) -> Result<PhysicalDevice<'_>, PhysicalDeviceError> {
        let selected_devices = self.select_impl();

        match selected_devices {
            Ok(devices) => {
                if devices.is_empty() {
                    return Err(PhysicalDeviceError::NoSuitableDevice);
                }
                devices
                    .into_iter()
                    .next()
                    .ok_or(PhysicalDeviceError::NoSuitableDevice)
            }
            Err(err) => Err(err),
        }
    }
    pub fn select_devices(&self) -> Result<Vec<PhysicalDevice>, PhysicalDeviceError> {
        let selected_devices = self.select_impl();
        match selected_devices {
            Ok(devices) => {
                if devices.is_empty() {
                    return Err(PhysicalDeviceError::NoSuitableDevice);
                }
                Ok(devices)
            }
            Err(err) => Err(err),
        }
    }
    pub fn select_device_names(&self) -> Result<Vec<String>, PhysicalDeviceError> {
        let selected_devices = self.select_impl();

        match selected_devices {
            Ok(devices) => {
                if devices.is_empty() {
                    return Err(PhysicalDeviceError::NoSuitableDevice);
                }
                Ok(devices.into_iter().map(|p| p.name).collect())
            }
            Err(err) => Err(err),
        }
    }

    pub fn set_surface(mut self, surface: vk::SurfaceKHR) -> Self {
        self.instance.surface = Some(surface);
        self
    }
    pub fn set_name(mut self, name: impl Into<String>) -> Self {
        self.criteria.name = name.into();
        self
    }
    pub fn prefer_gpu_device_type(mut self, device_type: vk::PhysicalDeviceType) -> Self {
        self.criteria.preferred_type = device_type;
        self
    }
    pub fn allow_any_gpu_device_type(mut self, allow_any: bool) -> Self {
        self.criteria.allow_any_type = allow_any;
        self
    }
    pub fn require_present(mut self, require_present: bool) -> Self {
        self.criteria.require_present = require_present;
        self
    }

    pub fn require_dedicated_compute_queue(mut self) -> Self {
        self.criteria.require_dedicated_compute_queue = true;
        self
    }
    pub fn require_dedicated_transfer_queue(mut self) -> Self {
        self.criteria.require_dedicated_transfer_queue = true;
        self
    }

    pub fn require_separate_transfer_queue(mut self) -> Self {
        self.criteria.require_separate_transfer_queue = true;
        self
    }
    pub fn require_separate_compute_queue(mut self) -> Self {
        self.criteria.require_separate_compute_queue = true;
        self
    }

    pub fn required_device_memory_size(mut self, size: vk::DeviceSize) -> Self {
        self.criteria.required_mem_size = size;
        self
    }

    pub fn add_required_extension(mut self, extension: impl Into<String>) -> Self {
        self.criteria.required_extensions.push(extension.into());
        self
    }
    pub fn add_required_extensions<I, A>(mut self, extensions: I) -> Self
    where
        I: IntoIterator<Item = A>,
        A: Into<String>,
    {
        extensions.into_iter().for_each(|ext| {
            self.criteria.required_extensions.push(ext.into());
        });
        self
    }

    pub fn set_minimum_version(mut self, major: u32, minor: u32) -> Self {
        self.criteria.required_version = vk::make_api_version(0, major, minor, 0);
        self
    }

    pub fn disable_portability_subset(mut self) -> Self {
        self.criteria.enable_portability_subset = false;
        self
    }

    pub fn set_required_features(mut self, features: &vk::PhysicalDeviceFeatures) -> Self {
        utils::combine_features(&mut self.criteria.required_features, features);
        self
    }

    pub fn set_required_additional_features(
        mut self,
        features: impl vk::ExtendsPhysicalDeviceFeatures2,
    ) -> Self {
        self.criteria
            .feature_chain
            .add_node(GenericFeatureNode::from_device_feature(features));

        self
    }
    pub fn defer_surface_initialization(mut self) -> Self {
        self.criteria.defer_surface_initialization = true;
        self
    }
    pub fn select_first_gpu_unconditionally(mut self, unconditionally: bool) -> Self {
        self.criteria.use_first_gpu_unconditionally = unconditionally;
        self
    }

    fn is_device_suitable(&self, pd: &PhysicalDevice) -> PhysicalDeviceSuitability {
        let mut suitable = PhysicalDeviceSuitability::Suitable;
        if !self.criteria.name.is_empty() && self.criteria.name != pd.name {
            return PhysicalDeviceSuitability::Unsuitable;
        }
        if self.criteria.required_version > pd.properties.api_version {
            return PhysicalDeviceSuitability::Unsuitable;
        }

        let dedicated_compute = utils::get_dedicated_queue_index(
            &pd.queue_families,
            vk::QueueFlags::COMPUTE,
            vk::QueueFlags::TRANSFER,
        )
        .is_some();
        let dedicated_transfer = utils::get_dedicated_queue_index(
            &pd.queue_families,
            vk::QueueFlags::TRANSFER,
            vk::QueueFlags::COMPUTE,
        )
        .is_some();

        let separate_compute = utils::get_separate_queue_index(
            &pd.queue_families,
            vk::QueueFlags::COMPUTE,
            vk::QueueFlags::TRANSFER,
        )
        .is_some();
        let separate_transfer = utils::get_separate_queue_index(
            &pd.queue_families,
            vk::QueueFlags::TRANSFER,
            vk::QueueFlags::COMPUTE,
        )
        .is_some();

        let present_queue = if let Some(surface) = self.instance.surface {
            utils::get_present_queue_index(
                &self.entry,
                &self.instance.instance,
                pd.physical_device.clone(),
                surface,
                &pd.queue_families,
            )
            .is_some()
        } else {
            false
        };

        if self.criteria.require_dedicated_compute_queue && !dedicated_compute {
            return PhysicalDeviceSuitability::Unsuitable;
        }
        if self.criteria.require_dedicated_transfer_queue && !dedicated_transfer {
            return PhysicalDeviceSuitability::Unsuitable;
        }
        if self.criteria.require_separate_compute_queue && !separate_compute {
            return PhysicalDeviceSuitability::Unsuitable;
        }
        if self.criteria.require_separate_transfer_queue && !separate_transfer {
            return PhysicalDeviceSuitability::Unsuitable;
        }
        if self.criteria.require_present
            && !present_queue
            && !self.criteria.defer_surface_initialization
        {
            return PhysicalDeviceSuitability::Unsuitable;
        }

        let required_extensions_supported = utils::check_device_extension_support(
            &pd.available_extensions,
            &self.criteria.required_extensions,
        );
        if required_extensions_supported.len() != self.criteria.required_extensions.len() {
            return PhysicalDeviceSuitability::Unsuitable;
        }

        if !self.criteria.defer_surface_initialization && self.criteria.require_present {
            let khr_instance = ash::khr::surface::Instance::new(&self.entry, &self.instance.instance);
            let formats = unsafe {
                khr_instance.get_physical_device_surface_formats(
                    pd.physical_device,
                    self.instance.surface.unwrap(),
                )
            };
            let present_modes = unsafe {
                khr_instance.get_physical_device_surface_present_modes(
                    pd.physical_device,
                    self.instance.surface.unwrap(),
                )
            };

            match formats {
                Ok(f) if !f.is_empty() => {}
                _ => return PhysicalDeviceSuitability::Unsuitable,
            }

            match present_modes {
                Ok(p) if !p.is_empty() => {}
                _ => return PhysicalDeviceSuitability::Unsuitable,
            }
        }

        if !self.criteria.allow_any_type
            && pd.properties.device_type != self.criteria.preferred_type
        {
            suitable = PhysicalDeviceSuitability::PartiallySuitable;
        }

        let required_features_supported = /*pd.features.supports(&self.criteria.required_features);*/ self.criteria.required_features.supports(&pd.features);
        let required_additional_features_supported =
            pd.feature_chain.contains_all(&self.criteria.feature_chain);

        //TODO: check vulkan2 1.1 1.2 1.3 features

        if !required_features_supported || !required_additional_features_supported {
            return PhysicalDeviceSuitability::Unsuitable;
        }

        let mut has_required_memory = false;
        for i in 0..pd.memory_properties.memory_heap_count {
            if pd.memory_properties.memory_heaps[i as usize]
                .flags
                .contains(vk::MemoryHeapFlags::DEVICE_LOCAL)
            {
                if pd.memory_properties.memory_heaps[i as usize].size
                    >= self.criteria.required_mem_size {
                    has_required_memory = true;
                    break;
                }
            }
        }

        if !has_required_memory {
            return PhysicalDeviceSuitability::Unsuitable;
        }

        return suitable;
    }

    /// SAFETY: Valid `device` enumerated from current device selector
    unsafe fn populate_device_details<'a, 'b>(
        &self,
        vk_phys_device: vk::PhysicalDevice,
        src_feature_chain: GenericFeatureChain<'a>,
    ) -> PhysicalDevice<'b>
    where
        'a: 'b,
    {
        let queue_families = unsafe {
            self.instance
                .instance
                .get_physical_device_queue_family_properties(vk_phys_device)
        };
        let properties = unsafe {
            self.instance
                .instance
                .get_physical_device_properties(vk_phys_device)
        };
        let features = unsafe {
            self.instance
                .instance
                .get_physical_device_features(vk_phys_device)
        };
        let memory_properties = unsafe {
            self.instance
                .instance
                .get_physical_device_memory_properties(vk_phys_device)
        };

        let name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
            .to_string_lossy().to_string();

        let available_extensions = if let Ok(extensions) = unsafe {
            self.instance
                .instance
                .enumerate_device_extension_properties(vk_phys_device)
        } {
            extensions
                .iter()
                .map(|ext| {
                    unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) }
                        .to_string_lossy().to_string()
                })
                .collect()
        } else {
            vec![]
        };

        let instance_is_1_1 = self.instance.version >= vk::API_VERSION_1_1;

        let mut fill_chain = src_feature_chain.clone();

        if !fill_chain.nodes.is_empty() && (instance_is_1_1 || self.instance.properties2_ext_enabled) {
            let mut local_features = vk::PhysicalDeviceFeatures2::default();

            unsafe{
                fill_chain.chain_up_physical_device(&mut local_features);
            }
            if instance_is_1_1 {
                unsafe {self.instance.instance.get_physical_device_features2(vk_phys_device, &mut local_features)};
            }else {
                let instance = ash::khr::get_physical_device_properties2::Instance::new(&self.entry, &self.instance.instance);
                unsafe {instance.get_physical_device_features2(vk_phys_device, &mut local_features)};
            }
        }

        PhysicalDevice {
            name,
            physical_device: vk_phys_device,
            surface: self.instance.surface,
            features,
            properties,
            memory_properties,
            instance_version: self.instance.version,
            extensions_to_enable: vec![],
            available_extensions,
            queue_families,
            feature_chain: fill_chain,
            defer_surface_initialization: self.criteria.defer_surface_initialization,
            properties2_ext_enabled: self.instance.properties2_ext_enabled,
            suitability: PhysicalDeviceSuitability::Suitable,
            instance: self.instance.instance.clone(),
            entry: self.entry.clone(),
        }
    }

    fn select_impl(&self) -> Result<Vec<PhysicalDevice>, PhysicalDeviceError> {
        if self.criteria.require_present
            && !self.criteria.defer_surface_initialization
            && self.instance.surface.is_none()
        {
            return Err(PhysicalDeviceError::NoSurfaceProvided);
        }

        let vk_physical_devices =
            if let Ok(devices) = unsafe { self.instance.instance.enumerate_physical_devices() } {
                if devices.is_empty() {
                    return Err(PhysicalDeviceError::NoPhysicalDevicesFound);
                }
                devices
            } else {
                return Err(PhysicalDeviceError::FailedEnumeratePhysicalDevices);
            };

        fn fill_out_phys_dev_with_criteria<'a>(
            phys_dev: &mut PhysicalDevice<'a>,
            criteria: &SelectionCriteria<'a>,
        ) {
            phys_dev.features = criteria.required_features;
            phys_dev.feature_chain = criteria.feature_chain.clone();
            let portability_ext_available = if criteria.enable_portability_subset {
                phys_dev.available_extensions.iter().any(|x| {
                    x == &vk::KHR_PORTABILITY_SUBSET_NAME
                        .to_string_lossy().to_string()
                })
            } else {
                false
            };
            phys_dev.extensions_to_enable.clear();
            phys_dev
                .extensions_to_enable
                .extend(criteria.required_extensions.clone());
            if portability_ext_available {
                phys_dev.extensions_to_enable.push(
                    vk::KHR_PORTABILITY_SUBSET_NAME
                        .to_string_lossy().to_string(),
                )
            }
        }

        if self.criteria.use_first_gpu_unconditionally && !vk_physical_devices.is_empty() {
            let mut physical_device = unsafe {
                self.populate_device_details(
                    vk_physical_devices[0],
                    self.criteria.feature_chain.clone(),
                )
            };
            fill_out_phys_dev_with_criteria(&mut physical_device, &self.criteria);
            return Ok(vec![physical_device]);
        }

        let mut physical_devices: Vec<_> = vk_physical_devices
            .iter()
            .map(|p| {
                let mut physical_device = unsafe {
                    self.populate_device_details(*p, self.criteria.feature_chain.clone())
                };
                physical_device.suitability = self.is_device_suitable(&physical_device);
                physical_device
            })
            .filter(|p| p.suitability != PhysicalDeviceSuitability::Unsuitable)
            .collect();

        physical_devices.sort_by_key(|p| match p.suitability {
            PhysicalDeviceSuitability::Suitable => 0,
            _ => 1,
        });

        physical_devices.iter_mut().for_each(|p| {
            fill_out_phys_dev_with_criteria(p, &self.criteria);
        });
        Ok(physical_devices)
    }
}

struct SelectionCriteria<'a> {
    name: String,
    preferred_type: vk::PhysicalDeviceType,
    allow_any_type: bool,
    require_present: bool,
    require_dedicated_transfer_queue: bool,
    require_dedicated_compute_queue: bool,
    require_separate_transfer_queue: bool,
    require_separate_compute_queue: bool,
    required_mem_size: vk::DeviceSize,

    required_extensions: Vec<String>,
    required_version: u32,

    required_features: vk::PhysicalDeviceFeatures,

    feature_chain: GenericFeatureChain<'a>,

    defer_surface_initialization: bool,
    use_first_gpu_unconditionally: bool,
    enable_portability_subset: bool,
}

impl Default for SelectionCriteria<'_> {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            preferred_type: vk::PhysicalDeviceType::DISCRETE_GPU,
            allow_any_type: true,
            require_present: true,
            require_dedicated_transfer_queue: false,
            require_dedicated_compute_queue: false,
            require_separate_transfer_queue: false,
            require_separate_compute_queue: false,
            required_mem_size: 0,
            required_extensions: vec![],
            required_version: vk::API_VERSION_1_0,
            required_features: Default::default(),
            feature_chain: Default::default(),
            defer_surface_initialization: false,
            use_first_gpu_unconditionally: false,
            enable_portability_subset: true,
        }
    }
}

struct InstanceInfo {
    instance: ash::Instance,
    surface: Option<vk::SurfaceKHR>,
    version: u32,
    headless: bool,
    properties2_ext_enabled: bool,
}
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum PhysicalDeviceSuitability {
    Suitable,
    PartiallySuitable,
    Unsuitable,
}

pub struct PhysicalDevice<'a> {
    pub name: String,
    pub physical_device: vk::PhysicalDevice,
    pub surface: Option<vk::SurfaceKHR>,
    pub features: vk::PhysicalDeviceFeatures,
    pub properties: vk::PhysicalDeviceProperties,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,

    pub instance_version: u32,
    pub(crate) extensions_to_enable: Vec<String>,
    pub(crate) available_extensions: Vec<String>,
    pub(crate) queue_families: Vec<vk::QueueFamilyProperties>,
    pub(crate) feature_chain: GenericFeatureChain<'a>,

    pub(crate) defer_surface_initialization: bool,
    pub(crate) properties2_ext_enabled: bool,
    suitability: PhysicalDeviceSuitability,

    pub instance: ash::Instance,
    pub entry: ash::Entry,
}

impl PhysicalDevice<'_> {
    pub fn has_dedicated_compute_queue(&self) -> bool {
        utils::get_dedicated_queue_index(
            &self.queue_families,
            vk::QueueFlags::COMPUTE,
            vk::QueueFlags::TRANSFER,
        )
        .is_some()
    }
    pub fn has_dedicated_transfer_queue(&self) -> bool {
        utils::get_dedicated_queue_index(
            &self.queue_families,
            vk::QueueFlags::TRANSFER,
            vk::QueueFlags::COMPUTE,
        )
        .is_some()
    }

    pub fn has_separate_compute_queue(&self) -> bool {
        utils::get_separate_queue_index(
            &self.queue_families,
            vk::QueueFlags::COMPUTE,
            vk::QueueFlags::TRANSFER,
        )
        .is_some()
    }
    pub fn has_separate_transfer_queue(&self) -> bool {
        utils::get_separate_queue_index(
            &self.queue_families,
            vk::QueueFlags::TRANSFER,
            vk::QueueFlags::COMPUTE,
        )
        .is_some()
    }

    pub fn get_queue_families(&self) -> Vec<vk::QueueFamilyProperties> {
        self.queue_families.clone()
    }
    pub fn get_extensions(&self) -> Vec<String> {
        self.extensions_to_enable.clone()
    }
    pub fn get_available_extensions(&self) -> Vec<String> {
        self.available_extensions.clone()
    }
    pub fn is_extension_present(&self, ext: impl Into<String>) -> bool {
        let ext = ext.into();
        self.available_extensions.iter().any(|avl| *avl == ext)
    }
    pub fn enable_extension_if_present(&mut self, ext: impl Into<String>) -> bool {
        let ext = ext.into();
        if self.is_extension_present(ext.clone()) && !self.extensions_to_enable.contains(&ext) {
            self.extensions_to_enable.push(ext);
            return true;
        }
        false
    }
    pub fn enable_extensions_if_present(&mut self, extensions: Vec<impl Into<String>>) -> bool {
        for ext in extensions {
            let ext = ext.into();
            if !self.enable_extension_if_present(ext) {
                return false;
            }
        }

        true
    }
}
