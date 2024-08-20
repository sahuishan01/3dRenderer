const PI: f32 = 3.141592653589793;

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

struct PixelCalculator{
   cam_pos: vec3<f32>,
   view_port_center: vec3<f32>,
   size: vec2<f32>,
   x: vec3<f32>,
   y: vec3<f32>,
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

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var<uniform> camInfos: CamInfos;

@group(2) @binding(0)
var<uniform>  pixelCalculator: PixelCalculator;


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

    
    var i = (in.vert_pos.x + 1.0);
    var j = 1. - (in.vert_pos.y);

    // Extract values from camInfos
    let cam_pos = vec3<f32>(camInfos.cam_info[0].x, camInfos.cam_info[0].y, camInfos.cam_info[0].z);
    let view_port_center = vec3<f32>(camInfos.cam_info[0].w, camInfos.cam_info[1].x, camInfos.cam_info[1].y);
    let half_width = camInfos.cam_info[2].x;
    let half_height = camInfos.cam_info[2].y;
    let x = vec3<f32>(camInfos.cam_info[2].z, camInfos.cam_info[2].w, camInfos.cam_info[3].x);
    let y = vec3<f32>(camInfos.cam_info[3].y, camInfos.cam_info[3].z, camInfos.cam_info[3].w);

    // Convert i and j into screen space (-half_width to half_width)
    let u = (i - 1.) * half_width;
    let v = (j - 1.) * half_height;

    // Calculate pixel center
    let pixel_center = view_port_center + x * u + y * v;

    // Ray direction and inverse direction
    let ray_direction = normalize(pixel_center - cam_pos);
    let ray_inv = 1.0 / ray_direction;

    let ray: Ray = Ray(cam_pos, ray_direction, ray_inv);

    // Perform hit detection (similar to Rust's `hitSphere` function)
    let hit_distance = hit_sphere(ray, 5.0); // Replace with your hit detection logic

    // Determine color based on hit
    if (hit_distance > 0.0) {
        let hit_point = ray_at(ray, hit_distance);
        var t_normal = vec3<f32>(0., 0., 0.);
        t_normal = normalize(hit_point - t_normal);
        let red_shade = (angle_between_vectors(t_normal, vec3<f32>(1., 0., 0.))) / PI;
        return vec4<f32>(red_shade, 0.0, 0.0, 1.0);
    }

    // No hit: return background color (e.g., blue)
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
fn ray_at(ray: Ray, distance: f32) -> vec3<f32>{
    return ray.origin + ray.direction * distance;
}
fn angle_between_vectors(a: vec3<f32>, b: vec3<f32>) -> f32 {
    let dot_product = dot(a, b);
    let magnitudes = length(a) * length(b);
    return acos(dot_product / magnitudes);
}

fn hit_sphere(r: Ray, sphere_radius: f32) -> f32 {
    let a = dot(r.direction, r.direction);
    let b = 2.0 * dot(r.origin, r.direction);
    let c = dot(r.origin, r.origin) - sphere_radius * sphere_radius;
    let discriminant = b * b - 4.0 * a * c;

    if (discriminant < 0.0) {
        return -1.0; // No intersection
    }

    let sqrt_discriminant = sqrt(discriminant);
    let tmin = (-b - sqrt_discriminant) / (2.0 * a);
    let tmax = (-b + sqrt_discriminant) / (2.0 * a);

    if (tmin > 0.0) {
        return tmin;
    } else if (tmax > 0.0) {
        return tmax;
    } else {
        return 0.;
    }
}