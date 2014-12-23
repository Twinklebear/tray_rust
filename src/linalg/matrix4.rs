/// Matrix4 is a 4x4 matrix stored in row-major format
#[deriving(Show, PartialEq, Copy)]
pub struct Matrix4 {
    mat: [f32, ..16],
}

impl Matrix4 {
    /// Return the zero matrix
    pub fn zero() -> Matrix4 {
        Matrix4 { mat: [0f32, ..16] }
    }
    /// Return the identity matrix
    pub fn identity() -> Matrix4 {
        Matrix4 { mat:
            [1f32, 0f32, 0f32, 0f32,
             0f32, 1f32, 0f32, 0f32,
             0f32, 0f32, 1f32, 0f32,
             0f32, 0f32, 0f32, 1f32]
        }
    }
    /// Create the matrix using the values passed
    pub fn new(mat: [f32, ..16]) -> Matrix4 {
        Matrix4 { mat: mat }
    }
    /// Access the element at row `i` column `j`
    pub fn at(&self, i: uint, j: uint) -> &f32 {
        &self.mat[4 * i + j]
    }
    /// Mutably access the element at row `i` column `j`
    pub fn at_mut(&mut self, i: uint, j: uint) -> &mut f32 {
        &mut self.mat[4 * i + j]
    }
    /// Compute and return the transpose of this matrix
    pub fn transpose(&self) -> Matrix4 {
        let mut res = Matrix4::zero();
        for i in range(0u, 4u) {
            for j in range(0u, 4u) {
                *res.at_mut(i, j) = *self.at(j, i);
            }
        }
        res
    }
    /// Compute and return the inverse of this matrix
    pub fn inverse(&self) -> Matrix4 {
        //MESA's matrix inverse, tweaked for row-major matrices
        let mut inv = Matrix4::zero();
        inv.mat[0] = self.mat[5] * self.mat[10] * self.mat[15]
            - self.mat[5]  * self.mat[11] * self.mat[14]
            - self.mat[9]  * self.mat[6]  * self.mat[15]
            + self.mat[9]  * self.mat[7]  * self.mat[14]
            + self.mat[13] * self.mat[6]  * self.mat[11]
            - self.mat[13] * self.mat[7]  * self.mat[10];

        inv.mat[4] = -self.mat[4]  * self.mat[10] * self.mat[15]
            + self.mat[4]  * self.mat[11] * self.mat[14]
            + self.mat[8]  * self.mat[6]  * self.mat[15]
            - self.mat[8]  * self.mat[7]  * self.mat[14]
            - self.mat[12] * self.mat[6]  * self.mat[11]
            + self.mat[12] * self.mat[7]  * self.mat[10];

        inv.mat[8] = self.mat[4]  * self.mat[9] * self.mat[15]
            - self.mat[4]  * self.mat[11] * self.mat[13]
            - self.mat[8]  * self.mat[5] * self.mat[15]
            + self.mat[8]  * self.mat[7] * self.mat[13]
            + self.mat[12] * self.mat[5] * self.mat[11]
            - self.mat[12] * self.mat[7] * self.mat[9];

        inv.mat[12] = -self.mat[4]  * self.mat[9] * self.mat[14]
            + self.mat[4]  * self.mat[10] * self.mat[13]
            + self.mat[8]  * self.mat[5] * self.mat[14]
            - self.mat[8]  * self.mat[6] * self.mat[13]
            - self.mat[12] * self.mat[5] * self.mat[10]
            + self.mat[12] * self.mat[6] * self.mat[9];

        inv.mat[1] = -self.mat[1]  * self.mat[10] * self.mat[15]
            + self.mat[1]  * self.mat[11] * self.mat[14]
            + self.mat[9]  * self.mat[2] * self.mat[15]
            - self.mat[9]  * self.mat[3] * self.mat[14]
            - self.mat[13] * self.mat[2] * self.mat[11]
            + self.mat[13] * self.mat[3] * self.mat[10];

        inv.mat[5] = self.mat[0]  * self.mat[10] * self.mat[15]
            - self.mat[0]  * self.mat[11] * self.mat[14]
            - self.mat[8]  * self.mat[2] * self.mat[15]
            + self.mat[8]  * self.mat[3] * self.mat[14]
            + self.mat[12] * self.mat[2] * self.mat[11]
            - self.mat[12] * self.mat[3] * self.mat[10];

        inv.mat[9] = -self.mat[0]  * self.mat[9] * self.mat[15]
            + self.mat[0]  * self.mat[11] * self.mat[13]
            + self.mat[8]  * self.mat[1] * self.mat[15]
            - self.mat[8]  * self.mat[3] * self.mat[13]
            - self.mat[12] * self.mat[1] * self.mat[11]
            + self.mat[12] * self.mat[3] * self.mat[9];

        inv.mat[13] = self.mat[0]  * self.mat[9] * self.mat[14]
            - self.mat[0]  * self.mat[10] * self.mat[13]
            - self.mat[8]  * self.mat[1] * self.mat[14]
            + self.mat[8]  * self.mat[2] * self.mat[13]
            + self.mat[12] * self.mat[1] * self.mat[10]
            - self.mat[12] * self.mat[2] * self.mat[9];

        inv.mat[2] = self.mat[1]  * self.mat[6] * self.mat[15]
            - self.mat[1]  * self.mat[7] * self.mat[14]
            - self.mat[5]  * self.mat[2] * self.mat[15]
            + self.mat[5]  * self.mat[3] * self.mat[14]
            + self.mat[13] * self.mat[2] * self.mat[7]
            - self.mat[13] * self.mat[3] * self.mat[6];

        inv.mat[6] = -self.mat[0]  * self.mat[6] * self.mat[15]
            + self.mat[0]  * self.mat[7] * self.mat[14]
            + self.mat[4]  * self.mat[2] * self.mat[15]
            - self.mat[4]  * self.mat[3] * self.mat[14]
            - self.mat[12] * self.mat[2] * self.mat[7]
            + self.mat[12] * self.mat[3] * self.mat[6];

        inv.mat[10] = self.mat[0]  * self.mat[5] * self.mat[15]
            - self.mat[0]  * self.mat[7] * self.mat[13]
            - self.mat[4]  * self.mat[1] * self.mat[15]
            + self.mat[4]  * self.mat[3] * self.mat[13]
            + self.mat[12] * self.mat[1] * self.mat[7]
            - self.mat[12] * self.mat[3] * self.mat[5];

        inv.mat[14] = -self.mat[0]  * self.mat[5] * self.mat[14]
            + self.mat[0]  * self.mat[6] * self.mat[13]
            + self.mat[4]  * self.mat[1] * self.mat[14]
            - self.mat[4]  * self.mat[2] * self.mat[13]
            - self.mat[12] * self.mat[1] * self.mat[6]
            + self.mat[12] * self.mat[2] * self.mat[5];

        inv.mat[3] = -self.mat[1] * self.mat[6] * self.mat[11]
            + self.mat[1] * self.mat[7] * self.mat[10]
            + self.mat[5] * self.mat[2] * self.mat[11]
            - self.mat[5] * self.mat[3] * self.mat[10]
            - self.mat[9] * self.mat[2] * self.mat[7]
            + self.mat[9] * self.mat[3] * self.mat[6];

        inv.mat[7] = self.mat[0] * self.mat[6] * self.mat[11]
            - self.mat[0] * self.mat[7] * self.mat[10]
            - self.mat[4] * self.mat[2] * self.mat[11]
            + self.mat[4] * self.mat[3] * self.mat[10]
            + self.mat[8] * self.mat[2] * self.mat[7]
            - self.mat[8] * self.mat[3] * self.mat[6];

        inv.mat[11] = -self.mat[0] * self.mat[5] * self.mat[11]
            + self.mat[0] * self.mat[7] * self.mat[9]
            + self.mat[4] * self.mat[1] * self.mat[11]
            - self.mat[4] * self.mat[3] * self.mat[9]
            - self.mat[8] * self.mat[1] * self.mat[7]
            + self.mat[8] * self.mat[3] * self.mat[5];

        inv.mat[15] = self.mat[0] * self.mat[5] * self.mat[10]
            - self.mat[0] * self.mat[6] * self.mat[9]
            - self.mat[4] * self.mat[1] * self.mat[10]
            + self.mat[4] * self.mat[2] * self.mat[9]
            + self.mat[8] * self.mat[1] * self.mat[6]
            - self.mat[8] * self.mat[2] * self.mat[5];

        let mut det = self.mat[0] * inv.mat[0] + self.mat[1] * inv.mat[4]
            + self.mat[2] * inv.mat[8] + self.mat[3] * inv.mat[12];
        assert!(det != 0f32);
        det = 1f32 / det;

        for x in inv.mat.iter_mut() {
            *x *= det;
        }
        inv
    }
}

impl Add<Matrix4, Matrix4> for Matrix4 {
    /// Add two matrices together
    fn add(self, rhs: Matrix4) -> Matrix4 {
        // TODO: Is there not a way to fill an array from an iterator?
        let mut it = self.mat.iter().zip(rhs.mat.iter()).map(|(&x, &y)| x + y).enumerate();
        let mut res = Matrix4::zero();
        for (i, x) in it {
            res.mat[i] = x;
        }
        res
    }
}

impl Sub<Matrix4, Matrix4> for Matrix4 {
    /// Subtract two matrices
    fn sub(self, rhs: Matrix4) -> Matrix4 {
        // TODO: Is there not a way to fill an array from an iterator?
        let mut it = self.mat.iter().zip(rhs.mat.iter()).map(|(&x, &y)| x - y).enumerate();
        let mut res = Matrix4::zero();
        for (i, x) in it {
            res.mat[i] = x;
        }
        res
    }
}

impl Mul<Matrix4, Matrix4> for Matrix4 {
    /// Multiply two matrices
    fn mul(self, rhs: Matrix4) -> Matrix4 {
        let mut res = Matrix4::zero();
        for i in range(0u, 4u) {
            for j in range(0u, 4u) {
                *res.at_mut(i, j) = *self.at(i, 0) * *rhs.at(0, j)
                    + *self.at(i, 1) * *rhs.at(1, j)
                    + *self.at(i, 2) * *rhs.at(2, j)
                    + *self.at(i, 3) * *rhs.at(3, j);
            }
        }
        res
    }
}

