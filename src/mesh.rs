use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{self,Write, Read, Seek, SeekFrom};
use std::collections::hash_map::DefaultHasher;
use std::path::PathBuf;
use rayon::{max_num_threads, prelude::*};
use std::fs::File;
use crate::Vec3;

impl Hash for Vec3<f32>{
    // pub fn hash<H: Hasher>(&self, state: &mut H) {
    //     let mut hasher = DefaultHasher::new();
    //
    //     // Hash each component individually
    //     hasher.write_u32(self.v[0].to_bits());
    //     state.write_u64(hasher.finish());
    //
    //     hasher = DefaultHasher::new();
    //     hasher.write_u32(self.v[1].to_bits());
    //     state.write_u64(hasher.finish());
    //
    //     hasher = DefaultHasher::new();
    //     hasher.write_u32(self.v[2].to_bits());
    //     state.write_u64(hasher.finish());
    // }
    fn hash<H: Hasher>(&self, state: &mut H) {

        let bytes_x = self.v[0].to_le_bytes();
        let bytes_y = self.v[1].to_le_bytes();
        let bytes_z = self.v[2].to_le_bytes();

        // Concatenate the byte representations of x, y, and z
        let combined_bytes: Vec<u8> = [&bytes_x, &bytes_y, &bytes_z]
            .iter()
            .flat_map(|&array| array.iter().cloned())
            .collect();

        state.write(&combined_bytes);
    }
}
pub fn hash_point(p: &Vec3<f32>) -> u64 {
    let mut hasher = DefaultHasher::new();
    p.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, Copy)]
pub struct Face{
    pub v1: u32,
    pub v2: u32,
    pub v3: u32,
}
impl Face{
    pub fn new() -> Self{
        Self{
            v1: u32::MAX,
            v2: u32::MAX,
            v3: u32::MAX
        }
    }
}


#[repr(C, packed)]
pub struct Triangle{
    pub normal: Vec3<f32>,
    pub vertices: [Vec3<f32>; 3],
    pub padding: [u8; 2],
}
impl Triangle{
    pub fn new() -> Self {
        Self{
            normal: Vec3::new(0., 0., 0.),
            vertices: [Vec3::new(0., 0., 0.), Vec3::new(0., 0., 0.), Vec3::new(0., 0., 0.)],
            padding: [0, 0],
        }
    }
}


#[derive(Clone)]
pub struct Triangle2{
    pub normal: Vec3<f32>,
    pub vertices: [Vec3<f32>; 3],
    pub padding: [u8; 2],
}

impl Triangle2{
    pub fn new() -> Self {
        Self{
            normal: Vec3::new(0., 0., 0.),
            vertices: [Vec3::new(0., 0., 0.), Vec3::new(0., 0., 0.), Vec3::new(0., 0., 0.)],
            padding: [0, 0],
        }
    }
}

#[derive(Clone)]
pub struct Mesh{
    pub normals: Vec<Vec3<f32>>,
    pub vertices: Vec<Vec3<f32>>,
    pub faces: Vec<Face>,
    pub num_faces: u32,
    pub loaded: bool,
    pub processed: bool,
}

impl Mesh{
    pub fn new(num_faces: usize) -> Self {
        let normals: Vec<Vec3<f32>> = vec![Vec3::new(0., 0., 0.); num_faces];
        let vertices: Vec<Vec3<f32>> = vec![Vec3::new(0., 0., 0.); num_faces*3];
        let faces: Vec<Face> = vec![Face::new(); num_faces];
        let num_faces = num_faces as u32;
        Self {normals, vertices, faces, num_faces, loaded: false, processed: false}
    }
    pub fn e_new() -> Self {
        Self {
            normals: vec![],
            vertices: vec![],
            faces: vec![],
            num_faces: 0,
            loaded: false,
            processed: false
        }
    }
    pub fn len(&self) -> usize {
        self.faces.len()
    }

    pub fn write_stl_file(&self, file_path: &str) -> io::Result<()> {
        let mut file = File::create(file_path)?;

        writeln!(file, "solid ASCII_STL")?;

        for i in 0..self.faces.len() {
            let face = &self.faces[i];
            let normal = &self.normals[i];

            writeln!(
                file,
                "  facet normal {} {} {}",
                normal.v[0], normal.v[1], normal.v[2]
            )?;
            writeln!(file, "    outer loop")?;

            let v1 = &self.vertices[face.v1 as usize];
            let v2 = &self.vertices[face.v2 as usize];
            let v3 = &self.vertices[face.v3 as usize];

            writeln!(file, "      vertex {} {} {}", v1.v[0], v1.v[1], v1.v[2])?;
            writeln!(file, "      vertex {} {} {}", v2.v[0], v2.v[1], v2.v[2])?;
            writeln!(file, "      vertex {} {} {}", v3.v[0], v3.v[1], v3.v[2])?;

            writeln!(file, "    endloop")?;
            writeln!(file, "  endfacet")?;
        }

        writeln!(file, "endsolid ASCII_STL")?;

        println!("saved {}", file_path);
        Ok(())
    }

}

pub fn load_mesh(path: &PathBuf) -> Result<Mesh, String>{

    // println!("Processing path {:?}", std::path::Path::new(path).file_name().ok_or_else(|| "Invalid file name").unwrap());

    let mut f = File::open(path).unwrap();

    f.seek(SeekFrom::Start(80)).unwrap();

    let mut num_triangles: u32 = 0;
    f.read_exact(unsafe { std::mem::transmute::<&mut u32, &mut [u8; 4]>(&mut num_triangles) }).unwrap();

    let data_start = f.seek(SeekFrom::Current(0)).unwrap();
    let data_end = f.seek(SeekFrom::End(0)).unwrap();
    let len_data = data_end - data_start;
    let expected_data_size = (std::mem::size_of::<Triangle>() as u64) * (num_triangles as u64);

    // println!("Triangle size is {:?}", std::mem::size_of::<Triangle>() as u64);

    if len_data != expected_data_size {
        eprintln!("Error: Binary STL has incorrect length in data section: {} vs {}", len_data, expected_data_size);
        return Err("Binary Data incorrect".to_string());
    }


    f.seek(SeekFrom::Start(data_start)).unwrap();
    let mut triangles: Vec<Triangle> = Vec::with_capacity(num_triangles as usize);

    unsafe {
        triangles.set_len(num_triangles as usize);
        f.read_exact(std::slice::from_raw_parts_mut(triangles.as_mut_ptr() as *mut u8, expected_data_size as usize)).unwrap();
    }
    let mut i = 0;
    let mut f_count =  0;
    let mut mesh: Mesh = Mesh::new(num_triangles as usize);
    while i < triangles.len() {
        mesh.normals[i] = Vec3::new(triangles[i].normal.v[0], triangles[i].normal.v[1], triangles[i].normal.v[2]);
        mesh.vertices[i*3] = Vec3::new(triangles[i].vertices[0].v[0], triangles[i].vertices[0].v[1], triangles[i].vertices[0].v[2]);
        mesh.vertices[i*3+1] = Vec3::new(triangles[i].vertices[1].v[0], triangles[i].vertices[1].v[1], triangles[i].vertices[1].v[2]);
        mesh.vertices[i*3+2] = Vec3::new(triangles[i].vertices[2].v[0], triangles[i].vertices[2].v[1], triangles[i].vertices[2].v[2]);
        mesh.faces[i] = Face{
            v1: f_count,
            v2: f_count + 1,
            v3: f_count + 2,
        };
        f_count += 3;
        i+=1;
    }
    // let mut mesh: Mesh = Mesh::e_new();
    // while i < triangles.len(){
    //     mesh.normals.push(triangles[i].normal);
    //     mesh.vertices.extend_from_slice(&triangles[i].vertices);
    //     mesh.faces.push(
    //         Face{
    //             v1: f_count,
    //             v2: f_count + 1,
    //             v3: f_count + 2,
    //         }
    //     );
    //     f_count += 3;
    //     i+=1;
    // }
    mesh.loaded = true;
    Ok(mesh)

}

pub fn process_mesh(mesh: &Mesh) -> Result<Mesh, String>{

    let mut temp_mesh = Mesh::e_new();
    let mut point_map: HashMap<u64, usize> = HashMap::new();
    for i in 0..mesh.faces.len(){
        let face = &mesh.faces[i];
        temp_mesh.normals.push(Vec3::new(mesh.normals[i].v[0], mesh.normals[i].v[1], mesh.normals[i].v[2]));
        let mut f = Face::new();
        let h1 = hash_point(&mesh.vertices[face.v1 as usize]);
        let h2 = hash_point(&mesh.vertices[face.v2 as usize]);
        let h3 = hash_point(&mesh.vertices[face.v3 as usize]);
        if let Some(index) = point_map.get(&h1) {
            f.v1 = index.clone() as u32;
        }
        else{
            point_map.insert(h1, temp_mesh.vertices.len());
            f.v1 = temp_mesh.vertices.len() as u32;
            temp_mesh.vertices.push(Vec3::new(mesh.vertices[face.v1 as usize].v[0], mesh.vertices[face.v1 as usize].v[1], mesh.vertices[face.v1 as usize].v[2]));
        }
        if let Some(index) = point_map.get(&h2) {
            f.v2 = index.clone() as u32;
        }
        else{
            point_map.insert(h2, temp_mesh.vertices.len());
            f.v2 = temp_mesh.vertices.len() as u32;
            temp_mesh.vertices.push(Vec3::new(mesh.vertices[face.v2 as usize].v[0], mesh.vertices[face.v2 as usize].v[1], mesh.vertices[face.v2 as usize].v[2]));
        }
        if let Some(index) = point_map.get(&h3) {
            f.v3 = index.clone() as u32;
        }
        else{
            point_map.insert(h3, temp_mesh.vertices.len());
            f.v3 = temp_mesh.vertices.len() as u32;
            temp_mesh.vertices.push(Vec3::new(mesh.vertices[face.v3 as usize].v[0], mesh.vertices[face.v3 as usize].v[1], mesh.vertices[face.v3 as usize].v[2]));
        }
        temp_mesh.faces.push(f);
    }
    temp_mesh.loaded = mesh.loaded;
    temp_mesh.num_faces = mesh.num_faces;
    temp_mesh.processed = mesh.processed;
    Ok(temp_mesh)
}

pub fn add_meshes(meshes: &mut Vec<(PathBuf, Mesh)>,  paths: Vec<PathBuf>) {

    let start_time = std::time::Instant::now();

    let start_index = meshes.len();

    meshes.extend((0..paths.len()).map(|i| (paths[i].clone(), Mesh::e_new())));

    // for multithreading

    // chunks for threads
    let chunk_size = (meshes.len() + max_num_threads() - 1) / max_num_threads();
    meshes.par_chunks_mut(chunk_size).enumerate().skip(start_index).for_each(|(_chunk_idx, chunk)|{
        for (_index, (path, mesh)) in chunk.iter_mut().enumerate(){
            // let global_index = chunk_idx * chunk_size + index + start_index;
            let mut result = match load_mesh(path) {
                Ok(m) => {
                    println!("Added successfully {:?}", path.file_name().unwrap());
                    m
                }
                Err(e) => {
                    println!(
                        "Failed to add {:?}, Error {:?}",
                        path.file_name().unwrap(),
                        e
                    );
                    Mesh::e_new()
                }
            };
            result.loaded = true;
            *mesh = result;

        }
    });

    // all on separate threads
    // meshes.par_iter_mut().enumerate().skip(start_index).for_each(|(index, (path, mesh))| {
    //     let mut result = match load_mesh(path) {
    //         Ok(m) => {
    //             println!("Added successfully {:?}", path.file_name().unwrap());
    //             m
    //         }
    //         Err(e) => {
    //             println!("Failed to add {:?}, Error {e:?}", path.file_name().unwrap());
    //             Mesh::e_new()
    //         },
    //     };
    //     result.loaded = true;
    //     *mesh = result;
    // });

    // for loading one by one

    // let mut i = 0;
    // while i < paths.len(){
    //     let mesh = match load_mesh(&paths[i]){
    //         Ok(mesh) => {
    //             println!("Added successfully {:?}", i+start_index);
    //             mesh
    //         },
    //         Err(e) => {
    //             println!("Failed to add {:?}, Error {e:?}", i+start_index);
    //             Mesh::e_new()
    //         },
    //     };
    //     meshes[i + start_index] = (paths[i].clone(), mesh);
    //     i += 1;
    // }
    println!("Total time for loading meshes is {} microseconds", start_time.elapsed().as_micros());
}

pub fn process_meshes(meshes: &mut Vec<(PathBuf, Mesh)>, start_idx: usize){

    let start_time = std::time::Instant::now();

    // processing with chunks
    let meshes_per_thread = meshes.len() / max_num_threads();
    let remaining_meshes = meshes.len() % max_num_threads();
    if meshes_per_thread == 0 {
        //processing individually on separate threads
        meshes.par_iter_mut().enumerate().skip(start_idx).for_each(|(_index, (path, mesh))| {
            let mut result = match process_mesh(mesh) {
                Ok(m) => {
                    println!("Processed successfully for {:?}", path.file_name().unwrap());
                    m
                }
                Err(e) => {
                    println!("Failed to process {:?}, Error {e:?}", path.file_name().unwrap());
                    Mesh::e_new()
                },
            };
            result.processed = true;
            *mesh = result;
        });
    }
    else {
        meshes
            .par_chunks_mut(meshes_per_thread)
            .enumerate()
            .for_each(|(thread_idx, chunk)| {
                let additional_meshes = if thread_idx < remaining_meshes {
                    1
                } else {
                    0
                };

                for (_index, (path, mesh)) in chunk.iter_mut().enumerate().take(meshes_per_thread + additional_meshes) {
                    let mut result = match process_mesh(mesh) {
                        Ok(m) => {
                            println!("Processed successfully for {:?}", path.file_name().unwrap());
                            m
                        }
                        Err(e) => {
                            println!("Failed to process {:?}, Error {:?}", path.file_name().unwrap(), e);
                            Mesh::e_new()
                        },
                    };
                    result.processed = true;
                    *mesh = result;
                }
            });
    }

    println!("Total time for processing mesh is {} microseconds", start_time.elapsed().as_micros());


}

pub fn sort_meshes_by_num_faces(meshes: &mut Vec<(PathBuf, Mesh)>){
    let swap_start = std::time::Instant::now();
    meshes.sort_by(|a,b| b.1.num_faces.cmp(&a.1.num_faces));
    let mut indices: Vec<Vec<usize>> = vec![Vec::new(); max_num_threads()];
    let mut i: usize = 0;
    let chunk_size = meshes.len()/ max_num_threads();
    let extra_files = meshes.len() % max_num_threads();
    while i  < max_num_threads() {
        let mut j: usize = 0;
        let mut reverse = false;
        while j < chunk_size{
            if reverse == false{
                indices[i].push(i + (j * max_num_threads()));
            }
            else{
                indices[i].push(((j+1) * max_num_threads()) - i - 1);
            }
            reverse = !reverse;
            j+=1;
        }
        i +=1;
    }
    let mut j = 0;
    println!("extra files: {}, last index is: {}", extra_files, max_num_threads() * chunk_size + j);
    while j < extra_files{
        indices[j].push(max_num_threads()  * chunk_size + j);
        j+=1;
    }

    let mut rearranged_meshes = vec![(PathBuf::new(), Mesh::e_new()); meshes.len()];
    let mut rearranged_index = 0;
    for index_list in indices.iter() {
        for &original_index in index_list.iter() {
            if original_index < meshes.len() {
                // Use mem::replace or mem::swap to avoid move errors
                let _ = std::mem::replace(&mut rearranged_meshes[rearranged_index], std::mem::replace(&mut meshes[original_index], (PathBuf::new(), Mesh::e_new())));
                rearranged_index += 1;
            }
        }
    }
    meshes.swap_with_slice(&mut rearranged_meshes);
    println!("Swap time is: {} micro seconds", swap_start.elapsed().as_micros());
}