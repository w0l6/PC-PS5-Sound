use crate::audio::com::{propvariant_to_string, take_pwstr};
use crate::audio::types::AudioDevice;
use crate::error::{AppError, AppResult};
use windows::Win32::Devices::FunctionDiscovery::{
    PKEY_Device_DeviceDesc, PKEY_Device_FriendlyName, PKEY_DeviceInterface_FriendlyName,
};
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Media::Audio::Endpoints::{IAudioEndpointVolume, IAudioMeterInformation};
use windows::Win32::Media::Audio::{
    eCapture, eCommunications, eConsole, eRender, IMMDevice, IMMDeviceEnumerator,
    MMDeviceEnumerator, PKEY_AudioEndpoint_FormFactor, DEVICE_STATE, DEVICE_STATE_ACTIVE,
    DEVICE_STATE_DISABLED, DEVICE_STATE_NOTPRESENT, DEVICE_STATE_UNPLUGGED, DEVICE_STATEMASK_ALL,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL, STGM_READ};
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;

pub struct EnumeratedDevice {
    pub info: AudioDevice,
    pub meter: Option<IAudioMeterInformation>,
}

pub struct Enumeration {
    pub devices: Vec<EnumeratedDevice>,
    pub default_output_name: Option<String>,
    pub default_output_form_factor: String,
}

pub fn create_enumerator() -> AppResult<IMMDeviceEnumerator> {
    unsafe {
        CoCreateInstance(
            &MMDeviceEnumerator,
            None::<&windows::core::IUnknown>,
            CLSCTX_ALL,
        )
        .map_err(AppError::from)
    }
}

fn device_id(device: &IMMDevice) -> Option<String> {
    unsafe { device.GetId().ok().map(|pw| take_pwstr(pw)) }
}

fn default_id(
    enumerator: &IMMDeviceEnumerator,
    flow: windows::Win32::Media::Audio::EDataFlow,
    role: windows::Win32::Media::Audio::ERole,
) -> Option<String> {
    unsafe {
        enumerator
            .GetDefaultAudioEndpoint(flow, role)
            .ok()
            .and_then(|device| device_id(&device))
    }
}

unsafe fn property_string(store: &IPropertyStore, key: &PROPERTYKEY) -> String {
    match store.GetValue(key) {
        Ok(value) => propvariant_to_string(&value),
        Err(_) => String::new(),
    }
}

pub unsafe fn friendly_name(device: &IMMDevice) -> String {
    match device.OpenPropertyStore(STGM_READ) {
        Ok(store) => {
            let name = property_string(&store, &PKEY_Device_FriendlyName);
            if name.is_empty() {
                property_string(&store, &PKEY_Device_DeviceDesc)
            } else {
                name
            }
        }
        Err(_) => String::new(),
    }
}

fn form_factor_info(value: u32) -> (&'static str, &'static str) {
    match value {
        0 => ("network", "Reseau distant"),
        1 => ("speakers", "Haut-parleurs"),
        2 => ("linelevel", "Niveau ligne"),
        3 => ("headphones", "Casque"),
        4 => ("microphone", "Microphone"),
        5 => ("headset", "Casque-micro"),
        6 => ("handset", "Combine"),
        7 => ("digital", "Passthrough numerique"),
        8 => ("spdif", "SPDIF"),
        9 => ("hdmi", "Ecran / HDMI"),
        _ => ("unknown", "Inconnu"),
    }
}

fn form_factor_key(device: &IMMDevice) -> String {
    unsafe {
        match device.OpenPropertyStore(STGM_READ) {
            Ok(store) => {
                let raw = property_string(&store, &PKEY_AudioEndpoint_FormFactor);
                let number = raw.trim().parse::<u32>().unwrap_or(10);
                form_factor_info(number).0.to_string()
            }
            Err(_) => "unknown".to_string(),
        }
    }
}

fn state_info(raw: u32) -> (&'static str, &'static str) {
    if raw & DEVICE_STATE_ACTIVE.0 != 0 {
        ("active", "Connecte")
    } else if raw & DEVICE_STATE_UNPLUGGED.0 != 0 {
        ("unplugged", "Deconnecte")
    } else if raw & DEVICE_STATE_DISABLED.0 != 0 {
        ("disabled", "Desactive")
    } else if raw & DEVICE_STATE_NOTPRESENT.0 != 0 {
        ("notpresent", "Non present")
    } else {
        ("unknown", "Inconnu")
    }
}

/// Enumere tous les endpoints de capture, quel que soit leur etat,
/// et lit leurs proprietes, volumes et metres.
pub fn enumerate_capture_devices(enumerator: &IMMDeviceEnumerator) -> AppResult<Enumeration> {
    unsafe {
        let collection = enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE(DEVICE_STATEMASK_ALL))?;
        let count = collection.GetCount()?;

        let default_capture_id = default_id(enumerator, eCapture, eConsole);
        let default_comm_capture_id = default_id(enumerator, eCapture, eCommunications);

        let render_device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).ok();
        let default_output_name = render_device
            .as_ref()
            .map(|device| friendly_name(device))
            .filter(|name| !name.is_empty());
        let default_output_form_factor = render_device
            .as_ref()
            .map(form_factor_key)
            .unwrap_or_else(|| "unknown".to_string());

        let mut devices = Vec::with_capacity(count as usize);

        for index in 0..count {
            let device = collection.Item(index)?;
            let Some(id) = device_id(&device) else {
                continue;
            };

            let raw_state = device.GetState().map(|state| state.0).unwrap_or(0);
            let (state, state_label) = state_info(raw_state);

            let store = device.OpenPropertyStore(STGM_READ).ok();
            let friendly = store
                .as_ref()
                .map(|store| property_string(store, &PKEY_Device_FriendlyName))
                .unwrap_or_default();
            let description = store
                .as_ref()
                .map(|store| property_string(store, &PKEY_Device_DeviceDesc))
                .unwrap_or_default();
            let interface = store
                .as_ref()
                .map(|store| property_string(store, &PKEY_DeviceInterface_FriendlyName))
                .unwrap_or_default();
            let form_factor_raw = store
                .as_ref()
                .map(|store| property_string(store, &PKEY_AudioEndpoint_FormFactor))
                .unwrap_or_default();
            let form_factor_number = form_factor_raw.trim().parse::<u32>().unwrap_or(10);
            let (form_factor, form_factor_label) = form_factor_info(form_factor_number);

            let volume_interface = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).ok();
            let (volume_percent, muted) = volume_interface
                .as_ref()
                .map(|volume| {
                    let percent = volume
                        .GetMasterVolumeLevelScalar()
                        .map(|scalar| (scalar * 100.0).clamp(0.0, 100.0))
                        .unwrap_or(0.0);
                    let muted = volume
                        .GetMute()
                        .map(|value| value.as_bool())
                        .unwrap_or(false);
                    (percent, muted)
                })
                .unwrap_or((0.0, false));

            let meter = device.Activate::<IAudioMeterInformation>(CLSCTX_ALL, None).ok();
            let name = if friendly.is_empty() {
                description.clone()
            } else {
                friendly.clone()
            };

            let info = AudioDevice {
                id: id.clone(),
                name: name.clone(),
                display_name: name,
                custom_name: None,
                device_desc: description,
                interface_name: interface,
                state: state.to_string(),
                state_label: state_label.to_string(),
                is_default: default_capture_id.as_deref() == Some(id.as_str()),
                is_default_communication: default_comm_capture_id.as_deref() == Some(id.as_str()),
                volume_percent,
                muted,
                form_factor: form_factor.to_string(),
                form_factor_label: form_factor_label.to_string(),
            };

            devices.push(EnumeratedDevice { info, meter });
        }

        devices.sort_by(|a, b| {
            let rank = |device: &EnumeratedDevice| match device.info.state.as_str() {
                "active" => 0,
                "unplugged" => 1,
                "disabled" => 2,
                _ => 3,
            };
            rank(a)
                .cmp(&rank(b))
                .then_with(|| a.info.name.to_lowercase().cmp(&b.info.name.to_lowercase()))
        });

        Ok(Enumeration {
            devices,
            default_output_name,
            default_output_form_factor,
        })
    }
}
