## Purpose

Makes the jump from the loop's end back to its start seamless, with a short crossfade instead of a hard cut.

## ADDED Requirements

### Requirement: Crossfade at the loop boundary
When playback reaches endSec, the engine SHALL crossfade over a few milliseconds into startSec instead of cutting abruptly.

#### Scenario: Reaching the loop end
- **WHEN** playback reaches endSec
- **THEN** it continues from startSec with no audible click

### Requirement: Loop start prepared in advance
The audio at startSec MUST be prepared before playback reaches endSec, so the crossfade needs no decoding, allocation or I/O in the callback.

#### Scenario: Approaching the loop end
- **WHEN** playback approaches endSec
- **THEN** the start-of-loop audio is already decoded and waiting in its own buffer
