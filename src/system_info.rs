use ash::vk;
use std::ffi::{CStr, CString};
use crate::errors::InstanceError;

pub const VALIDATION_LAYER_NAME: &CStr = c"VK_LAYER_KHRONOS_validation";

pub struct SystemInfo {
    available_layers: Vec<vk::LayerProperties>,
    available_extensions: Vec<vk::ExtensionProperties>,
    pub validation_layers_available: bool,
    pub debug_utils_available: bool,
    pub instance_api_version: u32,
}

impl SystemInfo {
    pub fn new() -> Result<Self, InstanceError> {
        unsafe {
            let entry = ash::Entry::load().map_err(|_| InstanceError::VulkanUnavailable)?;
            let available_layers = entry.enumerate_instance_layer_properties().unwrap();
            let available_extensions = entry.enumerate_instance_extension_properties(None).unwrap();

            let validation_layers_available = available_layers
                .iter()
                .any(|layer| CStr::from_ptr(layer.layer_name.as_ptr()) == VALIDATION_LAYER_NAME);

            let debug_utils_available = available_extensions
                .iter()
                .any(|ext| CStr::from_ptr(ext.extension_name.as_ptr()) == vk::EXT_DEBUG_UTILS_NAME);

            let instance_api_version = entry
                .try_enumerate_instance_version()
                .unwrap_or(Option::from(vk::make_api_version(0, 1, 0, 0)))
                .unwrap_or(vk::make_api_version(0, 1, 0, 0));

            Ok(Self {
                available_layers,
                available_extensions,
                validation_layers_available,
                debug_utils_available,
                instance_api_version,
            })
        }
    }

    pub fn is_layer_available(&self, layer_name: impl Into<String>) -> bool {
        let layer_name = layer_name.into();
        self.available_layers.iter().any(|layer| unsafe {
            CStr::from_ptr(layer.layer_name.as_ptr())
                == CString::new(layer_name.clone()).unwrap().as_ref()
        })
    }

    pub fn is_extension_available(&self, extension_name: impl Into<String>) -> bool {
        let extension_name = extension_name.into();
        self.available_extensions.iter().any(|ext| unsafe {
            CStr::from_ptr(ext.extension_name.as_ptr())
                == CString::new(extension_name.clone()).unwrap().as_ref()
        })
    }

    pub fn extensions_supported(&self, extensions: &[CString]) -> bool {
        extensions
            .iter()
            .all(|ext| self.is_extension_available(ext.to_str().unwrap()))
    }

    pub fn layers_supported(&self, layers: &[CString]) -> bool {
        layers
            .iter()
            .all(|layer| self.is_layer_available(layer.to_str().unwrap()))
    }

    pub fn is_instance_version_available(&self, major: u32, minor: u32, patch: u32) -> bool {
        let required_version = vk::make_api_version(0, major, minor, patch);
        self.is_instance_version_available_raw(required_version)
    }

    pub fn is_instance_version_available_raw(&self, version: u32) -> bool {
        self.instance_api_version >= version
    }
}
