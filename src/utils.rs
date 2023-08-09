pub fn print_mat(mat: &cgmath::Matrix4<f32>){
    let mat_raw: [[f32; 4]; 4] = (*mat).into();
    println!("{}, {}, {}, {}\n{}, {}, {}, {}\n{}, {}, {}, {}\n{}, {}, {}, {}", mat_raw[0][0], mat_raw[0][1], mat_raw[0][2], mat_raw[0][3], mat_raw[1][0], mat_raw[1][1], mat_raw[1][2], mat_raw[1][3], mat_raw[2][0], mat_raw[2][1], mat_raw[2][2], mat_raw[2][3], mat_raw[3][0], mat_raw[3][1], mat_raw[3][2], mat_raw[3][3]);
}

pub fn print_mat_named(mat: &cgmath::Matrix4<f32>, name: &str){
    let mat_raw: [[f32; 4]; 4] = (*mat).into();
    println!("[{}]\n{:.2}, {:.2}, {:.2}, {:.2}\n{:.2}, {:.2}, {:.2}, {:.2}\n{:.2}, {:.2}, {:.2}, {:.2}\n{:.2}, {:.2}, {:.2}, {:.2}\n", name, mat_raw[0][0], mat_raw[0][1], mat_raw[0][2], mat_raw[0][3], mat_raw[1][0], mat_raw[1][1], mat_raw[1][2], mat_raw[1][3], mat_raw[2][0], mat_raw[2][1], mat_raw[2][2], mat_raw[2][3], mat_raw[3][0], mat_raw[3][1], mat_raw[3][2], mat_raw[3][3]);
}

/*
pub fn print_vec(vec: &cgmath::Vector3<f32>){
    let vec_raw: [f32; 3] = (*vec).into();
    println!("[{}, {}, {}]", vec_raw[0], vec)
}
*/