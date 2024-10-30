use std::collections::HashMap;
use std::hash::Hasher;
use std::io::{self,Write, Read, Seek, SeekFrom};
use std::collections::hash_map::DefaultHasher;
use std::path::PathBuf;
use rayon::{max_num_threads, prelude::*};
use std::fs::File;

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
pub fn hash<H: Hasher>(data: [f32; 3], state: &mut H) {

    let bytes_x = data[0].to_le_bytes();
    let bytes_y = data[1].to_le_bytes();
    let bytes_z = data[2].to_le_bytes();
    // Concatenate the byte representations of x, y, and z
    let combined_bytes: Vec<u8> = [&bytes_x, &bytes_y, &bytes_z]
        .iter()
        .flat_map(|&array| array.iter().cloned())
        .collect();
    state.write(&combined_bytes);
}

pub fn hash_point(p: &[f32; 3]) -> u64 {
    let mut hasher = DefaultHasher::new();
    hash(*p, &mut hasher);
    hasher.finish()
}

#[derive(Clone, Copy)]
pub struct Face{
    pub v1: u32,
    pub v2: u32,
    pub v3: u32,
}

#[repr(C, packed)]
#[derive(Clone)]
pub struct Triangle{
    pub n: [f32; 3],
    pub v1: [f32; 3], 
    pub v2: [f32; 3],
    pub v3: [f32; 3],
    pub padding_: [u8; 2],
}

impl Default for Triangle {
    fn default() -> Self {
        Self {
            n: [0., 0., 0.],
            v1: [0., 0., 0.],
            v2: [0., 0., 0.],
            v3: [0., 0., 0.],
            padding_: [0, 0],
        }
    }
}

#[derive(Clone)]
pub struct Mesh{
    pub normals: Vec<[f32; 3]>,
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[usize; 3]>,
    pub num_faces: u32,
    pub loaded: bool,
    pub processed: bool,
}

impl Mesh{

    pub fn new(num_faces: usize) -> Self {
        let normals = vec![[0., 0., 0.]; num_faces];
        let vertices = vec![[0., 0., 0.]; num_faces * 3];
        let faces = vec![[0, 0, 0]; num_faces];
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

    pub fn write_stl_file(&self, file_path: &str) -> io::Result<()> {
        let mut file = File::create(file_path)?;

        writeln!(file, "solid ASCII_STL")?;

        for i in 0..self.faces.len() {
            let face = &self.faces[i];
            let normal = &self.normals[i];

            writeln!(
                file,
                "  facet normal {} {} {}",
                normal[0], normal[1], normal[2]
            )?;
            writeln!(file, "    outer loop")?;

            let v1 = &self.vertices[face[0]];
            let v2 = &self.vertices[face[1]];
            let v3 = &self.vertices[face[2]];

            writeln!(file, "      vertex {} {} {}", v1[0], v1[1], v1[2])?;
            writeln!(file, "      vertex {} {} {}", v2[0], v2[1], v2[2])?;
            writeln!(file, "      vertex {} {} {}", v3[0], v3[1], v3[2])?;

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

    let data_start = f.stream_position().unwrap();
    let data_end = f.seek(SeekFrom::End(0)).unwrap();
    let len_data = data_end - data_start;
    println!("{:?}", std::mem::size_of::<Triangle>());
    let expected_data_size = (std::mem::size_of::<Triangle>() as u64) * (num_triangles as u64);

    // println!("Triangle size is {:?}", std::mem::size_of::<Triangle>() as u64);

    if len_data != expected_data_size {
        eprintln!("Error: Binary STL has incorrect length in data section: {} vs {}", len_data, expected_data_size);
        return Err("Binary Data incorrect".to_string());
    }


    let mut mesh: Mesh = Mesh::new(num_triangles as usize);
    f.seek(SeekFrom::Start(data_start)).unwrap();
    
    let mut triangles: Vec<Triangle> = vec![Triangle::default(); num_triangles as usize];
    unsafe {
        f.read_exact(std::slice::from_raw_parts_mut(triangles.as_mut_ptr() as *mut u8, expected_data_size as usize)).unwrap();
    }

    let chunk_size = (num_triangles as usize + max_num_threads() - 1) / max_num_threads();

    let faces_chunks = mesh.faces.par_chunks_mut(chunk_size);
    let vertices_chunks = mesh.vertices.par_chunks_mut(3 * chunk_size);
    let normals_chunks = mesh.normals.par_chunks_mut(chunk_size);

    // Zip the chunks to process them together
    faces_chunks.zip(vertices_chunks).zip(normals_chunks)
        .enumerate()
        .for_each(|(chunk_idx, ((faces_chunk, vertices_chunk), normals_chunk))| {
            let global_offset = chunk_idx * chunk_size;

            // Calculate the actual end of the current chunk, ensuring it doesn't exceed num_triangles
            let end_idx = (global_offset + chunk_size).min(num_triangles as usize);

            for (local_idx, triangle) in triangles[global_offset..end_idx].iter().enumerate() {
                let global_idx = global_offset + local_idx;

                faces_chunk[local_idx] = [3 * global_idx, 3 * global_idx + 1, 3 * global_idx + 2];
                vertices_chunk[3 * local_idx] = [triangle.v1[0], triangle.v1[1], triangle.v1[2]];
                vertices_chunk[3 * local_idx + 1] = [triangle.v2[0], triangle.v2[1], triangle.v2[2]];
                vertices_chunk[3 * local_idx + 2] = [triangle.v3[0], triangle.v3[1], triangle.v3[2]];
                normals_chunk[local_idx] = [triangle.n[0], triangle.n[1], triangle.n[2]];
            }
        });
    mesh.loaded = true;
    Ok(mesh)

}

pub fn process_mesh(mesh: &Mesh) -> Result<Mesh, String>{

    let mut temp_mesh = Mesh::e_new();
    let mut point_map: HashMap<u64, usize> = HashMap::new();
    for i in 0..mesh.num_faces as usize {
        let face = &mesh.faces[i];
        let mut f: [usize; 3] = [0, 0, 0];
        let h1 = hash_point(&mesh.vertices[face[0]]);
        let h2 = hash_point(&mesh.vertices[face[1]]);
        let h3 = hash_point(&mesh.vertices[face[2]]);
        if let Some(index) = point_map.get(&h1){
            f[0] = *index;
        }
        else {
            point_map.insert(h1, temp_mesh.vertices.len());
            f[0] = temp_mesh.vertices.len();
            temp_mesh.vertices.push(mesh.vertices[face[0]]);
        }
        if let Some(index) = point_map.get(&h2) {
            f[1] = *index;
        }
        else {
            point_map.insert(h2, temp_mesh.vertices.len());
            f[1] = temp_mesh.vertices.len();
            temp_mesh.vertices.push(mesh.vertices[face[1]]);
        }
        if let Some(index) = point_map.get(&h3) {
            f[2] = *index;
        }
        else {
            point_map.insert(h3, temp_mesh.vertices.len());
            f[2] = temp_mesh.vertices.len();
            temp_mesh.vertices.push(mesh.vertices[face[2]]);
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
    meshes.par_chunks_mut(chunk_size).skip(start_index).for_each(|chunk|{
        for (path, mesh) in chunk.iter_mut(){
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

pub fn sort_meshes_by_num_faces(meshes: &mut [(PathBuf, Mesh)]){
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
            if !reverse {
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

#[cfg(test)]
mod tests {
    use std::{str::FromStr, time::Instant};

    use crate::utils::bvh;

    use super::*;

    #[test]
    fn mesh_loader_test(){
        let path: PathBuf = PathBuf::from_str("../../STLS/Raw/A.stl").unwrap();
        let start = Instant::now();
        let mesh: Mesh = load_mesh(&path).unwrap();
        println!("Mesh loading time: {:?} milliseconds", start.elapsed().as_millis());
        let start = Instant::now();
        let _res = bvh::create_bvh(&mesh, 10);
        //let res = mesh.write_stl_file("../1.stl");
        println!("BVH time: {:?} milliseconds", start.elapsed().as_millis());
    }
}
