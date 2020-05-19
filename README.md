# footile
A 2D vector graphics library written in Rust.

## Documentation
[https://docs.rs/footile](https://docs.rs/footile)

## Example
```rust
use footile::{FillRule, PathBuilder, Plotter};
use pix::matte::Matte8;
use pix::Raster;

let fish = PathBuilder::new()
    .relative()
    .pen_width(3.0)
    .move_to(112.0, 24.0)
    .line_to(-32.0, 24.0)
    .cubic_to(-96.0, -48.0, -96.0, 80.0, 0.0, 32.0)
    .line_to(32.0, 24.0)
    .line_to(-16.0, -40.0)
    .close()
    .build();
let raster = Raster::with_clear(128, 128);
let mut p = Plotter::new(128, 128);
p.fill(FillRule::NonZero, &fish, Matte8::new(255));
```

## Rasterizing: Bird's Eye View

There is nothing novel here — this is merely a guide to the code.

So we have a 2D *path* made up of lines, bézier splines, arcs, etc., and we want
to make a high-quality raster image out of it.  But how?

### Modules

* `path`: Defines struct to store 2D `Path`s
* `geom`: Points (`Pt`) and transforms used by `plotter` and `stroker`
* `plotter`: Defines `Plotter` struct and flattens curves
* `stroker`: Creates *stroked* paths for plotter
* `fixed`: Defines `Fixed` struct used by `fig` module
* `fig`: Rasterizes paths

### Curve Flattening

Here, *flattening* refers to approximating a curve with a series of line
segments, and has no relation to pandemic response.

Currently we use the recursive algorithm described by De Casteljau.  There might
be opportunities for optimization here if we ever determine this is a
bottleneck.  One other thing to note: this method could cause a stack overflow
with the wrong input data.

Once complete, we have a series of line segments forming one or more closed
polygons.

### Sorting Vertices

For the next step, we create a sorted list of the vertices in (Y, X) order.
This is needed because we will *scan* the polygon onto a grid one row at a time.

Every path has a **winding order**: either *clockwise* or the other direction,
sometimes called *counter-* or *anti-clockwise*.  Let's avoid that debate by
calling it *widdershins*, since clocks rarely go backwards.

The first vertex must be on the outside of the path, so we can check the angle
between its neighbors to determine the winding order.

### Active Edges

As rows are scanned from top to bottom, we keep track of a list of *active
edges*.  If an edge crosses the current row, it is added to the list, otherwise,
it is removed.  Since horizontal edges cannot *cross* a row, they can safely be
ignored.

For each row, vertices from the list are compared to its top and bottom.  If
the new vertex is above the bottom, one or more edges are *added*.  When the
new vertex is above the top, existing edges are *removed*.

```bob
           v0
           /\
          /  \ (a)
         /    \      v2
    (b) /      +-----+
       /      v1      \
      /                \ (c)
     /                  \
    +--------------------+
     v3                  v4
```

Example:
* Starting with `v0`, add edges `(a)` and `(b)`
* Scan until the bottom of the current row is below `v1` / `v2`
* Add edge `(c)`
* Scan until the row top is below `v1`
* Remove edge `(a)`
* Scan until the row top is below `v3` / `v4`
* Remove edges `(b)` and `(c)`

### Signed Area

The active edges are used to find the *signed area* at any point.  Count the
number of edge crossings needed to reach the exterior of the path.  For example:

```bob
    +------+
    |      |
    |  +1  |
    |      |
    +------+
```

A self-intersecting polygon looks like this:

```bob
    +-----------------+
    |                 |
    |      +------+   |
    |      |       \ /
    |  +1  |  +2    X
    |      |       / \
    |      +------+   |
    |                 |
    +-----------------+
```

What about *sub-paths* with opposite winding order?  In that case, subtract
instead of adding:

```bob
    +----------------+
    |      +-----+   |
    |      |     |   |
    |  +1  |  0  |   |
    |      |     |   |
    |      +-----+   |
    |        <--     |
    |                |
    +----------------+
          -->
```

### Scanning Rows

When scanning a row, the signed area is sampled for each pixel.  The direction
of each edge from top to bottom determines whether it adds to or subtracts from
the area.  In the normal winding order, it adds to the area; otherwise, it
subtracts.

This row is 4 pixels wide:

```bob
      -  -  - | -  -  -   -  -  - | -  -  -
    |         |         |         |         |
              | +1                | -1
    |         |         |         |         |
              |                   |      
    | -  -  - | -  -  - | -  -  - | -  -  - |
          0         1         1         0
```

The *cumulative sum* of these values is the signed area of each pixel.

#### Anti-Aliasing

Sometimes edges don't fall on pixel boundaries.  In this case, the trick is to
use fractional numbers for anti-aliasing.

```bob
      -  -  - | -  |  -   -  -  -   -  -  -
    |         |    |    |         |         |
              | +1 | -½   -½
    |         |    |    |         |         |
              |    |                     
    | -  -  - | -  |  - | -  -  - | -  -  - |
          0         ½         0         0
```

Notice how the remainder of the second edge coverage is added to the pixel to
the right (third pixel).  This is necessary to keep the cumulative sum correct.

```bob
      -  -  - | -  \  -   -  -  -   -  -  -
    |         |     \   |         |         |
              | +1   \-¼  -¾
    |         |       \ |         |         |
              |        \
    | -  -  - | -  -  - \ -  -  - | -  -  - |
          0         ¾         0         0
```

### Compositing

The signed area buffer can be composited with a raster, using a source color.
