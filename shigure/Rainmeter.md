# Shigure and Rainmeter

Shigure allows Mado skins to run in Rainmeter.
It expects the following to be available:

## WebNowPlaying

This is an example Rainmeter Skin that instantiates Mado:

```ini
[Rainmeter]
Update=100

;; Mado: WebNowPlaying Integration

[MadoWNPStatus]
Measure=Plugin
Plugin=WebNowPlaying
PlayerType=Status

[MadoWNPPlayer]
Measure=Plugin
Plugin=WebNowPlaying
PlayerType=Player

[MadoWNPTitle]
Measure=Plugin
Plugin=WebNowPlaying
PlayerType=Title

[MadoWNPArtist]
Measure=Plugin
Plugin=WebNowPlaying
PlayerType=Artist

[MadoWNPAlbum]
Measure=Plugin
Plugin=WebNowPlaying
PlayerType=Album

[MadoWNPAlbumCover]
Measure=Plugin
Plugin=WebNowPlaying
PlayerType=Cover

[MadoWNPDuration]
Measure=Plugin
Plugin=WebNowPlaying
PlayerType=Duration

[MadoWNPPosition]
Measure=Plugin
Plugin=WebNowPlaying
PlayerType=Position

[MadoWNPProgress]
Measure=Plugin
Plugin=WebNowPlaying
PlayerType=Progress

[MadoWNPVolume]
Measure=Plugin
Plugin=WebNowPlaying
PlayerType=Volume

[MadoWNPState]
Measure=Plugin
Plugin=WebNowPlaying
PlayerType=State

;; Mado: Core

[Shigure]
Measure=Plugin
Plugin=shigure
Url=https://www.example.com
Width=400
Height=300
X=[&Anch:MeterX]
Y=[&Anch:MeterY]
DynamicVariables=1
;; Mado: Core - WebNowPlaying
MadoWNPStatus=[MadoWNPStatus]
MadoWNPPlayer=[MadoWNPPlayer]
MadoWNPTitle=[MadoWNPTitle]
MadoWNPArtist=[MadoWNPArtist]
MadoWNPAlbum=[MadoWNPAlbum]
MadoWNPAlbumCover=[MadoWNPAlbumCover]
MadoWNPDuration=[MadoWNPDuration]
MadoWNPPosition=[MadoWNPPosition]
MadoWNPProgress=[MadoWNPProgress]
MadoWNPVolume=[MadoWNPVolume]
MadoWNPState=[MadoWNPState]

[Anch]
meter=string
W=400
H=300
SolidColor=0,0,0,1

```

### Bangs

All Bangs (commands) for Music are sent to the measure "MadoWNPTitle"
