use std::{
    fmt::Display,
    ops::{Index, IndexMut, Mul, MulAssign},
};

use bytemuck::{Zeroable, Pod};

use crate::{Vec3, Vec4};

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(transparent)]
pub struct Mat4([[f32; 4]; 4]);

impl Default for Mat4 {
    fn default() -> Self {
        Mat4::IDENTITY
    }
}

impl Display for Mat4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[{:?}", self.0[0])?;
        writeln!(f, " {:?}", self.0[1])?;
        writeln!(f, " {:?}", self.0[2])?;
        writeln!(f, " {:?}]", self.0[3])
    }
}

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    pub const OPENGL_TO_WGPU: Mat4 = Mat4([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 0.5, 0.0],
        [0.0, 0.0, 0.5, 1.0],
    ]);

    pub fn from_vector_rows(row1: Vec4, row2: Vec4, row3: Vec4, row4: Vec4) -> Mat4 {
        Mat4([
            row1.to_array(),
            row2.to_array(),
            row3.to_array(),
            row4.to_array(),
        ])
    }

    pub fn from_vector_cols(col1: Vec4, col2: Vec4, col3: Vec4, col4: Vec4) -> Mat4 {
        Mat4([
            [col1.x(), col2.x(), col3.x(), col4.x()],
            [col1.y(), col2.y(), col3.y(), col4.y()],
            [col1.z(), col2.z(), col3.z(), col4.z()],
            [col1.w(), col2.w(), col3.w(), col4.w()],
        ])
    }

    fn mat2_det(x0y0: f32, x1y0: f32, x0y1: f32, x1y1: f32) -> f32 {
        x0y0 * x1y1 - x1y0 * x0y1
    }

    fn mat3_det(mat3: [[f32; 3]; 3]) -> f32 {
        mat3[0][0] * Self::mat2_det(mat3[1][1], mat3[1][2], mat3[2][1], mat3[2][2])
            - mat3[0][1] * Self::mat2_det(mat3[1][0], mat3[1][2], mat3[2][0], mat3[2][2])
            + mat3[0][2] * Self::mat2_det(mat3[1][0], mat3[1][1], mat3[2][0], mat3[2][1])
    }

    pub fn det(&self) -> f32 {
        self[0][0]
            * Self::mat3_det([
                [self[1][1], self[2][1], self[3][1]],
                [self[1][2], self[2][2], self[3][2]],
                [self[1][3], self[2][3], self[3][3]],
            ])
            - self[0][1]
                * Self::mat3_det([
                    [self[0][1], self[2][1], self[3][1]],
                    [self[0][2], self[2][2], self[3][2]],
                    [self[0][3], self[2][3], self[3][3]],
                ])
            + self[0][2]
                * Self::mat3_det([
                    [self[0][1], self[1][2], self[3][1]],
                    [self[0][1], self[1][2], self[3][2]],
                    [self[0][1], self[1][2], self[3][3]],
                ])
            - self[0][3]
                * Self::mat3_det([
                    [self[0][1], self[1][1], self[2][1]],
                    [self[0][2], self[1][2], self[2][2]],
                    [self[0][3], self[1][3], self[2][3]],
                ])
    }

    pub fn transpose(&self) -> Mat4 {
        Mat4([
            [self[0][0], self[1][0], self[2][0], self[3][0]],
            [self[0][1], self[1][1], self[2][1], self[3][1]],
            [self[0][2], self[1][2], self[2][2], self[3][2]],
            [self[0][3], self[1][3], self[2][3], self[3][3]],
        ])
    }

    pub fn translation(pos: Vec3) -> Mat4 {
        let mut mat = Mat4::IDENTITY;
        mat[3][0] = pos.x();
        mat[3][1] = pos.y();
        mat[3][2] = pos.z();
        mat
    }

    pub fn scale(scale: Vec3) -> Mat4 {
        Mat4([
            [scale.x(), 0.0, 0.0, 0.0],
            [0.0, scale.y(), 0.0, 0.0],
            [0.0, 0.0, scale.z(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    #[rustfmt::skip]
    pub fn rotation_eular_x(angle_radians: f32) -> Mat4 {
        let s = angle_radians.sin();
        let c = angle_radians.cos();
        Mat4([
            [1.0, 0.0, 0.0, 0.0], 
            [0.0, c, -s, 0.0], 
            [0.0, s, c, 0.0], 
            [0.0, 0.0, 0.0, 1.0]
        ])
    }

    #[rustfmt::skip]
    pub fn rotation_eular_y(angle_radians: f32) -> Mat4 {
        let s = angle_radians.sin();
        let c = angle_radians.cos();
        Mat4([
            [c, 0.0, -s, 0.0], 
            [0.0, 1.0, 0.0, 0.0], 
            [s, 0.0, c, 0.0], 
            [0.0, 0.0, 0.0, 1.0]
        ])
    }

    #[rustfmt::skip]
    pub fn rotation_eular_z(angle_radians: f32) -> Mat4 {
        let s = angle_radians.sin();
        let c = angle_radians.cos();
        Mat4([
            [c, -s, 0.0, 0.0], 
            [s, c, 0.0, 0.0], 
            [0.0, 0.0, 1.0, 0.0], 
            [0.0, 0.0, 0.0, 1.0]
        ])
    }

    pub fn roation_eular_xyz(x_radians: f32, y_radians: f32, z_radians: f32) -> Mat4 {
        let x = Mat4::rotation_eular_x(x_radians);
        let y = Mat4::rotation_eular_y(y_radians);
        let z = Mat4::rotation_eular_z(z_radians);
        x * y * z
    }

    pub fn nth_column(&self, i: usize) -> Vec4 {
        Vec4::new(self[0][i], self[1][i], self[2][i], self[3][i])
    }

    pub fn nth_row(&self, i: usize) -> Vec4 {
        Vec4::from_array(self[i])
    }

    pub fn orthographic_projection(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near_clip: f32,
        far_clip: f32,
    ) -> Mat4 {
        let lr = 1.0 / (left - right);
        let bt = 1.0 / (bottom - top);
        let nf = 1.0 / (near_clip - far_clip);

        Mat4([
            [-2.0 * lr, 0.0, 0.0, 0.0],
            [0.0, -2.0 * bt, 0.0, 0.0],
            [0.0, 0.0, -2.0 * nf, 0.0],
            [
                (left + right) * lr,
                (bottom + top) * bt,
                (near_clip + far_clip) * nf,
                0.0,
            ],
        ])
    }

    pub fn perspective_projection(
        fov_radians: f32,
        aspect_ratio: f32,
        near_clip: f32,
        far_clip: f32,
    ) -> Mat4 {
        let fov = f32::tan(fov_radians * 0.5);

        Mat4([
            [fov / aspect_ratio, 0.0, 0.0, 0.0],
            [0.0, fov, 0.0, 0.0],
            [0.0, 0.0, (far_clip + near_clip) / (near_clip - far_clip), -1.0],
            [0.0, 0.0, (2.0 * far_clip * near_clip) / (near_clip - far_clip), 0.0],
        ])
    }

    pub fn look_at(pos: Vec3, target: Vec3, up: Vec3) -> Mat4 {
        let z_axis = (target - pos).normalize();

        let x_axis = z_axis.cross(up).normalize();
        let y_axis = x_axis.cross(z_axis);

        Mat4([
            [x_axis.x(), y_axis.x(), -z_axis.x(), 0.0],
            [x_axis.y(), y_axis.y(), -z_axis.y(), 0.0],
            [x_axis.z(), y_axis.z(), -z_axis.z(), 0.0],
            [-x_axis.dot(pos), -y_axis.dot(pos), z_axis.dot(pos), 1.0],
        ])
    }

    pub fn forward(&self) -> Vec3 {
        Vec3::new(-self[0][2], -self[1][2], -self[2][2]).normalize()
    }

    pub fn backward(&self) -> Vec3 {
        Vec3::new(self[0][2], self[1][2], self[2][2]).normalize()
    }

    pub fn up(&self) -> Vec3 {
        Vec3::new(self[0][1], self[1][1], self[2][1]).normalize()
    }

    pub fn down(&self) -> Vec3 {
        Vec3::new(-self[0][1], -self[1][1], -self[2][1]).normalize()
    }

    pub fn left(&self) -> Vec3 {
        Vec3::new(-self[0][0], -self[1][0], -self[2][0]).normalize()
    }

    pub fn right(&self) -> Vec3 {
        Vec3::new(self[0][0], self[1][0], self[2][0]).normalize()
    }
}

impl Mul for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Self) -> Self::Output {
        Mat4([
            [
                self.nth_row(0).dot(rhs.nth_column(0)),
                self.nth_row(0).dot(rhs.nth_column(1)),
                self.nth_row(0).dot(rhs.nth_column(2)),
                self.nth_row(0).dot(rhs.nth_column(3)),
            ],
            [
                self.nth_row(1).dot(rhs.nth_column(0)),
                self.nth_row(1).dot(rhs.nth_column(1)),
                self.nth_row(1).dot(rhs.nth_column(2)),
                self.nth_row(1).dot(rhs.nth_column(3)),
            ],
            [
                self.nth_row(2).dot(rhs.nth_column(0)),
                self.nth_row(2).dot(rhs.nth_column(1)),
                self.nth_row(2).dot(rhs.nth_column(2)),
                self.nth_row(2).dot(rhs.nth_column(3)),
            ],
            [
                self.nth_row(3).dot(rhs.nth_column(0)),
                self.nth_row(3).dot(rhs.nth_column(1)),
                self.nth_row(3).dot(rhs.nth_column(2)),
                self.nth_row(3).dot(rhs.nth_column(3)),
            ],
        ])
    }
}

impl MulAssign<Mat4> for Mat4 {
    fn mul_assign(&mut self, rhs: Mat4) {
        *self = *self * rhs;
    }
}

impl Mul<f32> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: f32) -> Self::Output {
        Mat4::from_vector_rows(
            self.nth_row(0) * rhs,
            self.nth_row(1) * rhs,
            self.nth_row(2) * rhs,
            self.nth_row(3) * rhs,
        )
    }
}

impl MulAssign<f32> for Mat4 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Index<usize> for Mat4 {
    type Output = [f32; 4];

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Mat4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
