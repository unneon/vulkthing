use crate::config::DEFAULT_PLANET_SCALE;
use crate::mesh::MeshData;
use crate::renderer::util::{Buffer, Dev};
use crate::renderer::vertex::GrassBlade;
use crate::renderer::{GrassChunk, UNIFIED_MEMORY};
use crate::util::MpscPendingIterator;
use ash::vk;
use log::debug;
use nalgebra::{Rotation3, Unit, Vector3};
use noise::{NoiseFn, Perlin};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;
use std::f32::consts::PI;
use std::sync::mpsc;
use std::thread::JoinHandle;

pub struct GrassState {
    // TODO: Remove Options once moving in main is fixed.
    request_tx: Option<mpsc::Sender<GrassRequest>>,
    response_rx: MpscPendingIterator<GrassResponse>,
    thread_handle: Option<JoinHandle<()>>,
}

struct GrassThread {
    planet_mesh: MeshData,
    chunks: Vec<Vec<usize>>,
    loaded_chunks: HashSet<usize>,
    parameters: GrassParameters,
    response_tx: mpsc::Sender<GrassResponse>,
    dev: Dev,
}

#[derive(Clone)]
pub struct GrassParameters {
    pub enabled: bool,
    pub blades_per_triangle: usize,
    pub height_average: f32,
    pub height_max_variance: f32,
    pub height_noise_frequency: f32,
    pub width: f32,
    pub sway_frequency: f32,
    pub sway_amplitude: f32,
    pub chunk_count: usize,
    pub chunk_load_distance: f32,
    pub chunk_unload_distance: f32,
}

enum GrassRequest {
    Camera(Vector3<f32>),
    Parameters(GrassParameters),
}

pub enum GrassResponse {
    Load(usize, GrassChunk),
    Unload(usize),
}

const GOLDEN_RATIO: f32 = 1.618034;

impl GrassState {
    pub fn new(parameters: &GrassParameters, planet_mesh: &MeshData, dev: Dev) -> GrassState {
        let parameters = parameters.clone();
        let planet_mesh = planet_mesh.clone();
        let (request_tx, request_rx) = mpsc::channel();
        let (response_tx, response_rx) = mpsc::channel();
        let thread_handle = std::thread::spawn({
            move || {
                let chunks = build_triangle_chunks(&parameters, &planet_mesh);
                let mut thread = GrassThread {
                    planet_mesh,
                    chunks,
                    loaded_chunks: HashSet::new(),
                    parameters,
                    response_tx,
                    dev,
                };
                'outer: loop {
                    let mut camera_position = None;
                    let mut new_parameters = None;
                    let mut first_received = false;
                    while let Some(grass_request) = if first_received {
                        request_rx.try_recv().ok()
                    } else {
                        match request_rx.recv() {
                            Ok(grass_request) => Some(grass_request),
                            Err(_) => break 'outer,
                        }
                    } {
                        match grass_request {
                            GrassRequest::Camera(camera_position2) => {
                                camera_position = Some(camera_position2)
                            }
                            GrassRequest::Parameters(new_parameters2) => {
                                new_parameters = Some(new_parameters2)
                            }
                        }
                        first_received = true;
                    }
                    if let Some(camera_position) = camera_position {
                        thread.update_camera(camera_position);
                    }
                    if let Some(new_parameters) = new_parameters {
                        thread.update_parameters(new_parameters);
                    }
                }
            }
        });
        GrassState {
            request_tx: Some(request_tx),
            response_rx: MpscPendingIterator { rx: response_rx },
            thread_handle: Some(thread_handle),
        }
    }

    pub fn update_parameters(&mut self, new_parameters: &GrassParameters) {
        if let Some(request_tx) = self.request_tx.as_ref() {
            let _ = request_tx.send(GrassRequest::Parameters(new_parameters.clone()));
        }
    }

    pub fn update_camera(&mut self, camera_position: Vector3<f32>) {
        if let Some(request_tx) = self.request_tx.as_ref() {
            let _ = request_tx.send(GrassRequest::Camera(camera_position));
        }
    }

    pub fn events(&self) -> &MpscPendingIterator<GrassResponse> {
        &self.response_rx
    }

    pub fn shutdown(&mut self) {
        drop(self.request_tx.take());
        self.thread_handle.take().unwrap().join().unwrap();
    }
}

impl GrassThread {
    fn update_camera(&mut self, camera_position: Vector3<f32>) {
        if !self.parameters.enabled {
            return;
        }
        for (chunk_id, chunk) in self.chunks.iter().enumerate() {
            let triangle_id = chunk[0];
            let vertex = DEFAULT_PLANET_SCALE * self.planet_mesh.vertices[3 * triangle_id].position;
            let distance = (vertex - camera_position).norm();
            let loaded = self.loaded_chunks.contains(&chunk_id);
            if loaded && distance > self.parameters.chunk_unload_distance {
                self.loaded_chunks.remove(&chunk_id);
                let _ = self.response_tx.send(GrassResponse::Unload(chunk_id));
            }
            if !loaded && distance < self.parameters.chunk_load_distance {
                self.loaded_chunks.insert(chunk_id);
                let chunk = self.prepare_chunk(chunk_id);
                let _ = self.response_tx.send(GrassResponse::Load(chunk_id, chunk));
            }
        }
    }

    fn update_parameters(&mut self, new_parameters: GrassParameters) {
        if self.parameters.enabled && !new_parameters.enabled {
            for chunk_id in self.loaded_chunks.drain() {
                let _ = self.response_tx.send(GrassResponse::Unload(chunk_id));
            }
        }
        // TODO: Rebuild chunks if necessary.
        self.parameters = new_parameters;
    }

    fn prepare_chunk(&self, chunk_id: usize) -> GrassChunk {
        let chunk: &[usize] = self.chunks[chunk_id].as_slice();
        let blades_data = generate_grass_blades(&self.parameters, &self.planet_mesh, chunk);
        let blades_size = std::mem::size_of_val(blades_data.as_slice());
        let blades_buffer = Buffer::create(
            UNIFIED_MEMORY,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            blades_size,
            &self.dev,
        );
        blades_buffer.fill_from_slice_host_visible(&blades_data, &self.dev);
        crate::renderer::debug::set_label(
            blades_buffer.buffer,
            &format!("Grass buffer chunk={chunk_id}"),
            &self.dev,
        );
        crate::renderer::debug::set_label(
            blades_buffer.memory,
            &format!("Grass memory chunk={chunk_id}"),
            &self.dev,
        );
        GrassChunk {
            buffer: blades_buffer,
            triangle_count: blades_data.len(),
        }
    }
}

fn generate_grass_blades(
    parameters: &GrassParameters,
    planet_mesh: &MeshData,
    chunk: &[usize],
) -> Vec<GrassBlade> {
    let mut grass_blades = Vec::new();
    let mut rng = SmallRng::from_seed([23; 32]);
    let height_noise_generator = Perlin::new(907);
    for triangle_index in chunk {
        let triangle = &planet_mesh.vertices[3 * triangle_index..3 * triangle_index + 3];
        let d1 = triangle[1].position - triangle[0].position;
        let d2 = triangle[2].position - triangle[0].position;
        for _ in 0..parameters.blades_per_triangle {
            let mut t1: f32 = rng.gen();
            let mut t2: f32 = rng.gen();
            if t1 + t2 > 1. {
                t1 = 1. - t1;
                t2 = 1. - t2;
            }
            let position = triangle[0].position + t1 * d1 + t2 * d2;
            let up = position.normalize();
            let angle = rng.gen_range((0.)..(2. * PI));
            // https://math.stackexchange.com/a/4112622
            let right = (Rotation3::from_axis_angle(&Unit::new_normalize(up), angle)
                * Vector3::new(
                    up.z.copysign(up.x),
                    up.z.copysign(up.y),
                    -(up.x.abs() + up.y.abs()).copysign(up.z),
                ))
            .normalize();
            let front = up.cross(&right).normalize();
            let height_noise_arg = position * parameters.height_noise_frequency;
            let height_noise = height_noise_generator.get([
                height_noise_arg.x as f64,
                height_noise_arg.y as f64,
                height_noise_arg.z as f64,
            ]) as f32;
            grass_blades.push(GrassBlade {
                position,
                up,
                right,
                front,
                height_noise,
                ground_normal: triangle[0].normal,
            });
        }
    }
    grass_blades
}

pub fn build_triangle_chunks(grass: &GrassParameters, planet_model: &MeshData) -> Vec<Vec<usize>> {
    assert_eq!(grass.chunk_count % 2, 1);
    let fib = compute_fibonacci_sphere(grass.chunk_count as i64 / 2);
    let mut chunks = vec![Vec::new(); grass.chunk_count];
    for (i, triangle) in planet_model.vertices.array_chunks::<3>().enumerate() {
        // I love how Rust doesn't implement Ord on floats.
        let mut best_distance = f32::INFINITY;
        let mut best_chunk_id = usize::MAX;
        for (chunk_id, chunk_center) in fib.iter().enumerate() {
            let distance = (chunk_center - triangle[0].position).norm();
            if distance < best_distance {
                best_distance = distance;
                best_chunk_id = chunk_id;
            }
        }
        chunks[best_chunk_id].push(i);
    }
    debug!("grass chunk generated");
    chunks
}

/// Generates 2n+1 reasonably uniformly distributed points on the unit sphere.
fn compute_fibonacci_sphere(n: i64) -> Vec<Vector3<f32>> {
    // https://arxiv.org/pdf/0912.4540.pdf
    let mut points = Vec::new();
    for i in -n..=n {
        let latitude = ((2 * i) as f32 / (2 * n + 1) as f32).asin();
        let longitude = 2. * PI * (i as f32) * GOLDEN_RATIO;
        let position = Vector3::new(
            longitude.cos() * latitude.cos(),
            longitude.sin() * latitude.cos(),
            latitude.sin(),
        );
        points.push(position);
    }
    points
}
