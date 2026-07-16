# ViVerN is a node-based, no-code visual novel editor

## Demo
Exported visual novel demo: https://youtu.be/6UvIPH7LdKs \
Working with ViVerN: https://youtu.be/pbYYlJ5xXAE \

## Features:
### Node-based
every in-game event is a node on an infinite canvas. These nodes can be freely moved and connected. Connections between nodes define event order
<img width="1920" height="1036" alt="image" src="https://github.com/user-attachments/assets/a134b5d1-0d1b-4636-a296-47d21fd74171" />

### No-code
ViVerN does not require users to write game logic. Instead, a template is packaged when exporting a project

### Dialogue flow
ViVerN includes nodes for writing dialogue and choices to construct a branching narrative

### Graphics
Include background art and characters with multiple expressions support. After being defined, these sprites may be selected via a dropdown menu in corresponding nodes\
Character sprites can be positioned and scaled\

Animate sprites with following effects:
- Slide (animate position)
- Pulse (animate opacity)
- Breathe (animate scale)

### Sound
- Select music file to play in background. It can be looped and paused
- Play SFX. They are played only once per event in game

###  Scenes
Use scenes to organize your project. Every scene has a root node as an entry point

### Export to web
Currently ViVern supports export as JSON files of specific scenes or a HTML/JS game

## Developed with:
- Rust  programming language
- Eframe, Egui 0.33.0
- Hexi engine (JavaScript)
