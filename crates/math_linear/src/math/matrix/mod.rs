//! Matrices

use {
    crate::prelude::*,
    directx_math::*,
};

#[derive(Debug, Clone, Copy)]
pub struct FloatMatrix4x4 {
    inner: XMMATRIX,
}

impl FloatMatrix4x4 {
    /// Constructs matrix from 16 floats.
    pub fn new(
        a11: f32, a12: f32, a13: f32, a14: f32,
        a21: f32, a22: f32, a23: f32, a24: f32,
        a31: f32, a32: f32, a33: f32, a34: f32,
        a41: f32, a42: f32, a43: f32, a44: f32
    ) -> Self {
        Self {
            inner: XMMatrixSet(
                a11, a12, a13, a14,
                a21, a22, a23, a24,
                a31, a32, a33, a34,
                a41, a42, a43, a44
            )
        }
    }

    /// Constructs matrix from 1d float array.
    pub fn from_1d_array(mat: [f32; 16]) -> Self {
        FloatMatrix4x4 {
            inner: XMMatrixSet(
                mat[0],  mat[1],  mat[2],  mat[3],
                mat[4],  mat[5],  mat[6],  mat[7],
                mat[8],  mat[9],  mat[10], mat[11],
                mat[12], mat[13], mat[14], mat[15]
            )
        }
    }

    /// Constructs matrix from 2d float array.
    pub fn from_2d_array(mat: [[f32; 4]; 4]) -> Self {
        FloatMatrix4x4 {
            inner: XMMatrixSet(
                mat[0][0], mat[0][1], mat[0][2], mat[0][3],
                mat[1][0], mat[1][1], mat[1][2], mat[1][3],
                mat[2][0], mat[2][1], mat[2][2], mat[2][3],
                mat[3][0], mat[3][1], mat[3][2], mat[3][3]
            )
        }
    }

    /// Gives 2D array representation of matrix.
    pub fn as_2d_array(self) -> [[f32; 4]; 4] {
        XMMatrix(self.inner).into()
    }

    /// Sets all elements to given float.
    pub fn all(all: f32) -> Self {
        Self::from_1d_array([all; 16])
    }

    /// Sets main diagonal to given float.
    pub fn set_diagonal(a: f32) -> Self {
        Self::new(
            a,   0.0, 0.0, 0.0,
            0.0, a,   0.0, 0.0,
            0.0, 0.0, a,   0.0,
            0.0, 0.0, 0.0, a
        )
    }

    /// Sets main diagonal to given float except last is 1.0
    pub fn set_diagonal_last_unit(a: f32) -> Self {
        Self::new(
            a,   0.0, 0.0, 0.0,
            0.0, a,   0.0, 0.0,
            0.0, 0.0, a,   0.0,
            0.0, 0.0, 0.0, 1.0
        )
    }

    /// Gives identity matrix.
    pub fn identity() -> Self {
        Self::set_diagonal(1.0)
    }

    /// Gives view matrix for left-handed coordinate system
    pub fn look_at_lh(eye_pos: vec3, focus_pos: vec3, up_dir: vec3) -> Self {
        FloatMatrix4x4 {
            inner: XMMatrixLookAtLH(eye_pos.into(), focus_pos.into(), up_dir.into())
        }
    }

    /// Gives view matrix for right-handed coordinate system
    pub fn look_at_rh(eye_pos: vec3, focus_pos: vec3, up_dir: vec3) -> Self {
        FloatMatrix4x4 {
            inner: XMMatrixLookAtRH(eye_pos.into(), focus_pos.into(), up_dir.into())
        }
    }

    /// Gives perspective matrix for left-handed coordinate system.
    pub fn perspective_lh(view_width: f32, view_height: f32, near: f32, far: f32) -> Self {
        FloatMatrix4x4 {
            inner: XMMatrixPerspectiveLH(view_width, view_height, near, far)
        }
    }

    /// Gives perspective matrix for right-handed coordinate system.
    pub fn perspective_rh(view_width: f32, view_height: f32, near: f32, far: f32) -> Self {
        FloatMatrix4x4 {
            inner: XMMatrixPerspectiveRH(view_width, view_height, near, far)
        }
    }

    /// Gives perspective matrix for left-handed coordinate system with fov setting.
    pub fn perspective_fov_lh(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        FloatMatrix4x4 {
            inner: XMMatrixPerspectiveFovLH(fov, 1.0 / aspect_ratio, near, far)
        }
    }

    /// Gives perspective matrix for right-handed coordinate system with fov setting.
    pub fn perspective_fov_rh(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        FloatMatrix4x4 {
            inner: XMMatrixPerspectiveFovRH(fov, 1.0 / aspect_ratio, near, far)
        }
    }

    pub fn orthographic_lh(view_width: f32, view_height: f32, near: f32, far: f32) -> Self {
        FloatMatrix4x4 {
            inner: XMMatrixOrthographicLH(view_width, view_height, near, far)
        }
    }

    /// Gives rotation matrix from roll, pitch and yaw.
    pub fn rotation_rpy(roll: f32, pitch: f32, yaw: f32) -> Self {
        Self {
            inner: XMMatrixRotationRollPitchYaw(pitch, yaw, roll)
        }
    }

    pub fn scaling(amount: vec3) -> Self {
        Self {
            inner: XMMatrixScaling(amount.x, amount.y, amount.z),
        }
    }

    pub fn translation(offset: vec3) -> Self {
        Self {
            inner: XMMatrixTranslation(offset.x, offset.y, offset.z)
        }
    }

    /// Matrices multiply.
    pub fn multiply(&self, rhs: &Self) -> Self {
        FloatMatrix4x4 {
            inner: XMMatrixMultiply(self.inner, &rhs.inner)
        }
    }

    /// Multiplies matrix by another.
    pub fn multiply_by(&mut self, another: &Self) {
        self.inner = XMMatrixMultiply(self.inner, &another.inner);
    }

    /// Transposes the matrix.
    pub fn transpose(&mut self) {
        self.inner = XMMatrixTranspose(self.inner)
    }

    /// Inverses matrix.
    fn inverse(&mut self) {
        self.inner = XMMatrixInverse(None, self.inner);
    }
}

impl Default for FloatMatrix4x4 {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(feature = "byte_muck")]
unsafe impl bytemuck::Pod for FloatMatrix4x4 { }

#[cfg(feature = "byte_muck")]
unsafe impl bytemuck::Zeroable for FloatMatrix4x4 {
    fn zeroed() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl std::ops::Mul<&Self> for &FloatMatrix4x4 {
    type Output = FloatMatrix4x4;
    fn mul(self, another: &Self) -> Self::Output {
        Self::Output { inner: XMMatrixMultiply(another.inner, &self.inner) }
    }
}

impl std::ops::Mul for FloatMatrix4x4 {
    type Output = FloatMatrix4x4;
    fn mul(self, another: Self) -> Self::Output {
        Self::Output { inner: XMMatrixMultiply(another.inner, &self.inner) }
    }
}

impl std::ops::MulAssign<Self> for FloatMatrix4x4 {
    fn mul_assign(&mut self, another: Self) {
        self.inner = XMMatrixMultiply(self.inner, &another.inner)
    }
}

impl std::ops::Mul<vec4> for &FloatMatrix4x4 {
    type Output = vec4;
    fn mul(self, vec: vec4) -> vec4 {
        vec4 {
            i_vec: XMVector4Transform(vec.i_vec, self.inner)
        }
    }
}

impl std::ops::Mul<vec3> for FloatMatrix4x4 {
    type Output = vec3;
    fn mul(self, vec: vec3) -> vec3 {
        let xmvec = XMVector4Transform(vec4::from(vec).i_vec, self.inner);
        vec3::new(
            XMVectorGetX(xmvec),
            XMVectorGetY(xmvec),
            XMVectorGetZ(xmvec),
        )
    }
}
