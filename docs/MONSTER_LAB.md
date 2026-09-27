# Monster Lab

A player-authored creature editor inspired by the accessibility of pixel-skin editors, without copying another game's art/template.

## Flow
God Dock -> Monster -> Edit/New -> Monster Lab overlay.

Left:
- 16x16, 24x24 or 32x32 pixel canvas
- pencil, erase, fill, mirror, clear
- small restrained palette
- optional emissive pixels
- symmetry toggle
- animation preview later

Right:
- name
- visual scale
- body mass
- movement speed
- armor
- aggression
- intelligence
- reproduction
- later: senses, diet, thermal tolerance, aquatic/aerial locomotion, venom, group behavior

Bottom:
- Save Blueprint
- Spawn
- Duplicate
- Randomize parameters
- Procedural silhouette suggestion

## Important separation
Skin is visual identity. It must not secretly define combat stats.
Physics parameters determine physical behavior. Some optional derived geometric properties may use the silhouette (coverage, frontal area, leg/contact layout), but a player cannot paint one black pixel and receive infinite armor.

## Blueprint format
MonsterBlueprint = PixelSkin + MonsterArchetype + visual scale + metadata.

Blueprints are deterministic, serializable and shareable. Future import/export can use JSON plus a tiny indexed PNG representation.

## Future procedural pixel animation
The static skin becomes a source mask for:
- idle breathing offset
- walk cycle limb offsets
- hit flash
- attack pose
- death/collapse
- swimming/flying variants

Keep animation procedural so user-authored skins do not require hand-drawing every frame.
