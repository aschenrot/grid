# Dual-Grid Model

Dual-grid topology maps a `width x height` logic grid into a `(width + 1) x
(height + 1)` visual corner grid. Each visual corner stores a 4-bit occupancy
mask and a neutral visual descriptor.

The topology API distinguishes the two coordinate spaces:

- `LogicCellCoord` names a logic cell.
- `VisualCornerCoord` names an intersection/corner in the visual grid.

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

## Corner Coordinates vs Block Origins

For a tiny `2 x 2` logic grid:

```text
A B
C D
```

Current corner semantics make visual corner `(1, 1)` sample all four cells:

```text
A B
C D
```

With only `A` filled, these current visual-corner masks are produced:

```text
1000 0100 0000
0010 0001 0000
0000 0000 0000
```

The alternate block-origin helper names the same `2 x 2` sample by its top-left
logic cell. Block origin `(0, 0)` is equivalent to visual corner `(1, 1)`, so it
returns mask `0001` when only `A` is filled.

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
