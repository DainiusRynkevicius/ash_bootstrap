use crate::errors::SurfaceSupportError;
use crate::instance::Instance;
use ash::vk;
use ash::vk::{Bool32, Extent2D, Handle};

pub trait AsBool {
    fn as_bool(&self) -> bool;
}

impl AsBool for Bool32 {
    fn as_bool(&self) -> bool {
        *self != vk::FALSE
    }
}
pub fn combine_features(dest: &mut vk::PhysicalDeviceFeatures, src: &vk::PhysicalDeviceFeatures) {
    dest.robust_buffer_access =
        Bool32::from(dest.robust_buffer_access.as_bool() || src.robust_buffer_access.as_bool());
    dest.full_draw_index_uint32 =
        Bool32::from(dest.full_draw_index_uint32.as_bool() || src.full_draw_index_uint32.as_bool());
    dest.image_cube_array =
        Bool32::from(dest.image_cube_array.as_bool() || src.image_cube_array.as_bool());
    dest.independent_blend =
        Bool32::from(dest.independent_blend.as_bool() || src.independent_blend.as_bool());
    dest.geometry_shader =
        Bool32::from(dest.geometry_shader.as_bool() || src.geometry_shader.as_bool());
    dest.tessellation_shader =
        Bool32::from(dest.tessellation_shader.as_bool() || src.tessellation_shader.as_bool());
    dest.sample_rate_shading =
        Bool32::from(dest.sample_rate_shading.as_bool() || src.sample_rate_shading.as_bool());
    dest.dual_src_blend =
        Bool32::from(dest.dual_src_blend.as_bool() || src.dual_src_blend.as_bool());
    dest.logic_op = Bool32::from(dest.logic_op.as_bool() || src.logic_op.as_bool());
    dest.multi_draw_indirect =
        Bool32::from(dest.multi_draw_indirect.as_bool() || src.multi_draw_indirect.as_bool());
    dest.draw_indirect_first_instance = Bool32::from(
        dest.draw_indirect_first_instance.as_bool() || src.draw_indirect_first_instance.as_bool(),
    );
    dest.depth_clamp = Bool32::from(dest.depth_clamp.as_bool() || src.depth_clamp.as_bool());
    dest.depth_bias_clamp =
        Bool32::from(dest.depth_bias_clamp.as_bool() || src.depth_bias_clamp.as_bool());
    dest.fill_mode_non_solid =
        Bool32::from(dest.fill_mode_non_solid.as_bool() || src.fill_mode_non_solid.as_bool());
    dest.depth_bounds = Bool32::from(dest.depth_bounds.as_bool() || src.depth_bounds.as_bool());
    dest.wide_lines = Bool32::from(dest.wide_lines.as_bool() || src.wide_lines.as_bool());
    dest.large_points = Bool32::from(dest.large_points.as_bool() || src.large_points.as_bool());
    dest.alpha_to_one = Bool32::from(dest.alpha_to_one.as_bool() || src.alpha_to_one.as_bool());
    dest.multi_viewport =
        Bool32::from(dest.multi_viewport.as_bool() || src.multi_viewport.as_bool());
    dest.sampler_anisotropy =
        Bool32::from(dest.sampler_anisotropy.as_bool() || src.sampler_anisotropy.as_bool());
    dest.texture_compression_etc2 = Bool32::from(
        dest.texture_compression_etc2.as_bool() || src.texture_compression_etc2.as_bool(),
    );
    dest.texture_compression_astc_ldr = Bool32::from(
        dest.texture_compression_astc_ldr.as_bool() || src.texture_compression_astc_ldr.as_bool(),
    );
    dest.texture_compression_bc =
        Bool32::from(dest.texture_compression_bc.as_bool() || src.texture_compression_bc.as_bool());
    dest.occlusion_query_precise = Bool32::from(
        dest.occlusion_query_precise.as_bool() || src.occlusion_query_precise.as_bool(),
    );
    dest.pipeline_statistics_query = Bool32::from(
        dest.pipeline_statistics_query.as_bool() || src.pipeline_statistics_query.as_bool(),
    );
    dest.vertex_pipeline_stores_and_atomics = Bool32::from(
        dest.vertex_pipeline_stores_and_atomics.as_bool()
            || src.vertex_pipeline_stores_and_atomics.as_bool(),
    );
    dest.fragment_stores_and_atomics = Bool32::from(
        dest.fragment_stores_and_atomics.as_bool() || src.fragment_stores_and_atomics.as_bool(),
    );
    dest.shader_tessellation_and_geometry_point_size = Bool32::from(
        dest.shader_tessellation_and_geometry_point_size.as_bool()
            || src.shader_tessellation_and_geometry_point_size.as_bool(),
    );
    dest.shader_image_gather_extended = Bool32::from(
        dest.shader_image_gather_extended.as_bool() || src.shader_image_gather_extended.as_bool(),
    );
    dest.shader_storage_image_extended_formats = Bool32::from(
        dest.shader_storage_image_extended_formats.as_bool()
            || src.shader_storage_image_extended_formats.as_bool(),
    );
    dest.shader_storage_image_multisample = Bool32::from(
        dest.shader_storage_image_multisample.as_bool()
            || src.shader_storage_image_multisample.as_bool(),
    );
    dest.shader_storage_image_read_without_format = Bool32::from(
        dest.shader_storage_image_read_without_format.as_bool()
            || src.shader_storage_image_read_without_format.as_bool(),
    );
    dest.shader_storage_image_write_without_format = Bool32::from(
        dest.shader_storage_image_write_without_format.as_bool()
            || src.shader_storage_image_write_without_format.as_bool(),
    );
    dest.shader_uniform_buffer_array_dynamic_indexing = Bool32::from(
        dest.shader_uniform_buffer_array_dynamic_indexing.as_bool()
            || src.shader_uniform_buffer_array_dynamic_indexing.as_bool(),
    );
    dest.shader_sampled_image_array_dynamic_indexing = Bool32::from(
        dest.shader_sampled_image_array_dynamic_indexing.as_bool()
            || src.shader_sampled_image_array_dynamic_indexing.as_bool(),
    );
    dest.shader_storage_buffer_array_dynamic_indexing = Bool32::from(
        dest.shader_storage_buffer_array_dynamic_indexing.as_bool()
            || src.shader_storage_buffer_array_dynamic_indexing.as_bool(),
    );
    dest.shader_storage_image_array_dynamic_indexing = Bool32::from(
        dest.shader_storage_image_array_dynamic_indexing.as_bool()
            || src.shader_storage_image_array_dynamic_indexing.as_bool(),
    );
    dest.shader_clip_distance =
        Bool32::from(dest.shader_clip_distance.as_bool() || src.shader_clip_distance.as_bool());
    dest.shader_cull_distance =
        Bool32::from(dest.shader_cull_distance.as_bool() || src.shader_cull_distance.as_bool());
    dest.shader_float64 =
        Bool32::from(dest.shader_float64.as_bool() || src.shader_float64.as_bool());
    dest.shader_int64 = Bool32::from(dest.shader_int64.as_bool() || src.shader_int64.as_bool());
    dest.shader_int16 = Bool32::from(dest.shader_int16.as_bool() || src.shader_int16.as_bool());
    dest.shader_resource_residency = Bool32::from(
        dest.shader_resource_residency.as_bool() || src.shader_resource_residency.as_bool(),
    );
    dest.shader_resource_min_lod = Bool32::from(
        dest.shader_resource_min_lod.as_bool() || src.shader_resource_min_lod.as_bool(),
    );
    dest.sparse_binding =
        Bool32::from(dest.sparse_binding.as_bool() || src.sparse_binding.as_bool());
    dest.sparse_residency_buffer = Bool32::from(
        dest.sparse_residency_buffer.as_bool() || src.sparse_residency_buffer.as_bool(),
    );
    dest.sparse_residency_image2_d = Bool32::from(
        dest.sparse_residency_image2_d.as_bool() || src.sparse_residency_image2_d.as_bool(),
    );
    dest.sparse_residency_image3_d = Bool32::from(
        dest.sparse_residency_image3_d.as_bool() || src.sparse_residency_image3_d.as_bool(),
    );
    dest.sparse_residency2_samples = Bool32::from(
        dest.sparse_residency2_samples.as_bool() || src.sparse_residency2_samples.as_bool(),
    );
    dest.sparse_residency4_samples = Bool32::from(
        dest.sparse_residency4_samples.as_bool() || src.sparse_residency4_samples.as_bool(),
    );
    dest.sparse_residency8_samples = Bool32::from(
        dest.sparse_residency8_samples.as_bool() || src.sparse_residency8_samples.as_bool(),
    );
    dest.sparse_residency16_samples = Bool32::from(
        dest.sparse_residency16_samples.as_bool() || src.sparse_residency16_samples.as_bool(),
    );
    dest.sparse_residency_aliased = Bool32::from(
        dest.sparse_residency_aliased.as_bool() || src.sparse_residency_aliased.as_bool(),
    );
    dest.variable_multisample_rate = Bool32::from(
        dest.variable_multisample_rate.as_bool() || src.variable_multisample_rate.as_bool(),
    );
    dest.inherited_queries =
        Bool32::from(dest.inherited_queries.as_bool() || src.inherited_queries.as_bool());
}

pub fn get_dedicated_queue_index(
    families: &[vk::QueueFamilyProperties],
    desired_flags: vk::QueueFlags,
    undesired_flags: vk::QueueFlags,
) -> Option<u32> {
    families
        .iter()
        .position(|f| {
            f.queue_flags.contains(desired_flags)
                && !f.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && !f.queue_flags.intersects(undesired_flags)
        })
        .map(|i| i as u32)
}

pub fn get_first_queue_index(
    families: &[vk::QueueFamilyProperties],
    desired_flags: vk::QueueFlags,
) -> Option<u32> {
    families
        .iter()
        .position(|f| f.queue_flags.contains(desired_flags))
        .map(|i| i as u32)
}

pub fn get_separate_queue_index(
    families: &[vk::QueueFamilyProperties],
    desired_flags: vk::QueueFlags,
    undesired_flags: vk::QueueFlags,
) -> Option<u32> {
    let mut fallback: Option<u32> = None;

    for (i, family) in families.iter().enumerate() {
        if family.queue_flags.contains(desired_flags)
            && !family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        {
            if !family.queue_flags.intersects(undesired_flags) {
                return Some(i as u32);
            } else {
                fallback = Some(i as u32);
            }
        }
    }

    fallback
}

pub fn get_present_queue_index(
    entry: &ash::Entry,
    instance: &ash::Instance,
    phys_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
    families: &[vk::QueueFamilyProperties],
) -> Option<u32> {
    let khr_instance = ash::khr::surface::Instance::new(entry, instance);
    families
        .iter()
        .enumerate()
        .position(|(i, _)| unsafe {
            khr_instance
                .get_physical_device_surface_support(phys_device, i as u32, surface)
                .unwrap_or(false)
        })
        .map(|t| t as u32)
}

pub fn check_device_extension_support(available: &[String], required: &[String]) -> Vec<String> {
    required
        .iter()
        .filter(|req| available.iter().any(|avail| avail == *req))
        .cloned()
        .collect()
}

pub trait SupportsFeatures {
    fn supports(&self, other: &Self) -> bool;
}

impl SupportsFeatures for vk::PhysicalDeviceFeatures {
    fn supports(&self, other: &Self) -> bool {
        if self.robust_buffer_access.as_bool() && !other.robust_buffer_access.as_bool() {
            return false;
        }
        if self.full_draw_index_uint32.as_bool() && !other.full_draw_index_uint32.as_bool() {
            return false;
        }
        if self.image_cube_array.as_bool() && !other.image_cube_array.as_bool() {
            return false;
        }
        if self.independent_blend.as_bool() && !other.independent_blend.as_bool() {
            return false;
        }
        if self.geometry_shader.as_bool() && !other.geometry_shader.as_bool() {
            return false;
        }
        if self.tessellation_shader.as_bool() && !other.tessellation_shader.as_bool() {
            return false;
        }
        if self.sample_rate_shading.as_bool() && !other.sample_rate_shading.as_bool() {
            return false;
        }
        if self.dual_src_blend.as_bool() && !other.dual_src_blend.as_bool() {
            return false;
        }
        if self.logic_op.as_bool() && !other.logic_op.as_bool() {
            return false;
        }
        if self.multi_draw_indirect.as_bool() && !other.multi_draw_indirect.as_bool() {
            return false;
        }
        if self.draw_indirect_first_instance.as_bool()
            && !other.draw_indirect_first_instance.as_bool()
        {
            return false;
        }
        if self.depth_clamp.as_bool() && !other.depth_clamp.as_bool() {
            return false;
        }
        if self.depth_bias_clamp.as_bool() && !other.depth_bias_clamp.as_bool() {
            return false;
        }
        if self.fill_mode_non_solid.as_bool() && !other.fill_mode_non_solid.as_bool() {
            return false;
        }
        if self.depth_bounds.as_bool() && !other.depth_bounds.as_bool() {
            return false;
        }
        if self.wide_lines.as_bool() && !other.wide_lines.as_bool() {
            return false;
        }
        if self.large_points.as_bool() && !other.large_points.as_bool() {
            return false;
        }
        if self.alpha_to_one.as_bool() && !other.alpha_to_one.as_bool() {
            return false;
        }
        if self.multi_viewport.as_bool() && !other.multi_viewport.as_bool() {
            return false;
        }
        if self.sampler_anisotropy.as_bool() && !other.sampler_anisotropy.as_bool() {
            return false;
        }
        if self.texture_compression_etc2.as_bool() && !other.texture_compression_etc2.as_bool() {
            return false;
        }
        if self.texture_compression_astc_ldr.as_bool()
            && !other.texture_compression_astc_ldr.as_bool()
        {
            return false;
        }
        if self.texture_compression_bc.as_bool() && !other.texture_compression_bc.as_bool() {
            return false;
        }
        if self.occlusion_query_precise.as_bool() && !other.occlusion_query_precise.as_bool() {
            return false;
        }
        if self.pipeline_statistics_query.as_bool() && !other.pipeline_statistics_query.as_bool() {
            return false;
        }
        if self.vertex_pipeline_stores_and_atomics.as_bool()
            && !other.vertex_pipeline_stores_and_atomics.as_bool()
        {
            return false;
        }
        if self.fragment_stores_and_atomics.as_bool()
            && !other.fragment_stores_and_atomics.as_bool()
        {
            return false;
        }
        if self.shader_tessellation_and_geometry_point_size.as_bool()
            && !other.shader_tessellation_and_geometry_point_size.as_bool()
        {
            return false;
        }
        if self.shader_image_gather_extended.as_bool()
            && !other.shader_image_gather_extended.as_bool()
        {
            return false;
        }
        if self.shader_storage_image_extended_formats.as_bool()
            && !other.shader_storage_image_extended_formats.as_bool()
        {
            return false;
        }
        if self.shader_storage_image_multisample.as_bool()
            && !other.shader_storage_image_multisample.as_bool()
        {
            return false;
        }
        if self.shader_storage_image_read_without_format.as_bool()
            && !other.shader_storage_image_read_without_format.as_bool()
        {
            return false;
        }
        if self.shader_storage_image_write_without_format.as_bool()
            && !other.shader_storage_image_write_without_format.as_bool()
        {
            return false;
        }
        if self.shader_uniform_buffer_array_dynamic_indexing.as_bool()
            && !other.shader_uniform_buffer_array_dynamic_indexing.as_bool()
        {
            return false;
        }
        if self.shader_sampled_image_array_dynamic_indexing.as_bool()
            && !other.shader_sampled_image_array_dynamic_indexing.as_bool()
        {
            return false;
        }
        if self.shader_storage_buffer_array_dynamic_indexing.as_bool()
            && !other.shader_storage_buffer_array_dynamic_indexing.as_bool()
        {
            return false;
        }
        if self.shader_storage_image_array_dynamic_indexing.as_bool()
            && !other.shader_storage_image_array_dynamic_indexing.as_bool()
        {
            return false;
        }
        if self.shader_clip_distance.as_bool() && !other.shader_clip_distance.as_bool() {
            return false;
        }
        if self.shader_cull_distance.as_bool() && !other.shader_cull_distance.as_bool() {
            return false;
        }
        if self.shader_float64.as_bool() && !other.shader_float64.as_bool() {
            return false;
        }
        if self.shader_int64.as_bool() && !other.shader_int64.as_bool() {
            return false;
        }
        if self.shader_int16.as_bool() && !other.shader_int16.as_bool() {
            return false;
        }
        if self.shader_resource_residency.as_bool() && !other.shader_resource_residency.as_bool() {
            return false;
        }
        if self.shader_resource_min_lod.as_bool() && !other.shader_resource_min_lod.as_bool() {
            return false;
        }
        if self.sparse_binding.as_bool() && !other.sparse_binding.as_bool() {
            return false;
        }
        if self.sparse_residency_buffer.as_bool() && !other.sparse_residency_buffer.as_bool() {
            return false;
        }
        if self.sparse_residency_image2_d.as_bool() && !other.sparse_residency_image2_d.as_bool() {
            return false;
        }
        if self.sparse_residency_image3_d.as_bool() && !other.sparse_residency_image3_d.as_bool() {
            return false;
        }
        if self.sparse_residency2_samples.as_bool() && !other.sparse_residency2_samples.as_bool() {
            return false;
        }
        if self.sparse_residency4_samples.as_bool() && !other.sparse_residency4_samples.as_bool() {
            return false;
        }
        if self.sparse_residency8_samples.as_bool() && !other.sparse_residency8_samples.as_bool() {
            return false;
        }
        if self.sparse_residency16_samples.as_bool() && !other.sparse_residency16_samples.as_bool()
        {
            return false;
        }
        if self.sparse_residency_aliased.as_bool() && !other.sparse_residency_aliased.as_bool() {
            return false;
        }
        if self.variable_multisample_rate.as_bool() && !other.variable_multisample_rate.as_bool() {
            return false;
        }
        if self.inherited_queries.as_bool() && !other.inherited_queries.as_bool() {
            return false;
        }

        true
    }
}

pub struct SurfaceSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}
pub fn query_surface_support_details(
    entry: &ash::Entry,
    instance: ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Result<SurfaceSupportDetails, SurfaceSupportError> {
    if surface == vk::SurfaceKHR::null() {
        return Err(SurfaceSupportError::SurfaceHandleNull);
    }

    let khr_instance = ash::khr::surface::Instance::new(entry, &instance);

    let capabilities =
        unsafe { khr_instance.get_physical_device_surface_capabilities(physical_device, surface) }
            .map_err(|_| SurfaceSupportError::FailedToGetSurfaceCapabilities)?;
    let formats =
        unsafe { khr_instance.get_physical_device_surface_formats(physical_device, surface) }
            .map_err(|_| SurfaceSupportError::FailedToEnumerateSurfaceFormats)?;
    let present_modes =
        unsafe { khr_instance.get_physical_device_surface_present_modes(physical_device, surface) }
            .map_err(|_| SurfaceSupportError::FailedToEnumeratePresentModes)?;

    Ok(SurfaceSupportDetails {
        capabilities,
        formats,
        present_modes,
    })
}
pub fn find_desired_surface_format(
    available: &[vk::SurfaceFormatKHR],
    desired: &[vk::SurfaceFormatKHR],
) -> Result<vk::SurfaceFormatKHR, SurfaceSupportError> {
    desired
        .iter()
        .find(|d| {
            available
                .iter()
                .any(|a| a.format == d.format && a.color_space == d.color_space)
        })
        .copied()
        .ok_or(SurfaceSupportError::NoSuitableDesiredFormat)
}
pub fn find_best_surface_format(
    available: &[vk::SurfaceFormatKHR],
    desired: &[vk::SurfaceFormatKHR],
) -> vk::SurfaceFormatKHR {
    find_desired_surface_format(available, desired).unwrap_or(available[0])
}
pub fn find_extent(
    capabilities: &vk::SurfaceCapabilitiesKHR,
    desired_width: u32,
    desired_height: u32,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        Extent2D::default()
            .width(
                capabilities
                    .min_image_extent
                    .width
                    .max(capabilities.max_image_extent.width.min(desired_width)),
            )
            .height(
                capabilities
                    .min_image_extent
                    .height
                    .max(capabilities.max_image_extent.height.min(desired_height)),
            )
    }
}
pub fn find_present_mode(
    available: &[vk::PresentModeKHR],
    desired: &[vk::PresentModeKHR],
) -> vk::PresentModeKHR {
    desired
        .iter()
        .find(|&&desired_pm| {
            available
                .iter()
                .any(|&available_pm| desired_pm == available_pm)
        })
        .copied()
        .unwrap_or(vk::PresentModeKHR::FIFO)
}

pub unsafe fn destroy_surface(entry: &ash::Entry, instance: &Instance, surface: vk::SurfaceKHR) {
    if !surface.is_null() {
        unsafe {
            let khr_instance = ash::khr::surface::Instance::new(entry, &instance.instance);
            khr_instance.destroy_surface(surface, instance.allocation_callbacks.as_ref());
        }
    }
}
