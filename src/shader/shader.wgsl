const PI: f32 = 3.141592653589793;
const MAX_FLOAT: f32 = 3.4028235e38;
const MAX_U32: u32 = 0xffffffff; 


struct Sphere{
    center: vec3<f32>,
    radius: f32,
    color: vec4<f32>,
    material: f32,
    refactivity: f32,
}

struct CamInfos {
    cam_info: mat4x4<f32>,
    //[cam_pos[0], cam_pos[1], cam_pos[2], view_port_center[0]]
    //[view_port_center[1], view_port_center[2], pixel_width, pixel_height]
    //[half_width, half_height, x[0], x[1]]
    //[x[2], y[0], y[1], y[2]]
}

struct Light{
    position: vec3<f32>,
    is_valid: u32,
    color: vec4<f32>,
    intensity: f32,
}

struct Triangle{
  normal: vec3<f32>,
  v1: vec3<f32>,
  v2: vec3<f32>,
  v3: vec3<f32>,
  padding: vec4<u32>
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
    inv: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec3<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct HitResult {
    distance: f32,
    normal: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> cams: CamInfos;

@group(1) @binding(0)
var<storage, read_write> spheres: array<Sphere>;
@group(1) @binding(1)
var<uniform> sphere_count: u32;

@group(2) @binding(0)
var<storage, read_write> lights: array<Light>;
@group(2) @binding(1)
var<uniform> light_count: u32;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.vert_pos = vec3<f32>(model.position);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    
    var i = (in.vert_pos.x + 1.);
    var j = 1. - (in.vert_pos.y);

    // Extract values from camInfos
    let cam_pos = vec3<f32>(cams.cam_info[0].x, cams.cam_info[0].y, cams.cam_info[0].z);
    let view_port_center = vec3<f32>(cams.cam_info[0].w, cams.cam_info[1].x, cams.cam_info[1].y);
    let half_width = cams.cam_info[2].x;
    let half_height = cams.cam_info[2].y;
    let x = vec3<f32>(cams.cam_info[2].z, cams.cam_info[2].w, cams.cam_info[3].x);
    let y = vec3<f32>(cams.cam_info[3].y, cams.cam_info[3].z, cams.cam_info[3].w);

    // Convert i and j into screen space (-half_width to half_width)
    let u = (i - 1.) * half_width;
    let v = (j - 1.) * half_height;

    // Calculate pixel center
    let pixel_center = view_port_center + x * u + y * v;

    // Ray direction and inverse direction
    let ray_direction = normalize(pixel_center - cam_pos);
    let ray_inv = 1.0 / ray_direction;

    var ray: Ray = Ray(cam_pos, ray_direction, ray_inv);
    var final_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    var attenuation = vec3<f32>(1.0, 1.0, 1.0);

    // Perform hit detection (similar to Rust's `hitSphere` function)
    for (var bounce = 0u; bounce < 5u; bounce++) {
        var closest_hit = HitResult(MAX_FLOAT, vec3<f32>(0.0));
        var hit_sphere: Sphere;
        var hit_light: Light;
        for (var l = 0u; l < sphere_count; l++) {
            let sphere = spheres[l];
            if (sphere.radius <= 0.0) {
                continue;
            }

            let hit_result = hit_sphere(ray, sphere.center, sphere.radius);

            if (hit_result.distance < closest_hit.distance) {
                closest_hit = hit_result;
                hit_sphere = sphere;
           }
        }
        if (closest_hit.distance == MAX_FLOAT ) {
            let sky_contribution = sample_skybox(ray.direction) * vec4<f32>(attenuation, 1.) * 0.1;
            final_color += sky_contribution;
            break;
        }
        let hit_point = ray_at(ray, closest_hit.distance);
        for(var l = 0u; l < light_count; l++) {
            if (lights[l].is_valid == 1)
            {
                let light = lights[l];
                let r_d =  normalize(light.position - hit_point);
                let ray_2 = Ray(light.position, r_d, 1.0 / r_d);
                var hit_found = false;
                for (var k = 0u; k < sphere_count; k++) {
                    let sphere = spheres[k];
                    let hit_result = hit_sphere(ray_2, sphere.center, sphere.radius);
                    if (hit_result.distance != MAX_FLOAT){
                        hit_found = true;
                        break;
                    }
                }
                if (!hit_found){
                    let dist = length(light.position - hit_point);
                    let final_intensity = light.intensity / ( dist * dist / 1000);
                    let max_value = max(dot(closest_hit.normal, r_d), 0.);
                    var diffuse = vec4<f32>(
                        hit_sphere.color.xyz * max_value,
                        hit_sphere.color.w
                    );
                    final_color+= light.color * final_intensity * diffuse;
                }
            }
        }
        if (hit_sphere.material < 1.e-1){
            final_color += vec4<f32>(0.1, 0.1, 0.1, 1.0) * hit_sphere.color;
            break;
        }
        else if (hit_sphere.material <= 1.){
            let specular_ratio = smoothstep(0.0, 1.0, hit_sphere.material);
            let reflected_dir = reflect(ray.direction, closest_hit.normal);
            let diffuse_dir = random_hemisphere_direction(closest_hit.normal, vec2<f32>(10.)); // You'd need to implement this
            ray.direction = mix(diffuse_dir, reflected_dir, specular_ratio);
            // ray.direction = reflect(ray.direction, closest_hit.normal);
        }
        else{
            let cos_theta = min(dot(-ray.direction, closest_hit.normal), 1.0);
            let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
            let refraction_ratio = 1.0 / hit_sphere.refactivity;  // Air to glass
            if (refraction_ratio * sin_theta > 1.0) {
                ray.direction = reflect(ray.direction, closest_hit.normal);
            } else {
                ray.direction = refract_ray(ray.direction, closest_hit.normal, 1.0, 1.5);
            }
        }
        ray.origin = hit_point + closest_hit.normal*0.001;

        ray.inv = 1.0 / ray.direction;
        attenuation *=  hit_sphere.material;

    }

    return final_color;
}

fn random_hemisphere_direction(normal: vec3<f32>, seed: vec2<f32>) -> vec3<f32> {
    // Generate random angles
    let phi = 2.0 * 3.14159 * random_float(seed);
    let cos_theta = random_float(seed + vec2<f32>(1.0, 0.0));
    let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
    
    // Create a vector in local space
    let local_dir = vec3<f32>(
        cos(phi) * sin_theta,
        sin(phi) * sin_theta,
        cos_theta
    );
    
    // Create a coordinate system around the normal
    let tangent = normalize(cross(
        select(vec3<f32>(1.0, 0.0, 0.0), 
               vec3<f32>(0.0, 1.0, 0.0), 
               abs(normal.x) > 0.9),
        normal
    ));
    let bitangent = cross(normal, tangent);
    
    // Transform to world space
    return normalize(
        tangent * local_dir.x +
        bitangent * local_dir.y +
        normal * local_dir.z
    );
}

// Simple random number generator
fn random_float(seed: vec2<f32>) -> f32 {
    return fract(sin(dot(seed, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn sample_skybox(direction: vec3<f32>) -> vec4<f32> {
    // Basic gradient skybox example
    let t = 0.5 * (direction.y + 1.0);
    let sky_color = mix(
        vec4<f32>(0.0, 0.0, 1.0, 1.0),  // Horizon color
        vec4<f32>(0.5, 0.7, 1.0, 1.0),   // Sky color
        t
    );
    return sky_color;
}

fn ray_at(ray: Ray, distance: f32) -> vec3<f32>{
    return ray.origin + ray.direction * distance;
}

fn angle_between_vectors(a: vec3<f32>, b: vec3<f32>) -> f32 {
    let dot_product = dot(a, b);
    let magnitudes = length(a) * length(b);
    return acos(dot_product / magnitudes);
}

fn hit_sphere(r: Ray, sphere_center: vec3<f32>, sphere_radius: f32) -> HitResult {
    let det = r.origin - sphere_center;
    let a = dot(r.direction, r.direction);
    let b = 2.0 * dot(det, r.direction);
    let c = dot(det, det) - sphere_radius * sphere_radius;
    let discriminant = b * b - 4.0 * a * c;

    if (discriminant < 0.0) {
        return HitResult(MAX_FLOAT, vec3<f32>(0.0));; // No intersection
    }

    let sqrt_discriminant = sqrt(discriminant);
    var t = (-b - sqrt_discriminant) / (2.0 * a);
    if (t < 0.0) {
        t = (-b + sqrt_discriminant) / (2.0 * a);
    }
    if (t < 0.0) {
        return HitResult(MAX_FLOAT, vec3<f32>(0.0));
    }
    let hit_point = ray_at(r, t);
    let normal = normalize(hit_point - sphere_center);

    return HitResult(t, normal);
}
fn refract_ray(incident: vec3<f32>, normal: vec3<f32>, n1: f32, n2: f32) -> vec3<f32> {
    let n = n1 / n2;
    let cos_i = -dot(normal, incident);
    let sin_t2 = n * n * (1.0 - cos_i * cos_i);
    
    // Check for total internal reflection
    if (sin_t2 > 1.0) {
        // Total internal reflection occurs
        return reflect(incident, normal);
    }
    
    let cos_t = sqrt(1.0 - sin_t2);
    return n * incident + (n * cos_i - cos_t) * normal;
}
