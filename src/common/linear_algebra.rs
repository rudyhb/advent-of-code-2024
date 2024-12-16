use crate::common::models::NumericNeg;
use std::ops::{Add, Index, IndexMut};

#[derive(Debug, Clone)]
pub struct Vector<T: NumericNeg, const S: usize>([T; S]);
#[derive(Debug, Clone)]
pub struct Matrix<T: NumericNeg, const SX: usize, const SY: usize>([Vector<T, SX>; SY]);

pub fn solve_2x2_matrix_ax_b<T: NumericNeg>(
    matrix: &Matrix<T, 2, 2>,
    b: &Vector<T, 2>,
) -> Vector<T, 2> {
    let inverse = matrix.inverse();
    inverse.multiply_vec(b)
}

impl<T: NumericNeg, const S: usize> Vector<T, S> {
    pub fn new(value: [T; S]) -> Self {
        Self(value)
    }
    pub fn dot(&self, other: &Self) -> T {
        assert_eq!(self.0.len(), other.0.len());
        self.0
            .iter()
            .zip(other.0.iter())
            .map(|(&left, &right)| left * right)
            .sum()
    }
}

#[allow(dead_code)]
impl<T: NumericNeg, const SX: usize, const SY: usize> Matrix<T, SX, SY> {
    pub fn new(value: [Vector<T, SX>; SY]) -> Self {
        Self(value)
    }
    pub fn multiply_vec(&self, vec: &Vector<T, SX>) -> Vector<T, SY> {
        Vector::new(
            self.0
                .iter()
                .map(|row| row.dot(vec))
                .collect::<Vec<_>>()
                .try_into()
                .expect("matrix vector multiplication size mismatch"),
        )
    }
    pub fn transpose(&self) -> Matrix<T, SY, SX> {
        let mut result: Matrix<T, SY, SX> = Default::default();
        for i in 0..SX {
            for j in 0..SY {
                result[i][j] = self.0[j][i];
            }
        }
        result
    }
    pub fn scale(&mut self, value: T) {
        for i in 0..SX {
            for j in 0..SY {
                self[i][j] = self[i][j] * value;
            }
        }
    }
}

impl<T: NumericNeg> Matrix<T, 2, 2> {
    pub fn inverse(&self) -> Self {
        let mut result: Matrix<T, 2, 2> = Default::default();

        result[0][0] = self[1][1];
        result[1][1] = self[0][0];
        result[0][1] = -self[0][1];
        result[1][0] = -self[1][0];

        result.scale(self.determinant().invert());
        result
    }
    pub fn determinant(&self) -> T {
        self[0][0] * self[1][1] - self[0][1] * self[1][0]
    }
}

impl<T: NumericNeg, const S: usize> From<[T; S]> for Vector<T, S> {
    fn from(value: [T; S]) -> Self {
        Self(value)
    }
}

impl<T: NumericNeg, const SX: usize, const SY: usize> From<[[T; SX]; SY]> for Matrix<T, SX, SY> {
    fn from(value: [[T; SX]; SY]) -> Self {
        Self(
            value
                .into_iter()
                .map(Vector::from)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }
}

impl<T: NumericNeg, const S: usize> Default for Vector<T, S> {
    fn default() -> Self {
        Self::new([T::default(); S])
    }
}

impl<T: NumericNeg, const SX: usize, const SY: usize> Default for Matrix<T, SX, SY> {
    fn default() -> Self {
        [[T::default(); SX]; SY].into()
    }
}

impl<T: NumericNeg, const S: usize> Index<usize> for Vector<T, S> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T: NumericNeg, const S: usize> IndexMut<usize> for Vector<T, S> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<T: NumericNeg, const SX: usize, const SY: usize> Index<usize> for Matrix<T, SX, SY> {
    type Output = Vector<T, SX>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T: NumericNeg, const SX: usize, const SY: usize> IndexMut<usize> for Matrix<T, SX, SY> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<T: NumericNeg, const S: usize> Add for &Vector<T, S> {
    type Output = Vector<T, S>;

    fn add(self, rhs: Self) -> Self::Output {
        Vector(
            self.0
                .into_iter()
                .zip(rhs.0.into_iter())
                .map(|(a, b)| a + b)
                .collect::<Vec<_>>()
                .try_into()
                .expect("Vector length mismatch"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_creation_and_indexing() {
        let vec = Vector::new([1, 2, 3, 4]);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);
        assert_eq!(vec[3], 4);
    }

    #[test]
    fn test_vector_dot_product() {
        let vec1 = Vector::new([1, 2, 3]);
        let vec2 = Vector::new([4, 5, 6]);
        let dot = vec1.dot(&vec2);
        assert_eq!(dot, 32); // 1*4 + 2*5 + 3*6 = 32
    }

    #[test]
    fn test_vector_addition() {
        let vec1 = Vector::new([1, 2, 3]);
        let vec2 = Vector::new([4, 5, 6]);
        let result = &vec1 + &vec2;
        assert_eq!(result.0, [5, 7, 9]);
    }

    #[test]
    fn test_matrix_creation_and_indexing() {
        let mat: Matrix<i32, 2, 2> = Matrix::new([Vector::new([1, 2]), Vector::new([3, 4])]);
        assert_eq!(mat[0][0], 1);
        assert_eq!(mat[0][1], 2);
        assert_eq!(mat[1][0], 3);
        assert_eq!(mat[1][1], 4);
    }

    #[test]
    fn test_matrix_transpose() {
        let mat: Matrix<i32, 2, 3> = Matrix::new([
            Vector::new([1, 2]),
            Vector::new([3, 4]),
            Vector::new([5, 6]),
        ]);
        let transposed = mat.transpose();
        assert_eq!(transposed[0][0], 1);
        assert_eq!(transposed[1][0], 2);
        assert_eq!(transposed[0][1], 3);
        assert_eq!(transposed[1][1], 4);
        assert_eq!(transposed[0][2], 5);
        assert_eq!(transposed[1][2], 6);
    }

    #[test]
    fn test_matrix_vector_multiplication() {
        let mat: Matrix<i32, 3, 2> = Matrix::new([Vector::new([1, 2, 3]), Vector::new([4, 5, 6])]);
        let vec = Vector::new([1, 2, 3]);
        let result = mat.multiply_vec(&vec);
        assert_eq!(result.0, [14, 32]); // [1*1+2*2+3*3, 4*1+5*2+6*3]
    }

    #[test]
    fn test_matrix_determinant() {
        let mat: Matrix<i32, 2, 2> = Matrix::new([Vector::new([1, 2]), Vector::new([3, 4])]);
        assert_eq!(mat.determinant(), -2);
    }

    #[test]
    fn test_matrix_inverse() {
        let mat: Matrix<f32, 2, 2> =
            Matrix::new([Vector::new([1.0, 2.0]), Vector::new([3.0, 4.0])]);
        let det = mat.determinant();
        println!("det: {}", det);
        let mat = mat.inverse();
        assert_eq!(mat[0][0], 4.0 / det);
        assert_eq!(mat[0][1], -2.0 / det);
        assert_eq!(mat[1][0], -3.0 / det);
        assert_eq!(mat[1][1], 1.0 / det);
    }

    #[test]
    fn test_solve_matrix_ax_b() {
        let mat: Matrix<f64, 2, 2> =
            Matrix::new([Vector::new([2.0, 1.0]), Vector::new([5.0, 7.0])]);
        let b = Vector::new([11.0, 13.0]);
        let result = solve_2x2_matrix_ax_b(&mat, &b);
        // For matrix [[2, 1], [5, 7]] and b=[11, 13]
        // The solution should be x=[7.1111, -3.22222]
        assert!((result.0[0] - 7.1111).abs() < 0.001);
        assert!((result.0[1] - -3.2222).abs() < 0.001);
    }
}
