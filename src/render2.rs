use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
struct Circle {
    pub centerx: f32,
    pub centery: f32,
    pub radius: f32,
}

impl std::hash::Hash for Circle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.centerx.to_bits().hash(state);
        self.centery.to_bits().hash(state);
        self.radius.to_bits().hash(state);
    }
}

impl lyon::tessellation::geometry_builder::GeometryBuilder for VertexBuffer {
    fn add_triangle(
        &mut self,
        a: lyon::tessellation::VertexId,
        b: lyon::tessellation::VertexId,
        c: lyon::tessellation::VertexId,
    ) {
        // Add the three vertices to the current geometry
        self.indices.push(a.0 as u32);
        self.indices.push(b.0 as u32);
        self.indices.push(c.0 as u32);

        // Update the last index id
        self.last_index_id = c.0 as u32;
    }

    fn abort_geometry(&mut self) {
        // we cannot abort the geometry and need to panic
        panic!("Something went wrong while tessellating a geometry, cannot proceed.");
    }
}

impl lyon::tessellation::geometry_builder::FillGeometryBuilder for VertexBuffer {
    fn add_fill_vertex(
        &mut self,
        vertex: lyon::tessellation::FillVertex,
    ) -> Result<lyon::tessellation::VertexId, lyon::tessellation::GeometryBuilderError> {
        // Update the last vertex id
        self.last_vertex_id = self.vertices.len() as u32;

        // Add the vertex to the vertex buffer
        self.vertices.push(Vertex {
            x: vertex.position().x,
            y: vertex.position().y,
        });

        Ok(lyon::tessellation::VertexId(
            (self.vertices.len() - 1) as u32,
        ))
    }
}

impl std::cmp::Eq for Circle {}

fn main() {
    let n_times = 100;

    let mut cache: HashMap<Circle, (Vec<Vertex>, Vec<u32>)> = HashMap::new();

    let mut output = VertexBuffer::new_with_capacity(100_000, 100_000);

    // benchmark tessellating a simple geometry
    let t0 = std::time::Instant::now();

    for i in 0..n_times {
        let circle = Circle {
            centerx: 0.0,
            centery: 0.0,
            radius: 1000.0,
        };
        tess(circle, &mut output, &mut cache);
    }

    let t1 = std::time::Instant::now();

    println!(
        "tessellate_circle - total: {:?}, per iteration: {:?}",
        t1 - t0,
        (t1 - t0) / n_times
    );

    println!(
        "Output vertices: {:?}, Output indices: {:?}",
        output.vertices.len(),
        output.indices.len()
    );
}

fn tess(
    circle: Circle,
    buffers: &mut VertexBuffer,
    cache: &mut HashMap<Circle, (Vec<Vertex>, Vec<u32>)>,
) {
    if let Some(v) = cache.get(&circle) {
        // add the vertices to the buffer
        buffers.vertices.extend(v.0.iter().cloned());
        buffers.indices.extend(v.1.iter().cloned());

        return;
    }

    let mut tessellator = lyon::tessellation::FillTessellator::new();

    let center = lyon::math::Point::new(circle.centerx, circle.centery);
    let radius = circle.radius;

    let options = lyon::tessellation::FillOptions::default();

    tessellator
        .tessellate_circle(center, radius, &options, buffers)
        .unwrap();

    // cache the buffers
    cache.insert(
        circle,
        (buffers.get_last_geomtery(), buffers.get_last_indices()),
    );
}
