use crate::utils::AsBool;
use ash::vk;
use ash::vk::{ExtendsPhysicalDeviceFeatures2, ExtendsSwapchainCreateInfoKHR, StructureType};
use std::ffi::c_void;
use std::ptr::null_mut;
use std::{mem, ptr};

#[derive(Default)]
pub struct GenericFeatureChain {
    pub(crate) nodes: Vec<GenericFeatureNode>,
}

impl GenericFeatureChain {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_node(&mut self, node: GenericFeatureNode) {
        if let Some(duplicate) = self
            .nodes
            .iter_mut()
            .find(|x| x.structure_type == node.structure_type)
        {
            duplicate.combine(&node);
        } else {
            self.nodes.push(node);
        }
    }
    pub fn contains_all(&self, extensions_requested: &Self) -> bool {
        if self.nodes.len() != extensions_requested.nodes.len() {
            return false;
        }

        self.nodes
            .iter()
            .zip(extensions_requested.nodes.iter())
            .all(|(node, extension)| extension.contains(node))
    }
    pub fn find_and_match(&self, extensions_requested: &Self) -> bool {
        extensions_requested.nodes.iter().all(|requested_node| {
            self.nodes.iter().any(|supported_node| {
                supported_node.structure_type == requested_node.structure_type
                    && supported_node.contains(requested_node)
            })
        })
    }
    //TODO: Switch this with a trait that has push next or do it manually without helper method
    pub unsafe fn chain_up_physical_device<'b>(
        &'b mut self,
        feats2: &mut vk::PhysicalDeviceFeatures2<'b>,
    ) {
        for node in &mut self.nodes {
            *feats2 = feats2.push_next(node);
        }
    }
    pub unsafe fn chain_up_swapchain<'b>(
        &'b mut self,
        feats2: &mut vk::SwapchainCreateInfoKHR<'b>,
    ) {
        for node in &mut self.nodes {
            *feats2 = feats2.push_next(node);
        }
    }

    pub fn combine(&mut self, right: Self) {
        for right_node in right.nodes {
            let mut already_contained = false;
            for left_node in &mut self.nodes {
                if left_node.structure_type == right_node.structure_type {
                    left_node.combine(&right_node);
                    already_contained = true;
                }
            }
            if !already_contained {
                self.nodes.push(right_node);
            }
        }
    }
}

impl Clone for GenericFeatureChain {
    // Resets p next chain and clones
    fn clone(&self) -> Self {
        let chain = Self {
            nodes: self
                .nodes
                .iter()
                .map(|x| unsafe {
                    let mut node = x.clone_raw();
                    node.p_next = null_mut();
                    node
                })
                .collect(),
        };
        chain
    }
}

pub const NODE_FIELD_CAPACITY: usize = 256;

#[repr(C)]
pub struct GenericFeatureNode {
    pub(crate) structure_type: StructureType,
    p_next: *mut c_void,
    fields: [vk::Bool32; NODE_FIELD_CAPACITY],
}

impl GenericFeatureNode {
    pub fn from_device_feature<T>(feature: T) -> GenericFeatureNode
    where
        T: ExtendsPhysicalDeviceFeatures2,
    {
        unsafe { Self::from_raw(feature) }
    }

    pub fn from_swapchain_feature<T>(feature: T) -> GenericFeatureNode
    where
        T: ExtendsSwapchainCreateInfoKHR,
    {
        unsafe { Self::from_raw(feature) }
    }

    /// # Safety: feature must be a chainable vulkan feature struct
    pub unsafe fn from_raw<T>(feature: T) -> GenericFeatureNode {
        assert!(
            mem::size_of::<T>() <= mem::size_of::<Self>(),
            "Not enough space to copy fields."
        );
        let mut node = Self::default();
        unsafe {
            let node_ptr = &mut node as *mut GenericFeatureNode as *mut u8;
            let src_ptr = &feature as *const T as *const u8;
            let copy_size = std::cmp::min(size_of::<T>(), size_of::<GenericFeatureNode>());
            ptr::copy_nonoverlapping(src_ptr, node_ptr, copy_size);
        }
        node.p_next = null_mut();
        node
    }

    pub fn contains(&self, contains: &Self) -> bool {
        assert_eq!(
            self.structure_type, contains.structure_type,
            "Nodes' types should match"
        );

        self.fields
            .iter()
            .zip(contains.fields.iter())
            .all(|(req, has)| !req.as_bool() || has.as_bool())
    }

    pub fn combine(&mut self, right: &Self) {
        assert_eq!(
            self.structure_type, right.structure_type,
            "Nodes' types should match"
        );
        for (field, right_field) in self.fields.iter_mut().zip(right.fields.iter()) {
            *field = vk::Bool32::from(field.as_bool() || right_field.as_bool())
        }
    }

    /// SAFETY: Must not live more than original feature chain lives
    pub unsafe fn clone_raw(&self) -> Self {
        Self {
            structure_type: self.structure_type.clone(),
            p_next: self.p_next.clone(),
            fields: self.fields.clone(),
        }
    }
}

impl Default for GenericFeatureNode {
    fn default() -> Self {
        Self {
            structure_type: vk::StructureType::from_raw(0),
            p_next: null_mut(),
            fields: [vk::FALSE; NODE_FIELD_CAPACITY],
        }
    }
}

unsafe impl ExtendsPhysicalDeviceFeatures2 for GenericFeatureNode {}
unsafe impl ExtendsSwapchainCreateInfoKHR for GenericFeatureNode {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_contains_subset() {
        let f1 = vk::PhysicalDeviceVulkan11Features::default().multiview(true);
        let n1 = GenericFeatureNode::from_device_feature(f1);

        let f2 = vk::PhysicalDeviceVulkan11Features::default()
            .multiview(true)
            .shader_draw_parameters(true);
        let n2 = GenericFeatureNode::from_device_feature(f2);

        assert!(n1.contains(&n2));

        assert!(!n2.contains(&n1));
    }

    #[test]
    fn add_node_merges_same_structure_type() {
        let mut chain = GenericFeatureChain::new();
        chain.add_node(GenericFeatureNode::from_device_feature(
            vk::PhysicalDeviceVulkan11Features::default().multiview(true),
        ));
        chain.add_node(GenericFeatureNode::from_device_feature(
            vk::PhysicalDeviceVulkan11Features::default().shader_draw_parameters(true),
        ));

        assert_eq!(chain.nodes.len(), 1);

        let mut requested = GenericFeatureChain::new();
        requested.add_node(GenericFeatureNode::from_device_feature(
            vk::PhysicalDeviceVulkan11Features::default()
                .multiview(true)
                .shader_draw_parameters(true),
        ));

        assert!(chain.contains_all(&requested));
    }

    #[test]
    fn clone_resets_p_next_chain() {
        let mut chain = GenericFeatureChain::new();
        chain.add_node(GenericFeatureNode::from_device_feature(
            vk::PhysicalDeviceVulkan11Features::default().multiview(true),
        ));

        let cloned = chain.clone();
        assert!(cloned.nodes.iter().all(|n| n.p_next.is_null()));
    }
}
