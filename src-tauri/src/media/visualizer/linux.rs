//! Per-sink-input monitors support PulseAudio and PipeWire's PulseAudio server.
use super::{
    pulse::*,
    spectrum::{self, BANDS, RATE, SIZE},
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    ffi::{CStr, CString},
    sync::Arc,
    time::{Duration, Instant},
};
use tauri::AppHandle;

pub fn roots(_: &AppHandle) -> Vec<u32> {
    vec![std::process::id()]
}
fn engine_process(mut pid: u32, root: u32) -> bool {
    for _ in 0..64 {
        if pid == root {
            return true;
        }
        let path = format!("/proc/{pid}");
        let Ok(exe) = std::fs::read_link(format!("{path}/exe")) else {
            return false;
        };
        let name = exe.file_name().and_then(|s| s.to_str()).unwrap_or_default();
        // Another Spark process (including a private session), external browser or
        // arbitrary child application is a hard boundary, even if it descends from Spark.
        if !matches!(
            name,
            "WebKitWebProcess" | "WebKitNetworkProcess" | "WebKitGPUProcess" | "bwrap"
        ) {
            return false;
        }
        let Ok(stat) = std::fs::read_to_string(format!("{path}/stat")) else {
            return false;
        };
        let Some(parent) = stat
            .rsplit_once(')')
            .and_then(|(_, s)| s.split_whitespace().nth(1))
            .and_then(|s| s.parse::<u32>().ok())
        else {
            return false;
        };
        if parent == pid || parent <= 1 {
            return false;
        }
        pid = parent;
    }
    false
}
struct Discovery {
    api: Arc<Api>,
    root: u32,
    targets: Vec<(u32, u32)>,
    sinks: HashMap<u32, String>,
    done: bool,
    pending: usize,
}
unsafe extern "C" fn inputs(_: Ptr, info: *const SinkInput, end: i32, user: Ptr) {
    let mut d = (&*user.cast::<RefCell<Discovery>>()).borrow_mut();
    if end != 0 {
        d.done = true;
        return;
    }
    let Some(info) = info.as_ref() else {
        return;
    };
    if info.mute != 0
        || info.corked != 0
        || d.targets.len() >= 256
        || (info.has_volume != 0
            && info.volume.values[..usize::from(info.volume.channels).min(32)]
                .iter()
                .all(|v| *v == 0))
    {
        return;
    }
    let property = (d.api.property)(info.proplist, c"application.process.id".as_ptr());
    if property.is_null() {
        return;
    }
    let pid = CStr::from_ptr(property)
        .to_str()
        .ok()
        .and_then(|s| s.parse::<u32>().ok());
    if pid.is_some_and(|pid| engine_process(pid, d.root)) {
        d.targets.push((info.index, info.sink));
    }
}
unsafe extern "C" fn sink(_: Ptr, info: *const Sink, end: i32, user: Ptr) {
    let mut d = (&*user.cast::<RefCell<Discovery>>()).borrow_mut();
    if end != 0 {
        d.pending = d.pending.saturating_sub(1);
        return;
    }
    if let Some(info) = info.as_ref() {
        if !info.monitor_source_name.is_null() {
            if let Ok(name) = CStr::from_ptr(info.monitor_source_name).to_str() {
                d.sinks.insert(info.index, name.to_owned());
            }
        }
    }
}
struct Samples {
    api: Arc<Api>,
    queue: VecDeque<f32>,
    fresh: bool,
}
unsafe extern "C" fn read(stream: Ptr, _: usize, user: Ptr) {
    let mut data = (&*user.cast::<RefCell<Samples>>()).borrow_mut();
    let mut ptr = std::ptr::null();
    let mut size = 0;
    if (data.api.peek)(stream, &mut ptr, &mut size) < 0 || size == 0 {
        return;
    }
    if size <= 768000 {
        let frames = size / 8;
        for index in frames.saturating_sub(SIZE)..frames {
            let value = if ptr.is_null() {
                0.0
            } else {
                let values = ptr.cast::<f32>().add(index * 2);
                (values.read_unaligned() + values.add(1).read_unaligned()) * 0.5
            };
            if data.queue.len() == SIZE {
                data.queue.pop_front();
            }
            data.queue.push_back(value);
        }
        data.fresh = true;
    }
    (data.api.drop_packet)(stream);
}
struct Capture {
    stream: Ptr,
    samples: Box<RefCell<Samples>>,
    sink: u32,
}
impl Drop for Capture {
    fn drop(&mut self) {
        unsafe {
            let samples = self.samples.borrow();
            (samples.api.stream_callback)(self.stream, None, std::ptr::null_mut());
            (samples.api.stream_disconnect)(self.stream);
            (samples.api.stream_unref)(self.stream);
        }
    }
}
pub struct Backend {
    api: Arc<Api>,
    mainloop: Ptr,
    context: Ptr,
    discovery: Box<RefCell<Discovery>>,
    captures: HashMap<u32, Capture>,
    probe: Instant,
}
impl Backend {
    pub fn open(roots: &[u32]) -> Result<Self, &'static str> {
        let root = *roots.first().ok_or("audio_unavailable")?;
        let api = Arc::new(Api::load()?);
        unsafe {
            let mainloop = (api.mainloop_new)();
            if mainloop.is_null() {
                return Err("audio_unavailable");
            }
            let context =
                (api.context_new)((api.mainloop_api)(mainloop), c"Spark equalizer".as_ptr());
            if context.is_null() {
                (api.mainloop_free)(mainloop);
                return Err("audio_unavailable");
            }
            let backend = Self {
                api: api.clone(),
                mainloop,
                context,
                discovery: Box::new(RefCell::new(Discovery {
                    api,
                    root,
                    targets: Vec::new(),
                    sinks: HashMap::new(),
                    done: false,
                    pending: 0,
                })),
                captures: HashMap::new(),
                probe: Instant::now(),
            };
            if (backend.api.context_connect)(context, std::ptr::null(), 0, std::ptr::null_mut()) < 0
            {
                return Err("audio_unavailable");
            }
            backend.wait(|| (backend.api.context_state)(context) == 4)?;
            Ok(backend)
        }
    }
    fn pump(&self) -> Result<(), &'static str> {
        unsafe {
            for _ in 0..32 {
                let result = (self.api.iterate)(self.mainloop, 0, std::ptr::null_mut());
                if result < 0 {
                    return Err("audio_unavailable");
                }
                if result == 0 {
                    break;
                }
            }
            if (self.api.context_state)(self.context) >= 5 {
                return Err("audio_unavailable");
            }
            Ok(())
        }
    }
    fn wait(&self, done: impl Fn() -> bool) -> Result<(), &'static str> {
        let deadline = Instant::now() + Duration::from_millis(750);
        while !done() {
            self.pump()?;
            if Instant::now() >= deadline {
                return Err("audio_unavailable");
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        Ok(())
    }
    fn discover(&mut self) -> Result<(), &'static str> {
        unsafe {
            {
                let mut d = self.discovery.borrow_mut();
                d.targets.clear();
                d.sinks.clear();
                d.done = false;
            }
            let user = (&mut *self.discovery as *mut RefCell<Discovery>).cast();
            let op = (self.api.input_list)(self.context, Some(inputs), user);
            if op.is_null() {
                return Err("audio_unavailable");
            }
            (self.api.operation_unref)(op);
            self.wait(|| self.discovery.borrow().done)?;
            let sinks: HashSet<_> = self
                .discovery
                .borrow()
                .targets
                .iter()
                .map(|(_, s)| *s)
                .collect();
            self.discovery.borrow_mut().pending = sinks.len();
            for sink in sinks {
                let op = (self.api.sink_info)(
                    self.context,
                    sink,
                    Some(crate::media::visualizer::linux::sink),
                    user,
                );
                if op.is_null() {
                    return Err("audio_unavailable");
                }
                (self.api.operation_unref)(op);
            }
            self.wait(|| self.discovery.borrow().pending == 0)?;
            let discovery = self.discovery.borrow();
            self.captures
                .retain(|index, capture| discovery.targets.contains(&(*index, capture.sink)));
            for &(index, sink) in &discovery.targets {
                if self.captures.contains_key(&index) {
                    continue;
                }
                let source = discovery.sinks.get(&sink).ok_or("audio_unavailable")?;
                let source = CString::new(source.as_str()).map_err(|_| "audio_unavailable")?;
                let spec = SampleSpec {
                    format: 5,
                    rate: RATE,
                    channels: 2,
                };
                let stream = (self.api.stream_new)(
                    self.context,
                    c"Spark spectrum monitor".as_ptr(),
                    &spec,
                    std::ptr::null(),
                );
                if stream.is_null() {
                    return Err("audio_unavailable");
                }
                let mut capture = Capture {
                    stream,
                    sink,
                    samples: Box::new(RefCell::new(Samples {
                        api: self.api.clone(),
                        queue: VecDeque::with_capacity(SIZE),
                        fresh: false,
                    })),
                };
                (self.api.stream_callback)(
                    stream,
                    Some(read),
                    (&mut *capture.samples as *mut RefCell<Samples>).cast(),
                );
                let attr = BufferAttr {
                    maxlength: u32::MAX,
                    tlength: u32::MAX,
                    prebuf: u32::MAX,
                    minreq: u32::MAX,
                    fragsize: (SIZE * 8) as u32,
                };
                if (self.api.stream_monitor)(stream, index) < 0
                    || (self.api.stream_connect)(stream, source.as_ptr(), &attr, 0) < 0
                {
                    return Err("audio_unavailable");
                }
                self.captures.insert(index, capture);
            }
            self.probe = Instant::now() + Duration::from_secs(1);
            Ok(())
        }
    }
    pub fn read(&mut self) -> Result<[f32; BANDS], &'static str> {
        if Instant::now() >= self.probe {
            self.discover()?;
        }
        self.pump()?;
        if self.captures.is_empty() {
            return Err("audio_unavailable");
        }
        let mut spectra = Vec::new();
        for capture in self.captures.values_mut() {
            if unsafe { (self.api.stream_state)(capture.stream) } >= 3 {
                return Err("audio_unavailable");
            }
            let mut samples = capture.samples.borrow_mut();
            if samples.fresh {
                spectra.push(spectrum::analyze(samples.queue.make_contiguous()));
                samples.fresh = false;
            }
        }
        Ok(spectrum::combine(spectra.into_iter()))
    }
}
impl Drop for Backend {
    fn drop(&mut self) {
        self.captures.clear();
        unsafe {
            (self.api.context_disconnect)(self.context);
            (self.api.context_unref)(self.context);
            (self.api.mainloop_free)(self.mainloop);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn process_filter_requires_the_normal_profile_root() {
        assert!(engine_process(std::process::id(), std::process::id()));
        assert!(!engine_process(0, std::process::id()));
        assert!(!engine_process(u32::MAX, std::process::id()));
    }
    #[test]
    fn pulse_abi_keeps_the_native_field_alignment() {
        assert_eq!(std::mem::size_of::<SampleSpec>(), 12);
        assert_eq!(std::mem::size_of::<ChannelMap>(), 132);
        assert_eq!(std::mem::size_of::<Volume>(), 132);
        if std::mem::size_of::<usize>() == 8 {
            assert_eq!(std::mem::offset_of!(SinkInput, proplist), 344);
            assert_eq!(std::mem::offset_of!(Sink, monitor_source_name), 312);
        }
    }
}
