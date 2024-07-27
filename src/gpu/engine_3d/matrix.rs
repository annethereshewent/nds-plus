pub const UNIT_MATRIX: [[i32; 4]; 4] = [
  [0x1000,0,0,0],
  [0,0x1000,0,0],
  [0,0,0x1000,0],
  [0,0,0,0x1000]
];

#[derive(Clone, Copy, Debug)]
pub struct Matrix(pub [[i32; 4]; 4]);

impl Matrix {
  pub fn new() -> Self {
    Matrix(UNIT_MATRIX)
  }

  pub fn from(data: [[i32; 4]; 4]) -> Self {
    Matrix(data)
  }

  pub fn create_vector_position_stack() -> [Matrix; 32] {
    let mut vec = Vec::new();

    for _ in 0..32 {
      vec.push(Matrix::new());
    }

    vec.try_into().unwrap_or_else(|vec: Vec<Matrix>| panic!("expected a vector of length 32 but got a vector of length {}", vec.len()))
  }
}