const PI: f32 = 3.141592653589793;
const MAX_FLOAT: f32 = 3.4028235e38;
const MAX_U32: u32 = 0xffffffff; 


struct Sphere{
    center: vec3<f32>,
    radius: f32,
    color: vec4<f32>,
    material: u32,
}

struct Uniforms {
    mouse_pos: vec2<f32>,
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
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var<uniform> cams: CamInfos;
@group(1) @binding(1)
var<uniform> spheres: array<Sphere, 200>;
@group(1) @binding(2)
var<uniform> lights: array<Light, 200>;
@group(1) @binding(3)
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

    var hit_indices: array<u32, 10> = array<u32, 10>(
        MAX_U32, MAX_U32, MAX_U32, MAX_U32, MAX_U32,
        MAX_U32, MAX_U32, MAX_U32, MAX_U32, MAX_U32
    );
    var current_hit_index = 0u;
    // Perform hit detection (similar to Rust's `hitSphere` function)
    for (var bounce = 0u; bounce < 5u; bounce++) {
        var closest_hit = HitResult(MAX_FLOAT, vec3<f32>(0.0));
        var hit_sphere: Sphere;
        for (var l = 0u; l < 200u; l++) {
            let sphere = spheres[l];
            if (sphere.radius <= 0.0) {
                continue;
            }

            let hit_result = hit_sphere(ray, sphere.center, sphere.radius);
            var break_loop = false;
            if (hit_result.distance < closest_hit.distance){
                for (var k = 0u; k < current_hit_index; k++)
                {
                    if (hit_indices[k] == l){
                        break_loop = true;
                        break;
                    }
                }
            }
            if (break_loop){
                break;
            }

            if (hit_result.distance < closest_hit.distance) {
                closest_hit = hit_result;
                hit_sphere = sphere;
                hit_indices[current_hit_index] = l;
                current_hit_index += 1u;
            }
        }

        if (closest_hit.distance == MAX_FLOAT || hit_sphere.material == 0) {
            break;
        }

        let hit_point = ray_at(ray, closest_hit.distance);
        let normal = closest_hit.normal;

        var diffuse = vec3<f32>(0., 0., 0.);
        for(var k = 0u; k < light_count; k++) {
            let light = lights[k];
            if (light.is_valid == 1){
                let light_dir = normalize(light.position);
                let max_value = max(dot(normal, light_dir), 0.);
                diffuse += vec3<f32>(max_value);
            }
            else{
                continue;
            }
        }
        let sphere_color = hit_sphere.color.rgb;
        final_color += vec4<f32>(sphere_color * diffuse * attenuation, 0.0);
        // final_color = hit_sphere.color;

        // Update ray for next bounce
        ray.origin = hit_point + closest_hit.normal*0.001;
        ray.direction = reflect(ray.direction, normal);

        ray.inv = 1.0 / ray.direction;
        attenuation *= 0.7;

    }

    return final_color;
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