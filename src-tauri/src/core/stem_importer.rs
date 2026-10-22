//! `Stem Importer` -- receives stem files (local, or later the output of
//! automatic separation), enforces consistency across a project's stems,
//! and writes them into the project. See
//! docs/architecture.md#consistency-across-a-projects-stems.
//!
//! v1 accepts WAV (integer PCM or IEEE float, plain or
//! `WAVE_FORMAT_EXTENSIBLE`) -- the common case for already-separated stems.
//! Everything here reads the RIFF header, never decodes audio: the sample
//! rate and `durationSec` come straight from `fmt `/`data`, and the mono
//! upmix copies each sample's bytes to both channels without converting
//! them.

use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::persistence::{ProjectPersistence, StemEntry};

/// What a WAV's header says, after walking its RIFF chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WavInfo {
    /// 1 = integer PCM, 3 = IEEE float (an extensible header's subformat,
    /// already resolved to one of these two).
    pub format: u16,
    pub channels: u16,
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    /// Bytes per frame (one sample for every channel).
    pub block_align: u16,
    /// Where the `data` chunk's samples start, from the start of the file.
    pub data_offset: u64,
    /// Length of the `data` chunk's samples, in bytes.
    pub data_len: u64,
}

impl WavInfo {
    /// Frames / sample rate, from the header alone.
    pub fn duration_sec(&self) -> f64 {
        let frames = self.data_len / u64::from(self.block_align.max(1));
        frames as f64 / f64::from(self.sample_rate)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    /// The file couldn't be read, or the project folder couldn't be written.
    Io { file: PathBuf, message: String },
    /// Not a WAV this importer understands (not RIFF/WAVE, no `fmt `/`data`
    /// chunk, truncated, an encoding other than PCM/float).
    Unsupported { file: PathBuf, message: String },
    /// More than two channels: there's no lossless way to fold them to
    /// stereo, unlike mono.
    TooManyChannels { file: PathBuf, channels: u16 },
    /// Never resampled silently -- docs/architecture.md.
    SampleRateMismatch {
        file: PathBuf,
        found: u32,
        expected: u32,
    },
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportError::Io { file, message } => {
                write!(f, "{}: {message}", file.display())
            }
            ImportError::Unsupported { file, message } => {
                write!(f, "{} isn't a supported WAV file: {message}", file.display())
            }
            ImportError::TooManyChannels { file, channels } => write!(
                f,
                "{} has {channels} channels; only mono and stereo stems can be imported",
                file.display()
            ),
            ImportError::SampleRateMismatch {
                file,
                found,
                expected,
            } => write!(
                f,
                "{} is {found} Hz but this project's stems are {expected} Hz; \
                 resample it to {expected} Hz before importing",
                file.display()
            ),
        }
    }
}

impl std::error::Error for ImportError {}

pub struct StemImporter {
    persistence: Arc<ProjectPersistence>,
}

impl StemImporter {
    pub fn new(persistence: Arc<ProjectPersistence>) -> Self {
        Self { persistence }
    }

    /// Imports one or more stem files into the current project. Still a
    /// stub: the `Command` path (which project is open, saving the new
    /// entries) is wired by TASK-009 on top of [`Self::import`].
    pub fn import_stems(&self, _file_paths: Vec<PathBuf>) -> Result<(), String> {
        Err("import_stems: not yet implemented".into())
    }

    /// Copies `files` into `project_dir` and returns their new
    /// `StemEntry`s (the caller adds them to the project and marks it
    /// dirty). All or nothing:
    /// 1. Read every header first. The expected sample rate is the first
    ///    existing stem's (read from its file in `project_dir`), else the
    ///    first new file's; any file that differs fails the whole import
    ///    with `SampleRateMismatch` before anything is written.
    /// 2. Each file gets an id from [`stem_id`] (unique among `existing`,
    ///    this import and the files already in `project_dir`) and is written to `project_dir/<id>.wav` through
    ///    `ProjectPersistence::atomic_write_with`: a mono file upmixed by
    ///    [`write_upmixed`], a stereo one copied byte for byte.
    /// 3. If a write fails, the files this call already wrote are removed
    ///    again, and the error is returned.
    #[allow(dead_code)] // called from Session once TASK-009 wires the Command
    pub fn import(
        &self,
        project_dir: &Path,
        existing: &[StemEntry],
        files: &[PathBuf],
    ) -> Result<Vec<StemEntry>, ImportError> {
        // Pass 1: validate every file before anything is written.
        let mut expected: Option<u32> = match existing.first() {
            Some(stem) => Some(open_header(&project_dir.join(&stem.file))?.1.sample_rate),
            None => None,
        };
        let mut sources: Vec<(PathBuf, File, WavInfo)> = Vec::with_capacity(files.len());
        for path in files {
            let (file, info) = open_header(path)?;
            if info.channels > 2 {
                return Err(ImportError::TooManyChannels {
                    file: path.clone(),
                    channels: info.channels,
                });
            }
            let expected_rate = *expected.get_or_insert(info.sample_rate);
            if info.sample_rate != expected_rate {
                return Err(ImportError::SampleRateMismatch {
                    file: path.clone(),
                    found: info.sample_rate,
                    expected: expected_rate,
                });
            }
            sources.push((path.clone(), file, info));
        }

        // Pass 2: write each stem; undo this call's writes if one fails.
        let mut taken: Vec<String> = existing.iter().map(|s| s.id.clone()).collect();
        let mut written: Vec<PathBuf> = Vec::with_capacity(sources.len());
        let mut entries = Vec::with_capacity(sources.len());
        for (path, mut file, info) in sources {
            // Also skip ids whose file is already on disk without being a
            // stem of the project (a leftover, a file the user put there):
            // an import never overwrites a file it didn't write.
            let mut id = stem_id(&path, &taken);
            while project_dir.join(format!("{id}.wav")).exists() {
                taken.push(id);
                id = stem_id(&path, &taken);
            }
            taken.push(id.clone());
            let file_name = format!("{id}.wav");
            let dest = project_dir.join(&file_name);
            let result = self.persistence.atomic_write_with(&dest, |out| {
                if info.channels == 1 {
                    write_upmixed(&info, &mut file, out)
                } else {
                    file.seek(SeekFrom::Start(0))?;
                    io::copy(&mut file, out).map(|_| ())
                }
            });
            if let Err(error) = result {
                for done in &written {
                    let _ = fs::remove_file(done);
                }
                return Err(ImportError::Io {
                    file: dest,
                    message: error.to_string(),
                });
            }
            written.push(dest);
            entries.push(StemEntry {
                id,
                file: file_name,
                duration_sec: info.duration_sec(),
                waveform_cache: None,
            });
        }
        Ok(entries)
    }
}

/// Opens `path` and reads its WAV header, mapping failures to `ImportError`.
fn open_header(path: &Path) -> Result<(File, WavInfo), ImportError> {
    let mut file = File::open(path).map_err(|e| ImportError::Io {
        file: path.to_path_buf(),
        message: e.to_string(),
    })?;
    let info = read_wav_header(&mut file).map_err(|message| ImportError::Unsupported {
        file: path.to_path_buf(),
        message,
    })?;
    Ok((file, info))
}

/// Reads exactly `buf.len()` bytes, turning end-of-file into `what`.
fn read_exact_or(file: &mut File, buf: &mut [u8], what: &str) -> Result<(), String> {
    file.read_exact(buf).map_err(|e| {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            what.to_string()
        } else {
            format!("read failed: {e}")
        }
    })
}

/// Reads the RIFF header of `file` (from its start; leaves the position
/// anywhere). Walks the chunks after `RIFF....WAVE` -- each one is a 4-byte
/// id, a little-endian u32 size, the body, and a pad byte when the size is
/// odd -- until both `fmt ` and `data` are found; unknown chunks (`LIST`,
/// ...) are skipped. `fmt `'s format tag 1 or 3 is taken as is; 0xFFFE
/// (extensible) uses the first two bytes of its subformat GUID (offset 24
/// in the `fmt ` body), which must be 1 or 3. Errors (as a message):
/// not RIFF/WAVE, a missing chunk, any other format, `block_align` not
/// `channels * bits_per_sample / 8`, a zero sample rate or channel count,
/// or a `data` chunk running past the end of the file.
fn read_wav_header(file: &mut File) -> Result<WavInfo, String> {
    let file_len = file
        .metadata()
        .map_err(|e| format!("can't read the file size: {e}"))?
        .len();
    file.seek(SeekFrom::Start(0))
        .map_err(|e| format!("seek failed: {e}"))?;
    let mut riff = [0u8; 12];
    read_exact_or(file, &mut riff, "file is too short to be a WAV")?;
    if &riff[0..4] != b"RIFF" || &riff[8..12] != b"WAVE" {
        return Err("not a RIFF/WAVE file".into());
    }

    let mut fmt: Option<WavInfo> = None;
    let mut data: Option<(u64, u64)> = None;
    let mut pos: u64 = 12;
    while fmt.is_none() || data.is_none() {
        let mut head = [0u8; 8];
        read_exact_or(file, &mut head, "no fmt and data chunks found")?;
        let id = [head[0], head[1], head[2], head[3]];
        let size = u64::from(u32::from_le_bytes([head[4], head[5], head[6], head[7]]));
        let body_start = pos + 8;
        // Odd-sized bodies are followed by one pad byte.
        let next = body_start + size + (size & 1);

        match &id {
            b"fmt " if fmt.is_none() => {
                if size < 16 {
                    return Err("fmt chunk is shorter than 16 bytes".into());
                }
                // The extensible subformat sits at offset 24, so 40 bytes
                // is all that's ever needed; skip the rest of a longer one.
                let take = size.min(40) as usize;
                let mut body = vec![0u8; take];
                read_exact_or(file, &mut body, "fmt chunk is truncated")?;
                let tag = u16::from_le_bytes([body[0], body[1]]);
                let format = match tag {
                    1 | 3 => tag,
                    0xFFFE => {
                        if body.len() < 26 {
                            return Err("extensible fmt chunk has no subformat".into());
                        }
                        match u16::from_le_bytes([body[24], body[25]]) {
                            sub @ (1 | 3) => sub,
                            sub => {
                                return Err(format!(
                                    "unsupported extensible subformat {sub:#06x}"
                                ))
                            }
                        }
                    }
                    other => return Err(format!("unsupported format tag {other:#06x}")),
                };
                let channels = u16::from_le_bytes([body[2], body[3]]);
                let sample_rate = u32::from_le_bytes([body[4], body[5], body[6], body[7]]);
                let block_align = u16::from_le_bytes([body[12], body[13]]);
                let bits_per_sample = u16::from_le_bytes([body[14], body[15]]);
                if channels == 0 || sample_rate == 0 {
                    return Err("zero channel count or sample rate".into());
                }
                let expected_align = u32::from(channels) * u32::from(bits_per_sample) / 8;
                if block_align == 0 || u32::from(block_align) != expected_align {
                    return Err(format!(
                        "block_align {block_align} isn't {channels} channels x {bits_per_sample} bits"
                    ));
                }
                fmt = Some(WavInfo {
                    format,
                    channels,
                    sample_rate,
                    bits_per_sample,
                    block_align,
                    data_offset: 0,
                    data_len: 0,
                });
            }
            b"data" if data.is_none() => {
                if body_start + size > file_len {
                    return Err("data chunk runs past the end of the file".into());
                }
                data = Some((body_start, size));
            }
            _ => {}
        }

        file.seek(SeekFrom::Start(next))
            .map_err(|e| format!("seek failed: {e}"))?;
        pos = next;
    }

    let (mut info, (data_offset, data_len)) = match (fmt, data) {
        (Some(info), Some(data)) => (info, data),
        _ => return Err("missing fmt or data chunk".into()),
    };
    info.data_offset = data_offset;
    info.data_len = data_len;
    Ok(info)
}

/// Writes a stereo copy of the mono WAV `src` (described by `info`) to
/// `dst`: a canonical 44-byte header (`RIFF`, `WAVE`, a 16-byte `fmt ` with
/// `info.format`, 2 channels, the same rate and bit depth, doubled
/// `block_align` and byte rate, then `data`), followed by every source
/// frame written twice (left, then right). Streams in bounded chunks --
/// never the whole file in memory.
fn write_upmixed(info: &WavInfo, src: &mut File, dst: &mut File) -> io::Result<()> {
    const CHUNK_BYTES: usize = 64 * 1024;
    let too_big = || io::Error::new(io::ErrorKind::InvalidInput, "stem too large for a WAV file");

    let width = usize::from(info.block_align.max(1));
    let frames = info.data_len / width as u64;
    let out_data_len = u32::try_from(frames * width as u64 * 2).map_err(|_| too_big())?;
    let out_align = u16::try_from(width * 2).map_err(|_| too_big())?;
    let byte_rate = info.sample_rate.checked_mul(u32::from(out_align)).ok_or_else(too_big)?;
    let riff_size = out_data_len.checked_add(36).ok_or_else(too_big)?;

    let mut header = Vec::with_capacity(44);
    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&riff_size.to_le_bytes());
    header.extend_from_slice(b"WAVE");
    header.extend_from_slice(b"fmt ");
    header.extend_from_slice(&16u32.to_le_bytes());
    header.extend_from_slice(&info.format.to_le_bytes());
    header.extend_from_slice(&2u16.to_le_bytes());
    header.extend_from_slice(&info.sample_rate.to_le_bytes());
    header.extend_from_slice(&byte_rate.to_le_bytes());
    header.extend_from_slice(&out_align.to_le_bytes());
    header.extend_from_slice(&info.bits_per_sample.to_le_bytes());
    header.extend_from_slice(b"data");
    header.extend_from_slice(&out_data_len.to_le_bytes());
    dst.write_all(&header)?;

    // Whole frames per chunk, about 64 KiB of source per round.
    let frames_per_chunk = (CHUNK_BYTES / width).max(1) as u64;
    let mut input = vec![0u8; frames_per_chunk as usize * width];
    let mut output = Vec::with_capacity(input.len() * 2);
    src.seek(SeekFrom::Start(info.data_offset))?;
    let mut remaining = frames;
    while remaining > 0 {
        let n = remaining.min(frames_per_chunk);
        let bytes = n as usize * width;
        src.read_exact(&mut input[..bytes])?;
        output.clear();
        for frame in input[..bytes].chunks_exact(width) {
            output.extend_from_slice(frame);
            output.extend_from_slice(frame);
        }
        dst.write_all(&output)?;
        remaining -= n;
    }
    Ok(())
}

/// A stem id from the file name: its stem (no extension), lowercased, every
/// run of characters other than ASCII letters/digits turned into one `-`,
/// no leading/trailing `-`; `stem` if nothing is left. If the id is in
/// `taken`, append `-2`, `-3`, ... until it isn't.
fn stem_id(file: &Path, taken: &[String]) -> String {
    let stem = file
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut base = String::new();
    let mut separator_pending = false;
    for c in stem.chars() {
        if c.is_ascii_alphanumeric() {
            if separator_pending && !base.is_empty() {
                base.push('-');
            }
            separator_pending = false;
            base.push(c.to_ascii_lowercase());
        } else {
            separator_pending = true;
        }
    }
    if base.is_empty() {
        base = "stem".into();
    }
    if !taken.contains(&base) {
        return base;
    }
    let mut n = 2;
    loop {
        let candidate = format!("{base}-{n}");
        if !taken.contains(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

#[cfg(test)]
mod tests {
    // The contract for TASK-003 (openspec/changes/task-003-stem-importer).
    // Written before the implementation, by a different author than it --
    // never loosen one of these to make it pass (AGENTS.md, testing policy).
    // Fixtures: src-tauri/tests/fixtures/stems/, made by generate.py there.
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/stems")
            .join(name)
    }

    fn header(name: &str) -> WavInfo {
        read_wav_header(&mut File::open(fixture(name)).unwrap()).unwrap()
    }

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "stemloft-import-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn files_in(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    fn importer() -> StemImporter {
        StemImporter::new(Arc::new(ProjectPersistence::new()))
    }

    fn data_of(path: &Path) -> (WavInfo, Vec<u8>) {
        let mut file = File::open(path).unwrap();
        let info = read_wav_header(&mut file).unwrap();
        let mut data = vec![0; info.data_len as usize];
        file.seek(SeekFrom::Start(info.data_offset)).unwrap();
        file.read_exact(&mut data).unwrap();
        (info, data)
    }

    /// A project folder that already holds one 44.1 kHz stem, `guitar`.
    fn project_with_guitar(label: &str) -> (PathBuf, Vec<StemEntry>) {
        let dir = temp_dir(label);
        fs::copy(fixture("stereo-44100-16.wav"), dir.join("guitar.wav")).unwrap();
        let existing = vec![StemEntry {
            id: "guitar".into(),
            file: "guitar.wav".into(),
            duration_sec: 0.2,
            waveform_cache: None,
        }];
        (dir, existing)
    }

    // ---- headers --------------------------------------------------------

    #[test]
    fn reads_plain_pcm_headers() {
        let mono = header("mono-44100-16.wav");
        assert_eq!(
            (mono.format, mono.channels, mono.sample_rate, mono.bits_per_sample, mono.block_align),
            (1, 1, 44100, 16, 2)
        );
        assert_eq!(mono.data_offset, 44);
        assert_eq!(mono.data_len, 4410 * 2);

        let stereo = header("stereo-48000-16.wav");
        assert_eq!(
            (stereo.channels, stereo.sample_rate, stereo.block_align),
            (2, 48000, 4)
        );
        assert_eq!(stereo.data_len, 2400 * 4);
    }

    #[test]
    fn reads_float_and_extensible_headers_and_skips_unknown_chunks() {
        let float = header("mono-44100-float.wav");
        assert_eq!((float.format, float.bits_per_sample, float.block_align), (3, 32, 4));

        // fmt (extensible, 40-byte body), then a LIST chunk with an odd
        // size and its pad byte, then data.
        let ext = header("mono-44100-24-extensible.wav");
        assert_eq!(
            (ext.format, ext.channels, ext.bits_per_sample, ext.block_align),
            (1, 1, 24, 3)
        );
        assert_eq!(ext.data_offset, 12 + 8 + 40 + 8 + 22 + 8);
        assert_eq!(ext.data_len, 4410 * 3);
    }

    #[test]
    fn duration_comes_from_the_header() {
        assert_eq!(header("mono-44100-16.wav").duration_sec(), 0.1);
        assert_eq!(header("stereo-44100-16.wav").duration_sec(), 0.2);
        assert_eq!(header("stereo-48000-16.wav").duration_sec(), 0.05);
        assert_eq!(header("mono-44100-float.wav").duration_sec(), 0.05);
        assert_eq!(header("mono-44100-24-extensible.wav").duration_sec(), 0.1);
    }

    fn header_of_bytes(label: &str, bytes: &[u8]) -> Result<WavInfo, String> {
        let dir = temp_dir(label);
        let path = dir.join("x.wav");
        fs::write(&path, bytes).unwrap();
        let result = read_wav_header(&mut File::open(&path).unwrap());
        fs::remove_dir_all(&dir).ok();
        result
    }

    #[test]
    fn rejects_what_is_not_a_supported_wav() {
        let good = fs::read(fixture("mono-44100-16.wav")).unwrap();

        assert!(header_of_bytes("text", b"just some text, not audio").is_err());
        assert!(header_of_bytes("empty", b"").is_err());

        let mut not_wave = good.clone();
        not_wave[8..12].copy_from_slice(b"AVI ");
        assert!(header_of_bytes("avi", &not_wave).is_err());

        // MPEG-in-WAV (format tag 0x55) is not PCM/float.
        let mut mp3 = good.clone();
        mp3[20..22].copy_from_slice(&0x55u16.to_le_bytes());
        assert!(header_of_bytes("mp3", &mp3).is_err());

        let mut bad_align = good.clone();
        bad_align[32..34].copy_from_slice(&3u16.to_le_bytes());
        assert!(header_of_bytes("align", &bad_align).is_err());

        let mut zero_rate = good.clone();
        zero_rate[24..28].copy_from_slice(&0u32.to_le_bytes());
        assert!(header_of_bytes("rate", &zero_rate).is_err());

        // Header promises more samples than the file holds.
        assert!(header_of_bytes("truncated", &good[..good.len() - 100]).is_err());

        // No data chunk at all.
        assert!(header_of_bytes("no-data", &good[..36]).is_err());
    }

    // ---- ids ------------------------------------------------------------

    #[test]
    fn stem_ids_are_slugs_unique_in_the_project() {
        let none: Vec<String> = vec![];
        assert_eq!(stem_id(Path::new("/x/Bass.wav"), &none), "bass");
        assert_eq!(stem_id(Path::new("/x/Lead Vocals (take 2).WAV"), &none), "lead-vocals-take-2");
        assert_eq!(stem_id(Path::new("/x/__drums__.wav"), &none), "drums");
        assert_eq!(stem_id(Path::new("/x/ação.wav"), &none), "a-o");
        assert_eq!(stem_id(Path::new("/x/!!!.wav"), &none), "stem");

        let taken = vec!["bass".to_string(), "bass-2".to_string()];
        assert_eq!(stem_id(Path::new("/x/bass.wav"), &taken), "bass-3");
    }

    // ---- upmix ----------------------------------------------------------

    #[test]
    fn upmix_writes_a_stereo_wav_with_the_same_signal_on_both_channels() {
        for name in ["mono-44100-16.wav", "mono-44100-float.wav", "mono-44100-24-extensible.wav"] {
            let dir = temp_dir("upmix");
            let out = dir.join("out.wav");
            let (mono, mono_data) = data_of(&fixture(name));

            write_upmixed(
                &mono,
                &mut File::open(fixture(name)).unwrap(),
                &mut File::create(&out).unwrap(),
            )
            .unwrap();

            let (stereo, stereo_data) = data_of(&out);
            assert_eq!(stereo.channels, 2, "{name}");
            assert_eq!(stereo.format, mono.format, "{name}");
            assert_eq!(stereo.sample_rate, mono.sample_rate, "{name}");
            assert_eq!(stereo.bits_per_sample, mono.bits_per_sample, "{name}");
            assert_eq!(stereo.block_align, mono.block_align * 2, "{name}");
            assert_eq!(stereo.data_offset, 44, "{name}");
            assert_eq!(stereo.duration_sec(), mono.duration_sec(), "{name}");

            let width = mono.block_align as usize;
            for (frame, sample) in mono_data.chunks(width).enumerate() {
                let left = &stereo_data[frame * 2 * width..frame * 2 * width + width];
                let right = &stereo_data[frame * 2 * width + width..(frame + 1) * 2 * width];
                assert_eq!((left, right), (sample, sample), "{name}, frame {frame}");
            }
            // Byte rate and RIFF size are consistent too.
            let bytes = fs::read(&out).unwrap();
            let byte_rate = u32::from_le_bytes(bytes[28..32].try_into().unwrap());
            assert_eq!(byte_rate, stereo.sample_rate * stereo.block_align as u32, "{name}");
            let riff_size = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
            assert_eq!(riff_size as usize, bytes.len() - 8, "{name}");
            fs::remove_dir_all(&dir).ok();
        }
    }

    // ---- import ---------------------------------------------------------

    #[test]
    fn imports_into_an_empty_project_upmixing_mono_and_copying_stereo() {
        let dir = temp_dir("empty");

        let entries = importer()
            .import(
                &dir,
                &[],
                &[fixture("mono-44100-16.wav"), fixture("stereo-44100-16.wav")],
            )
            .unwrap();

        assert_eq!(
            entries,
            vec![
                StemEntry {
                    id: "mono-44100-16".into(),
                    file: "mono-44100-16.wav".into(),
                    duration_sec: 0.1,
                    waveform_cache: None,
                },
                StemEntry {
                    id: "stereo-44100-16".into(),
                    file: "stereo-44100-16.wav".into(),
                    duration_sec: 0.2,
                    waveform_cache: None,
                },
            ]
        );
        assert_eq!(files_in(&dir), vec!["mono-44100-16.wav", "stereo-44100-16.wav"]);
        assert_eq!(data_of(&dir.join("mono-44100-16.wav")).0.channels, 2);
        assert_eq!(
            fs::read(dir.join("stereo-44100-16.wav")).unwrap(),
            fs::read(fixture("stereo-44100-16.wav")).unwrap()
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_stem_at_another_sample_rate_is_rejected_naming_both_rates() {
        let (dir, existing) = project_with_guitar("mismatch");

        let err = importer()
            .import(&dir, &existing, &[fixture("stereo-48000-16.wav")])
            .unwrap_err();

        assert_eq!(
            err,
            ImportError::SampleRateMismatch {
                file: fixture("stereo-48000-16.wav"),
                found: 48000,
                expected: 44100,
            }
        );
        let message = err.to_string();
        assert!(message.contains("48000") && message.contains("44100"), "{message}");
        assert_eq!(files_in(&dir), vec!["guitar.wav"], "nothing may be written");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn one_bad_file_rejects_the_whole_import() {
        let dir = temp_dir("all-or-nothing");

        // The first file sets the rate for an empty project.
        let err = importer()
            .import(
                &dir,
                &[],
                &[fixture("mono-44100-16.wav"), fixture("stereo-48000-16.wav")],
            )
            .unwrap_err();
        assert!(matches!(
            err,
            ImportError::SampleRateMismatch { found: 48000, expected: 44100, .. }
        ));

        let err = importer()
            .import(&dir, &[], &[fixture("mono-44100-16.wav"), fixture("quad-44100-16.wav")])
            .unwrap_err();
        assert_eq!(
            err,
            ImportError::TooManyChannels {
                file: fixture("quad-44100-16.wav"),
                channels: 4
            }
        );

        let err = importer()
            .import(&dir, &[], &[fixture("mono-44100-16.wav"), dir.join("missing.wav")])
            .unwrap_err();
        assert!(matches!(err, ImportError::Io { .. }), "{err:?}");

        let err = importer()
            .import(&dir, &[], &[fixture("generate.py")])
            .unwrap_err();
        assert!(matches!(err, ImportError::Unsupported { .. }), "{err:?}");

        assert!(files_in(&dir).is_empty(), "nothing may be written");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn new_stems_get_ids_that_do_not_collide_with_existing_ones() {
        let (dir, existing) = project_with_guitar("ids");
        let source = temp_dir("ids-source");
        fs::copy(fixture("mono-44100-16.wav"), source.join("Guitar.wav")).unwrap();

        let entries = importer()
            .import(
                &dir,
                &existing,
                &[source.join("Guitar.wav"), source.join("Guitar.wav")],
            )
            .unwrap();

        let ids: Vec<&str> = entries.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["guitar-2", "guitar-3"]);
        assert_eq!(files_in(&dir), vec!["guitar-2.wav", "guitar-3.wav", "guitar.wav"]);
        fs::remove_dir_all(&dir).ok();
        fs::remove_dir_all(&source).ok();
    }

    #[test]
    fn an_import_never_overwrites_a_file_already_in_the_project_folder() {
        let (dir, existing) = project_with_guitar("stray");
        fs::write(dir.join("bass.wav"), b"not part of the project").unwrap();
        let source = temp_dir("stray-source");
        fs::copy(fixture("stereo-44100-16.wav"), source.join("bass.wav")).unwrap();

        let entries = importer()
            .import(&dir, &existing, &[source.join("bass.wav")])
            .unwrap();

        assert_eq!(entries[0].id, "bass-2");
        assert_eq!(fs::read(dir.join("bass.wav")).unwrap(), b"not part of the project");
        fs::remove_dir_all(&dir).ok();
        fs::remove_dir_all(&source).ok();
    }

    #[test]
    fn importing_into_a_missing_project_folder_is_an_io_error() {
        let dir = temp_dir("no-folder").join("not-created");
        let err = importer()
            .import(&dir, &[], &[fixture("mono-44100-16.wav")])
            .unwrap_err();
        assert!(matches!(err, ImportError::Io { .. }), "{err:?}");
    }
}
