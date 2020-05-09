# footile
A 2D vector graphics library written in Rust.

## Documentation
[https://docs.rs/footile](https://docs.rs/footile)

## Example
```rust
use footile::{FillRule, PathBuilder, Plotter};

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
let mut p = Plotter::new(128, 128);
let raster = p.fill(&fish, FillRule::NonZero);
```

## Code Walkthrough

There is nothing novel here — this is merely a guide to the code.

So we have a 2D *path* made up of lines, bézier splines, arcs, etc., and we want
to make a high-quality raster image out of it.  But how?

### Modules

* `path`: Defines `Path` struct to store 2D paths
* `geom`: Points (`Pt`) and transforms used by `plotter` and `stroker`
* `plotter`: Defines `Plotter` struct and flattens curves
* `stroker`: Creates *stroked* paths for plotter
* `fixed`: Defines `Fixed` struct used by `fig` module
* `fig`: Rasterizes paths

### Curve Flattening

Here, *flattening* refers to approximating a curve with a series of line
segments, and has no relation to pandemic response.  Currently we use a basic
algorithm described by De Casteljau.  There might be opportunities for
optimization here if we ever determine this is a bottleneck.

Once complete, we have a series of line segments forming one or more closed
polygons.

### Sorting Vertices

For the next step, we create a sorted list of the vertices in (Y, X) order.
This is needed because we will *scan* the polygon onto a grid one row at a time.

Every path has a **winding order**: either *clockwise* or the other direction,
sometimes called *counter-* or *anti-clockwise*.  Let's avoid that debate by
calling it *widdershins*, since clocks rarely go backwards.

The first vertex must be on the outside of the path, so we can check the angle
to its neighbors to determine the winding order.

### Active Edges

Processing the rows in order, edges are added and removed from the *active edge*
list.  If an edge crosses the current row, it is added to the list, otherwise,
it is removed.  Once we reach a vertex with Y greater than the current row, the
row can be scanned.

As rows are scanned, the active edge list is updated by considering the vertices
in the sorted list.

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

For anti-aliasing, the trick is to use fractional numbers where an edge crosses
through a pixel.  Let's look at a triangle crossing two pixels:

```bob
    (0, 0)    (1, 0)
    +  -  -  -
    |\        |
    | \        
    |  \      |
    |   \     
    | -  \  -  (1, 1)
    |     \   |
    |      \  
    |       \ |
    |        \
    +---------+ (1, 2)
```

Ideally, the top pixel would be 25% filled and the bottom would be 75%.  As we
rasterize one row at a time, we keep a cumulative sum of values from left to
right.

```bob
      -  -  - +  -  -  -  -  -  -  
    |         |\        |         |
              | \ -0.75           
    |    0    |  \      |  -0.25  |
              |+1 \                
      -  -  - | -  \  -   -  -  -  
    |         |     \-0.25        |
              |      \                
    |    0    |  +1   \ |  -0.75  |
              |        \           
      -  -  - +---------+ -  -  -  
```

Notice how the remainder of the edge coverage is added to the pixel to the
right?  This ensures that the cumulative sum at that pixel returns to zero.

### Compositing

The signed area buffer can be composited with a raster, using a source color.
