use std::ops::{Deref, DerefMut, Mul};

use crate::{Mat4, Vec3, Vec4};

#[derive(Clone, Copy, Debug)]
pub struct Quat(Vec4);

impl Quat {
    pub const IDENTITY: Quat = Quat(Vec4::W);

    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Quat {
        Quat(Vec4::new(x, y, z, w))
    }

    pub const fn from_vec4(xyzw: Vec4) -> Quat {
        Quat(xyzw)
    }

    pub fn from_parts(imaginary: Vec3, real: f32) -> Quat {
        Quat(Vec4::from_xyz(imaginary, real))
    }

    pub fn normal(&self) -> f32 {
        self.magnitude()
    }

    pub fn normalize(&self) -> Quat {
        Quat(self.0.normalize())
    }

    pub fn conjugate(&self) -> Quat {
        Quat(Vec4::from_xyz(-self.xyz(), self.w()))
    }

    pub fn inverse(&self) -> Quat {
        self.conjugate().normalize()
    }

    pub fn dot(&self, other: Quat) -> f32 {
        self.0.dot(other.0)
    }

    pub fn to_mat4(&self) -> Mat4 {
        let normalized = self.normalize();
        let row_1 = Vec3::new(
            1.0 - 2.0 * normalized.y() * normalized.y() - 2.0 * normalized.z() * normalized.z(),
            2.0 * normalized.x() * normalized.y() - 2.0 * normalized.z() * normalized.w(),
            2.0 * normalized.x() * normalized.z() + 2.0 * normalized.y() * normalized.w(),
        );
        let row_2 = Vec3::new(
            2.0 * normalized.x() * normalized.y() + 2.0 * normalized.z() * normalized.w(),
            1.0 - 2.0 * normalized.x() * normalized.x() - 2.0 * normalized.z() * normalized.z(),
            2.0 * normalized.y() * normalized.z() - 2.0 * normalized.x() * normalized.w(),
        );

        let row_3 = Vec3::new(
            2.0 * normalized.x() * normalized.z() - 2.0 * normalized.y() * normalized.w(),
            2.0 * normalized.y() * normalized.z() + 2.0 * normalized.x() * normalized.w(),
            1.0 - 2.0 * normalized.x() * normalized.x() - 2.0 * normalized.y() * normalized.y(),
        );


        Mat4::from_vector_rows(
            Vec4::from_xyz(row_1, 0.0),
            Vec4::from_xyz(row_2, 0.0),
            Vec4::from_xyz(row_3, 0.0),
            Vec4::W,
        )
    }

    pub fn to_mat4_centered(&self, center: Vec3) -> Mat4 {
        let mut row_1 = Vec4::new(
            self.xw().magnitude_squared() - self.yz().magnitude_squared(),
            2.0 * self.x() * self.y() + self.z() * self.w(),
            2.0 * self.x() * self.z() - self.y() * self.w(),
            0.0,
        );
        *row_1.w_mut() =
            center.x() - center.x() * row_1.x() - center.y() * row_1.y() - center.z() * row_1.z();

        let mut row_2 = Vec4::new(
            2.0 * self.x() * self.y() + self.z() * self.w(),
            self.yw().magnitude_squared() - self.xz().magnitude_squared(),
            2.0 * self.y() * self.z() - self.x() * self.w(),
            0.0,
        );
        *row_2.w_mut() =
            center.y() - center.x() * row_1.x() - center.y() * row_1.y() - center.z() * row_1.z();

        let mut row_3 = Vec4::new(
            2.0 * self.x() * self.z() + self.y() * self.w(),
            2.0 * self.y() * self.z() - self.x() * self.w(),
            self.zw().magnitude_squared() - self.xy().magnitude_squared(),
            0.0,
        );
        *row_3.w_mut() =
            center.z() - center.x() * row_1.x() - center.y() * row_1.y() - center.z() * row_1.z();

        Mat4::from_vector_rows(row_1, row_2, row_3, Vec4::W)
    }

    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Quat {
        let half_angle = angle / 2.0;
        let s = half_angle.sin();
        let c = half_angle.cos();

        Quat::new(s * axis.x(), s * axis.y(), s * axis.z(), c)
    }

    fn lerp(from: Quat, to: Quat, t: f32) -> Quat {
        Quat::from_vec4(Vec4::lerp(from.0, to.0, t))
    }

    pub fn slerp(from: Quat, to: Quat, t: f32) -> Quat {
        let norm_f = from.normalize();
        let mut norm_t = to.normalize();
        let mut dot = norm_f.dot(norm_t);

        if dot < 0.0 {
            norm_t = Quat::from_vec4(-norm_t.0);
            dot = -dot;
        }

        const LERP_THRESHOLD: f32 = 0.9995;

        if dot > LERP_THRESHOLD {
            Quat::lerp(norm_f, norm_t, t).normalize()
        } else {
            let theta_0 = dot.acos();
            let theta = theta_0 * t;
            let sin_theta = theta.sin();
            let sin_theta_0 = theta_0.sin();

            let s0 = theta.cos() - dot * sin_theta / sin_theta_0;
            let s1 = sin_theta / sin_theta_0;
            Quat::from_vec4(from.0 * s0 + to.0 * s1)
        }
    }
}

impl Deref for Quat {
    type Target = Vec4;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Quat {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Mul for Quat {
    type Output = Quat;

    fn mul(self, rhs: Self) -> Self::Output {
        Quat(Vec4::new(
            self.x() * rhs.w() + self.y() * rhs.z() - self.z() * rhs.y() + self.w() * rhs.x(),
            -self.x() * rhs.z() + self.y() * rhs.w() + self.z() * rhs.x() + self.w() * rhs.y(),
            self.x() * rhs.y() - self.y() * rhs.x() + self.z() * rhs.w() + self.w() * rhs.z(),
            -self.x() * rhs.x() - self.y() * rhs.y() - self.z() * rhs.z() + self.w() * rhs.w(),
        ))
    }
}
