// types
// Define constants for pi, max float value, and max u32 value
const PI: f32 = 3.141592653589793; // Define pi constant
const MAX_FLOAT: f32 = 3.4028235e38; // Define max float value
const MAX_U32: u32 = 0xffffffff; // Define max u32 value

// Define a struct to represent a sphere
struct Sphere {
  center: vec3<f32>, // Center point of the sphere
  radius: f32, // Radius of the sphere
  color: vec4<f32>, // Color of the sphere
  material: f32, // Material properties of the sphere
  refactivity: f32, // Refractive index of the sphere
}

// Define a struct to represent camera information
struct CamInfos {
  cam_info: mat4x4<f32>, // 4x4 matrix containing camera information
}

// Define a struct to represent a light source
struct Light {
  position: vec3<f32>, // Position of the light source
  is_valid: u32, // Flag indicating whether the light is valid
  color: vec4<f32>, // Color of the light source
  intensity: f32, // Intensity of the light source
}

// Define a struct to represent a ray
struct Ray {
  origin: vec3<f32>, // Origin point of the ray
  direction: vec3<f32>, // Direction vector of the ray
  inv: vec3<f32>, // Inverse of the direction vector
}


//shpere

fn random_hemisphere_direction(normal: vec3<f32>, seed: vec2<f32>) -> vec3<f32> {
    // Generate random angles for spherical coordinates in the hemisphere
    let phi = 2.0 * 3.14159 * random_float(seed); // Random angle around the hemisphere
    let cos_theta = random_float(seed + vec2<f32>(1.0, 0.0)); // Random angle from the normal
    let sin_theta = sqrt(1.0 - cos_theta * cos_theta); // Compute sin(theta) from cos(theta)

    // Create a direction vector in local tangent space
    let local_dir = vec3<f32>(
        cos(phi) * sin_theta,
        sin(phi) * sin_theta,
        cos_theta
    );

    // Define a tangent coordinate system around the normal
    let tangent = normalize(cross(
        select(vec3<f32>(1.0, 0.0, 0.0),
            vec3<f32>(0.0, 1.0, 0.0),
            abs(normal.x) > 0.9), // Choose tangent axis based on normal's orientation
        normal
    ));
    let bitangent = cross(normal, tangent);

    // Convert the local direction to world space by transforming with the tangent space basis
    return normalize(
        tangent * local_dir.x + bitangent * local_dir.y + normal * local_dir.z
    );
}

// single pseudo-random float between 0-1
fn rand(seed: vec2<f32>) -> f32 {
  return 0.5 + 0.5 * 
     fract(sin(dot(seed.xy, vec2(12.9898, 78.233)))* 43758.5453);
}

// Generates a pseudo-random float based on a 2D seed
fn random_float(seed: vec2<f32>) -> f32 {
    return fract(sin(dot(seed, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}
// Calculates the point along a ray at a given distance
fn ray_at(ray: Ray, distance: f32) -> vec3<f32> {
    return ray.origin + ray.direction * distance;
}

// Computes the angle in radians between two vectors
fn angle_between_vectors(a: vec3<f32>, b: vec3<f32>) -> f32 {
    let dot_product = dot(a, b);
    let magnitudes = length(a) * length(b);
    return acos(dot_product / magnitudes); // Compute the angle using the arccos of the dot product
}

// Tests for an intersection between a ray and a sphere
fn hit_sphere_result(r: Ray, center: vec3<f32>, radius: f32) -> HitResult {
    let oc   = r.origin - center;
    let half_b = dot(oc, r.direction);         // b/2
    let c    = dot(oc, oc) - radius * radius;
    let disc = half_b * half_b - c;            // discriminant simplified

    if disc < 0.0 {
        return HitResult(MAX_FLOAT, vec3<f32>(0.0));
    }

    let sq = sqrt(disc);
    var t  = -half_b - sq;       // no /2a since a=1
    if t < 0.001 { t = -half_b + sq; }    // 0.001 replaces your epsilon push
    if t < 0.001 {
        return HitResult(MAX_FLOAT, vec3<f32>(0.0));
    }

    let hit_point = ray_at(r, t);
    let normal    = normalize(hit_point - center);
    return HitResult(t, normal);
}

// Calculates refraction or reflection of a ray through a surface
fn refract_ray(incident: vec3<f32>, normal: vec3<f32>, n1: f32, n2: f32) -> vec3<f32> {
    let n = n1 / n2; // Ratio of refraction indices
    let cos_i = -dot(normal, incident); // Cosine of incident angle
    let sin_t2 = n * n * (1.0 - cos_i * cos_i); // Calculate sin^2 of transmission angle
    
    // Check for total internal reflection
    if sin_t2 > 1.0 {
        // If total internal reflection occurs, return the reflected vector
        return reflect(incident, normal);
    }

    // Calculate cos(theta) for the transmitted angle
    let cos_t = sqrt(1.0 - sin_t2);

    // Compute the refracted direction using Snell's law
    return n * incident + (n * cos_i - cos_t) * normal;
}


// skybox 

fn hash3(p: vec3<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyz) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// 3D noise function for seamless cloud generation
fn noise3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    // Eight corners of the cube
    let a = hash3(i);
    let b = hash3(i + vec3<f32>(1.0, 0.0, 0.0));
    let c = hash3(i + vec3<f32>(0.0, 1.0, 0.0));
    let d = hash3(i + vec3<f32>(1.0, 1.0, 0.0));
    let e = hash3(i + vec3<f32>(0.0, 0.0, 1.0));
    let g = hash3(i + vec3<f32>(1.0, 0.0, 1.0));
    let h = hash3(i + vec3<f32>(0.0, 1.0, 1.0));
    let k = hash3(i + vec3<f32>(1.0, 1.0, 1.0));
    
    // Smooth interpolation
    let u = f * f * (3.0 - 2.0 * f);

    return mix(
        mix(mix(a, b, u.x), mix(c, d, u.x), u.y),
        mix(mix(e, g, u.x), mix(h, k, u.x), u.y),
        u.z
    );
}


// Fractal Brownian Motion for more natural-looking clouds
fn fbm3d(p: vec3<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 3.0;

    for (var i = 0; i < 5; i++) {
        value += amplitude * noise3d(p * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    return value;
}

fn worley(p: vec3<f32>) -> f32 {
    let i = floor(p);
    var min_dist = 10.0;
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            for (var z = -1; z <= 1; z++) {
                let cell = i + vec3<f32>(f32(x), f32(y), f32(z));
                // Random point within this cell
                let rand_pt = cell + vec3<f32>(
                    hash3(cell),
                    hash3(cell + vec3(7.1, 0.0, 0.0)),
                    hash3(cell + vec3(0.0, 3.7, 0.0))
                );
                min_dist = min(min_dist, length(p - rand_pt));
            }
        }
    }
    return min_dist;
}
fn fbm3d_octaves(p: vec3<f32>, count: u32) -> f32 {
    var value = 0.0; var amplitude = 0.5; var frequency = 3.0;
    for (var i = 0u; i < count; i++) {
        value += amplitude * noise3d(p * frequency);
        amplitude *= 0.5; frequency *= 2.0;
    }
    return value;
}
fn domain_warped_fbm(p: vec3<f32>) -> f32 {
    // Your warp vec already costs 3 × fbm3d = 3 × 5 × 8 = 120 hash calls
    // Reduce warp fbm to 3 octaves — imperceptible at skybox distance
    let warp = vec3<f32>(
        fbm3d_octaves(p + vec3(0.0, 0.0, 0.0), 3u),
        fbm3d_octaves(p + vec3(5.2, 1.3, 2.8), 3u),
        fbm3d_octaves(p + vec3(9.1, 3.7, 6.4), 3u)
    );
    return mix(worley(p), fbm3d(p + 1.5 * warp), 0.6);
}

fn sample_skybox(direction: vec3<f32>) -> vec4<f32> {
    // Generate cloud pattern using 3D direction directly to avoid seams
    let cloud_scale = 2.0;
    let cloud_density = domain_warped_fbm(direction * cloud_scale);
    
    // Base sky gradient
    let t = 0.5 * (direction.y + 1.0);
    let sky_color = mix(
        vec4<f32>(0.0, 0.0, 0.0, 1.0),  // Horizon color
        vec4<f32>(0.5, 0.7, 1.0, 1.0),  // Sky color at the top
        t
    );
    
    // Cloud color and transparency
    let cloud_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    let cloud_coverage = 0.4; // Adjust this value to control cloud coverage (0.0 - 1.0)
    let cloud_softness = 0.1; // Adjust this value to control cloud edge softness
    
    // Only show clouds above horizon (direction.y > 0)
    let cloud_mask = smoothstep(0.0, cloud_softness, cloud_density - (1.0 - cloud_coverage)) * smoothstep(-0.1, 0.1, direction.y);
    
    // Mix sky color with clouds
    return mix(sky_color, cloud_color, cloud_mask * 0.7); // 0.7 to make clouds slightly transparent
}

// BVH
struct BVHNode {
    bounds: array<f32, 6>,
    start_triangle: u32,
    triangle_count: u32,
    left_node: u32,
    right_node: u32,
    padding_: array<u32, 6>,
};
struct Triangle {
    n: vec3<f32>,
    pad_n: f32,
    v0: vec3<f32>,
    p0: f32,
    v1: vec3<f32>,
    p1: f32,
    v2: vec3<f32>,
    p2: f32,
};


// Define a struct to represent vertex output
struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>, // Clip space position
  @location(0) vert_pos: vec3<f32>, // Vertex position
}

// Define a struct to represent vertex input
struct VertexInput {
  @location(0) position: vec3<f32>, // Vertex position
}

// Define a struct to represent hit result
struct HitResult {
  distance: f32, // Distance to the hit point
  normal: vec3<f32>, // Normal vector at the hit point
}

// Bind camera information to a uniform buffer
@group(0) @binding(0)
var<uniform> cam_info: mat4x4<f32>;
// [cam_pos[0], cam_pos[1], cam_pos[2], view_port_center[0]]
// [view_port_center[1], view_port_center[2], pixel_width, pixel_height]
// [half_width, half_height, x[0], x[1]]
// [x[2], y[0], y[1], y[2]]

// Bind sphere data to a storage buffer
@group(1) @binding(0)
var<storage, read> spheres: array<Sphere>;

// Bind light data to a storage buffer
@group(2) @binding(0)
var<storage, read> lights: array<Light>;
// Bind light count to a uniform buffer

// Bind light data to a storage buffer
@group(3) @binding(0) var<storage, read> bvh_nodes: array<BVHNode>;
@group(3) @binding(1) var<storage, read> bvh_triangles: array<Triangle>;

struct Count {
    count: u32,
}

@group(3) @binding(2) var<uniform> bvh_nodes_count: Count;
@group(3) @binding(3) var<uniform> bvh_triangles_count: Count;

const offset_count = 8u;

fn intersect_aabb(ray: Ray, bounds: array<f32, 6>) -> f32 {
    let inv_dir = ray.inv;
    
    let t1 = (bounds[0] - ray.origin.x) * inv_dir.x;
    let t2 = (bounds[3] - ray.origin.x) * inv_dir.x;
    let t3 = (bounds[1] - ray.origin.y) * inv_dir.y;
    let t4 = (bounds[4] - ray.origin.y) * inv_dir.y;
    let t5 = (bounds[2] - ray.origin.z) * inv_dir.z;
    let t6 = (bounds[5] - ray.origin.z) * inv_dir.z;

    let tmin_x = min(t1, t2);
    let tmax_x = max(t1, t2);
    let tmin_y = min(t3, t4);
    let tmax_y = max(t3, t4);
    let tmin_z = min(t5, t6);
    let tmax_z = max(t5, t6);

    let tmin = max(max(tmin_x, tmin_y), tmin_z);
    let tmax = min(min(tmax_x, tmax_y), tmax_z);

    if (tmax >= tmin && tmax > 0.0) {
        return tmin;
    }
    return MAX_FLOAT;
}

fn intersect_triangle(ray: Ray, tri: Triangle) -> HitResult {
    let e1 = tri.v1 - tri.v0;
    let e2 = tri.v2 - tri.v0;
    let ray_cross_e2 = cross(ray.direction, e2);
    let det = dot(e1, ray_cross_e2);

    if (det > -0.0000001 && det < 0.0000001) {
        return HitResult(MAX_FLOAT, vec3<f32>(0.0));
    }

    let inv_det = 1.0 / det;
    let s = ray.origin - tri.v0;
    let u = inv_det * dot(s, ray_cross_e2);
    if (u < 0.0 || u > 1.0) {
        return HitResult(MAX_FLOAT, vec3<f32>(0.0));
    }

    let s_cross_e1 = cross(s, e1);
    let v = inv_det * dot(ray.direction, s_cross_e1);
    if (v < 0.0 || u + v > 1.0) {
        return HitResult(MAX_FLOAT, vec3<f32>(0.0));
    }

    let t = inv_det * dot(e2, s_cross_e1); 
    if (t > 0.001) {
        var normal = normalize(tri.n);
        if (dot(ray.direction, normal) > 0.0) {
            normal = -normal;
        }
        return HitResult(t, normal);
    }
    return HitResult(MAX_FLOAT, vec3<f32>(0.0));
}

fn traverse_bvh(ray: Ray) -> HitResult {
    var stack: array<u32, 32>;
    var stack_ptr: i32 = 0;
    stack[stack_ptr] = 0u;
    
    var closest_hit = HitResult(MAX_FLOAT, vec3<f32>(0.0));

    while (stack_ptr >= 0) {
        let node_idx = stack[stack_ptr];
        stack_ptr = stack_ptr - 1;
        
        let node = bvh_nodes[node_idx];
        
        let t_aabb = intersect_aabb(ray, node.bounds);
        if (t_aabb >= closest_hit.distance) {
            continue;
        }

        if (node.triangle_count > 0u) {
            for (var i = 0u; i < node.triangle_count; i = i + 1u) {
                let tri_idx = node.start_triangle + i;
                let tri = bvh_triangles[tri_idx];
                let hit = intersect_triangle(ray, tri);
                if (hit.distance < closest_hit.distance) {
                    closest_hit = hit;
                }
            }
        } else {
            let left_idx = node.left_node;
            let right_idx = node.right_node;
            
            let t_left = intersect_aabb(ray, bvh_nodes[left_idx].bounds);
            let t_right = intersect_aabb(ray, bvh_nodes[right_idx].bounds);
            
            if (t_left < closest_hit.distance && t_right < closest_hit.distance) {
                if (t_left < t_right) {
                    stack_ptr = stack_ptr + 1;
                    stack[stack_ptr] = right_idx;
                    stack_ptr = stack_ptr + 1;
                    stack[stack_ptr] = left_idx;
                } else {
                    stack_ptr = stack_ptr + 1;
                    stack[stack_ptr] = left_idx;
                    stack_ptr = stack_ptr + 1;
                    stack[stack_ptr] = right_idx;
                }
            } else if (t_left < closest_hit.distance) {
                stack_ptr = stack_ptr + 1;
                stack[stack_ptr] = left_idx;
            } else if (t_right < closest_hit.distance) {
                stack_ptr = stack_ptr + 1;
                stack[stack_ptr] = right_idx;
            }
        }
    }
    
    return closest_hit;
}

// Define the vertex shader
@vertex
fn vs_main(
    @builtin(vertex_index) vert_idx: u32
) -> VertexOutput {
    var out: VertexOutput;
    var pos = array(
        vec2(-1.0, -1.0),  // Triangle 1
        vec2(1.0, -1.0),
        vec2(-1.0, 1.0),
        vec2(1.0, -1.0),  // Triangle 2
        vec2(1.0, 1.0),
        vec2(-1.0, 1.0),
    );
    let xy = pos[vert_idx];
    out.clip_position = vec4<f32>(xy, 0.0, 1.0);
    out.vert_pos = vec3<f32>(xy, 0.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert the vertex position from [-1,1] range to [0,1] for screen space
    var i = (in.vert_pos.x + 1.);
    var j = 1. - (in.vert_pos.y);

    // Extract camera information (position, viewport center, dimensions, orientation) from camInfos
    let cam_pos = vec3<f32>(cam_info[0].x, cam_info[0].y, cam_info[0].z);
    let view_port_center = vec3<f32>(cam_info[0].w, cam_info[1].x, cam_info[1].y);
    let x = vec3<f32>(cam_info[2].z, cam_info[2].w, cam_info[3].x);
    let y = vec3<f32>(cam_info[3].y, cam_info[3].z, cam_info[3].w);

    // Map i and j into screen space coordinates
    // Initialize the final color as black and set initial attenuation for lighting calculations
    var final_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let half_count = offset_count / 2;
    var offsets: array<vec2<f32>, offset_count>;
    for (var idx = 0u; idx < half_count; idx++) {
        for (var idx2 = 0u; idx2 < half_count; idx2++) {
            let seed = vec2<f32>(in.vert_pos.x,in.vert_pos.y) * f32(idx + 1u) * 127.1;
            offsets[idx * half_count + idx2] = vec2<f32>(rand(seed), rand(seed + vec2<f32>(1.0, 0.0)));
        }
    }

    for (var idx = 0u; idx < offset_count; idx++) {
        var ray_color = vec4(0., 0., 0., 1.);
        let u = (i - 1. + offsets[idx].x * cam_info[1].z) * cam_info[2].x;
        let v = (j - 1. + offsets[idx].y * cam_info[1].w) * cam_info[2].y;
    
        // Calculate the center of the pixel in world space
        let pixel_center = view_port_center + x * u + y * v;
    
        // Define ray direction and inverse direction based on camera position and pixel center
        let ray_direction = normalize(pixel_center - cam_pos);
        let ray_inv = 1.0 / ray_direction;
    
        // Initialize the ray with origin (camera position), direction, and inverse direction
        var ray: Ray = Ray(cam_pos, ray_direction, ray_inv);


        var attenuation = vec3<f32>(1.0, 1.0, 1.0);
    
        // Boolean to check if a sphere is hit during ray tracing
        var is_hit_sphere = false;
    
        // Loop over possible bounces (up to 5) for reflection/refraction effects
        for (var bounce = 0u; bounce < 5u; bounce++) {
            // Initialize closest hit result with maximum distance and no hit point
            var closest_hit = HitResult(MAX_FLOAT, vec3<f32>(0.0));
            var hit_sphere: Sphere; // Placeholder for the hit sphere
            var hit_light: Light;   // Placeholder for the hit light
    
            // Check for intersections with each sphere in the scene
            for (var l = 0u; l < arrayLength(&spheres); l++) {
                let sphere = spheres[l];
                if sphere.radius <= 0.0 { // Skip invalid spheres
                    continue;
                }
    
                // Check for ray-sphere intersection
                let hit_result = hit_sphere_result(ray, sphere.center, sphere.radius);
    
                // Update closest hit if a nearer hit is found
                if hit_result.distance < closest_hit.distance {
                    closest_hit = hit_result;
                    hit_sphere = sphere;
                    is_hit_sphere = true;
                }
            }

            // Check for intersections with the BVH
            if (bvh_nodes_count.count > 0u) {
                let hit_bvh = traverse_bvh(ray);
                if (hit_bvh.distance < closest_hit.distance) {
                    closest_hit = hit_bvh;
                    is_hit_sphere = true; // Set to true to proceed with lighting/skybox logic
                    // Assign default material for meshes
                    hit_sphere = Sphere(vec3<f32>(0.0), 0.0, vec4<f32>(0.8, 0.8, 0.8, 1.0), 2, 0.15);
                }
            }
    
            // If no intersection with spheres, calculate skybox color and break out of loop
            if closest_hit.distance == MAX_FLOAT {
                if is_hit_sphere {
                    let sky_contribution = (sample_skybox(ray.direction)) / (10.0 * f32(bounce + 1u));
                    ray_color += sky_contribution;
                } else {
                    let sky_contribution = sample_skybox(ray.direction);
                    ray_color = sky_contribution;
                }
                break;
            }
    
            // Calculate the hit point based on the ray and the closest hit distance
            let hit_point = ray_at(ray, closest_hit.distance);
    
            // Loop over lights to calculate direct illumination at the hit point
            for (var l = 0u; l < arrayLength(&lights); l++) {
                if lights[l].is_valid == 1 {
                    let light = lights[l];
                    let r_d = normalize(light.position - hit_point); // Direction to the light
                    let shadow_origin = hit_point + closest_hit.normal * 0.001;
                    let ray_2 = Ray(shadow_origin, r_d, 1.0 / r_d);
                    let dist_to_light = length(light.position - hit_point);
    
                    // Check if light is visible by tracing a shadow ray
                    var hit_found = false;
                    for (var k = 0u; k < arrayLength(&spheres); k++) {
                        let sphere = spheres[k];
                        let hit_result = hit_sphere_result(ray_2, sphere.center, sphere.radius);
                        if (hit_result.distance < dist_to_light) {
                            hit_found = true;
                            break;
                        }
                    }
                    if (!hit_found && bvh_nodes_count.count > 0u) {
                        let hit_bvh = traverse_bvh(ray_2);
                        if (hit_bvh.distance < dist_to_light) {
                            hit_found = true;
                        }
                    }
    
                    // If no objects obstruct the light, calculate lighting contribution
                    if !hit_found {
                        let dist = length(light.position - hit_point);
                        let final_intensity = light.intensity / (dist * dist / 1000);
                        let max_value = max(dot(closest_hit.normal, r_d), 0.);
                        var diffuse = vec4<f32>(
                            hit_sphere.color.xyz * max_value,
                            hit_sphere.color.w
                        );
                        ray_color += light.color * final_intensity * diffuse;
                    }
                }
            }
    
            // Handle different material types
            if hit_sphere.material < 1.e-1 { // Diffuse material
                ray_color += vec4<f32>(0.1, 0.1, 0.1, 1.0) * hit_sphere.color;
                break;
            } else if hit_sphere.material <= 1. { // Reflective material
                let specular_ratio = smoothstep(0.0, 1.0, hit_sphere.material);
                let reflected_dir = reflect(ray.direction, closest_hit.normal);
                // let seed = vec2<f32>(in.vert_pos.xy) * f32(bounce + 1u) * 127.1;
                let diffuse_dir = random_hemisphere_direction(closest_hit.normal, vec2<f32>(10.)); // Generate a random direction
                ray.direction = mix(diffuse_dir, reflected_dir, specular_ratio);
            } else { // Refractive material
                let cos_theta = min(dot(-ray.direction, closest_hit.normal), 1.0);
                let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
                let refraction_ratio = 1.0 / hit_sphere.refactivity;  // Air to glass
                if refraction_ratio * sin_theta > 1.0 { // Total internal reflection
                    ray.direction = reflect(ray.direction, closest_hit.normal);
                } else { // Calculate refraction direction
                    ray.direction = refract_ray(ray.direction, closest_hit.normal, 1.0, 1.5);
                }
            }
    
            // Move the ray origin slightly along the normal to prevent self-intersection
            ray.origin = hit_point + closest_hit.normal * 0.001;
            ray.inv = 1.0 / ray.direction;
            attenuation *= hit_sphere.material;
        }

        final_color += ray_color;
    }

    final_color /= f32(offset_count);

    // Return the calculated color after all bounces and lighting computations
    return final_color;
}
