use std::ops::Mul;

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

  pub fn multiply_3x3(&mut self, matrix2: Matrix) {
    let matrix1 = self.0;
    let matrix2 = matrix2.0;

    let mut result = Matrix::new();

    let result_mtx = &mut result.0;

    result_mtx[0][0] =
    ((matrix2[0][0] as i64 * matrix1[0][0] as i64 +
      matrix2[0][1] as i64 * matrix1[1][0] as i64 +
      matrix2[0][2] as i64 * matrix1[2][0] as i64) >> 12) as i32;

    result_mtx[0][1] =
    ((matrix2[0][0] as i64 * matrix1[0][1] as i64 +
      matrix2[0][1] as i64 * matrix1[1][1] as i64 +
      matrix2[0][2] as i64 * matrix1[2][1] as i64) >> 12) as i32;

    result_mtx[0][2] =
    ((matrix2[0][0] as i64 * matrix1[0][2] as i64 +
      matrix2[0][1] as i64 * matrix1[1][2] as i64 +
      matrix2[0][2] as i64 * matrix1[2][2] as i64) >> 12) as i32;

    result_mtx[0][3] =
    ((matrix2[0][0] as i64 * matrix1[0][3] as i64 +
      matrix2[0][1] as i64 * matrix1[1][3] as i64 +
      matrix2[0][2] as i64 * matrix1[2][3] as i64) >> 12) as i32;

    result_mtx[1][0] =
    ((matrix2[1][0] as i64 * matrix1[0][0] as i64 +
      matrix2[1][1] as i64 * matrix1[1][0] as i64 +
      matrix2[1][2] as i64 * matrix1[2][0] as i64) >> 12) as i32;

    result_mtx[1][1] =
    ((matrix2[1][0] as i64 * matrix1[0][1] as i64 +
      matrix2[1][1] as i64 * matrix1[1][1] as i64 +
      matrix2[1][2] as i64 * matrix1[2][1] as i64) >> 12) as i32;

    result_mtx[1][2] =
    ((matrix2[1][0] as i64 * matrix1[0][2] as i64 +
      matrix2[1][1] as i64 * matrix1[1][2] as i64 +
      matrix2[1][2] as i64 * matrix1[2][2] as i64) >> 12) as i32;

    result_mtx[1][3] =
    ((matrix2[1][0] as i64 * matrix1[0][3] as i64 +
      matrix2[1][1] as i64 * matrix1[1][3] as i64 +
      matrix2[1][2] as i64 * matrix1[2][3] as i64) >> 12) as i32;

    result_mtx[2][0] =
    ((matrix2[2][0] as i64 * matrix1[0][0] as i64 +
      matrix2[2][1] as i64 * matrix1[1][0] as i64 +
      matrix2[2][2] as i64 * matrix1[2][0] as i64) >> 12) as i32;

    result_mtx[2][1] =
    ((matrix2[2][0] as i64 * matrix1[0][1] as i64 +
      matrix2[2][1] as i64 * matrix1[1][1] as i64 +
      matrix2[2][2] as i64 * matrix1[2][1] as i64) >> 12) as i32;

    result_mtx[2][2] =
    ((matrix2[2][0] as i64 * matrix1[0][2] as i64 +
      matrix2[2][1] as i64 * matrix1[1][2] as i64 +
      matrix2[2][2] as i64 * matrix1[2][2] as i64) >> 12) as i32;

    result_mtx[2][3] =
    ((matrix2[2][0] as i64 * matrix1[0][3] as i64 +
      matrix2[2][1] as i64 * matrix1[1][3] as i64 +
      matrix2[2][2] as i64 * matrix1[2][3] as i64) >> 12) as i32;


    self.0 = *result_mtx;
  }

  pub fn multiply_4x3(&mut self, matrix2: Matrix) {
    let matrix1 = self.0;
    let matrix2 = matrix2.0;


    let mut result = Matrix::new();

    let result_mtx = &mut result.0;

    result_mtx[0][0] =
    ((matrix2[0][0] as i64 * matrix1[0][0] as i64 +
      matrix2[0][1] as i64 * matrix1[1][0] as i64 +
      matrix2[0][2] as i64 * matrix1[2][0] as i64) >> 12) as i32;

    result_mtx[0][1] =
    ((matrix2[0][0] as i64 * matrix1[0][1] as i64 +
      matrix2[0][1] as i64 * matrix1[1][1] as i64 +
      matrix2[0][2] as i64 * matrix1[2][1] as i64) >> 12) as i32;

    result_mtx[0][2] =
    ((matrix2[0][0] as i64 * matrix1[0][2] as i64 +
      matrix2[0][1] as i64 * matrix1[1][2] as i64 +
      matrix2[0][2] as i64 * matrix1[2][2] as i64) >> 12) as i32;

    result_mtx[0][3] =
    ((matrix2[0][0] as i64 * matrix1[0][3] as i64 +
      matrix2[0][1] as i64 * matrix1[1][3] as i64 +
      matrix2[0][2] as i64 * matrix1[2][3] as i64) >> 12) as i32;

    result_mtx[1][0] =
    ((matrix2[1][0] as i64 * matrix1[0][0] as i64 +
      matrix2[1][1] as i64 * matrix1[1][0] as i64 +
      matrix2[1][2] as i64 * matrix1[2][0] as i64) >> 12) as i32;

    result_mtx[1][1] =
    ((matrix2[1][0] as i64 * matrix1[0][1] as i64 +
      matrix2[1][1] as i64 * matrix1[1][1] as i64 +
      matrix2[1][2] as i64 * matrix1[2][1] as i64) >> 12) as i32;

    result_mtx[1][2] =
    ((matrix2[1][0] as i64 * matrix1[0][2] as i64 +
      matrix2[1][1] as i64 * matrix1[1][2] as i64 +
      matrix2[1][2] as i64 * matrix1[2][2] as i64) >> 12) as i32;

    result_mtx[1][3] =
    ((matrix2[1][0] as i64 * matrix1[0][3] as i64 +
      matrix2[1][1] as i64 * matrix1[1][3] as i64 +
      matrix2[1][2] as i64 * matrix1[2][3] as i64) >> 12) as i32;

    result_mtx[2][0] =
    ((matrix2[2][0] as i64 * matrix1[0][0] as i64 +
      matrix2[2][1] as i64 * matrix1[1][0] as i64 +
      matrix2[2][2] as i64 * matrix1[2][0] as i64) >> 12) as i32;

    result_mtx[2][1] =
    ((matrix2[2][0] as i64 * matrix1[0][1] as i64 +
      matrix2[2][1] as i64 * matrix1[1][1] as i64 +
      matrix2[2][2] as i64 * matrix1[2][1] as i64) >> 12) as i32;

    result_mtx[2][2] =
    ((matrix2[2][0] as i64 * matrix1[0][2] as i64 +
      matrix2[2][1] as i64 * matrix1[1][2] as i64 +
      matrix2[2][2] as i64 * matrix1[2][2] as i64) >> 12) as i32;

    result_mtx[2][3] =
    ((matrix2[2][0] as i64 * matrix1[0][3] as i64 +
      matrix2[2][1] as i64 * matrix1[1][3] as i64 +
      matrix2[2][2] as i64 * matrix1[2][3] as i64) >> 12) as i32;

    result_mtx[3][0] =
    ((matrix2[3][0] as i64 * matrix1[0][0] as i64 +
      matrix2[3][1] as i64 * matrix1[1][0] as i64 +
      matrix2[3][2] as i64 * matrix1[2][0] as i64 +
      0x1000 * matrix1[3][0] as i64) >> 12) as i32;

    result_mtx[3][1] =
    ((matrix2[3][0] as i64 * matrix1[0][1] as i64 +
      matrix2[3][1] as i64 * matrix1[1][1] as i64 +
      matrix2[3][2] as i64 * matrix1[2][1] as i64 +
      0x1000 * matrix1[3][1] as i64) >> 12) as i32;

    result_mtx[3][2] =
    ((matrix2[3][0] as i64 * matrix1[0][2] as i64 +
      matrix2[3][1] as i64 * matrix1[1][2] as i64 +
      matrix2[3][2] as i64 * matrix1[2][2] as i64 +
      0x1000 * matrix1[3][2] as i64) >> 12) as i32;

    result_mtx[3][3] =
    ((matrix2[3][0] as i64 * matrix1[0][3] as i64 +
      matrix2[3][1] as i64 * matrix1[1][3] as i64 +
      matrix2[3][2] as i64 * matrix1[2][3] as i64 +
      0x1000 * matrix1[3][3] as i64) >> 12) as i32;

    self.0 = *result_mtx;
  }

  pub fn multiply_row(&self, row: &[i32], shift: i32) -> [i32; 4] {
    let matrix = self.0;

    let cell0 =
    ((row[0] as i64 * matrix[0][0] as i64 +
      row[1] as i64 * matrix[1][0] as i64 +
      row[2] as i64 * matrix[2][0] as i64 +
      row[3] as i64 * matrix[3][0] as i64) >> shift) as i32;
    let cell1 =
    ((row[0] as i64 * matrix[0][1] as i64 +
      row[1] as i64 * matrix[1][1] as i64 +
      row[2] as i64 * matrix[2][1] as i64 +
      row[3] as i64 * matrix[3][1] as i64) >> shift) as i32;
    let cell2 =
    ((row[0] as i64 * matrix[0][2] as i64 +
      row[1] as i64 * matrix[1][2] as i64 +
      row[2] as i64 * matrix[2][2] as i64 +
      row[3] as i64 * matrix[3][2] as i64) >> shift) as i32;

    let cell3 =
    ((row[0] as i64 * matrix[0][3] as i64 +
      row[1] as i64 * matrix[1][3] as i64 +
      row[2] as i64 * matrix[2][3] as i64 +
      row[3] as i64 * matrix[3][3] as i64) >> shift) as i32;

    [cell0, cell1, cell2, cell3]
  }

  pub fn translate(&mut self, row: &[i32]) {
    let matrix = &mut self.0;

    matrix[3][0] +=
    ((row[0] as i64 * matrix[0][0] as i64 +
      row[1] as i64 * matrix[1][0] as i64 +
      row[2] as i64 * matrix[2][0] as i64) >> 12) as i32;
    matrix[3][1] +=
    ((row[0] as i64 * matrix[0][1] as i64 +
      row[1] as i64 * matrix[1][1] as i64 +
      row[2] as i64 * matrix[2][1] as i64) >> 12) as i32;
    matrix[3][2] +=
    ((row[0] as i64 * matrix[0][2] as i64 +
      row[1] as i64 * matrix[1][2] as i64 +
      row[2] as i64 * matrix[2][2] as i64) >> 12) as i32;
    matrix[3][3] +=
    ((row[0] as i64 * matrix[0][3] as i64 +
      row[1] as i64 * matrix[1][3] as i64 +
      row[2] as i64 * matrix[2][3] as i64) >> 12) as i32;
  }

  pub fn scale(&mut self, vector: &[i32]) {
    let matrix = &mut self.0;

    matrix[0][0] = ((vector[0] as i64 * matrix[0][0] as i64) >> 12) as i32;
    matrix[0][1] = ((vector[0] as i64 * matrix[0][1] as i64) >> 12) as i32;
    matrix[0][2] = ((vector[0] as i64 * matrix[0][2] as i64) >> 12) as i32;
    matrix[0][3] = ((vector[0] as i64 * matrix[0][3] as i64) >> 12) as i32;

    matrix[1][0] = ((vector[1] as i64 * matrix[1][0] as i64) >> 12) as i32;
    matrix[1][1] = ((vector[1] as i64 * matrix[1][1] as i64) >> 12) as i32;
    matrix[1][2] = ((vector[1] as i64 * matrix[1][2] as i64) >> 12) as i32;
    matrix[1][3] = ((vector[1] as i64 * matrix[1][3] as i64) >> 12) as i32;

    matrix[2][0] = ((vector[2] as i64 * matrix[2][0] as i64) >> 12) as i32;
    matrix[2][1] = ((vector[2] as i64 * matrix[2][1] as i64) >> 12) as i32;
    matrix[2][2] = ((vector[2] as i64 * matrix[2][2] as i64) >> 12) as i32;
    matrix[2][3] = ((vector[2] as i64 * matrix[2][3] as i64) >> 12) as i32;
  }
}

impl Mul for Matrix {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self::Output {
    let matrix1 = self.0;
    let matrix2 = rhs.0;

    let mut result = Matrix::new();

    let result_mtx = &mut result.0;

    result_mtx[0][0] =
    ((matrix1[0][0] as i64 * matrix2[0][0] as i64 +
      matrix1[0][1] as i64 * matrix2[1][0] as i64 +
      matrix1[0][2] as i64 * matrix2[2][0] as i64 +
      matrix1[0][3] as i64 * matrix2[3][0] as i64) >> 12) as i32;
    result_mtx[1][0] =
    ((matrix1[1][0] as i64 * matrix2[0][0] as i64 +
      matrix1[1][1] as i64 * matrix2[1][0] as i64 +
      matrix1[1][2] as i64 * matrix2[2][0] as i64 +
      matrix1[1][3] as i64 * matrix2[3][0] as i64) >> 12) as i32;
    result_mtx[2][0] =
    ((matrix1[2][0] as i64 * matrix2[0][0] as i64 +
      matrix1[2][1] as i64 * matrix2[1][0] as i64 +
      matrix1[2][2] as i64 * matrix2[2][0] as i64 +
      matrix1[2][3] as i64 * matrix2[3][0] as i64) >> 12) as i32;
    result_mtx[3][0] =
    ((matrix1[3][0] as i64 * matrix2[0][0] as i64 +
      matrix1[3][1] as i64 * matrix2[1][0] as i64 +
      matrix1[3][2] as i64 * matrix2[2][0] as i64 +
      matrix1[3][3] as i64 * matrix2[3][0] as i64) >> 12) as i32;

    result_mtx[0][1] =
    ((matrix1[0][0] as i64 * matrix2[0][1] as i64 +
      matrix1[0][1] as i64 * matrix2[1][1] as i64 +
      matrix1[0][2] as i64 * matrix2[2][1] as i64 +
      matrix1[0][3] as i64 * matrix2[3][1] as i64) >> 12) as i32;
    result_mtx[1][1] =
    ((matrix1[1][0] as i64 * matrix2[0][1] as i64 +
      matrix1[1][1] as i64 * matrix2[1][1] as i64 +
      matrix1[1][2] as i64 * matrix2[2][1] as i64 +
      matrix1[1][3] as i64 * matrix2[3][1] as i64) >> 12) as i32;
    result_mtx[2][1] =
    ((matrix1[2][0] as i64 * matrix2[0][1] as i64 +
      matrix1[2][1] as i64 * matrix2[1][1] as i64 +
      matrix1[2][2] as i64 * matrix2[2][1] as i64 +
      matrix1[2][3] as i64 * matrix2[3][1] as i64) >> 12) as i32;
    result_mtx[3][1] =
    ((matrix1[3][0] as i64 * matrix2[0][1] as i64 +
      matrix1[3][1] as i64 * matrix2[1][1] as i64 +
      matrix1[3][2] as i64 * matrix2[2][1] as i64 +
      matrix1[3][3] as i64 * matrix2[3][1] as i64) >> 12) as i32;

    result_mtx[0][2] =
    ((matrix1[0][0] as i64 * matrix2[0][2] as i64 +
      matrix1[0][1] as i64 * matrix2[1][2] as i64 +
      matrix1[0][2] as i64 * matrix2[2][2] as i64 +
      matrix1[0][3] as i64 * matrix2[3][2] as i64) >> 12) as i32;
    result_mtx[1][2] =
    ((matrix1[1][0] as i64 * matrix2[0][2] as i64 +
      matrix1[1][1] as i64 * matrix2[1][2] as i64 +
      matrix1[1][2] as i64 * matrix2[2][2] as i64 +
      matrix1[1][3] as i64 * matrix2[3][2] as i64) >> 12) as i32;
    result_mtx[2][2] =
    ((matrix1[2][0] as i64 * matrix2[0][2] as i64 +
      matrix1[2][1] as i64 * matrix2[1][2] as i64 +
      matrix1[2][2] as i64 * matrix2[2][2] as i64 +
      matrix1[2][3] as i64 * matrix2[3][2] as i64) >> 12) as i32;
    result_mtx[3][2] =
    ((matrix1[3][0] as i64 * matrix2[0][2] as i64 +
      matrix1[3][1] as i64 * matrix2[1][2] as i64 +
      matrix1[3][2] as i64 * matrix2[2][2] as i64 +
      matrix1[3][3] as i64 * matrix2[3][2] as i64) >> 12) as i32;


    result_mtx[0][3] =
    ((matrix1[0][0] as i64 * matrix2[0][3] as i64 +
      matrix1[0][1] as i64 * matrix2[1][3] as i64 +
      matrix1[0][2] as i64 * matrix2[2][3] as i64 +
      matrix1[0][3] as i64 * matrix2[3][3] as i64) >> 12) as i32;
    result_mtx[1][3] =
    ((matrix1[1][0] as i64 * matrix2[0][3] as i64 +
      matrix1[1][1] as i64 * matrix2[1][3] as i64 +
      matrix1[1][2] as i64 * matrix2[2][3] as i64 +
      matrix1[1][3] as i64 * matrix2[3][3] as i64) >> 12) as i32;
    result_mtx[2][3] =
    ((matrix1[2][0] as i64 * matrix2[0][3] as i64 +
      matrix1[2][1] as i64 * matrix2[1][3] as i64 +
      matrix1[2][2] as i64 * matrix2[2][3] as i64 +
      matrix1[2][3] as i64 * matrix2[3][3] as i64) >> 12) as i32;
    result_mtx[3][3] =
    ((matrix1[3][0] as i64 * matrix2[0][3] as i64 +
      matrix1[3][1] as i64 * matrix2[1][3] as i64 +
      matrix1[3][2] as i64 * matrix2[2][3] as i64 +
      matrix1[3][3] as i64 * matrix2[3][3] as i64) >> 12) as i32;

    result
  }
}