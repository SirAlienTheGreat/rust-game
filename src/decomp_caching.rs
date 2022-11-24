pub(crate) mod decomp_caching {
    use std::io::Write;
    use std::{path::Path, fs::File};
    #[cfg(feature="check-local-cache")]
    use std::io::Read;
    use bevy::prelude::Vec3;
    use bevy::utils::default;
    use serde::{Serialize, Deserialize};
    use bevy_rapier3d::rapier::prelude::SharedShape;
    use bevy_rapier3d::geometry::Collider;

    #[derive(Serialize, Deserialize, Clone)]
    pub(crate) struct RenderedDecomp {
        pub(crate) vertices: Vec<Vec3>,
        pub(crate) indices: Box<[[u32;3]]>,
        pub(crate) decomp: SharedShape
    }

    // This makes it so that you can compare 2 RenderedDecomps
    // It doesn't check the actual decomposition, just the inputs for the decomposition
    impl PartialEq for RenderedDecomp {
        fn eq(&self, other: &Self) -> bool {
            self.vertices == other.vertices && self.indices == other.indices
        }
    }


    /**
     * Checks to see if a convex collider has already been decomposed. If it has been, the existing decomposition is returned. If not, the new decomposition is calculated and returned
     */
    pub(crate) fn decompose(vertices: Vec<Vec3>, indices: Box<[[u32;3]]>, cache:&mut Vec<RenderedDecomp>) -> Option<RenderedDecomp> {


        match check_if_already_in_list(&vertices, &indices,&cache) {
            Some(item) => return Some(item),
            None => {
                println!("New item! About to decompose shape with {} ",vertices.len());
                
                let decomposition;
                //let indices = indices.clone();

                if vertices.len() < 4 || vertices.len() == 44{
                    return None
                } else if vertices.len() == 4 || check_if_flat(&vertices){
                    println!("using convex_hull on item with {} vertices",vertices.len());
                    decomposition = Collider::convex_hull(&vertices).unwrap();
                } else{
                    //let x = vertices.iter().map(|a|{return format!("Point A 1::{}::{}::{}::19465.17::13::A::0::0::0::1::0;", a.x, a.y, a.z)});
                    //println!("vertices: {:?}", x.collect::<String>());
                    decomposition = Collider::convex_decomposition_with_params(&vertices, &indices, &bevy_rapier3d::prelude::VHACDParameters { concavity:0.005, max_convex_hulls:2048*10, resolution:256, ..default() });
                }

                //let indeces = Box::new(indices);
                
                let rendered_decomp = RenderedDecomp{ vertices: vertices.to_vec(), indices:indices.into(), decomp: decomposition.raw};
                println!("finished decomposition");
                add_to_cache(rendered_decomp.clone(),cache);
                
                println!("here");
                return Some(rendered_decomp);
            },
        };
    }

    // Checks to see if an array of vertices is flat.
    // Note: this wont work if things are diagonally flat
    fn check_if_flat(verticies: &Vec<Vec3>) -> bool{
        // Iterates through each vertex and compares it to the previous one
        // if they don't match, then the shape must not be flat in that axis
        
        let mut flat_x = true;
        let mut flat_y = true;
        let mut flat_z = true;

        for i in 1..verticies.len(){
            if flat_x {
                flat_x = verticies[i].x == verticies[i-1].x
            }
            if flat_y {
                flat_y = verticies[i].y == verticies[i-1].y
            }
            if flat_z {
                flat_z = verticies[i].z == verticies[i-1].z
            }
            if !flat_x && !flat_y && !flat_z {
                return false
            }
        }
        return flat_x || flat_y || flat_z
    }

    fn add_to_cache(item:RenderedDecomp, current_cache:&mut Vec<RenderedDecomp>){
        let path = Path::new("assets/cache.bin");
        let display = path.display();

        // Add the new item to the cache
        current_cache.push(item);

        // Serialize the cache again
        let serialized = bincode::serialize(&current_cache).unwrap();

        // Write the new, serialized vector back to the cache
        let mut file_write = match File::create(&path) {
            Err(why) => panic!("couldn't open {} for writing: {}", display, why),
            Ok(file) => file,
        };

        match file_write.write_all(&serialized) {
            Ok(_) => println!("Cached new vector into {}",path.to_str().unwrap()),
            Err(_) => println!("WARNING! FAILED TO WRITE NEW CACHE TO {}",path.to_str().unwrap()),
        };

    }

    /**
     * Checks to see if the decomposition has already been done
     * If so, returns the decomposition very fast
     */
    fn check_if_already_in_list(vertices: &Vec<Vec3>, indices: &[[u32;3]], cache:&Vec<RenderedDecomp>) -> Option<RenderedDecomp> {

        for item in cache {
            if &item.vertices == vertices && &*item.indices == indices{
                println!("Found existing decomposition with {} vertexes",item.vertices.len());
                return Some(item.clone());
            }
        }

        return None
    }

    pub(crate) fn load_cache() -> Vec<RenderedDecomp>{
        // Open the cache file

        

        let embed_cache:Vec<RenderedDecomp> = bincode::deserialize(include_bytes!("../assets/cache.bin")).unwrap_or(vec![]);

        #[cfg(feature= "check-local-cache")]
        {
            let mut embed_cache = embed_cache;
            add_new_cache(&mut embed_cache);
            return embed_cache;
        }

        return embed_cache
    }

    #[cfg(feature= "check-local-cache")]
    fn add_new_cache(old_cache:&mut Vec<RenderedDecomp>){
        let mut local_cache;
        println!("checking local cache");

        let path = Path::new("assets/cache.bin");
        let display = path.display();

        // Open the path in read-only mode, returns `io::Result<File>`
        match File::open(&path) {
            Err(why) => {
                println!("couldn't open processed 3D model cache {}: {}", display, why);
                println!("All 3D meshes will now be processed (This may take a long time)");
                local_cache = vec![];
            },
            Ok(mut file) => {
                // Read the file contents into a string, returns `io::Result<usize>`
                let mut cache:Vec<u8> = vec![];
                match file.read_to_end(&mut cache) {
                    Err(why) => panic!("couldn't read {}: {}", display, why),
                    Ok(_) => {},
                }

                // Deserialize the current cache
                local_cache = bincode::deserialize(&cache).unwrap_or(vec![]);
            },
        };

        old_cache.append(&mut local_cache);
        old_cache.dedup();

    }

}