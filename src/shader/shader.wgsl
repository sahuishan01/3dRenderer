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
  // [cam_pos[0], cam_pos[1], cam_pos[2], view_port_center[0]]
  // [view_port_center[1], view_port_center[2], pixel_width, pixel_height]
  // [half_width, half_height, x[0], x[1]]
  // [x[2], y[0], y[1], y[2]]
}

// Define a struct to represent a light source
struct Light {
  position: vec3<f32>, // Position of the light source
  is_valid: u32, // Flag indicating whether the light is valid
  color: vec4<f32>, // Color of the light source
  intensity: f32, // Intensity of the light source
}

// Define a struct to represent a triangle
struct Triangle {
  normal: vec3<f32>, // Normal vector of the triangle
  v1: vec3<f32>, // First vertex of the triangle
  v2: vec3<f32>, // Second vertex of the triangle
  v3: vec3<f32>, // Third vertex of the triangle
  padding: vec4<u32> // Padding to ensure proper alignment
}

// Define a struct to represent a ray
struct Ray {
  origin: vec3<f32>, // Origin point of the ray
  direction: vec3<f32>, // Direction vector of the ray
  inv: vec3<f32>, // Inverse of the direction vector
}

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
var<uniform> cams: CamInfos;

// Bind sphere data to a storage buffer
@group(1) @binding(0)
var<storage, read_write> spheres: array<Sphere>;
// Bind sphere count to a uniform buffer
@group(1) @binding(1)
var<uniform> sphere_count: u32;

// Bind light data to a storage buffer
@group(2) @binding(0)
var<storage, read_write> lights: array<Light>;
// Bind light count to a uniform buffer
@group(2) @binding(1)
var<uniform> light_count: u32;

// Define the vertex shader
@vertex
fn vs_main(
  model: VertexInput, // Input vertex data
) -> VertexOutput {
  var out: VertexOutput;
  out.clip_position = vec4<f32>(model.position, 1.0);
  out.vert_pos = vec3<f32>(model.position);
  
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert the vertex position from [-1,1] range to [0,1] for screen space
    var i = (in.vert_pos.x + 1.);
    var j = 1. - (in.vert_pos.y);

    // Extract camera information (position, viewport center, dimensions, orientation) from camInfos
    let cam_pos = vec3<f32>(cams.cam_info[0].x, cams.cam_info[0].y, cams.cam_info[0].z);
    let view_port_center = vec3<f32>(cams.cam_info[0].w, cams.cam_info[1].x, cams.cam_info[1].y);
    let half_width = cams.cam_info[2].x;
    let half_height = cams.cam_info[2].y;
    let x = vec3<f32>(cams.cam_info[2].z, cams.cam_info[2].w, cams.cam_info[3].x);
    let y = vec3<f32>(cams.cam_info[3].y, cams.cam_info[3].z, cams.cam_info[3].w);
    let pixel_width = cams.cam_info[1].z;
    let pixel_height = cams.cam_info[1].w;

    // Map i and j into screen space coordinates
    // Initialize the final color as black and set initial attenuation for lighting calculations
    var final_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let offset_count = 16u;
    var offsets = array<vec2<f32>, 16>(
        // Row 1
        vec2<f32>(0.125, 0.125),
        vec2<f32>(0.375, 0.125),
        vec2<f32>(0.625, 0.125),
        vec2<f32>(0.875, 0.125),
        
        // Row 2
        vec2<f32>(0.125, 0.375),
        vec2<f32>(0.375, 0.375),
        vec2<f32>(0.625, 0.375),
        vec2<f32>(0.875, 0.375),
        
        // Row 3
        vec2<f32>(0.125, 0.625),
        vec2<f32>(0.375, 0.625),
        vec2<f32>(0.625, 0.625),
        vec2<f32>(0.875, 0.625),
        
        // Row 4
        vec2<f32>(0.125, 0.875),
        vec2<f32>(0.375, 0.875),
        vec2<f32>(0.625, 0.875),
        vec2<f32>(0.875, 0.875),
    );

    for (var idx = 0u; idx < offset_count; idx++){
        var ray_color = vec4(0., 0., 0., 1.);
        let u = (i - 1. + offsets[idx].x * pixel_width) * half_width;
        let v = (j - 1. + offsets[idx].y * pixel_height) * half_height;
    
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
            for (var l = 0u; l < sphere_count; l++) {
                let sphere = spheres[l];
                if (sphere.radius <= 0.0) { // Skip invalid spheres
                    continue;
                }
    
                // Check for ray-sphere intersection
                let hit_result = hit_sphere(ray, sphere.center, sphere.radius);
    
                // Update closest hit if a nearer hit is found
                if (hit_result.distance < closest_hit.distance) {
                    closest_hit = hit_result;
                    hit_sphere = sphere;
                    is_hit_sphere = true;
                }
            }
    
            // If no intersection with spheres, calculate skybox color and break out of loop
            if (closest_hit.distance == MAX_FLOAT) {
                if (is_hit_sphere) {
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
            for(var l = 0u; l < light_count; l++) {
                if (lights[l].is_valid == 1) {
                    let light = lights[l];
                    let r_d = normalize(light.position - hit_point); // Direction to the light
                    let ray_2 = Ray(light.position, r_d, 1.0 / r_d);
    
                    // Check if light is visible by tracing a shadow ray
                    var hit_found = false;
                    for (var k = 0u; k < sphere_count; k++) {
                        let sphere = spheres[k];
                        let hit_result = hit_sphere(ray_2, sphere.center, sphere.radius);
                        if (hit_result.distance != MAX_FLOAT) {
                            hit_found = true;
                            break;
                        }
                    }
    
                    // If no objects obstruct the light, calculate lighting contribution
                    if (!hit_found) {
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
            if (hit_sphere.material < 1.e-1) { // Diffuse material
                ray_color += vec4<f32>(0.1, 0.1, 0.1, 1.0) * hit_sphere.color;
                break;
            } else if (hit_sphere.material <= 1.) { // Reflective material
                let specular_ratio = smoothstep(0.0, 1.0, hit_sphere.material);
                let reflected_dir = reflect(ray.direction, closest_hit.normal);
                let diffuse_dir = random_hemisphere_direction(closest_hit.normal, vec2<f32>(10.)); // Generate a random direction
                ray.direction = mix(diffuse_dir, reflected_dir, specular_ratio);
            } else { // Refractive material
                let cos_theta = min(dot(-ray.direction, closest_hit.normal), 1.0);
                let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
                let refraction_ratio = 1.0 / hit_sphere.refactivity;  // Air to glass
                if (refraction_ratio * sin_theta > 1.0) { // Total internal reflection
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
        tangent * local_dir.x +
        bitangent * local_dir.y +
        normal * local_dir.z
    );
}

// Generates a pseudo-random float based on a 2D seed
fn random_float(seed: vec2<f32>) -> f32 {
    return fract(sin(dot(seed, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}
fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// 2D noise function for cloud generation
fn noise2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    // Four corners of the tile
    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));
    
    // Smooth interpolation
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(
        mix(a, b, u.x),
        mix(c, d, u.x),
        u.y
    );
}

// Fractal Brownian Motion for more natural-looking clouds
fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 3.0;
    
    for(var i = 0; i < 5; i++) {
        value += amplitude * noise2d(p * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    return value;
}

fn sample_skybox(direction: vec3<f32>) -> vec4<f32> {
    // Map direction to UV coordinates for cloud sampling
    let phi = atan2(direction.z, direction.x);
    let theta = acos(direction.y);
    let uv = vec2<f32>(phi / (2.0 * 3.14159) + 0.5, theta / 3.14159);
    
    // Generate cloud pattern
    let cloud_scale = 4.0;
    let cloud_density = fbm(uv * cloud_scale + vec2<f32>(0.1)); // Add small offset for variation
    
    // Base sky gradient
    let t = 0.5 * (direction.y + 1.0);
    let sky_color = mix(
        vec4<f32>(0.1, 0.2, 1.0, 1.0),  // Horizon color
        vec4<f32>(0.5, 0.7, 1.0, 1.0),  // Sky color at the top
        t
    );
    
    // Cloud color and transparency
    let cloud_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    let cloud_coverage = 0.4; // Adjust this value to control cloud coverage (0.0 - 1.0)
    let cloud_softness = 0.1; // Adjust this value to control cloud edge softness
    
    // Only show clouds above horizon (direction.y > 0)
    let cloud_mask = smoothstep(0.0, cloud_softness, cloud_density - (1.0 - cloud_coverage)) * 
                     smoothstep(-0.1, 0.1, direction.y);
    
    // Mix sky color with clouds
    return mix(sky_color, cloud_color, cloud_mask * 0.7); // 0.7 to make clouds slightly transparent
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
fn hit_sphere(r: Ray, sphere_center: vec3<f32>, sphere_radius: f32) -> HitResult {
    let det = r.origin - sphere_center; // Vector from ray origin to sphere center
    let a = dot(r.direction, r.direction); // Quadratic term coefficient
    let b = 2.0 * dot(det, r.direction); // Linear term coefficient
    let c = dot(det, det) - sphere_radius * sphere_radius; // Constant term
    let discriminant = b * b - 4.0 * a * c; // Calculate discriminant

    // Check if the discriminant is negative, meaning no intersection
    if (discriminant < 0.0) {
        return HitResult(MAX_FLOAT, vec3<f32>(0.0)); // No intersection
    }

    // Calculate the nearest intersection point using the discriminant
    let sqrt_discriminant = sqrt(discriminant);
    var t = (-b - sqrt_discriminant) / (2.0 * a); // First possible intersection point
    if (t < 0.0) { // If behind the ray, check second intersection point
        t = (-b + sqrt_discriminant) / (2.0 * a);
    }
    if (t < 0.0) { // If both are behind the ray, no intersection
        return HitResult(MAX_FLOAT, vec3<f32>(0.0));
    }

    // Calculate the hit point and normal at the intersection
    let hit_point = ray_at(r, t);
    let normal = normalize(hit_point - sphere_center);

    return HitResult(t, normal);
}

// Calculates refraction or reflection of a ray through a surface
fn refract_ray(incident: vec3<f32>, normal: vec3<f32>, n1: f32, n2: f32) -> vec3<f32> {
    let n = n1 / n2; // Ratio of refraction indices
    let cos_i = -dot(normal, incident); // Cosine of incident angle
    let sin_t2 = n * n * (1.0 - cos_i * cos_i); // Calculate sin^2 of transmission angle
    
    // Check for total internal reflection
    if (sin_t2 > 1.0) {
        // If total internal reflection occurs, return the reflected vector
        return reflect(incident, normal);
    }

    // Calculate cos(theta) for the transmitted angle
    let cos_t = sqrt(1.0 - sin_t2);

    // Compute the refracted direction using Snell's law
    return n * incident + (n * cos_i - cos_t) * normal;
}
