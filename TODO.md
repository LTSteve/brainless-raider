UI
X Death -> Respawn
X Lives - Game Over
X Treasure Tracker
(oops same branch)
The Real Tiled-as-scene experience
X Tool Sprite (editor only)
X Camera-as-Component
X NoTearDown-as-Component
X Background-music-as-Component
X Refactor World hydrators (remove world, and move that functionality to a single-call 'initiailze' system, see Pits & Planks)
X Label-as-component
X Click to start

Flavor
X Falling into pits animations
X BUG: Game Over causes issues
X Pixel Perfect
X BUG~: Module Tree
X Spinning Treasure
X BUG: Adventurer clips under exit?
X UI Scales with window size
X Font
X BUG: Text glitches out
X Make planks render above dead movers
X You Win screen

Exporting!!!! -- do this feature before post mortem/errata episodes

Post Mortem 1
---

*what will wait until more non-shtevlog features are added*
- adding Doors
- mouse handling
  - make start button and 'you win' screen function with the 'clickable_area' module (rather than unique snowflakes)
  - support circle, box, and everything colliders
  - split collision shape properties from clickable area component
- pits-and-planks should have a system that 'initializes' movers with overpit and overplank counters

*what I'll be changing for next Shtevlog project (orbital cleanup? kepler syndrome?)*
- level-as-code (vs tiled)
- is raising 'event'-like things by inserting a Component a good practice?
  - optionally: we could raise an event instead
  - the 'insert dead' method relies on my understanding that that will cause a separate system somewhere else to occur
  - using events, that understanding is less of a logical leap
  - therefore, using component insertion as a method of event handling is no-go

*maybe as it's own project/devlog*
- components list is getting a little tough to work with
  - using custom types in tiled to sync up project data structure with tiled editor
  - may be pointless once bevy releases with scene editor
  - define "TiledExport" trait in separate crate
  - separate tiled-main entry point
    - scans all components for "TiledExport"
    - creates a propertytypes.json containing all metadata
  - create a tiled script that runs the project from the tiled-main entry point
    - somehow sync current custom types with those defined in propertytypes.json

*wontfix*
- bug with overriding 'obj type' between template & object ref -- fixed in vid, but not main branch
- run if in state MLS::Done
  - can we set up plugins to be loaded after map is loaded?
  - would be cool, but no, can't really do that
- is having all collision events in one file a good design?
  - Yes, it is for this project, but if the file gets too big, we can break things out into smaller chunks within the same module

Errata for some bugs (collisions using z index, plank z-index, etc):
---
X Remove Audio Server dependency on Command -- Borrow checker makes this a bad plan (would need an audio player and audio server, at this point i don't care enough)
  - audio server list for "to-play" audio events
  - fn for adding events to those lists
  - audio server System for running queued audio events
X Don't need "Option<Res<AudioServer>>"
  - not needed as AudioServer is built in Startup
X remove 'tags' module, move to where they're used most
- Better Map State handling
  - next level()
  - get current level()
  - a system that checks if 'next level' has been called and updates map & scene settings accordingly
