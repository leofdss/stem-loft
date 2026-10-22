//! `Score Metadata Manager` -- parses the `.cho` (extended ChordPro) file
//! referenced by the project: chords, tablature, lyrics, metronome, tuning.
//! See docs/architecture.md#score-metadata-chords-tablature-and-lyrics and
//! the `chordpro-format` playbook for the grammar implemented here (one
//! chord per line, `{t: m:ss.cc}` time anchors, tablature technique
//! notation, `startSec`/`endSec` resolution).
//!
//! Read-only in v1: this module parses, it never writes the `.cho`. A parse
//! failure is a [`ScoreParseError`] (the `.cho` line and a message) handed
//! back to the caller -- never a panic, and nothing here touches playback,
//! so the rest of the app keeps working with a broken score.
//!
//! Tablature (`{start_of_tab}` ... `{end_of_tab}`) is an ASCII staff: six
//! lines, string 1 (highest) on top, each `<name>|<columns>|`. Every run of
//! non-`-` characters on a line is one note in the playbook's `note`
//! grammar, with its `.string` given by the line it sits on.

// Not called by a `Command` yet: the `score_loaded`/`score_parse_error`
// events and the call from `Session` land with TASK-012.
#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

use serde::{Serialize, Serializer};

/// A parsed `.cho` file -- the payload behind `score_loaded`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub key: Option<String>,
    pub tempo: Option<f64>,
    pub time: Option<TimeSignature>,
    /// Lowest string to highest, as written in `{tuning}`.
    pub tuning: Option<Vec<String>>,
    /// Bar 1, beat 1: the first `{t:}` in the file, not second 0 of the
    /// audio. `None` when the file has no anchor at all.
    pub grid_origin_sec: Option<f64>,
    /// Every `[chord]`, in file order, with its resolved window.
    pub chords: Vec<TimedChord>,
    /// Every chord/lyrics line, in file order.
    pub lines: Vec<LyricLine>,
    pub tabs: Vec<TabBlock>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TimeSignature {
    pub numerator: u32,
    pub denominator: u32,
}

/// Fingering from `{define}`: base fret plus one entry per string, lowest
/// string first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChordDiagram {
    pub base_fret: u32,
    pub frets: Vec<DiagramFret>,
}

/// One string in a [`ChordDiagram`]: serialized as its fret number, or
/// `"x"` for a muted string (the shape `ChordDiagram.frets` has in Angular).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramFret {
    Muted,
    Fret(u8),
}

impl Serialize for DiagramFret {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            DiagramFret::Muted => serializer.serialize_str("x"),
            DiagramFret::Fret(n) => serializer.serialize_u8(*n),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimedChord {
    pub name: String,
    pub start_sec: f64,
    pub end_sec: f64,
    pub diagram: Option<ChordDiagram>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricLine {
    /// The `{t:}` anchoring this line, if it has its own.
    pub start_sec: Option<f64>,
    pub chord: Option<String>,
    /// Character index in `text` where the chord enters.
    pub chord_column: Option<usize>,
    /// The line with the `[chord]` removed.
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TabBlock {
    pub start_sec: f64,
    pub end_sec: f64,
    /// The six staff lines, verbatim, for display.
    pub lines: Vec<String>,
    pub notes: Vec<TabNote>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TabNote {
    /// 1 = highest string (the top staff line) ... 6 = lowest.
    pub string: u8,
    /// Index of the note's first character within the staff columns
    /// (after the leading `<name>|`).
    pub column: usize,
    /// `startSec + column / totalColumns * (endSec - startSec)`.
    pub start_sec: f64,
    pub steps: Vec<TabStep>,
}

/// One step of a tab note: the first is always `Fret`, the rest follow the
/// playbook's `op fret | "~"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "fret", rename_all = "camelCase")]
pub enum TabStep {
    Fret(u8),
    HammerOn(u8),
    PullOff(u8),
    Bend(u8),
    Release(u8),
    SlideUp(u8),
    SlideDown(u8),
    Vibrato,
}

/// The payload behind `score_parse_error`: the 1-based `.cho` line where
/// parsing failed (0 when the file couldn't be read at all) and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreParseError {
    pub line: u32,
    pub message: String,
}

impl fmt::Display for ScoreParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ScoreParseError {}

pub struct ScoreMetadataManager;

impl ScoreMetadataManager {
    pub fn new() -> Self {
        Self
    }

    /// Reads and parses the `.cho` at `path`. `project_duration_sec` is the
    /// project's largest `stems[].durationSec` (from the project `.json`,
    /// never decoded here): it closes the last chord/tab block.
    pub fn load_score(
        &self,
        path: &Path,
        project_duration_sec: f64,
    ) -> Result<Score, ScoreParseError> {
        let text = fs::read_to_string(path).map_err(|e| ScoreParseError {
            line: 0,
            message: format!("couldn't read {}: {e}", path.display()),
        })?;
        self.parse(&text, project_duration_sec)
    }

    /// Parses `.cho` text. See the module doc and the `chordpro-format`
    /// playbook for the rules.
    pub fn parse(&self, text: &str, project_duration_sec: f64) -> Result<Score, ScoreParseError> {
        let mut parser = Parser::new(project_duration_sec);
        for (index, raw) in text.lines().enumerate() {
            parser.line(raw, index as u32 + 1)?;
        }
        parser.finish()
    }
}

impl Default for ScoreMetadataManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---- Directive and line helpers ------------------------------------------
// Each one works on a single line's text and reports a plain message; the
// parser below adds the line number.

/// ASCII digits only (a leading `+` or `-` is not a digit), as a `u64`;
/// `None` when empty, not all digits, or too large for a `u64`.
fn digits(s: &str) -> Option<u64> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

/// `{t:}`'s value -> seconds. `minute ":" second "." hundredth`: minute is
/// 1+ ASCII digits, second exactly 2 digits (00-59), hundredth exactly 2
/// digits. Surrounding whitespace is ignored; anything else (including a
/// minute too large for a `u64`) is an error, never a panic.
fn parse_time_anchor(value: &str) -> Result<f64, String> {
    let value = value.trim();
    let bad = || format!("expected `m:ss.cc`, got `{value}`");
    let (minute, rest) = value.split_once(':').ok_or_else(bad)?;
    let (second, hundredth) = rest.split_once('.').ok_or_else(bad)?;
    let minute = digits(minute).ok_or_else(bad)?;
    let second = digits(second)
        .filter(|_| second.len() == 2)
        .ok_or_else(bad)?;
    let hundredth = digits(hundredth)
        .filter(|_| hundredth.len() == 2)
        .ok_or_else(bad)?;
    if second > 59 {
        return Err(format!("second {second:02} is outside 00-59"));
    }
    Ok(minute as f64 * 60.0 + second as f64 + hundredth as f64 / 100.0)
}

/// A directive line: `Ok(None)` when `line` (already trimmed) doesn't start
/// with `{`; otherwise `(name, value, rest)` -- name lowercased, ending at
/// the first `:` or whitespace; value trimmed (empty when there is none);
/// rest = whatever follows the closing `}`, trimmed. `Err` when there's no
/// closing `}`.
fn split_directive(line: &str) -> Result<Option<(String, String, String)>, String> {
    let Some(body) = line.strip_prefix('{') else {
        return Ok(None);
    };
    let Some(close) = body.find('}') else {
        return Err(format!("`{line}` has no closing `}}`"));
    };
    let inside = &body[..close];
    let rest = body[close + 1..].trim().to_string();
    let name_end = inside
        .find(|c: char| c == ':' || c.is_whitespace())
        .unwrap_or(inside.len());
    let name = inside[..name_end].to_lowercase();
    let after = inside[name_end..].trim_start();
    let after = after.strip_prefix(':').unwrap_or(after);
    Ok(Some((name, after.trim().to_string(), rest)))
}

/// `{tempo}`: a positive, finite number of beats per minute (`80`, `92.5`).
fn parse_tempo(value: &str) -> Result<f64, String> {
    let value = value.trim();
    let tempo: f64 = value
        .parse()
        .map_err(|_| format!("tempo `{value}` is not a number"))?;
    if tempo.is_finite() && tempo > 0.0 {
        Ok(tempo)
    } else {
        Err(format!(
            "tempo must be a positive, finite number, got `{value}`"
        ))
    }
}

/// `{time}`: `numerator/denominator`, both integers > 0, whitespace around
/// either part allowed.
fn parse_time_signature(value: &str) -> Result<TimeSignature, String> {
    let part = |s: &str| {
        digits(s.trim())
            .and_then(|n| u32::try_from(n).ok())
            .filter(|n| *n > 0)
    };
    let (num, den) = value
        .split_once('/')
        .ok_or_else(|| format!("time must be `numerator/denominator`, got `{value}`"))?;
    match (part(num), part(den)) {
        (Some(numerator), Some(denominator)) => Ok(TimeSignature {
            numerator,
            denominator,
        }),
        _ => Err(format!("time must be two integers above 0, got `{value}`")),
    }
}

/// `{tuning}`: exactly 6 whitespace-separated note names, lowest string
/// first.
fn parse_tuning(value: &str) -> Result<Vec<String>, String> {
    let names: Vec<String> = value.split_whitespace().map(String::from).collect();
    if names.len() == 6 {
        Ok(names)
    } else {
        Err(format!(
            "tuning needs 6 note names, lowest string first, got {}",
            names.len()
        ))
    }
}

/// `{define}`: `NAME [base-fret N] frets F F F F F F [fingers ...]`.
/// `base-fret` defaults to 1 and must be >= 1; `frets` is required and has
/// exactly 6 entries, each `x`/`X`/`N` (muted) or an integer 0-24; whatever
/// follows `fingers` is ignored; any other keyword is an error.
fn parse_define(value: &str) -> Result<(String, ChordDiagram), String> {
    let fret = |token: &str| -> Result<DiagramFret, String> {
        if matches!(token, "x" | "X" | "N") {
            return Ok(DiagramFret::Muted);
        }
        digits(token)
            .and_then(|n| u8::try_from(n).ok())
            .filter(|n| *n <= 24)
            .map(DiagramFret::Fret)
            .ok_or_else(|| format!("fret must be x, X, N or 0-24, got `{token}`"))
    };

    let mut tokens = value.split_whitespace();
    let name = tokens.next().ok_or("`{define}` needs a chord name")?;
    let mut base_fret = 1;
    let mut frets = None;
    while let Some(keyword) = tokens.next() {
        match keyword {
            "base-fret" => {
                let token = tokens.next().ok_or("`base-fret` needs a number")?;
                base_fret = digits(token)
                    .and_then(|n| u32::try_from(n).ok())
                    .filter(|n| *n >= 1)
                    .ok_or_else(|| format!("base-fret must be 1 or more, got `{token}`"))?;
            }
            "frets" => {
                let mut list = Vec::with_capacity(6);
                for _ in 0..6 {
                    let token = tokens.next().ok_or("`frets` needs exactly 6 entries")?;
                    list.push(fret(token)?);
                }
                frets = Some(list);
            }
            "fingers" => break,
            other => return Err(format!("unknown `{{define}}` keyword `{other}`")),
        }
    }
    let frets = frets.ok_or("`{define}` needs `frets`")?;
    Ok((name.to_string(), ChordDiagram { base_fret, frets }))
}

/// A chord/lyrics line -> `(Some((chord, column)), text)` or `(None, text)`.
/// `text` is the line with the `[chord]` removed; `column` is the number of
/// characters (not bytes) before the chord in `text`. Errors: more than one
/// `[chord]` on the line (the dialect allows at most one; the message must
/// contain "one chord per line"), a `[` with no `]`, a `]` with no `[`, an
/// empty `[]`.
fn parse_chord_line(line: &str) -> Result<(Option<(String, usize)>, String), String> {
    let mut text = String::new();
    let mut chord: Option<(String, usize)> = None;
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match c {
            '[' => {
                if chord.is_some() {
                    return Err(
                        "more than one chord on a line; the dialect allows one chord per line"
                            .into(),
                    );
                }
                let mut name = String::new();
                loop {
                    match chars.next() {
                        Some(']') => break,
                        Some('[') => return Err("`[` inside a chord name".into()),
                        Some(ch) => name.push(ch),
                        None => return Err(format!("`[{name}` has no closing `]`")),
                    }
                }
                let name = name.trim().to_string();
                if name.is_empty() {
                    return Err("empty chord `[]`".into());
                }
                chord = Some((name, text.chars().count()));
            }
            ']' => return Err("`]` with no `[` before it".into()),
            other => text.push(other),
        }
    }
    Ok((chord, text))
}

/// One tab note (a run of non-`-` characters on a staff line):
/// `fret (op fret | "~")*` with `op` in `h p b r / \`. A fret is 1-2 digits,
/// 0-24, no leading zero; `r` needs an earlier `b` in the same note.
fn parse_tab_note(token: &str) -> Result<Vec<TabStep>, String> {
    let mut chars = token.chars().peekable();
    let mut steps = Vec::new();
    let mut bent = false;

    let fret =
        |chars: &mut std::iter::Peekable<std::str::Chars<'_>>, after: &str| -> Result<u8, String> {
            let mut digits = String::new();
            while digits.len() < 2 {
                match chars.peek() {
                    Some(c) if c.is_ascii_digit() => {
                        digits.push(*c);
                        chars.next();
                    }
                    _ => break,
                }
            }
            if digits.is_empty() {
                return Err(format!("`{token}`: expected a fret number {after}"));
            }
            if digits.len() == 2 && digits.starts_with('0') {
                return Err(format!("`{token}`: fret `{digits}` has a leading zero"));
            }
            let n: u8 = digits.parse().expect("1-2 ASCII digits");
            if n > 24 {
                return Err(format!("`{token}`: fret {n} is outside 0-24"));
            }
            Ok(n)
        };

    steps.push(TabStep::Fret(fret(&mut chars, "at the start of the note")?));
    while let Some(c) = chars.next() {
        let step = match c {
            '~' => TabStep::Vibrato,
            'h' => TabStep::HammerOn(fret(&mut chars, "after `h`")?),
            'p' => TabStep::PullOff(fret(&mut chars, "after `p`")?),
            'b' => {
                bent = true;
                TabStep::Bend(fret(&mut chars, "after `b`")?)
            }
            'r' => {
                if !bent {
                    return Err(format!("`{token}`: release `r` with no bend `b` before it"));
                }
                TabStep::Release(fret(&mut chars, "after `r`")?)
            }
            '/' => TabStep::SlideUp(fret(&mut chars, "after `/`")?),
            '\\' => TabStep::SlideDown(fret(&mut chars, "after `\\`")?),
            other => return Err(format!("`{token}`: unexpected `{other}`")),
        };
        steps.push(step);
    }
    Ok(steps)
}

// ---- The parser ----------------------------------------------------------

/// A tab block still being read: its anchor and raw staff lines.
struct OpenTab {
    start_line: u32,
    start_sec: f64,
    staff: Vec<(u32, String)>,
}

/// A chord or tab block, in file order: the next one closes the previous.
enum Event {
    Chord(usize),
    Tab(usize),
}

/// A tab block's notes before its `end_sec` is known: (string, column,
/// steps), plus the staff width they're timed against.
struct PendingNotes {
    total_columns: usize,
    notes: Vec<(u8, usize, Vec<TabStep>)>,
}

struct Parser {
    duration_sec: f64,
    score: Score,
    defines: HashMap<String, ChordDiagram>,
    /// The latest `{t:}` not yet claimed by a line.
    pending_anchor: Option<f64>,
    last_anchor: Option<f64>,
    open_tab: Option<OpenTab>,
    events: Vec<(f64, Event)>,
    tab_notes: Vec<PendingNotes>,
}

impl Parser {
    fn new(duration_sec: f64) -> Self {
        Parser {
            // NaN or negative (no stems yet): treat as an empty project.
            duration_sec: if duration_sec.is_finite() {
                duration_sec.max(0.0)
            } else {
                0.0
            },
            score: Score {
                title: None,
                artist: None,
                key: None,
                tempo: None,
                time: None,
                tuning: None,
                grid_origin_sec: None,
                chords: Vec::new(),
                lines: Vec::new(),
                tabs: Vec::new(),
            },
            defines: HashMap::new(),
            pending_anchor: None,
            last_anchor: None,
            open_tab: None,
            events: Vec::new(),
            tab_notes: Vec::new(),
        }
    }

    fn line(&mut self, raw: &str, line: u32) -> Result<(), ScoreParseError> {
        let err = |message: String| ScoreParseError { line, message };
        // A UTF-8 byte-order mark on the first line isn't content.
        let text = raw.trim_start_matches('\u{feff}').trim();

        if self.open_tab.is_some() {
            return self.tab_line(text, line);
        }
        if text.is_empty() || text.starts_with('#') {
            return Ok(());
        }

        let Some((name, value, rest)) = split_directive(text).map_err(err)? else {
            return self.content(text, line);
        };
        if name == "t" {
            self.anchor(&value, line)?;
            // `{t: ...}[G]lyrics`: the rest is this anchor's line. One level
            // only -- a second directive there is an error, not recursion.
            if rest.is_empty() {
                return Ok(());
            }
            return match split_directive(&rest).map_err(err)? {
                None => self.content(&rest, line),
                Some((next, _, _)) if next == "start_of_tab" || next == "sot" => {
                    self.directive(&next, String::new(), String::new(), line)
                }
                Some((next, _, _)) => Err(err(format!(
                    "{{{next}}} after a {{t:}} on the same line; put it on its own line"
                ))),
            };
        }
        self.directive(&name, value, rest, line)
    }

    /// Every directive but `{t:}`. Text after its `}` is an error rather
    /// than silently dropped.
    fn directive(
        &mut self,
        name: &str,
        value: String,
        rest: String,
        line: u32,
    ) -> Result<(), ScoreParseError> {
        let err = |message: String| ScoreParseError { line, message };
        if !rest.is_empty() {
            return Err(err(format!(
                "text after {{{name}}} on the same line; put `{rest}` on its own line"
            )));
        }
        match name {
            "title" => self.score.title = Some(value),
            "artist" => self.score.artist = Some(value),
            "key" => self.score.key = Some(value),
            "tempo" => self.score.tempo = Some(parse_tempo(&value).map_err(err)?),
            "time" => self.score.time = Some(parse_time_signature(&value).map_err(err)?),
            "tuning" => self.score.tuning = Some(parse_tuning(&value).map_err(err)?),
            "define" => {
                let (chord, diagram) = parse_define(&value).map_err(err)?;
                self.defines.insert(chord, diagram);
            }
            "start_of_tab" | "sot" => {
                let start_sec = self
                    .pending_anchor
                    .take()
                    .ok_or_else(|| err("tab block has no {t:} anchor before it".into()))?;
                self.open_tab = Some(OpenTab {
                    start_line: line,
                    start_sec,
                    staff: Vec::new(),
                });
            }
            "end_of_tab" | "eot" => {
                return Err(err("{end_of_tab} with no {start_of_tab} before it".into()));
            }
            // Unknown directives (and known ones StemLoft doesn't use, like
            // `{c:}` or `{capo}`) are the format's extension mechanism:
            // ignored, never an error.
            _ => {}
        }
        Ok(())
    }

    /// `{t:}`: must parse, must not go backwards, and must be claimed by a
    /// line before the next one. An anchor past the project's end is fine
    /// (stems can be re-imported shorter, durations round): `finish` never
    /// lets an end come before its start.
    fn anchor(&mut self, value: &str, line: u32) -> Result<(), ScoreParseError> {
        let err = |message: String| ScoreParseError { line, message };
        let sec = parse_time_anchor(value)
            .map_err(|m| err(format!("invalid time anchor {{t: {value}}}: {m}")))?;
        if self.pending_anchor.is_some() {
            return Err(err(format!(
                "time anchor {{t: {value}}} follows another {{t:}} that no line used"
            )));
        }
        if let Some(last) = self.last_anchor {
            if sec < last {
                return Err(err(format!(
                    "time anchor {{t: {value}}} is earlier than the previous one"
                )));
            }
        }
        self.last_anchor = Some(sec);
        self.pending_anchor = Some(sec);
        self.score.grid_origin_sec.get_or_insert(sec);
        Ok(())
    }

    /// A line inside `{start_of_tab}` ... `{end_of_tab}`: a staff line, a
    /// blank, a `#` comment or a `{c:}`/`{comment:}` (both ignored).
    fn tab_line(&mut self, text: &str, line: u32) -> Result<(), ScoreParseError> {
        let err = |message: String| ScoreParseError { line, message };
        if text.is_empty() || text.starts_with('#') {
            return Ok(());
        }
        match split_directive(text).map_err(err)? {
            Some((name, _, _)) if name == "end_of_tab" || name == "eot" => self.close_tab(),
            Some((name, _, _)) if name == "c" || name == "comment" => Ok(()),
            Some((name, _, _)) => Err(err(format!(
                "directive {{{name}}} inside a tab block; close it with {{end_of_tab}} first"
            ))),
            None => {
                let tab = self
                    .open_tab
                    .as_mut()
                    .expect("only called inside a tab block");
                tab.staff.push((line, text.to_string()));
                Ok(())
            }
        }
    }

    /// A chord/lyrics line: claims the pending anchor, if any.
    fn content(&mut self, text: &str, line: u32) -> Result<(), ScoreParseError> {
        let (chord, text) =
            parse_chord_line(text).map_err(|message| ScoreParseError { line, message })?;
        let start_sec = self.pending_anchor.take();
        if let Some((name, _)) = &chord {
            let start = start_sec.ok_or_else(|| ScoreParseError {
                line,
                message: format!("chord [{name}] has no {{t:}} anchor before it"),
            })?;
            self.events
                .push((start, Event::Chord(self.score.chords.len())));
            self.score.chords.push(TimedChord {
                name: name.clone(),
                start_sec: start,
                end_sec: start, // resolved in `finish`
                diagram: None,
            });
        }
        let (chord, chord_column) = match chord {
            Some((name, column)) => (Some(name), Some(column)),
            None => (None, None),
        };
        self.score.lines.push(LyricLine {
            start_sec,
            chord,
            chord_column,
            text,
        });
        Ok(())
    }

    fn close_tab(&mut self) -> Result<(), ScoreParseError> {
        let tab = self
            .open_tab
            .take()
            .expect("only called inside a tab block");
        if tab.staff.len() != 6 {
            return Err(ScoreParseError {
                line: tab.start_line,
                message: format!(
                    "a tab block needs exactly 6 string lines, found {}",
                    tab.staff.len()
                ),
            });
        }

        let mut total_columns = None;
        let mut notes = Vec::new();
        for (index, (line, staff_line)) in tab.staff.iter().enumerate() {
            let err = |message: String| ScoreParseError {
                line: *line,
                message,
            };
            let (_, after_name) = staff_line
                .split_once('|')
                .ok_or_else(|| err(format!("tab line `{staff_line}` has no `|`")))?;
            let columns: Vec<char> = after_name
                .strip_suffix('|')
                .unwrap_or(after_name)
                .chars()
                .collect();
            match total_columns {
                None => total_columns = Some(columns.len()),
                Some(n) if n != columns.len() => {
                    return Err(err(format!(
                        "tab line has {} columns, the first one has {n}",
                        columns.len()
                    )))
                }
                Some(_) => {}
            }

            // A note is a run of anything but `-` (a rest) or `|` (a bar line).
            let is_gap = |c: char| c == '-' || c == '|';
            let mut column = 0;
            while column < columns.len() {
                if is_gap(columns[column]) {
                    column += 1;
                    continue;
                }
                let start = column;
                while column < columns.len() && !is_gap(columns[column]) {
                    column += 1;
                }
                let token: String = columns[start..column].iter().collect();
                let steps = parse_tab_note(&token).map_err(err)?;
                notes.push((index as u8 + 1, start, steps));
            }
        }

        self.events
            .push((tab.start_sec, Event::Tab(self.score.tabs.len())));
        self.score.tabs.push(TabBlock {
            start_sec: tab.start_sec,
            end_sec: tab.start_sec, // resolved in `finish`
            lines: tab.staff.into_iter().map(|(_, l)| l).collect(),
            notes: Vec::new(),
        });
        self.tab_notes.push(PendingNotes {
            total_columns: total_columns.unwrap_or(0),
            notes,
        });
        Ok(())
    }

    /// Closes every chord and tab block at the next event's start (the
    /// project's end for the last one), then times the tab notes.
    fn finish(mut self) -> Result<Score, ScoreParseError> {
        if let Some(tab) = &self.open_tab {
            return Err(ScoreParseError {
                line: tab.start_line,
                message: "{start_of_tab} is never closed with {end_of_tab}".into(),
            });
        }

        for i in 0..self.events.len() {
            let start = self.events[i].0;
            let end = self
                .events
                .get(i + 1)
                .map_or(self.duration_sec.max(start), |(next, _)| *next);
            match self.events[i].1 {
                Event::Chord(c) => self.score.chords[c].end_sec = end,
                Event::Tab(t) => self.score.tabs[t].end_sec = end,
            }
        }

        for (tab, pending) in self.score.tabs.iter_mut().zip(self.tab_notes) {
            let span = tab.end_sec - tab.start_sec;
            // A block with no notes may have zero columns: never divided.
            tab.notes = pending
                .notes
                .into_iter()
                .map(|(string, column, steps)| TabNote {
                    string,
                    column,
                    start_sec: tab.start_sec + column as f64 / pending.total_columns as f64 * span,
                    steps,
                })
                .collect();
        }

        for chord in &mut self.score.chords {
            chord.diagram = self.defines.get(&chord.name).cloned();
        }
        Ok(self.score)
    }
}

#[cfg(test)]
mod tests {
    // The contract for TASK-011 (openspec/changes/task-011-cho-parser).
    // Written before the implementation, by a different author than it --
    // never loosen one of these to make it pass (AGENTS.md, testing policy).
    use super::*;

    fn parse(text: &str, duration: f64) -> Result<Score, ScoreParseError> {
        ScoreMetadataManager::new().parse(text, duration)
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// docs/architecture.md, "Full example -- chords + lyrics", verbatim.
    const CHORDS_EXAMPLE: &str = "{title: Study in G Major}
{artist: StemLoft - example}
{key: G}
{time: 4/4}
{tempo: 80}
{tuning: E A D G B E}

{define: G base-fret 1 frets 3 2 0 0 0 3}
{define: C base-fret 1 frets x 3 2 0 1 0}
{define: D base-fret 1 frets x x 0 2 3 2}

{t: 0:00.00}
[G]First time I pick up the guitar
{t: 0:03.00}
[C]My fingers still hurt, but I'll keep going
{t: 0:06.00}
[D]Every chord is a step I climb
{t: 0:09.00}
[G]One day I'll play this song without thinking
";

    /// docs/architecture.md, "Example with techniques", verbatim.
    const TAB_EXAMPLE: &str = r"{title: Phrase with Techniques}
{key: Am}
{time: 4/4}
{tempo: 100}
{tuning: E A D G B E}

{c: Instrumental interlude - A minor pentatonic}
{t: 0:00.00}
{start_of_tab}
E|----------------------------------------------------------------|
B|-5h8-----8p5---------------------------------------------8~-----|
G|-----------------7-----------------------7b9-----7b9r7----------|
D|-------------------------5/7-----7\5----------------------------|
A|----------------------------------------------------------------|
E|----------------------------------------------------------------|
{end_of_tab}
";

    fn staff(lines: [&str; 6]) -> String {
        lines.join("\n")
    }

    // ---- {t:} anchors ---------------------------------------------------

    #[test]
    fn time_anchor_accepts_the_documented_valid_forms() {
        assert_eq!(parse_time_anchor("0:00.00"), Ok(0.0));
        assert!(close(parse_time_anchor("1:05.30").unwrap(), 65.3));
        assert!(close(parse_time_anchor("12:40.00").unwrap(), 760.0));
        assert!(close(parse_time_anchor(" 0:03.00 ").unwrap(), 3.0));
    }

    #[test]
    fn time_anchor_rejects_wrong_digit_counts_and_ranges() {
        for bad in [
            "00.5",
            "1:5.3",
            "1:05.3",
            "1:05.300",
            "1:60.00",
            ":05.30",
            "1:05,30",
            "a:05.30",
            "1:0a.30",
            "1:05.3x",
            "",
            "1:05",
            "-1:05.30",
            "+1:05.30",
            "99999999999999999999:00.00",
        ] {
            assert!(parse_time_anchor(bad).is_err(), "accepted `{bad}`");
        }
    }

    // ---- directives -----------------------------------------------------

    #[test]
    fn split_directive_reads_name_value_and_rest() {
        let s = |n: &str, v: &str, r: &str| Ok(Some((n.to_string(), v.to_string(), r.to_string())));
        assert_eq!(
            split_directive("{title: Study in G}"),
            s("title", "Study in G", "")
        );
        assert_eq!(split_directive("{start_of_tab}"), s("start_of_tab", "", ""));
        assert_eq!(
            split_directive("{t: 0:03.00}[C]My fingers"),
            s("t", "0:03.00", "[C]My fingers")
        );
        assert_eq!(
            split_directive("{t: 0:03.00}  [C]x "),
            s("t", "0:03.00", "[C]x")
        );
        assert_eq!(split_directive("{TEMPO 80}"), s("tempo", "80", ""));
        assert_eq!(split_directive("{c:  a comment }"), s("c", "a comment", ""));
    }

    #[test]
    fn split_directive_ignores_other_lines_and_rejects_an_unclosed_brace() {
        assert_eq!(split_directive("[G]First time"), Ok(None));
        assert_eq!(split_directive("plain lyrics"), Ok(None));
        assert!(split_directive("{title: never closed").is_err());
    }

    #[test]
    fn tempo_time_and_tuning_parse_and_validate() {
        assert_eq!(parse_tempo("80"), Ok(80.0));
        assert_eq!(parse_tempo(" 92.5 "), Ok(92.5));
        for bad in ["0", "-5", "abc", "", "inf", "NaN"] {
            assert!(parse_tempo(bad).is_err(), "tempo accepted `{bad}`");
        }

        let four_four = TimeSignature {
            numerator: 4,
            denominator: 4,
        };
        assert_eq!(parse_time_signature("4/4"), Ok(four_four));
        assert_eq!(
            parse_time_signature(" 6 / 8 "),
            Ok(TimeSignature {
                numerator: 6,
                denominator: 8
            })
        );
        for bad in ["4", "0/4", "4/0", "a/4", "4/4/4", ""] {
            assert!(parse_time_signature(bad).is_err(), "time accepted `{bad}`");
        }

        assert_eq!(
            parse_tuning("E A D G B E"),
            Ok(vec!["E", "A", "D", "G", "B", "E"]
                .into_iter()
                .map(String::from)
                .collect())
        );
        assert_eq!(parse_tuning("D A D G B E").unwrap()[0], "D");
        assert!(parse_tuning("E A D G").is_err());
        assert!(parse_tuning("E A D G B E A").is_err());
    }

    #[test]
    fn define_reads_base_fret_and_six_frets() {
        assert_eq!(
            parse_define("C base-fret 1 frets x 3 2 0 1 0"),
            Ok((
                "C".to_string(),
                ChordDiagram {
                    base_fret: 1,
                    frets: vec![
                        DiagramFret::Muted,
                        DiagramFret::Fret(3),
                        DiagramFret::Fret(2),
                        DiagramFret::Fret(0),
                        DiagramFret::Fret(1),
                        DiagramFret::Fret(0),
                    ],
                }
            ))
        );
        let (name, barre) =
            parse_define("Bm7 base-fret 2 frets X 1 3 1 2 N fingers 0 1 3 1 2 0").unwrap();
        assert_eq!(name, "Bm7");
        assert_eq!(barre.base_fret, 2);
        assert_eq!(barre.frets[0], DiagramFret::Muted);
        assert_eq!(barre.frets[5], DiagramFret::Muted);
        // base-fret is optional.
        assert_eq!(parse_define("G frets 3 2 0 0 0 3").unwrap().1.base_fret, 1);
    }

    #[test]
    fn define_rejects_malformed_diagrams() {
        for bad in [
            "G base-fret 1 frets 3 2 0 0 3",     // 5 frets
            "G base-fret 1 frets 3 2 0 0 0 3 1", // 7 frets
            "G base-fret 1",                     // no frets
            "G base-fret 0 frets 3 2 0 0 0 3",   // base-fret starts at 1
            "G base-fret 1 frets 3 2 0 0 0 25",  // fret past 24
            "G base-fret 1 frets 3 2 0 0 0 q",   // not a fret
            "G capo 1 frets 3 2 0 0 0 3",        // unknown keyword
            "",                                  // no name
        ] {
            assert!(parse_define(bad).is_err(), "define accepted `{bad}`");
        }
    }

    // ---- chord lines ----------------------------------------------------

    #[test]
    fn chord_line_extracts_its_one_chord_and_column() {
        assert_eq!(
            parse_chord_line("[G]First time"),
            Ok((Some(("G".into(), 0)), "First time".into()))
        );
        assert_eq!(
            parse_chord_line("Coração [Am7]bate"),
            Ok((Some(("Am7".into(), 8)), "Coração bate".into()))
        );
        assert_eq!(
            parse_chord_line("[D]"),
            Ok((Some(("D".into(), 0)), String::new()))
        );
        assert_eq!(
            parse_chord_line("only lyrics"),
            Ok((None, "only lyrics".into()))
        );
    }

    #[test]
    fn chord_line_rejects_a_second_chord_and_broken_brackets() {
        let two = parse_chord_line("[G]Twinkle twinkle [C]little star").unwrap_err();
        assert!(two.contains("one chord per line"), "{two}");
        for bad in ["[G First", "First] time", "[]First"] {
            assert!(parse_chord_line(bad).is_err(), "accepted `{bad}`");
        }
    }

    // ---- tab notes ------------------------------------------------------

    #[test]
    fn tab_note_reads_every_technique() {
        use TabStep::*;
        assert_eq!(parse_tab_note("5h7"), Ok(vec![Fret(5), HammerOn(7)]));
        assert_eq!(parse_tab_note("7p5"), Ok(vec![Fret(7), PullOff(5)]));
        assert_eq!(parse_tab_note("7b9"), Ok(vec![Fret(7), Bend(9)]));
        assert_eq!(
            parse_tab_note("7b9r7"),
            Ok(vec![Fret(7), Bend(9), Release(7)])
        );
        assert_eq!(parse_tab_note("5/7"), Ok(vec![Fret(5), SlideUp(7)]));
        assert_eq!(parse_tab_note("7\\5"), Ok(vec![Fret(7), SlideDown(5)]));
        assert_eq!(parse_tab_note("8~"), Ok(vec![Fret(8), Vibrato]));
        assert_eq!(
            parse_tab_note("8~~b10r8"),
            Ok(vec![Fret(8), Vibrato, Vibrato, Bend(10), Release(8)])
        );
        assert_eq!(parse_tab_note("0"), Ok(vec![Fret(0)]));
        assert_eq!(parse_tab_note("24"), Ok(vec![Fret(24)]));
    }

    #[test]
    fn tab_note_rejects_what_the_grammar_forbids() {
        for bad in ["25", "05", "7r5", "b7", "5h", "5x", "~", "5h8r7", "123"] {
            assert!(parse_tab_note(bad).is_err(), "accepted `{bad}`");
        }
    }

    // ---- whole files: startSec/endSec resolution -------------------------

    #[test]
    fn the_architecture_chords_example_resolves_every_chord() {
        let score = parse(CHORDS_EXAMPLE, 12.0).unwrap();

        assert_eq!(score.title.as_deref(), Some("Study in G Major"));
        assert_eq!(score.artist.as_deref(), Some("StemLoft - example"));
        assert_eq!(score.key.as_deref(), Some("G"));
        assert_eq!(score.tempo, Some(80.0));
        assert_eq!(
            score.time,
            Some(TimeSignature {
                numerator: 4,
                denominator: 4
            })
        );
        assert_eq!(score.tuning.as_ref().map(Vec::len), Some(6));
        assert_eq!(score.grid_origin_sec, Some(0.0));

        let windows: Vec<(&str, f64, f64)> = score
            .chords
            .iter()
            .map(|c| (c.name.as_str(), c.start_sec, c.end_sec))
            .collect();
        assert_eq!(
            windows,
            vec![
                ("G", 0.0, 3.0),
                ("C", 3.0, 6.0),
                ("D", 6.0, 9.0),
                ("G", 9.0, 12.0)
            ]
        );
        assert_eq!(
            score.chords[1].diagram.as_ref().unwrap().frets[0],
            DiagramFret::Muted
        );
        assert!(score.chords.iter().all(|c| c.diagram.is_some()));
        assert_eq!(score.lines.len(), 4);
        assert_eq!(score.lines[0].text, "First time I pick up the guitar");
        assert_eq!(score.lines[0].start_sec, Some(0.0));
    }

    #[test]
    fn a_lyric_line_without_a_chord_does_not_end_the_current_chord() {
        let score = parse(
            "{t: 0:00.00}\n[G]first line\n{t: 0:02.00}\nsecond line, same chord\n{t: 0:04.00}\n[C]third\n",
            10.0,
        )
        .unwrap();

        assert_eq!(score.chords[0].end_sec, 4.0);
        assert_eq!(score.chords[1].end_sec, 10.0);
        assert_eq!(score.lines[1].chord, None);
        assert_eq!(score.lines[1].start_sec, Some(2.0));
    }

    #[test]
    fn the_last_chord_ends_at_the_project_duration() {
        let score = parse("{t: 0:01.00}\n[G]only chord\n", 187.42).unwrap();
        assert_eq!(score.chords[0].end_sec, 187.42);
    }

    #[test]
    fn an_anchor_and_its_line_can_share_a_line() {
        let score = parse("{t: 0:01.00}[G]inline\n{t: 0:02.50} [C]spaced\n", 5.0).unwrap();
        assert_eq!(score.chords[0].start_sec, 1.0);
        assert_eq!(score.chords[0].end_sec, 2.5);
        assert_eq!(score.lines[1].text, "spaced");
    }

    #[test]
    fn the_grid_origin_is_the_first_anchor_not_second_zero() {
        let score = parse(
            "{title: x}\nintro without anchor\n{t: 0:02.50}\n[G]x\n",
            9.0,
        )
        .unwrap();
        assert_eq!(score.grid_origin_sec, Some(2.5));
        assert_eq!(score.lines[0].start_sec, None);
    }

    #[test]
    fn unknown_directives_and_comments_are_ignored() {
        let score = parse(
            "{x_stemloft_future: whatever}\n{c: a comment}\n{capo: 2}\n# a ChordPro comment\n{t: 0:00.00}\n[G]x\n",
            4.0,
        )
        .unwrap();
        assert_eq!(score.chords.len(), 1);
        assert_eq!(score.lines.len(), 1);
    }

    #[test]
    fn the_architecture_tab_example_times_each_note_by_column() {
        let score = parse(TAB_EXAMPLE, 6.4).unwrap();

        assert!(score.chords.is_empty());
        let tab = &score.tabs[0];
        assert_eq!((tab.start_sec, tab.end_sec), (0.0, 6.4));
        assert_eq!(tab.lines.len(), 6);
        assert_eq!(
            tab.lines[1],
            "B|-5h8-----8p5---------------------------------------------8~-----|"
        );

        let notes: Vec<(u8, usize)> = tab.notes.iter().map(|n| (n.string, n.column)).collect();
        assert_eq!(
            notes,
            vec![
                (2, 1),
                (2, 9),
                (2, 57),
                (3, 17),
                (3, 41),
                (3, 49),
                (4, 25),
                (4, 33)
            ]
        );
        // 64 columns over 6.4 s: column N starts at N * 0.1 s.
        assert!(close(tab.notes[1].start_sec, 0.9));
        assert!(close(tab.notes[6].start_sec, 2.5));
        assert_eq!(
            tab.notes[5].steps,
            vec![TabStep::Fret(7), TabStep::Bend(9), TabStep::Release(7)]
        );
    }

    #[test]
    fn a_tab_block_ends_the_chord_before_it_and_the_next_chord_ends_it() {
        let text = format!(
            "{{t: 0:00.00}}\n[G]x\n{{t: 0:04.00}}\n{{start_of_tab}}\n{}\n{{end_of_tab}}\n{{t: 0:08.00}}\n[C]y\n",
            staff(["e|-0--|", "B|----|", "G|----|", "D|----|", "A|----|", "E|----|"])
        );
        let score = parse(&text, 10.0).unwrap();

        assert_eq!(score.chords[0].end_sec, 4.0);
        assert_eq!((score.tabs[0].start_sec, score.tabs[0].end_sec), (4.0, 8.0));
        assert!(close(score.tabs[0].notes[0].start_sec, 5.0)); // column 1 of 4
        assert_eq!(score.chords[1].end_sec, 10.0);
    }

    #[test]
    fn short_tab_aliases_work_and_bar_lines_split_notes() {
        let text = format!(
            "{{t: 0:00.00}}\n{{sot}}\n{}\n{{eot}}\n",
            staff([
                "e|-3-|-5-|",
                "B|---|---|",
                "G|---|---|",
                "D|---|---|",
                "A|---|---|",
                "E|---|---|"
            ])
        );
        let score = parse(&text, 8.0).unwrap();
        let columns: Vec<usize> = score.tabs[0].notes.iter().map(|n| n.column).collect();
        assert_eq!(columns, vec![1, 5]);
    }

    #[test]
    fn score_serializes_in_the_shape_angular_expects() {
        let score = parse(CHORDS_EXAMPLE, 12.0).unwrap();
        let json = serde_json::to_value(&score).unwrap();

        assert_eq!(json["gridOriginSec"], 0.0);
        assert_eq!(json["chords"][1]["name"], "C");
        assert_eq!(json["chords"][1]["startSec"], 3.0);
        assert_eq!(json["chords"][1]["endSec"], 6.0);
        assert_eq!(
            json["chords"][1]["diagram"],
            serde_json::json!({ "baseFret": 1, "frets": ["x", 3, 2, 0, 1, 0] })
        );
        assert_eq!(
            serde_json::to_value(TabStep::Bend(9)).unwrap(),
            serde_json::json!({ "kind": "bend", "fret": 9 })
        );
        assert_eq!(
            serde_json::to_value(TabStep::Vibrato).unwrap(),
            serde_json::json!({ "kind": "vibrato" })
        );
    }

    // ---- score_parse_error: line + message, never a panic ----------------

    fn error_line(text: &str) -> u32 {
        parse(text, 60.0).expect_err("should not parse").line
    }

    #[test]
    fn two_chords_on_one_line_is_a_parse_error_on_that_line() {
        let err = parse(
            "{title: x}\n{t: 0:00.00}\n[G]Twinkle twinkle [C]little star\n",
            60.0,
        )
        .unwrap_err();
        assert_eq!(err.line, 3);
        assert!(
            err.message.contains("one chord per line"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_malformed_anchor_names_its_line() {
        let err = parse("{title: x}\n{t: 1:5.3}\n[G]x\n", 60.0).unwrap_err();
        assert_eq!(err.line, 2);
        assert!(err.message.contains("1:5.3"), "{}", err.message);
    }

    #[test]
    fn chords_need_an_anchor_and_anchors_must_move_forward_within_the_project() {
        assert_eq!(error_line("[G]no anchor\n"), 1);
        // An anchor is consumed by the line it anchors.
        assert_eq!(error_line("{t: 0:00.00}\n[G]a\n[C]b\n"), 3);
        assert_eq!(error_line("{t: 0:05.00}\n[G]a\n{t: 0:04.00}\n[C]b\n"), 3);
        // A second anchor before any line used the first one.
        assert_eq!(error_line("{t: 0:01.00}\n{t: 0:02.00}\n[G]a\n"), 2);
    }

    #[test]
    fn an_anchor_past_the_project_end_is_kept_and_never_ends_before_it_starts() {
        // Added after review: a stem re-imported shorter, or a rounded
        // durationSec, must not break the whole score.
        let score = parse("{t: 0:30.00}\n[G]a\n{t: 1:01.00}\n[C]b\n", 60.0).unwrap();
        assert_eq!(score.chords[0].end_sec, 61.0);
        assert_eq!(
            (score.chords[1].start_sec, score.chords[1].end_sec),
            (61.0, 61.0)
        );

        // No stems yet (or a bad duration): nothing becomes NaN or negative.
        for duration in [0.0, -1.0, f64::NAN] {
            let score = parse("{t: 0:02.00}\n[G]a\n", duration).unwrap();
            assert_eq!(score.chords[0].end_sec, 2.0, "duration {duration}");
        }
    }

    #[test]
    fn text_after_a_directive_is_an_error_not_silently_dropped() {
        // Added after review.
        assert_eq!(error_line("{title: x}[G]hello\n"), 1);
        assert_eq!(error_line("{t: 0:00.00}\n{c: verse}[G]hello\n"), 2);
        assert_eq!(error_line("{t: 0:00.00}{title: x}\n"), 1);
        // A tab block may open on its anchor's line.
        let text = format!(
            "{{t: 0:00.00}}{{start_of_tab}}\n{}\n{{end_of_tab}}\n",
            staff(["e|-1-|", "B|---|", "G|---|", "D|---|", "A|---|", "E|---|"])
        );
        assert_eq!(parse(&text, 4.0).unwrap().tabs[0].notes.len(), 1);
    }

    #[test]
    fn a_long_chain_of_anchors_on_one_line_is_an_error_not_a_stack_overflow() {
        // Added after review: this used to recurse once per directive.
        let line = "{t: 0:00.00}".repeat(200_000);
        assert_eq!(error_line(&line), 1);
    }

    #[test]
    fn comments_inside_a_tab_block_are_not_staff_lines() {
        // Added after review.
        let text = "{t: 0:00.00}\n{start_of_tab}\n# riff\n{c: slow}\ne|-1-|\nB|---|\nG|---|\nD|---|\nA|---|\nE|---|\n{end_of_tab}\n";
        assert_eq!(parse(text, 4.0).unwrap().tabs[0].lines.len(), 6);
    }

    #[test]
    fn broken_directives_name_their_line() {
        assert_eq!(error_line("{title: x}\n{tempo: fast}\n"), 2);
        assert_eq!(error_line("\n\n{time: 4}\n"), 3);
        assert_eq!(error_line("{tuning: E A D}\n"), 1);
        assert_eq!(error_line("{define: G frets 3 2 0}\n"), 1);
        assert_eq!(error_line("{title: never closed\n"), 1);
    }

    #[test]
    fn broken_tab_blocks_name_their_line() {
        let block = |lines: [&str; 6]| {
            format!(
                "{{t: 0:00.00}}\n{{start_of_tab}}\n{}\n{{end_of_tab}}\n",
                staff(lines)
            )
        };
        let ok = [
            "e|----|", "B|----|", "G|----|", "D|----|", "A|----|", "E|----|",
        ];

        let mut release = ok;
        release[2] = "G|7r5-|";
        assert_eq!(error_line(&block(release)), 5);

        let mut high = ok;
        high[0] = "e|-25-|";
        assert_eq!(error_line(&block(high)), 3);

        let mut ragged = ok;
        ragged[3] = "D|------|";
        assert_eq!(error_line(&block(ragged)), 6);

        let mut no_bar = ok;
        no_bar[1] = "B-----";
        assert_eq!(error_line(&block(no_bar)), 4);

        // Five string lines: reported on {start_of_tab}.
        assert_eq!(
            error_line(
                "{t: 0:00.00}\n{start_of_tab}\ne|-|\nB|-|\nG|-|\nD|-|\nA|-|\n{end_of_tab}\n"
            ),
            2
        );
        assert_eq!(error_line("{start_of_tab}\n"), 1); // no anchor
        assert_eq!(error_line("{t: 0:00.00}\n{start_of_tab}\ne|-|\n"), 2); // never closed
        assert_eq!(error_line("{title: x}\n{end_of_tab}\n"), 2);
        assert_eq!(error_line("{t: 0:00.00}\n{start_of_tab}\n{title: y}\n"), 3);
    }

    #[test]
    fn no_input_panics_the_parser() {
        for text in [
            "",
            "{",
            "}",
            "[",
            "]",
            "{t:}",
            "{t: }",
            "{define:}",
            "{tuning:}",
            "{time:/}",
            "{t: 0:00.00}\n{sot}\n|\n|\n|\n|\n|\n|\n{eot}",
            "{t: 0:00.00}[",
            "\u{feff}{title: bom}",
            "{t: 0:00.00}\n[ä]ü",
            "{t: 99999999999999999999:00.00}",
        ] {
            let _ = parse(text, 60.0);
        }
    }

    #[test]
    fn a_byte_order_mark_does_not_hide_the_first_directive() {
        let score = parse("\u{feff}{title: bom}\n", 1.0).unwrap();
        assert_eq!(score.title.as_deref(), Some("bom"));
    }

    #[test]
    fn load_score_reads_a_file_and_reports_a_missing_one_as_line_zero() {
        let dir = std::env::temp_dir().join(format!("stemloft-cho-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("song.cho");
        fs::write(&path, CHORDS_EXAMPLE).unwrap();
        let manager = ScoreMetadataManager::new();

        assert_eq!(manager.load_score(&path, 12.0).unwrap().chords.len(), 4);

        let missing = manager
            .load_score(&dir.join("missing.cho"), 12.0)
            .unwrap_err();
        assert_eq!(missing.line, 0);
        assert!(missing.to_string().starts_with("line 0: "));
        fs::remove_dir_all(&dir).ok();
    }
}
