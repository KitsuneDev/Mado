# MusicPlayerService Commands

| Command | Args | Return | Description |
|---------|------|--------|-------------|
| [get_data](#get_data) | `()` | `MusicPlayerState` |  |
| [next](#next) | `()` | `()` |  |
| [pause](#pause) | `()` | `()` |  |
| [play](#play) | `()` | `()` |  |
| [previous](#previous) | `()` | `()` |  |
| [seek_absolute](#seek_absolute) | `f64` | `()` |  |
| [set_volume](#set_volume) | `f64` | `()` |  |

## get_data

**Signature:** `fn get_data() -> MusicPlayerState`


## next

**Signature:** `fn next() -> ()`


## pause

**Signature:** `fn pause() -> ()`


## play

**Signature:** `fn play() -> ()`


## previous

**Signature:** `fn previous() -> ()`


## seek_absolute

**Signature:** `fn seek_absolute(f64) -> ()`


## set_volume

**Signature:** `fn set_volume(f64) -> ()`


# Struct Reference

## `MusicPlayerState`

| Field | Type | Description |
|-------|------|-------------|
| `is_connected` | `bool` |  |
| `player` | `String` |  |
| `title` | `String` |  |
| `artist` | `String` |  |
| `album` | `String` |  |
| `cover` | `String` | URL to the album cover image |
| `duration` | `String` | Duration in seconds |
| `position` | `String` | Position in seconds |
| `progress` | `f64` | Prrogress Percentage (0.0 to 1.0) |
| `volume` | `f64` | Volume Percentage (0.0 to 1.0) |
| `status` | `MusicPlayerStatus` |  |

