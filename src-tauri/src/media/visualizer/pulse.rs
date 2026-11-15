//! Minimal libpulse ABI, loaded only when the effect is used. No microphone APIs.
use libloading::Library;
use std::ffi::{c_char, c_int, c_void};
pub type Ptr = *mut c_void;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SampleSpec {
    pub format: c_int,
    pub rate: u32,
    pub channels: u8,
}
#[repr(C)]
pub struct ChannelMap {
    pub channels: u8,
    pub map: [c_int; 32],
}
#[repr(C)]
pub struct Volume {
    pub channels: u8,
    pub values: [u32; 32],
}
#[repr(C)]
pub struct SinkInput {
    pub index: u32,
    pub name: *const c_char,
    pub owner_module: u32,
    pub client: u32,
    pub sink: u32,
    pub sample_spec: SampleSpec,
    pub channel_map: ChannelMap,
    pub volume: Volume,
    pub buffer_usec: u64,
    pub sink_usec: u64,
    pub resample_method: *const c_char,
    pub driver: *const c_char,
    pub mute: c_int,
    pub proplist: Ptr,
    pub corked: c_int,
    pub has_volume: c_int,
    pub volume_writable: c_int,
    pub format: Ptr,
}
// Prefix of pa_sink_info. Only the fields through monitor_source_name are accessed.
#[repr(C)]
pub struct Sink {
    pub index: u32,
    pub name: *const c_char,
    pub description: *const c_char,
    pub sample_spec: SampleSpec,
    pub channel_map: ChannelMap,
    pub owner_module: u32,
    pub volume: Volume,
    pub mute: c_int,
    pub monitor_source: u32,
    pub monitor_source_name: *const c_char,
}
#[repr(C)]
pub struct BufferAttr {
    pub maxlength: u32,
    pub tlength: u32,
    pub prebuf: u32,
    pub minreq: u32,
    pub fragsize: u32,
}
pub type InputCallback = unsafe extern "C" fn(Ptr, *const SinkInput, c_int, Ptr);
pub type SinkCallback = unsafe extern "C" fn(Ptr, *const Sink, c_int, Ptr);
pub type ReadCallback = unsafe extern "C" fn(Ptr, usize, Ptr);

pub struct Api {
    _library: Library,
    pub mainloop_new: unsafe extern "C" fn() -> Ptr,
    pub mainloop_free: unsafe extern "C" fn(Ptr),
    pub mainloop_api: unsafe extern "C" fn(Ptr) -> Ptr,
    pub iterate: unsafe extern "C" fn(Ptr, c_int, *mut c_int) -> c_int,
    pub context_new: unsafe extern "C" fn(Ptr, *const c_char) -> Ptr,
    pub context_connect: unsafe extern "C" fn(Ptr, *const c_char, c_int, Ptr) -> c_int,
    pub context_state: unsafe extern "C" fn(Ptr) -> c_int,
    pub context_disconnect: unsafe extern "C" fn(Ptr),
    pub context_unref: unsafe extern "C" fn(Ptr),
    pub input_list: unsafe extern "C" fn(Ptr, Option<InputCallback>, Ptr) -> Ptr,
    pub sink_info: unsafe extern "C" fn(Ptr, u32, Option<SinkCallback>, Ptr) -> Ptr,
    pub operation_unref: unsafe extern "C" fn(Ptr),
    pub property: unsafe extern "C" fn(Ptr, *const c_char) -> *const c_char,
    pub stream_new:
        unsafe extern "C" fn(Ptr, *const c_char, *const SampleSpec, *const ChannelMap) -> Ptr,
    pub stream_monitor: unsafe extern "C" fn(Ptr, u32) -> c_int,
    pub stream_callback: unsafe extern "C" fn(Ptr, Option<ReadCallback>, Ptr),
    pub stream_connect: unsafe extern "C" fn(Ptr, *const c_char, *const BufferAttr, c_int) -> c_int,
    pub stream_state: unsafe extern "C" fn(Ptr) -> c_int,
    pub peek: unsafe extern "C" fn(Ptr, *mut *const c_void, *mut usize) -> c_int,
    pub drop_packet: unsafe extern "C" fn(Ptr) -> c_int,
    pub stream_disconnect: unsafe extern "C" fn(Ptr) -> c_int,
    pub stream_unref: unsafe extern "C" fn(Ptr),
}
impl Api {
    pub fn load() -> Result<Self, &'static str> {
        unsafe {
            let library = Library::new("libpulse.so.0").map_err(|_| "audio_unavailable")?;
            macro_rules! load {
                ($name:literal) => {
                    *library
                        .get(concat!($name, "\0").as_bytes())
                        .map_err(|_| "audio_unavailable")?
                };
            }
            Ok(Self {
                mainloop_new: load!("pa_mainloop_new"),
                mainloop_free: load!("pa_mainloop_free"),
                mainloop_api: load!("pa_mainloop_get_api"),
                iterate: load!("pa_mainloop_iterate"),
                context_new: load!("pa_context_new"),
                context_connect: load!("pa_context_connect"),
                context_state: load!("pa_context_get_state"),
                context_disconnect: load!("pa_context_disconnect"),
                context_unref: load!("pa_context_unref"),
                input_list: load!("pa_context_get_sink_input_info_list"),
                sink_info: load!("pa_context_get_sink_info_by_index"),
                operation_unref: load!("pa_operation_unref"),
                property: load!("pa_proplist_gets"),
                stream_new: load!("pa_stream_new"),
                stream_monitor: load!("pa_stream_set_monitor_stream"),
                stream_callback: load!("pa_stream_set_read_callback"),
                stream_connect: load!("pa_stream_connect_record"),
                stream_state: load!("pa_stream_get_state"),
                peek: load!("pa_stream_peek"),
                drop_packet: load!("pa_stream_drop"),
                stream_disconnect: load!("pa_stream_disconnect"),
                stream_unref: load!("pa_stream_unref"),
                _library: library,
            })
        }
    }
}
