#!/usr/bin/env python3
"""Regenerates the small WAV fixtures the Stem Importer tests read
(src-tauri/src/core/stem_importer.rs). Deterministic: same bytes every run.

    python3 src-tauri/tests/fixtures/stems/generate.py
"""
import math
import struct
from pathlib import Path

HERE = Path(__file__).resolve().parent


def samples(frames, channels, rate):
    """A 440 Hz sine on channel 0, 660 Hz on the others, in [-0.5, 0.5]."""
    for i in range(frames):
        yield [0.5 * math.sin(2 * math.pi * (440 if c == 0 else 660) * i / rate)
               for c in range(channels)]


def pack(value, fmt, bits):
    if fmt == 3:
        return struct.pack("<f", value)
    peak = (1 << (bits - 1)) - 1
    n = int(round(value * peak))
    if bits == 16:
        return struct.pack("<h", n)
    if bits == 24:
        return struct.pack("<i", n)[:3]
    raise ValueError(bits)


def wav(name, *, rate, channels, frames, bits=16, fmt=1, extensible=False, extra_chunk=False):
    data = b"".join(pack(v, fmt, bits) for frame in samples(frames, channels, rate) for v in frame)
    block_align = channels * bits // 8
    if extensible:
        # WAVE_FORMAT_EXTENSIBLE: cbSize 22, valid bits, channel mask, subformat GUID.
        guid = struct.pack("<H", fmt) + b"\x00\x00\x00\x00\x10\x00\x80\x00\x00\xaa\x00\x38\x9b\x71"
        fmt_body = struct.pack("<HHIIHHHHI", 0xFFFE, channels, rate, rate * block_align,
                               block_align, bits, 22, bits, 0x4) + guid
    else:
        fmt_body = struct.pack("<HHIIHH", fmt, channels, rate, rate * block_align, block_align, bits)
    chunks = b"fmt " + struct.pack("<I", len(fmt_body)) + fmt_body
    if extra_chunk:
        info = b"INFOISFT" + struct.pack("<I", 9) + b"StemLoft\x00" + b"\x00"  # odd size + pad byte
        chunks += b"LIST" + struct.pack("<I", len(info) - 1) + info
    chunks += b"data" + struct.pack("<I", len(data)) + data
    (HERE / name).write_bytes(b"RIFF" + struct.pack("<I", 4 + len(chunks)) + b"WAVE" + chunks)


wav("mono-44100-16.wav", rate=44100, channels=1, frames=4410)
wav("stereo-44100-16.wav", rate=44100, channels=2, frames=8820)
wav("stereo-48000-16.wav", rate=48000, channels=2, frames=2400)
wav("mono-44100-float.wav", rate=44100, channels=1, frames=2205, bits=32, fmt=3)
wav("mono-44100-24-extensible.wav", rate=44100, channels=1, frames=4410, bits=24,
    extensible=True, extra_chunk=True)
wav("quad-44100-16.wav", rate=44100, channels=4, frames=441)
