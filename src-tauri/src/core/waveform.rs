//! Waveform peaks and their disk cache -- the `Audio Engine`'s "pre-compute
//! the waveform once per stem" duty, with the file written through `Project
//! Persistence`'s atomic write. See
//! docs/architecture.md#design-note-waveform-cache-on-disk and the
//! `atomic-persistence` playbook.
//!
//! Decoding a whole stem is expensive on weak hardware, so peaks are
//! computed once and cached next to the project as `<stem id>.waveform`.
//! The cache records the source file's size and modification time: a stem
//! replaced on disk no longer matches and is recomputed, and re-importing a
//! stem calls [`WaveformCache::invalidate`]. This runs off the real-time
//! thread, like all decoding (`realtime-audio-safety` playbook).
//!
//! Cache file layout (all integers little-endian):
//!
//! | bytes  | field |
//! |--------|-------|
//! | 0..4   | magic `SLWF` |
//! | 4..8   | u32 format version, [`CACHE_VERSION`] |
//! | 8..16  | u64 source file size in bytes |
//! | 16..24 | u64 source modification time, nanoseconds since the Unix epoch (0 if unknown) |
//! | 24..28 | u32 peaks per second, [`PEAKS_PER_SECOND`] |
//! | 28..32 | u32 number of peaks |
//! | 32..   | that many f32 peaks |

// Not called by Session or an event yet: `waveform_ready` and computing on
// import are wired by TASK-010 (IPC events).
#![allow(dead_code)]

use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use super::persistence::{ProjectPersistence, StemEntry};
use super::stem_importer::{read_wav_header, WavInfo};

/// Waveform resolution: one peak per 10 ms of audio.
pub const PEAKS_PER_SECOND: u32 = 100;
/// Bumped whenever the cache layout or the peak definition changes; a cache
/// with another version is recomputed.
pub const CACHE_VERSION: u32 = 1;
const MAGIC: &[u8; 4] = b"SLWF";
const HEADER_LEN: usize = 32;

/// A stem's peaks, and whether they had to be computed (`false` = read
/// from the cache).
#[derive(Debug, Clone, PartialEq)]
pub struct Waveform {
    pub peaks: Vec<f32>,
    pub computed: bool,
}

/// Peaks of the WAV at `path`: one per bucket of `sample_rate /
/// PEAKS_PER_SECOND` frames (at least 1 frame; the last bucket may be
/// shorter and still counts). A peak is the largest absolute sample value
/// in the bucket across all channels, with samples scaled to -1.0..=1.0:
/// 8-bit PCM is unsigned (`(x - 128) / 128`), 16/24/32-bit PCM is signed
/// (`x / 2^(bits-1)`), 32/64-bit float is taken as is; the result is clamped
/// to 0.0..=1.0. Reads the samples in bounded chunks, never the whole file
/// at once. An unreadable or unsupported file is an error (its message).
pub fn compute_peaks(path: &Path) -> Result<Vec<f32>, String> {
    const CHUNK_BYTES: usize = 64 * 1024;

    let mut file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let info = read_wav_header(&mut file)?;
    let width = usize::from(info.block_align);
    let sample_bytes = usize::from(info.bits_per_sample / 8);
    if sample_bytes == 0 || sample_value(&info, &vec![0u8; sample_bytes]).is_none() {
        return Err(format!(
            "{}: unsupported sample format ({} bits, format {})",
            path.display(),
            info.bits_per_sample,
            info.format
        ));
    }

    let frames_total = info.data_len / width as u64;
    let frames_per_bucket = u64::from((info.sample_rate / PEAKS_PER_SECOND).max(1));
    let frames_per_chunk = (CHUNK_BYTES / width).max(1);
    let mut buf = vec![0u8; frames_per_chunk * width];

    file.seek(SeekFrom::Start(info.data_offset))
        .map_err(|e| format!("seek failed: {e}"))?;
    let mut peaks = Vec::new();
    let mut current = 0.0f32;
    let mut in_bucket = 0u64;
    let mut remaining = frames_total;
    while remaining > 0 {
        let n = remaining.min(frames_per_chunk as u64) as usize;
        let bytes = n * width;
        file.read_exact(&mut buf[..bytes])
            .map_err(|e| format!("read failed: {e}"))?;
        for frame in buf[..bytes].chunks_exact(width) {
            for sample in frame.chunks_exact(sample_bytes) {
                if let Some(value) = sample_value(&info, sample) {
                    current = current.max(value.abs());
                }
            }
            in_bucket += 1;
            if in_bucket == frames_per_bucket {
                peaks.push(current.clamp(0.0, 1.0));
                current = 0.0;
                in_bucket = 0;
            }
        }
        remaining -= n as u64;
    }
    if in_bucket > 0 {
        peaks.push(current.clamp(0.0, 1.0));
    }
    Ok(peaks)
}

/// Decodes one sample (`bytes.len()` = `info.bits_per_sample / 8`) to
/// -1.0..=1.0 by the rules in [`compute_peaks`]; `None` for a bit depth /
/// format pair it doesn't support (e.g. 12-bit, or 16-bit float).
fn sample_value(info: &WavInfo, bytes: &[u8]) -> Option<f32> {
    match (info.format, info.bits_per_sample) {
        (1, 8) => Some((f32::from(*bytes.first()?) - 128.0) / 128.0),
        (1, 16) => {
            let v = i16::from_le_bytes(bytes.try_into().ok()?);
            Some(f32::from(v) / 32768.0)
        }
        (1, 24) => {
            let b: [u8; 3] = bytes.try_into().ok()?;
            let v = i32::from_le_bytes([0, b[0], b[1], b[2]]) >> 8;
            Some(v as f32 / 8_388_608.0)
        }
        (1, 32) => {
            let v = i32::from_le_bytes(bytes.try_into().ok()?);
            Some(v as f32 / 2_147_483_648.0)
        }
        (3, 32) => Some(f32::from_le_bytes(bytes.try_into().ok()?)),
        (3, 64) => Some(f64::from_le_bytes(bytes.try_into().ok()?) as f32),
        _ => None,
    }
}

pub struct WaveformCache {
    persistence: Arc<ProjectPersistence>,
}

impl WaveformCache {
    pub fn new(persistence: Arc<ProjectPersistence>) -> Self {
        Self { persistence }
    }

    /// The cache file name for a stem: `<stem id>.waveform`, next to the
    /// project file -- what `stems[].waveformCache` records.
    pub fn cache_file_name(stem_id: &str) -> String {
        format!("{stem_id}.waveform")
    }

    /// The peaks of `stem` (whose file is `project_dir/stem.file`). Reads
    /// `project_dir/<id>.waveform` when it's valid -- right magic and
    /// version, same peaks per second, a peak count that matches the file
    /// length, and the source file's current size and modification time --
    /// and returns it with `computed: false`, without decoding. Otherwise
    /// (missing, stale or corrupt) computes the peaks, writes the cache with
    /// `ProjectPersistence::atomic_write`, and returns `computed: true`. A
    /// corrupt cache, or one that can't be written, is never an error, only
    /// a reason to recompute; a missing or unreadable source is.
    pub fn peaks(&self, project_dir: &Path, stem: &StemEntry) -> Result<Waveform, String> {
        let source = project_dir.join(&stem.file);
        let (size, mtime) =
            source_stamp(&source).map_err(|e| format!("{}: {e}", source.display()))?;
        let cache_path = project_dir.join(Self::cache_file_name(&stem.id));
        if let Some(peaks) = read_cache(&cache_path, size, mtime) {
            return Ok(Waveform {
                peaks,
                computed: false,
            });
        }

        let peaks = compute_peaks(&source)?;
        // The cache only saves work next time: if it can't be written, the
        // peaks are still good, and the next open just computes them again.
        if let Ok(count) = u32::try_from(peaks.len()) {
            let bytes = encode_cache(size, mtime, count, &peaks);
            let _ = self.persistence.atomic_write(&cache_path, &bytes);
        }
        Ok(Waveform {
            peaks,
            computed: true,
        })
    }

    /// Discards a stem's cached peaks, e.g. when it's re-imported. A cache
    /// that doesn't exist is not an error.
    pub fn invalidate(&self, project_dir: &Path, stem_id: &str) -> io::Result<()> {
        match fs::remove_file(project_dir.join(Self::cache_file_name(stem_id))) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }
}

/// The source file's (size, modification time in ns since the epoch, 0
/// when the platform can't say).
fn source_stamp(path: &Path) -> io::Result<(u64, u64)> {
    let meta = fs::metadata(path)?;
    let modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos() as u64);
    Ok((meta.len(), modified))
}

/// The peaks in the cache at `path`, if it's valid for a source of `size`
/// bytes and modification time `mtime` (see the layout in the module doc).
/// Any other content -- missing, unreadable, foreign, stale, truncated --
/// is `None`: a reason to recompute, never an error.
fn read_cache(path: &Path, size: u64, mtime: u64) -> Option<Vec<f32>> {
    let bytes = fs::read(path).ok()?;
    if bytes.len() < HEADER_LEN || &bytes[0..4] != MAGIC {
        return None;
    }
    let u32_at = |at: usize| -> Option<u32> {
        Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
    };
    let u64_at = |at: usize| -> Option<u64> {
        Some(u64::from_le_bytes(bytes.get(at..at + 8)?.try_into().ok()?))
    };
    if u32_at(4)? != CACHE_VERSION
        || u64_at(8)? != size
        || u64_at(16)? != mtime
        || u32_at(24)? != PEAKS_PER_SECOND
    {
        return None;
    }
    let count = usize::try_from(u32_at(28)?).ok()?;
    if bytes.len() != HEADER_LEN.checked_add(count.checked_mul(4)?)? {
        return None;
    }
    Some(
        bytes[HEADER_LEN..]
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect(),
    )
}

/// The cache file's bytes for `peaks` of a source with `size` bytes and
/// modification time `mtime`, laid out as the module doc says.
fn encode_cache(size: u64, mtime: u64, count: u32, peaks: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER_LEN + 4 * peaks.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&CACHE_VERSION.to_le_bytes());
    out.extend_from_slice(&size.to_le_bytes());
    out.extend_from_slice(&mtime.to_le_bytes());
    out.extend_from_slice(&PEAKS_PER_SECOND.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    for peak in peaks {
        out.extend_from_slice(&peak.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    // The contract for TASK-007 (openspec/changes/task-007-waveform-cache).
    // Written before the implementation, by a different author than it --
    // never loosen one of these to make it pass (AGENTS.md, testing policy).
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/stems")
            .join(name)
    }

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "stemloft-waveform-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A WAV with the given raw sample bytes (interleaved), PCM or float.
    fn wav(format: u16, channels: u16, rate: u32, bits: u16, data: &[u8]) -> Vec<u8> {
        let align = channels * bits / 8;
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
        out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&format.to_le_bytes());
        out.extend_from_slice(&channels.to_le_bytes());
        out.extend_from_slice(&rate.to_le_bytes());
        out.extend_from_slice(&(rate * align as u32).to_le_bytes());
        out.extend_from_slice(&align.to_le_bytes());
        out.extend_from_slice(&bits.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(data);
        out
    }

    fn i16s(values: &[i16]) -> Vec<u8> {
        values.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    fn peaks_of(label: &str, bytes: &[u8]) -> Result<Vec<f32>, String> {
        let dir = temp_dir(label);
        let path = dir.join("x.wav");
        fs::write(&path, bytes).unwrap();
        let result = compute_peaks(&path);
        fs::remove_dir_all(&dir).ok();
        result
    }

    fn stem(id: &str, file: &str) -> StemEntry {
        StemEntry {
            id: id.into(),
            file: file.into(),
            duration_sec: 0.0,
            waveform_cache: None,
        }
    }

    fn cache() -> WaveformCache {
        WaveformCache::new(Arc::new(ProjectPersistence::new()))
    }

    /// A project folder holding the 0.2 s stereo fixture as `bass.wav`.
    fn project(label: &str) -> PathBuf {
        let dir = temp_dir(label);
        fs::copy(fixture("stereo-44100-16.wav"), dir.join("bass.wav")).unwrap();
        dir
    }

    // ---- peaks ---------------------------------------------------------

    #[test]
    fn one_peak_per_bucket_including_a_short_last_one() {
        // 200 Hz -> 2 frames per bucket; 5 frames -> 3 buckets.
        let data = i16s(&[0, 16384, -32768, 0, 8192]);
        assert_eq!(
            peaks_of("buckets", &wav(1, 1, 200, 16, &data)).unwrap(),
            vec![0.5, 1.0, 0.25]
        );
    }

    #[test]
    fn a_peak_is_the_loudest_channel_in_the_bucket() {
        // Stereo frames (L, R): bucket 1 = (0.25, -0.5), (0, 0);
        // bucket 2 = (0, 0), (-0.75, 0.125).
        let data = i16s(&[8192, -16384, 0, 0, 0, 0, -24576, 4096]);
        assert_eq!(
            peaks_of("channels", &wav(1, 2, 200, 16, &data)).unwrap(),
            vec![0.5, 0.75]
        );
    }

    #[test]
    fn every_supported_sample_format_is_scaled_to_one() {
        // 200 Hz: 2 frames per bucket. 8-bit unsigned: 0 is -1.0, 128 is
        // silence, 192 is +0.5.
        assert_eq!(
            peaks_of("u8", &wav(1, 1, 200, 8, &[128, 192])).unwrap(),
            vec![0.5]
        );
        assert_eq!(
            peaks_of("u8-min", &wav(1, 1, 100, 8, &[0])).unwrap(),
            vec![1.0]
        );

        // 24-bit signed: -2^22 is -0.5.
        let s24 = [0x00, 0x00, 0xC0];
        assert_eq!(
            peaks_of("s24", &wav(1, 1, 100, 24, &s24)).unwrap(),
            vec![0.5]
        );

        // 32-bit signed: 2^30 is +0.5.
        let s32 = (1i32 << 30).to_le_bytes();
        assert_eq!(
            peaks_of("s32", &wav(1, 1, 100, 32, &s32)).unwrap(),
            vec![0.5]
        );

        // Float, clamped to 1.0.
        let f32s: Vec<u8> = [-0.25f32, 1.5]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        assert_eq!(
            peaks_of("f32", &wav(3, 1, 200, 32, &f32s)).unwrap(),
            vec![1.0]
        );
        let f64s: Vec<u8> = [0.125f64].iter().flat_map(|v| v.to_le_bytes()).collect();
        assert_eq!(
            peaks_of("f64", &wav(3, 1, 100, 64, &f64s)).unwrap(),
            vec![0.125]
        );
    }

    #[test]
    fn real_fixtures_give_their_sine_amplitude() {
        // Every fixture is a 0.5-amplitude sine (generate.py): each 10 ms
        // bucket holds several periods, so each peak is ~0.5.
        for (name, count) in [
            ("mono-44100-16.wav", 10),
            ("stereo-44100-16.wav", 20),
            ("stereo-48000-16.wav", 5),
            ("mono-44100-float.wav", 5),
            ("mono-44100-24-extensible.wav", 10),
        ] {
            let peaks = compute_peaks(&fixture(name)).unwrap();
            assert_eq!(peaks.len(), count, "{name}");
            assert!(
                peaks.iter().all(|p| (p - 0.5).abs() < 0.01),
                "{name}: {peaks:?}"
            );
        }
    }

    #[test]
    fn unsupported_or_missing_files_are_errors() {
        assert!(compute_peaks(&fixture("missing.wav")).is_err());
        assert!(compute_peaks(&fixture("generate.py")).is_err());
        // 12-bit PCM isn't a byte-aligned format this decoder knows.
        assert!(peaks_of("12bit", &wav(1, 1, 100, 12, &[0, 0])).is_err());
    }

    // ---- cache -----------------------------------------------------------

    #[test]
    fn peaks_are_computed_on_first_access_and_cached_atomically() {
        let dir = project("first");

        let first = cache().peaks(&dir, &stem("bass", "bass.wav")).unwrap();

        assert!(first.computed);
        assert_eq!(first.peaks, compute_peaks(&dir.join("bass.wav")).unwrap());
        let mut files: Vec<String> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        files.sort();
        assert_eq!(
            files,
            vec!["bass.wav", "bass.waveform"],
            "no temp files left"
        );

        let bytes = fs::read(dir.join("bass.waveform")).unwrap();
        assert_eq!(&bytes[0..4], b"SLWF");
        assert_eq!(
            u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            CACHE_VERSION
        );
        assert_eq!(
            u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
            fs::metadata(dir.join("bass.wav")).unwrap().len()
        );
        assert_eq!(
            u32::from_le_bytes(bytes[24..28].try_into().unwrap()),
            PEAKS_PER_SECOND
        );
        assert_eq!(u32::from_le_bytes(bytes[28..32].try_into().unwrap()), 20);
        assert_eq!(bytes.len(), HEADER_LEN + 20 * 4);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_next_open_reads_the_cache_instead_of_decoding() {
        let dir = project("reuse");
        let bass = stem("bass", "bass.wav");
        cache().peaks(&dir, &bass).unwrap();

        // Overwrite the cached peaks with a marker value: if the next call
        // decoded the stem again, it would get ~0.5 instead.
        let mut bytes = fs::read(dir.join("bass.waveform")).unwrap();
        for chunk in bytes[HEADER_LEN..].chunks_mut(4) {
            chunk.copy_from_slice(&0.25f32.to_le_bytes());
        }
        fs::write(dir.join("bass.waveform"), &bytes).unwrap();

        let second = cache().peaks(&dir, &bass).unwrap();

        assert!(!second.computed);
        assert_eq!(second.peaks, vec![0.25; 20]);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_replaced_stem_file_is_recomputed() {
        let dir = project("replaced");
        let bass = stem("bass", "bass.wav");
        cache().peaks(&dir, &bass).unwrap();

        fs::copy(fixture("stereo-48000-16.wav"), dir.join("bass.wav")).unwrap();
        let after = cache().peaks(&dir, &bass).unwrap();

        assert!(after.computed);
        assert_eq!(after.peaks.len(), 5);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn invalidate_discards_the_cache_so_it_is_recomputed() {
        let dir = project("invalidate");
        let bass = stem("bass", "bass.wav");
        let cache = cache();
        cache.peaks(&dir, &bass).unwrap();

        cache.invalidate(&dir, "bass").unwrap();

        assert!(!dir.join("bass.waveform").exists());
        assert!(cache.peaks(&dir, &bass).unwrap().computed);
        // Nothing to discard is fine too.
        cache.invalidate(&dir, "never-cached").unwrap();
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_corrupt_or_foreign_cache_is_recomputed_not_an_error() {
        let dir = project("corrupt");
        let bass = stem("bass", "bass.wav");
        let cache = cache();
        cache.peaks(&dir, &bass).unwrap();
        let good = fs::read(dir.join("bass.waveform")).unwrap();

        let mut wrong_version = good.clone();
        wrong_version[4..8].copy_from_slice(&99u32.to_le_bytes());
        let mut wrong_count = good.clone();
        wrong_count[28..32].copy_from_slice(&21u32.to_le_bytes());
        let truncated = good[..good.len() - 3].to_vec();
        for bad in [
            b"garbage".to_vec(),
            Vec::new(),
            wrong_version,
            wrong_count,
            truncated,
        ] {
            fs::write(dir.join("bass.waveform"), &bad).unwrap();
            let waveform = cache.peaks(&dir, &bass).unwrap();
            assert!(waveform.computed);
            assert_eq!(waveform.peaks.len(), 20);
        }
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_cache_that_cannot_be_written_still_returns_the_peaks() {
        // Added after review: a directory where the cache file should go
        // makes both reading and writing it fail.
        let dir = project("unwritable");
        fs::create_dir(dir.join("bass.waveform")).unwrap();

        let waveform = cache().peaks(&dir, &stem("bass", "bass.wav")).unwrap();

        assert!(waveform.computed);
        assert_eq!(waveform.peaks.len(), 20);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_missing_stem_file_is_an_error() {
        let dir = temp_dir("no-source");
        assert!(cache().peaks(&dir, &stem("bass", "bass.wav")).is_err());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_cache_file_name_is_the_stem_id() {
        assert_eq!(
            WaveformCache::cache_file_name("lead-vocals"),
            "lead-vocals.waveform"
        );
    }
}
