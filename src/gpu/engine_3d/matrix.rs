pub const UNIT_MATRIX: [[i32; 4]; 4] = [
  [1,0,0,0],
  [0,1,0,0],
  [0,0,1,0],
  [0,0,0,1]
];

#[derive(Clone)]
pub struct Matrix {
  pub data: Vec<Vec<i32>>
}

impl Matrix {
  pub fn new() -> Self {
    Self {
      data: UNIT_MATRIX.iter().map(|row| row.to_vec()).collect()
    }
  }

  pub fn create_vector_position_stack() -> [Matrix; 32] {
    let mut vec = Vec::new();

    for _ in 0..32 {
      vec.push(Matrix::new());
    }

    vec.try_into().unwrap_or_else(|vec: Vec<Matrix>| panic!("expected a vector of length 32 but got a vector of length {}", vec.len()))
  }
}