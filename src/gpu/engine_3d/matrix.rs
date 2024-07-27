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

  pub fn multiply_row(&self, row: &[i32], shift: i32) -> [i32; 4] {
    let matrix = self.0;
    [
      (row[0] * matrix[0][0] + row[1] * matrix[1][0] + row[2] * matrix[2][0] + row[3] * matrix[3][0]) >> shift,
      (row[0] * matrix[0][1] + row[1] * matrix[1][1] + row[2] * matrix[2][1] + row[3] * matrix[3][1]) >> shift,
      (row[0] * matrix[0][2] + row[1] * matrix[1][2] + row[2] * matrix[2][2] + row[3] * matrix[3][2]) >> shift,
      (row[0] * matrix[0][3] + row[1] * matrix[1][3] + row[2] * matrix[2][3] + row[3] * matrix[3][3]) >> shift,
    ]
  }
}