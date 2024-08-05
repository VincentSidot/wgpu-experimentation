# wgpu-experimentation

This project is an exploration of the WGPU graphics library. The goal is to create a simple 3D graphics program that renders a cube and allows the user to rotate and zoom in/out. Debug information is displayed using the EGUI library.

# Presentation

todo

# Modules Documentattion

The redaction of this section is still on progress.

## `debug` Module Docuementation

todo

## `camera` Module Documentation

todo

## `shape` Macro Documentation

### Overview

The `shape` macro helps in creating a collection of vertices and indices for a shape, typically used in graphics programming. It takes a color for the vertices and a set of named points with their positions. Additionally, it specifies the indices to define the shape's faces.

### Macro Structure

#### Syntax

```rust
shape!(
    color;
    name1 => pos1,
    name2 => pos2,
    ...;
    pointA1 pointB1 pointC1,
    pointA2 pointB2 pointC2,
    ...
)
```

#### Parameters

1. **color**: The color for all vertices.
2. **name => pos**: Named points with their respective positions.
3. **pointA pointB pointC**: Triangles defined by the indices of named points to form the shape.

#### Example Usage

```rust
let (mut vertices, indices) = shape!(
    [1.0, 0.0, 0.0]; // Color for vertices
    // Vertex positions
    A => [0.0, 1.0, 1.0],
    B => [1.0, 1.0, 1.0],
    C => [1.0, 0.0, 1.0],
    D => [0.0, 0.0, 1.0],
    E => [0.0, 1.0, 0.0],
    F => [1.0, 1.0, 0.0],
    G => [1.0, 0.0, 0.0],
    H => [0.0, 0.0, 0.0];
    // Indices for triangles
    // Front face
    A D C,
    A C B,
    // Back face
    E F G,
    G H E,
    // Top face
    E A B,
    B F E,
    // Bottom face
    H G C,
    C D H,
    // Left face
    A E H,
    H D A,
    // Right face
    F B C,
    C G F,
);
```

### Detailed Breakdown

1. **Color Definition**:
   ```rust
    [1.0, 0.0, 0.0]; // Color for vertices
   ```
   This part defines the color used for all vertices.

2. **Vertex Definitions**:
   ```rust
   A => [0.0, 1.0, 1.0],
   B => [1.0, 1.0, 1.0],
   ...
   ```
   Each vertex is given a name (e.g., `A`, `B`, etc.) and a position in 3D space.

3. **Triangle Definitions**:
   ```rust
   // Front face
   A D C,
   A C B,
   ...
   ```
   These define the faces of the shape by specifying the vertices that make up each triangle.

### Macro Internals

The macro expands into code that initializes a vector of vertices and indices based on the provided definitions.

- **Vertex Initialization**:
  ```rust
  shape!(@step 0u16, vertex, $color, $($name, $pos),*);
  ```
  This recursive part of the macro assigns indices to each named vertex and pushes them into the `vertex` vector.

- **Indices Initialization**:
  ```rust
  $(
      indices.push($pointA);
      indices.push($pointB);
      indices.push($pointC);
  )*
  ```
  This part iterates over the provided triangles and adds the corresponding indices to the `indices` vector.

### Final Output

The macro returns a tuple containing two vectors:
- **vertices**: A vector of `Vertex` instances, each created with a position and color.
- **indices**: A vector of indices defining the triangles that make up the shape.

### Notes

- Ensure the `Vertex` struct and `crate::graphics::Vertex::new` function are correctly defined in your project.
- The `#[allow(non_snake_case)]` attribute is used to permit non-standard naming for the generated vertex variables.

## Todo

- Better camera handle
- Understand projection system
- Render spheres.