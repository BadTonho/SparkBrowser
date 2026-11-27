//! Process-scoped WASAPI loopback. Never opens a microphone or a global output monitor.
use super::spectrum::{self, BANDS, RATE, SIZE};
use crate::sync::Mutex;
use std::{collections::VecDeque, mem::ManuallyDrop, sync::mpsc, time::Duration};
use windows::Win32::{
    Media::Audio::*,
    System::{Com::StructuredStorage::*, Com::*, Variant::VT_BLOB},
};
use windows_core::{implement, Interface};

// WASAPI's activated audio client is free-threaded. Both the completion thread and the
// owning capture worker use MTA; no interface is ever sent to the UI apartment.
struct Activated(IAudioClient);
unsafe impl Send for Activated {}

#[implement(IActivateAudioInterfaceCompletionHandler)]
struct Completion {
    sender: Mutex<Option<mpsc::Sender<windows_core::Result<Activated>>>>,
    _parameters: Box<AUDIOCLIENT_ACTIVATION_PARAMS>,
}
impl IActivateAudioInterfaceCompletionHandler_Impl for Completion_Impl {
    fn ActivateCompleted(
        &self,
        operation: windows_core::Ref<IActivateAudioInterfaceAsyncOperation>,
    ) -> windows_core::Result<()> {
        let result = (|| unsafe {
            let mut status = windows_core::HRESULT(0);
            let mut interface = None;
            operation
                .ok()?
                .GetActivateResult(&mut status, &mut interface)?;
            status.ok()?;
            Ok(Activated(
                interface
                    .ok_or_else(windows_core::Error::from_thread)?
                    .cast()?,
            ))
        })();
        if let Ok(mut slot) = self.sender.lock() {
            if let Some(sender) = slot.take() {
                crate::error_log::ignored(sender.send(result));
            }
        }
        Ok(())
    }
}

struct Capture {
    client: IAudioClient,
    reader: IAudioCaptureClient,
    samples: VecDeque<f32>,
}
impl Capture {
    unsafe fn open(pid: u32) -> Result<Self, &'static str> {
        let (sender, receive) = mpsc::channel();
        let mut parameters = Box::new(AUDIOCLIENT_ACTIVATION_PARAMS {
            ActivationType: AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK,
            Anonymous: AUDIOCLIENT_ACTIVATION_PARAMS_0 {
                ProcessLoopbackParams: AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS {
                    TargetProcessId: pid,
                    ProcessLoopbackMode: PROCESS_LOOPBACK_MODE_INCLUDE_TARGET_PROCESS_TREE,
                },
            },
        });
        // The BLOB borrows the handler-owned Box; PROPVARIANT's automatic
        // PropVariantClear would otherwise CoTaskMemFree Rust-owned memory.
        let variant = ManuallyDrop::new(PROPVARIANT {
            Anonymous: PROPVARIANT_0 {
                Anonymous: ManuallyDrop::new(PROPVARIANT_0_0 {
                    vt: VT_BLOB,
                    Anonymous: PROPVARIANT_0_0_0 {
                        blob: BLOB {
                            cbSize: std::mem::size_of::<AUDIOCLIENT_ACTIVATION_PARAMS>() as u32,
                            pBlobData: (&mut *parameters as *mut AUDIOCLIENT_ACTIVATION_PARAMS)
                                .cast(),
                        },
                    },
                    ..Default::default()
                }),
            },
        });
        let handler: IActivateAudioInterfaceCompletionHandler = Completion {
            sender: Mutex::new(Some(sender)),
            _parameters: parameters,
        }
        .into();
        let _operation = ActivateAudioInterfaceAsync(
            VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK,
            &IAudioClient::IID,
            Some(&*variant),
            &handler,
        )
        .map_err(|_| "audio_unavailable")?;
        let Activated(client) = receive
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| "audio_unavailable")?
            .map_err(|_| "audio_unavailable")?;
        let format = WAVEFORMATEX {
            wFormatTag: 3,
            nChannels: 2,
            nSamplesPerSec: RATE,
            nAvgBytesPerSec: RATE * 8,
            nBlockAlign: 8,
            wBitsPerSample: 32,
            cbSize: 0,
        };
        client
            .Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_LOOPBACK
                    | AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM
                    | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
                1_000_000,
                0,
                &format,
                None,
            )
            .map_err(|_| "audio_unavailable")?;
        let reader = client
            .GetService::<IAudioCaptureClient>()
            .map_err(|_| "audio_unavailable")?;
        client.Start().map_err(|_| "audio_unavailable")?;
        Ok(Self {
            client,
            reader,
            samples: VecDeque::with_capacity(SIZE),
        })
    }
    fn read(&mut self) -> Result<[f32; BANDS], &'static str> {
        let mut received = false;
        unsafe {
            // A bounded drain also protects the worker from a stalled/device-reset stream.
            for _ in 0..32 {
                if self
                    .reader
                    .GetNextPacketSize()
                    .map_err(|_| "audio_unavailable")?
                    == 0
                {
                    break;
                }
                let mut data = std::ptr::null_mut();
                let mut frames = 0;
                let mut flags = 0;
                self.reader
                    .GetBuffer(&mut data, &mut frames, &mut flags, None, None)
                    .map_err(|_| "audio_unavailable")?;
                if frames > 96000 {
                    crate::error_log::ignored(self.reader.ReleaseBuffer(frames));
                    return Err("audio_unavailable");
                }
                received = true;
                let silent = flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0 || data.is_null();
                for frame in 0..frames as usize {
                    let sample = if silent {
                        0.0
                    } else {
                        let values = data.cast::<f32>().add(frame * 2);
                        (values.read_unaligned() + values.add(1).read_unaligned()) * 0.5
                    };
                    if self.samples.len() == SIZE {
                        self.samples.pop_front();
                    }
                    self.samples.push_back(sample);
                }
                self.reader
                    .ReleaseBuffer(frames)
                    .map_err(|_| "audio_unavailable")?;
            }
        }
        if !received {
            return Ok([0.0; BANDS]);
        }
        Ok(spectrum::analyze(self.samples.make_contiguous()))
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        unsafe {
            crate::error_log::ignored(self.client.Stop());
        }
    }
}

pub struct Backend {
    captures: Vec<Capture>,
}
impl Backend {
    pub fn open(roots: &[u32]) -> Result<Self, &'static str> {
        let mut captures = Vec::new();
        for &pid in roots {
            captures.push(unsafe { Capture::open(pid)? });
        }
        if captures.is_empty() {
            return Err("audio_unavailable");
        }
        Ok(Self { captures })
    }
    pub fn read(&mut self) -> Result<[f32; BANDS], &'static str> {
        let spectra = self
            .captures
            .iter_mut()
            .map(Capture::read)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(spectrum::combine(spectra.into_iter()))
    }
}
pub struct Apartment;
impl Apartment {
    pub fn enter() -> Result<Self, &'static str> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .map_err(|_| "audio_unavailable")?;
        }
        Ok(Self)
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

pub fn roots(app: &tauri::AppHandle) -> Vec<u32> {
    use tauri::Manager;
    let (sender, receiver) = mpsc::channel();
    let app = app.clone();
    let handle = app.clone();
    crate::error_log::ignored(app.run_on_main_thread(move || {
        let tabs = handle
            .state::<crate::state::browser_state::BrowserState>()
            .snapshot_tabs();
        for tab in tabs {
            let media = handle.state::<crate::media::MediaService>().tab(&tab.id);
            if !media.audible || media.muted || media.volume == 0 {
                continue;
            }
            if let Some(view) =
                handle.get_webview(&crate::state::tab_model::webview_label_for_tab(&tab.id))
            {
                let sender = sender.clone();
                crate::error_log::ignored(view.with_webview(move |platform| unsafe {
                    let mut pid = 0;
                    if let Ok(core) = platform.controller().CoreWebView2() {
                        crate::error_log::ignored(core.BrowserProcessId(&mut pid));
                    }
                    if pid != 0 {
                        crate::error_log::ignored(sender.send(pid));
                    }
                }));
            }
        }
    }));
    let deadline = std::time::Instant::now() + Duration::from_millis(500);
    let mut roots = Vec::new();
    while let Ok(pid) =
        receiver.recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
    {
        roots.push(pid);
    }
    roots.sort_unstable();
    roots.dedup();
    roots
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Requires Windows process-loopback support; captures only this test process"]
    fn native_loopback_activation_and_silent_packets() {
        let _apartment = Apartment::enter().unwrap();
        let mut backend = Backend::open(&[std::process::id()]).unwrap();
        for _ in 0..5 {
            let bands = backend.read().unwrap();
            assert!(bands
                .iter()
                .all(|value| value.is_finite() && (0.0..=1.0).contains(value)));
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}
