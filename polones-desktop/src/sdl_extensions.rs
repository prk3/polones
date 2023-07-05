extern "C" {
    // Added in SDL 2.4
    fn SDL_GetDefaultAudioInfo(
        name: *mut *mut u8,
        info: *mut SDL_AudioSpec,
        iscapture: c_int,
    ) -> c_int;
    fn SDL_free(data: *mut u8);
}

#[allow(non_camel_case_types)]
type c_int = i32;

#[allow(non_camel_case_types)]
type SDL_AudioCallback = unsafe extern "C" fn(*mut u8, *mut u8, c_int);

#[repr(C)]
struct SDL_AudioSpec {
    freq: c_int,
    format: u16,
    channels: u8,
    silence: u8,
    samples: u16,
    size: u32,
    callback: SDL_AudioCallback,
    userdata: *mut u8,
}

pub fn get_default_playback_device_name() -> Option<String> {
    let mut name: *mut u8 = std::ptr::null_mut();
    let mut info = SDL_AudioSpec {
        freq: 0,
        format: 0,
        channels: 0,
        silence: 0,
        samples: 0,
        size: 0,
        callback: {
            unsafe extern "C" fn noop_audio_callback(_: *mut u8, _: *mut u8, _: c_int) {}
            noop_audio_callback
        },
        userdata: std::ptr::null_mut(),
    };
    let iscapture = 0;

    let error = unsafe {
        SDL_GetDefaultAudioInfo(
            &mut name as *mut *mut u8,
            &mut info as *mut SDL_AudioSpec,
            iscapture,
        )
    };

    if error != 0 {
        return None;
    }

    if name.is_null() {
        return None;
    }

    let mut buffer = Vec::new();

    for i in 0..1000 {
        let byte = unsafe { *name.offset(i) };
        if byte != 0 {
            buffer.push(byte);
        } else {
            break;
        }
    }

    unsafe {
        SDL_free(name);
    }

    String::from_utf8(buffer).ok()
}
