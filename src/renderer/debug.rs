use crate::renderer::util::Dev;
use ash::ext::debug_utils;
use ash::vk;
use ash::vk::Handle;
use std::ffi::CStr;

pub fn create_debug_messenger(debug_ext: &debug_utils::Instance) -> vk::DebugUtilsMessengerEXT {
    // vulkan-tutorial.com also shows how to enable this for creating instances, but the ash
    // example doesn't include this.
    let severity_filter = vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
        | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE;
    let type_filter = vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE;
    let info = vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(severity_filter)
        .message_type(type_filter)
        .pfn_user_callback(Some(callback));
    unsafe { debug_ext.create_debug_utils_messenger(&info, None) }.unwrap()
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

    // This can be NULL only in when using DEVICE_ADDRESS_BINDING message type.
    let message = CStr::from_ptr(callback_data.p_message).to_string_lossy();

    if !callback_data.p_message_id_name.is_null() {
        let message_id = CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy();

        // mesa prints some pointless device/loader selection logs when info level is enabled. Info
        // also enables debug printf, so let's filter this out here instead.
        if message_id == "Loader Message" {
            return vk::FALSE;
        }

        if message_id == "WARNING-DEBUG-PRINTF" {
            if let Some((_, message)) = message.split_once("vkQueueSubmit():  ") {
                log::trace!("{message}");
                return vk::FALSE;
            }
        }
    }
    let level = if message_severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) {
        log::Level::Error
    } else if message_severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING) {
        log::Level::Warn
    } else if message_severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::INFO) {
        log::Level::Info
    } else {
        log::Level::Trace
    };
    log::log!(level, "{message}");
    vk::FALSE
}

pub fn begin_label(buf: vk::CommandBuffer, text: &CStr, color: [u8; 3], dev: &Dev) {
    let color = [
        color[0] as f32 / 255.,
        color[1] as f32 / 255.,
        color[2] as f32 / 255.,
        1.,
    ];
    let label = vk::DebugUtilsLabelEXT::default()
        .label_name(text)
        .color(color);
    unsafe { dev.debug_ext.cmd_begin_debug_utils_label(buf, &label) };
}

pub fn end_label(buf: vk::CommandBuffer, dev: &Dev) {
    unsafe { dev.debug_ext.cmd_end_debug_utils_label(buf) };
}

pub fn set_label<T: Handle>(object: T, name: &CStr, dev: &Dev) {
    let name_info = vk::DebugUtilsObjectNameInfoEXT::default()
        .object_handle(object)
        .object_name(name);
    unsafe { dev.debug_ext.set_debug_utils_object_name(&name_info) }.unwrap();
}
