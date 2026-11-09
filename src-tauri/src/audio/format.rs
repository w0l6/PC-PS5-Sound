use windows::Win32::Media::Audio::{WAVEFORMATEX, WAVEFORMATEXTENSIBLE, WAVEFORMATEXTENSIBLE_0};
use windows::Win32::Media::KernelStreaming::WAVE_FORMAT_EXTENSIBLE;
use windows::Win32::Media::Multimedia::KSDATAFORMAT_SUBTYPE_IEEE_FLOAT;

const WAVE_FORMAT_IEEE_FLOAT: u16 = 3;

/// Description normalisee d'un format audio Windows.
#[derive(Debug, Clone, Copy, Default)]
pub struct SampleFormat {
    pub channels: u16,
    pub sample_rate: u32,
    pub block_align: u16,
    pub bytes_per_sample: u16,
    pub is_float: bool,
}

impl SampleFormat {
    pub fn valid(&self) -> bool {
        self.channels > 0
            && self.sample_rate > 0
            && self.bytes_per_sample > 0
            && self.block_align > 0
    }
}

/// Lit un WAVEFORMATEX (eventuellement etendu) sans conserver le pointeur.
pub unsafe fn parse_format(ptr: *const WAVEFORMATEX) -> SampleFormat {
    if ptr.is_null() {
        return SampleFormat::default();
    }
    let base = *ptr;
    let mut is_float = base.wFormatTag == WAVE_FORMAT_IEEE_FLOAT;

    if base.wFormatTag == WAVE_FORMAT_EXTENSIBLE as u16 {
        let ext = *(ptr as *const WAVEFORMATEXTENSIBLE);
        // WAVEFORMATEXTENSIBLE est `packed(1)` : une lecture non alignee est
        // obligatoire pour eviter une reference invalide.
        let subformat = std::ptr::read_unaligned(std::ptr::addr_of!(ext.SubFormat));
        is_float = subformat == KSDATAFORMAT_SUBTYPE_IEEE_FLOAT;
    }

    let bytes_per_sample = if base.wBitsPerSample > 0 {
        (base.wBitsPerSample as usize).div_ceil(8) as u16
    } else {
        0
    };

    SampleFormat {
        channels: base.nChannels,
        sample_rate: base.nSamplesPerSec,
        block_align: base.nBlockAlign,
        bytes_per_sample,
        is_float,
    }
}

/// Construit un format de rendu canonique : float32 stereo a la frequence
/// d'echantillonnage de la capture. Le moteur audio Windows se charge de la
/// conversion vers le format materiel (AUTOCONVERTPCM).
pub fn build_render_format(sample_rate: u32) -> WAVEFORMATEXTENSIBLE {
    let channels: u16 = 2;
    let bits: u16 = 32;
    let block_align = channels * (bits / 8);
    WAVEFORMATEXTENSIBLE {
        Format: WAVEFORMATEX {
            wFormatTag: WAVE_FORMAT_EXTENSIBLE as u16,
            nChannels: channels,
            nSamplesPerSec: sample_rate,
            nAvgBytesPerSec: sample_rate * block_align as u32,
            nBlockAlign: block_align,
            wBitsPerSample: bits,
            cbSize: 22,
        },
        Samples: WAVEFORMATEXTENSIBLE_0 {
            wValidBitsPerSample: bits,
        },
        dwChannelMask: 0x3, // SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT
        SubFormat: KSDATAFORMAT_SUBTYPE_IEEE_FLOAT,
    }
}

fn read_sample(format: &SampleFormat, data: &[u8], offset: usize) -> f32 {
    let bps = format.bytes_per_sample as usize;
    let Some(bytes) = data.get(offset..offset + bps) else {
        return 0.0;
    };

    if format.is_float {
        return match bps {
            4 => f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            8 => f64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]) as f32,
            _ => 0.0,
        };
    }

    match bps {
        1 => (bytes[0] as f32 - 128.0) / 128.0,
        2 => i16::from_le_bytes([bytes[0], bytes[1]]) as f32 / 32768.0,
        3 => {
            let raw = (bytes[0] as i32) | ((bytes[1] as i32) << 8) | ((bytes[2] as i32) << 16);
            let signed = if raw & 0x80_0000 != 0 {
                raw | !0xFF_FFFF
            } else {
                raw
            };
            signed as f32 / 8_388_608.0
        }
        4 => i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f32
            / 2_147_483_648.0,
        _ => 0.0,
    }
}

/// Convertit des trames PCM/float en trames float32 stereo entrelacees.
/// Retourne le nombre de trames produites.
pub fn frames_to_stereo_f32(
    format: &SampleFormat,
    data: &[u8],
    output: &mut Vec<f32>,
) -> usize {
    output.clear();
    let channels = format.channels as usize;
    let bps = format.bytes_per_sample as usize;
    if channels == 0 || bps == 0 {
        return 0;
    }
    // `block_align` est la taille reelle d'une trame dans le tampon WASAPI.
    // Il peut inclure un remplissage supplementaire selon le pilote.
    let frame_bytes = (format.block_align as usize).max(channels * bps);
    if frame_bytes == 0 {
        return 0;
    }
    let frames = data.len() / frame_bytes;
    output.reserve(frames * 2);

    for frame in 0..frames {
        let base = frame * frame_bytes;
        let left = read_sample(format, data, base);
        let right = if channels >= 2 {
            read_sample(format, data, base + bps)
        } else {
            left
        };
        output.push(left);
        output.push(right);
    }

    frames
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_format(
        channels: u16,
        sample_rate: u32,
        block_align: u16,
        bytes_per_sample: u16,
        is_float: bool,
    ) -> SampleFormat {
        SampleFormat {
            channels,
            sample_rate,
            block_align,
            bytes_per_sample,
            is_float,
        }
    }

    #[test]
    fn converts_stereo_i16_to_float() {
        let format = sample_format(2, 48_000, 4, 2, false);
        let data = [0x00, 0x00, 0xFF, 0x7F];
        let mut output = Vec::new();

        let frames = frames_to_stereo_f32(&format, &data, &mut output);

        assert_eq!(frames, 1);
        assert_eq!(output.len(), 2);
        assert!((output[0] - 0.0).abs() < 1e-6);
        assert!((output[1] - 0.999_969_5).abs() < 1e-4);
    }

    #[test]
    fn duplicates_mono_input() {
        let format = sample_format(1, 44_100, 2, 2, false);
        let data = [0x00, 0x40];
        let mut output = Vec::new();

        frames_to_stereo_f32(&format, &data, &mut output);

        assert_eq!(output, vec![0.5, 0.5]);
    }

    #[test]
    fn decodes_packed_24_bit() {
        let format = sample_format(1, 48_000, 3, 3, false);
        let data = [0x00, 0x00, 0x40];
        let mut output = Vec::new();

        frames_to_stereo_f32(&format, &data, &mut output);

        assert_eq!(output, vec![0.5, 0.5]);
    }

    #[test]
    fn honours_block_align_padding() {
        let format = sample_format(1, 48_000, 4, 2, false);
        let data = [
            0x00, 0x40, 0xFF, 0xFF, // 0.5 + remplissage
            0x00, 0xC0, 0xFF, 0xFF, // -0.5 + remplissage
        ];
        let mut output = Vec::new();

        let frames = frames_to_stereo_f32(&format, &data, &mut output);

        assert_eq!(frames, 2);
        assert_eq!(output, vec![0.5, 0.5, -0.5, -0.5]);
    }

    #[test]
    fn decodes_float32_stereo() {
        let format = sample_format(2, 48_000, 8, 4, true);
        let mut data = Vec::new();
        for value in [0.25f32, -0.75f32] {
            data.extend_from_slice(&value.to_le_bytes());
        }
        let mut output = Vec::new();

        frames_to_stereo_f32(&format, &data, &mut output);

        assert_eq!(output, vec![0.25, -0.75]);
    }

    #[test]
    fn render_format_is_float32_stereo() {
        let format = build_render_format(48_000);

        // WAVEFORMATEXTENSIBLE est `packed(1)` : chaque champ doit etre lu
        // par lecture non alignee avant toute assertion.
        let channels: u16 =
            unsafe { std::ptr::read_unaligned(std::ptr::addr_of!(format.Format.nChannels)) };
        let sample_rate: u32 =
            unsafe { std::ptr::read_unaligned(std::ptr::addr_of!(format.Format.nSamplesPerSec)) };
        let bits: u16 =
            unsafe { std::ptr::read_unaligned(std::ptr::addr_of!(format.Format.wBitsPerSample)) };
        let block_align: u16 =
            unsafe { std::ptr::read_unaligned(std::ptr::addr_of!(format.Format.nBlockAlign)) };
        let bytes_per_second: u32 = unsafe {
            std::ptr::read_unaligned(std::ptr::addr_of!(format.Format.nAvgBytesPerSec))
        };
        let valid_bits: u16 = unsafe {
            std::ptr::read_unaligned(std::ptr::addr_of!(format.Samples.wValidBitsPerSample))
        };
        let subformat = unsafe { std::ptr::read_unaligned(std::ptr::addr_of!(format.SubFormat)) };

        assert_eq!(channels, 2);
        assert_eq!(sample_rate, 48_000);
        assert_eq!(bits, 32);
        assert_eq!(block_align, 8);
        assert_eq!(bytes_per_second, 48_000 * 8);
        assert_eq!(valid_bits, 32);
        assert_eq!(subformat, KSDATAFORMAT_SUBTYPE_IEEE_FLOAT);
    }
}
