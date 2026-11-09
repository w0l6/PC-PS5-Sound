use std::ffi::c_void;
use windows::core::PWSTR;
use windows::Win32::System::Com::StructuredStorage::{
    PropVariantToStringAlloc, PROPVARIANT,
};
use windows::Win32::System::Com::{CoInitializeEx, CoTaskMemFree, CoUninitialize, COINIT_MULTITHREADED};

/// Garde RAII pour l'initialisation COM du thread courant.
pub struct ComGuard {
    uninit: bool,
}

impl ComGuard {
    pub fn mta() -> Self {
        // CoInitializeEx retourne un HRESULT brut dans windows 0.62.
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        // S_OK (0) et S_FALSE (1) doivent tous deux etre equilibres par CoUninitialize.
        // Un HRESULT negatif (ex: RPC_E_CHANGED_MODE) ne doit pas etre desinitialise.
        Self {
            uninit: hr.0 >= 0,
        }
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.uninit {
            unsafe {
                CoUninitialize();
            }
        }
    }
}

/// Convertit une chaine Rust en chaine UTF-16 terminee par NUL.
pub fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Copie une PWSTR allouee par COM puis libere la memoire d'origine.
pub unsafe fn take_pwstr(pw: PWSTR) -> String {
    if pw.0.is_null() {
        return String::new();
    }
    let text = String::from_utf16_lossy(pw.as_wide());
    CoTaskMemFree(Some(pw.0 as *const c_void));
    text
}

/// Convertit un PROPVARIANT en chaine lisible.
pub unsafe fn propvariant_to_string(value: &PROPVARIANT) -> String {
    match PropVariantToStringAlloc(value) {
        Ok(pw) => take_pwstr(pw).trim().to_string(),
        Err(_) => String::new(),
    }
}
