# Dual-Grid Model

Dual-grid topology maps a `width x height` logic grid into a `(width + 1) x
(height + 1)` visual corner grid. Each visual corner stores a 4-bit occupancy
mask and a neutral visual descriptor.

## Mask Bits

For visual corner `(cx, cy)`, bits are assigned in clockwise reading order from
the top-left sampled logic cell:

| Bit | Name | Logic coordinate |
| --- | --- | --- |
| 0 | northwest | `(cx - 1, cy - 1)` |
| 1 | northeast | `(cx, cy - 1)` |
| 2 | southwest | `(cx - 1, cy)` |
| 3 | southeast | `(cx, cy)` |

Any sampled logic coordinate outside the logic grid is empty.

## Classification

| Mask count/pattern | Kind |
| --- | --- |
| `0000` | `Empty` |
| one occupied bit | `Corner` |
| adjacent two occupied bits | `Edge` |
| opposite two occupied bits | `Diagonal` |
| three occupied bits | `T` |
| `1111` | `Full` |

The exact mask-to-kind and mask-to-rotation mapping is covered by
`tile_topology` tests for all 16 masks.

## Visual Descriptors

`VisualTileDescriptor` contains only:

- the 4-bit `TileMask4`
- the neutral `VisualTileKind`
- clockwise `VisualTileRotation`

It does not contain a mesh id, transform, node path, scene handle, material,
texture, or ECS component.
