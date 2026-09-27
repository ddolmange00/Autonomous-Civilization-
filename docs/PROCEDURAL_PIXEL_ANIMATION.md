# Procedural Pixel Animation

Goal: one user-authored static skin should be sufficient for readable motion.

No extra sprite sheets are required. Runtime transforms create:
- idle: subtle vertical breathing/squash
- walk: bob + lower-body stride
- run: stronger bob, lean and shear
- attack: forward squash/stretch and front-pixel lunge
- hit: recoil
- death: collapse, flatten and rotate

The renderer may infer coarse regions from normalized pixel coordinates rather than semantic bones. Later Monster Lab tools may optionally let the player mark anchors (eyes/head/feet/weapon/tail) for better motion.

Animation never changes simulation physics or attack success. It visualizes authoritative state.

Benefits:
- tiny storage footprint
- works with arbitrary player skins
- procedural designs and user designs share one animation system
- no need to generate 6-20 frames per creature
