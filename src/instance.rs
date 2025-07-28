use crate::errors::InstanceError;
use crate::system_info::{SystemInfo, VALIDATION_LAYER_NAME};
use ash::ext::debug_utils;
use ash::prelude::VkResult;
use ash::{vk, Entry};
use std::ffi::{c_void, CString};
use ash::vk::Handle;

pub unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    unsafe {
        let callback_data = *p_callback_data;
        let message_id_number = callback_data.message_id_number;

        let message_id_name = if callback_data.p_message_id_name.is_null() {
            std::borrow::Cow::from("")
        } else {
            std::ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
        };

        let message = if callback_data.p_message.is_null() {
            std::borrow::Cow::from("")
        } else {
            std::ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy()
        };

        let formated =
            format!("{message_type:?} [{message_id_name} ({message_id_number})] : {message}");

        #[cfg(feature = "logger")]
        {
            match message_severity {
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => log::error!("{}", formated),
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => log::warn!("{}", formated),
                vk::DebugUtilsMessageSeverityFlagsEXT::INFO => log::info!("{}", formated),
                vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => log::debug!("{}", formated),
                _ => log::error!("Unknown vulkan message severity: {}", formated),
            }
        }

        #[cfg(not(feature = "logger"))]
        {
            println!("{message_severity:?}: {formated}");
        }

        vk::FALSE
    }
}

pub struct InstanceBuilder<'a> {
    info: InstanceInfo<'a>,
}

impl<'a> InstanceBuilder<'a> {
    pub fn new() -> Self {
        Self {
            info: Default::default(),
        }
    }

    pub fn build(self) -> Result<Instance<'a>, InstanceError> {
        let sys_info = SystemInfo::new()?;

        let entry = unsafe { ash::Entry::load().unwrap() };

        let mut instance_version = vk::API_VERSION_1_0;

        if self.info.minimum_instance_version > vk::API_VERSION_1_0
            || self.info.required_api_version > vk::API_VERSION_1_0
        {
            if let Some(ver) = unsafe { entry.try_enumerate_instance_version().unwrap() } {
                instance_version = ver;
            } else {
                return Err(InstanceError::VulkanVersionUnavailable);
            }

            if instance_version < self.info.minimum_instance_version
                || (self.info.minimum_instance_version == 0
                    && instance_version < self.info.required_api_version)
            {
                return if vk::api_version_minor(self.info.required_api_version) == 4 {
                    Err(InstanceError::VulkanVersion1_4Unavailable)
                } else if vk::api_version_minor(self.info.required_api_version) == 3 {
                    Err(InstanceError::VulkanVersion1_3Unavailable)
                } else if vk::api_version_minor(self.info.required_api_version) == 2 {
                    Err(InstanceError::VulkanVersion1_2Unavailable)
                } else if vk::api_version_minor(self.info.required_api_version) == 1 {
                    Err(InstanceError::VulkanVersion1_1Unavailable)
                } else {
                    Err(InstanceError::VulkanVersionUnavailable)
                };
            }
        }

        let api_version = if instance_version < vk::API_VERSION_1_1 {
            instance_version
        } else {
            self.info.required_api_version
        };

        let app_name = CString::new(self.info.app_name).unwrap();
        let engine_name = CString::new(self.info.engine_name).unwrap();
        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(self.info.app_version)
            .engine_name(&engine_name)
            .engine_version(self.info.engine_version)
            .api_version(api_version);

        let mut extensions: Vec<CString> = vec![];
        let mut layers: Vec<CString> = vec![];

        for extension in self.info.enabled_extensions {
            extensions.push(CString::new(extension).unwrap())
        }

        if self.info.debug_callback.is_some()
            && self.info.use_debug_messenger
            && sys_info.debug_utils_available
        {
            extensions.push(vk::EXT_DEBUG_UTILS_NAME.to_owned());
        }

        let properties2_ext_enabled = api_version < vk::API_VERSION_1_1
            && sys_info.is_extension_available(
                vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_NAME
                    .to_str()
                    .unwrap(),
            );

        if properties2_ext_enabled {
            extensions.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_NAME.to_owned());
        }

        if !self.info.layer_settings.is_empty() {
            extensions.push(vk::EXT_LAYER_SETTINGS_NAME.to_owned())
        }

        let portability_enumeration_supported = if cfg!(feature = "portability_enumeration") {
            let portability_enumeration_supported = sys_info
                .is_extension_available(vk::KHR_PORTABILITY_ENUMERATION_NAME.to_str().unwrap());
            if portability_enumeration_supported {
                extensions.push(vk::KHR_PORTABILITY_ENUMERATION_NAME.to_owned())
            }
            portability_enumeration_supported
        } else {
            false
        };

        if !self.info.headless_context {
            let mut check_add_window_ext = |name: &str| -> bool {
                if !sys_info.is_extension_available(name) {
                    return false;
                }
                extensions.push(CString::new(name).unwrap());
                true
            };

            let khr_surface_added = check_add_window_ext(vk::KHR_SURFACE_NAME.to_str().unwrap());

            let added_window_exts = {
                #[cfg(target_os = "windows")]
                {
                    check_add_window_ext(vk::KHR_WIN32_SURFACE_NAME.to_str().unwrap())
                }
                #[cfg(target_os = "android")]
                {
                    check_add_window_ext(vk::KHR_ANDROID_SURFACE_NAME.to_str().unwrap())
                }
                #[cfg(any(target_os = "linux", target_os = "freebsd"))]
                {
                    let mut added_window_exts = false;
                    added_window_exts |=
                        check_add_window_ext(vk::KHR_XCB_SURFACE_NAME.to_str().unwrap());
                    added_window_exts |=
                        check_add_window_ext(vk::KHR_XLIB_SURFACE_NAME.to_str().unwrap());
                    added_window_exts |=
                        check_add_window_ext(vk::KHR_WAYLAND_SURFACE_NAME.to_str().unwrap());
                    added_window_exts
                }
                #[cfg(target_os = "macos")]
                {
                    check_add_window_ext(vk::EXT_METAL_SURFACE_NAME.to_str().unwrap())
                }
                #[cfg(not(any(
                    target_os = "windows",
                    target_os = "android",
                    target_os = "linux",
                    target_os = "freebsd",
                    target_os = "macos",
                )))]
                {
                    false
                }
            };

            if !khr_surface_added || !added_window_exts {
                return Err(InstanceError::WindowingExtensionsUnavailable);
            }
        }

        let all_extensions_supported = sys_info.extensions_supported(&extensions);
        if !all_extensions_supported {
            return Err(InstanceError::RequestedExtensionsUnavailable);
        }

        for layer in self.info.enabled_layers {
            layers.push(CString::new(layer).unwrap());
        }

        if self.info.enable_validation_layers
            || (self.info.request_validation_layers && sys_info.validation_layers_available)
        {
            layers.push(VALIDATION_LAYER_NAME.to_owned())
        }

        let all_layers_supported = sys_info.layers_supported(&layers);
        if !all_layers_supported {
            return Err(InstanceError::RequestedLayersUnavailable);
        }

        let mut messenger_create_info = if self.info.use_debug_messenger {
            vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(self.info.debug_message_severity)
                .message_type(self.info.debug_message_type)
                .pfn_user_callback(self.info.debug_callback)
                .user_data(
                    self.info
                        .debug_user_data_pointer
                        .unwrap_or(std::ptr::null_mut()),
                )
        } else {
            vk::DebugUtilsMessengerCreateInfoEXT::default()
        };

        let mut validation_features = if !self.info.enabled_validation_features.is_empty()
            || self.info.disabled_validation_features.is_empty()
        {
            vk::ValidationFeaturesEXT::default()
                .enabled_validation_features(&self.info.enabled_validation_features)
                .disabled_validation_features(&self.info.disabled_validation_features)
        } else {
            vk::ValidationFeaturesEXT::default()
        };

        let mut checks = if !self.info.disabled_validation_checks.is_empty() {
            vk::ValidationFlagsEXT::default()
                .disabled_validation_checks(self.info.disabled_validation_checks.as_slice())
        } else {
            Default::default()
        };

        let mut layer_settings_ci = if !self.info.layer_settings.is_empty() {
            vk::LayerSettingsCreateInfoEXT::default().settings(self.info.layer_settings.as_slice())
        } else {
            Default::default()
        };

        let extensions_raw = extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        let layers_raw = layers
            .iter()
            .map(|layer| layer.as_ptr())
            .collect::<Vec<_>>();

        let mut instance_create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(extensions_raw.as_slice())
            .enabled_layer_names(layers_raw.as_slice());

        if self.info.use_debug_messenger {
            instance_create_info = instance_create_info.push_next(&mut messenger_create_info);
        }
        if !self.info.enabled_validation_features.is_empty()
            || self.info.disabled_validation_features.is_empty()
        {
            instance_create_info = instance_create_info.push_next(&mut validation_features);
        }
        if !self.info.disabled_validation_checks.is_empty() {
            instance_create_info = instance_create_info.push_next(&mut checks);
        }
        if !self.info.layer_settings.is_empty() {
            instance_create_info = instance_create_info.push_next(&mut layer_settings_ci);
        }

        if portability_enumeration_supported {
            instance_create_info.flags |= vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
        }

        let vk_instance = unsafe {
            entry
                .create_instance(
                    &instance_create_info,
                    self.info.allocation_callbacks.as_ref(),
                )
                .map_err(|_| InstanceError::FailedToCreateInstance)?
        };

        let debug_messenger = if self.info.use_debug_messenger {
            unsafe {
                let dm = create_debug_utils_messenger(
                    &vk_instance,
                    self.info.debug_callback,
                    &entry,
                    self.info.debug_message_severity,
                    self.info.debug_message_type,
                    self.info.debug_user_data_pointer,
                    self.info.allocation_callbacks,
                );
                match dm {
                    Ok(x) => Some(x),
                    Err(_) => return Err(InstanceError::FailedToCreateDebugMessenger),
                }
            }
        } else {
            None
        };

        Ok(Instance {
            instance: vk_instance,
            debug_messenger,
            allocation_callbacks: self.info.allocation_callbacks,
            entry,
            instance_version,
            api_version,
            headless: self.info.headless_context,
            properties2_ext_enabled,
        })
    }
    pub fn set_app_name(mut self, app_name: impl Into<String>) -> Self {
        self.info.app_name = app_name.into();
        self
    }
    pub fn set_engine_name(mut self, engine_name: impl Into<String>) -> Self {
        self.info.engine_name = engine_name.into();
        self
    }

    pub fn set_app_version_raw(mut self, app_version: u32) -> Self {
        self.info.app_version = app_version;
        self
    }
    pub fn set_app_version(self, major: u32, minor: u32, patch: u32) -> Self {
        self.set_app_version_raw(vk::make_api_version(0, major, minor, patch))
    }

    pub fn require_api_version_raw(mut self, required_api_version: u32) -> Self {
        self.info.required_api_version = required_api_version;
        self
    }
    pub fn require_api_version(self, major: u32, minor: u32, patch: u32) -> Self {
        self.require_api_version_raw(vk::make_api_version(0, major, minor, patch))
    }

    pub fn set_minimum_instance_version_raw(mut self, minimum_instance_version: u32) -> Self {
        self.info.minimum_instance_version = minimum_instance_version;
        self
    }
    pub fn set_minimum_instance_version(self, major: u32, minor: u32, patch: u32) -> Self {
        self.set_minimum_instance_version_raw(vk::make_api_version(0, major, minor, patch))
    }

    pub fn enable_layer(mut self, layer_name: impl Into<String>) -> Self {
        self.info.enabled_layers.push(layer_name.into());
        self
    }
    pub fn enable_extension(mut self, extension_name: impl Into<String>) -> Self {
        self.info.enabled_extensions.push(extension_name.into());
        self
    }
    pub fn enable_extensions(mut self, extension_names: Vec<impl Into<String>>) -> Self {
        for name in extension_names {
            self.info.enabled_extensions.push(name.into());
        }

        self
    }
    pub fn set_headless(mut self, headless: bool) -> Self {
        self.info.headless_context = headless;
        self
    }

    pub fn enable_validation_layers(mut self, require_validation: bool) -> Self {
        self.info.enable_validation_layers = require_validation;
        self
    }
    pub fn request_validation_layers(mut self, enable_validation: bool) -> Self {
        self.info.request_validation_layers = enable_validation;
        self
    }

    pub fn use_default_debug_messenger(mut self) -> Self {
        self.info.use_debug_messenger = true;
        self.info.debug_callback = Some(vulkan_debug_callback);
        self
    }
    pub fn set_debug_callback(
        mut self,
        callback: vk::PFN_vkDebugUtilsMessengerCallbackEXT,
    ) -> Self {
        self.info.debug_callback = callback;
        self
    }
    pub fn set_debug_callback_user_data_pointer(mut self, user_data_pointer: *mut c_void) -> Self {
        self.info.debug_user_data_pointer = Some(user_data_pointer);
        self
    }
    pub fn set_debug_messenger_severity(
        mut self,
        severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    ) -> Self {
        self.info.debug_message_severity = severity;
        self
    }

    pub fn add_debug_messenger_severity(
        mut self,
        severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    ) -> Self {
        self.info.debug_message_severity |= severity;
        self
    }
    pub fn set_debug_messenger_type(
        mut self,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    ) -> Self {
        self.info.debug_message_type = message_type;
        self
    }
    pub fn add_debug_messenger_type(
        mut self,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    ) -> Self {
        self.info.debug_message_type |= message_type;
        self
    }
    pub fn add_validation_disable(mut self, check: vk::ValidationCheckEXT) -> Self {
        self.info.disabled_validation_checks.push(check);
        self
    }
    pub fn add_validation_feature_enable(mut self, enable: vk::ValidationFeatureEnableEXT) -> Self {
        self.info.enabled_validation_features.push(enable);
        self
    }
    pub fn add_validation_feature_disable(
        mut self,
        disable: vk::ValidationFeatureDisableEXT,
    ) -> Self {
        self.info.disabled_validation_features.push(disable);
        self
    }
    pub fn set_allocation_callbacks(
        mut self,
        allocation_callbacks: vk::AllocationCallbacks<'a>,
    ) -> Self {
        self.info.allocation_callbacks = Some(allocation_callbacks);
        self
    }
    pub fn add_layer_setting(mut self, layer_setting: vk::LayerSettingEXT<'a>) -> Self {
        self.info.layer_settings.push(layer_setting);
        self
    }
}

impl Default for InstanceBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe fn create_debug_utils_messenger(
    instance: &ash::Instance,
    debug_callback: vk::PFN_vkDebugUtilsMessengerCallbackEXT,
    entry: &Entry,
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    user_data_pointer: Option<*mut c_void>,
    allocation_callbacks: Option<vk::AllocationCallbacks>,
) -> VkResult<vk::DebugUtilsMessengerEXT> {
    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(severity)
        .message_type(message_type)
        .pfn_user_callback(debug_callback);

    let debug_info = if let Some(user_data_pointer) = user_data_pointer {
        debug_info.user_data(user_data_pointer)
    } else {
        debug_info
    };

    let debug_loader = debug_utils::Instance::new(entry, instance);
    unsafe { debug_loader.create_debug_utils_messenger(&debug_info, allocation_callbacks.as_ref()) }
}

struct InstanceInfo<'a> {
    // VkApplicationInfo
    app_name: String,
    engine_name: String,
    app_version: u32,
    engine_version: u32,
    minimum_instance_version: u32,
    required_api_version: u32,

    // VkInstanceCreateInfo
    enabled_layers: Vec<String>,
    enabled_extensions: Vec<String>,
    flags: vk::InstanceCreateFlags,
    layer_settings: Vec<vk::LayerSettingEXT<'a>>,

    // Debug Utils
    debug_callback: vk::PFN_vkDebugUtilsMessengerCallbackEXT,
    debug_message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    debug_message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    debug_user_data_pointer: Option<*mut c_void>,

    // Validation Features
    disabled_validation_checks: Vec<vk::ValidationCheckEXT>,
    enabled_validation_features: Vec<vk::ValidationFeatureEnableEXT>,
    disabled_validation_features: Vec<vk::ValidationFeatureDisableEXT>,

    // Custom allocator
    allocation_callbacks: Option<vk::AllocationCallbacks<'a>>,

    request_validation_layers: bool,
    enable_validation_layers: bool,
    use_debug_messenger: bool,
    headless_context: bool,
}

impl Default for InstanceInfo<'_> {
    fn default() -> Self {
        Self {
            app_name: "".to_string(),
            engine_name: "".to_string(),
            app_version: 0,
            engine_version: 0,
            minimum_instance_version: 0,
            required_api_version: vk::API_VERSION_1_0,
            enabled_layers: vec![],
            enabled_extensions: vec![],
            flags: Default::default(),
            layer_settings: vec![],
            debug_callback: None,
            debug_message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            debug_message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            debug_user_data_pointer: None,
            disabled_validation_checks: vec![],
            enabled_validation_features: vec![],
            disabled_validation_features: vec![],
            allocation_callbacks: None,
            request_validation_layers: false,
            enable_validation_layers: false,
            use_debug_messenger: false,
            headless_context: false,
        }
    }
}

pub struct Instance<'a> {
    pub instance: ash::Instance,
    pub debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    pub allocation_callbacks: Option<vk::AllocationCallbacks<'a>>,
    pub entry: ash::Entry,
    pub instance_version: u32,
    pub api_version: u32,
    pub headless: bool,
    pub properties2_ext_enabled: bool,
}

impl Instance<'_> {
    pub unsafe fn destroy(&self) {
        if let Some(messenger) = self.debug_messenger {
            if !messenger.is_null() {
                //TODO: add entry to instance, device, physical device, and swapchain
                let instance = unsafe {
                    ash::ext::debug_utils::Instance::new(&Entry::load().unwrap(), &self.instance)
                };
                unsafe {
                    instance.destroy_debug_utils_messenger(
                        messenger,
                        self.allocation_callbacks.as_ref(),
                    )
                }
            }
        }
        unsafe {
            self.instance
                .destroy_instance(self.allocation_callbacks.as_ref())
        };
    }
}
