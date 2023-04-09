use ash::extensions::ext::DebugUtils;
use ash::{vk, Instance};
use std::ffi::CStr;

pub struct VulkanDebug<'a> {
    debug_extension: &'a DebugUtils,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl<'a> VulkanDebug<'a> {
    pub(super) fn create(debug_extension: &'a DebugUtils) -> VulkanDebug<'a> {
        // Enable filtering by message severity and type. General and verbose levels seem to produce
        // too much noise related to physical device selection, so I turned them off.
        // vulkan-tutorial.com also shows how to enable this for creating instances, but the ash
        // example doesn't include this.
        let severity_filter = vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
            | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING;
        let type_filter = vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE;
        let info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(severity_filter)
            .message_type(type_filter)
            .pfn_user_callback(Some(callback));
        let messenger =
            unsafe { debug_extension.create_debug_utils_messenger(&info, None) }.unwrap();
        VulkanDebug {
            debug_extension,
            messenger,
        }
    }
}

impl Drop for VulkanDebug<'_> {
    fn drop(&mut self) {
        unsafe {
            self.debug_extension
                .destroy_debug_utils_messenger(self.messenger, None)
        };
    }
}

unsafe extern "system" fn callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    // In case of validation layer events, just the message field contains all the metadata the
    // event provides, as well as some data that is not accessible separately, including a URL to a
    // site explaining the problem. The most interesting part of the metadata is the queue/command
    // buffer/object labels/pointers, which the engine could tie back to some higher level
    // structures.
    let callback_data = *p_callback_data;
    assert!(!callback_data.p_message.is_null());
    let message = CStr::from_ptr(callback_data.p_message).to_string_lossy();
    let level = if message_severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) {
        log::Level::Error
    } else if message_severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING) {
        log::Level::Warn
    } else if message_severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::INFO) {
        log::Level::Info
    } else {
        log::Level::Trace
    };
    log::log!(level, "vulkan debug event: {message}");
    vk::FALSE
}
