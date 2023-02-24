use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use bytemuck::{Pod, Zeroable};
use paste::paste;
use petra_macros::swizzles;

macro_rules! vector {
    ($({$name: ident, $fields: tt, [$($($aliases: ident),*);*], $swizzle_types: tt})*) => {
        $(
            vector!(struct $name, $fields);

            impl $name {
                vector!(getters $fields, [$($($aliases),*);*]);
            }
        )*
    };
    (struct $name: ident, [$($field: ident),*]) => {
        #[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
        #[repr(C)]
        pub struct $name {
            $($field: f32),*
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "<")?;
                $(write!(f, "{}: {}", stringify!($field), self.$field)?;)*
                write!(f, ">")
            }
        }

        impl Add for $name {
            type Output = $name;

            fn add(self, rhs: Self) -> Self::Output {
                $name::new($(self.$field + rhs.$field),*)
            }
        }

        impl AddAssign for $name {
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        impl Sub for $name {
            type Output = $name;

            fn sub(self, rhs: Self) -> Self::Output {
                $name::new($(self.$field - rhs.$field),*)
            }
        }

        impl SubAssign for $name {
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }

        impl Mul<f32> for $name {
            type Output = $name;

            fn mul(self, rhs: f32) -> Self::Output {
                $name::new($(self.$field * rhs),*)
            }
        }

        impl Mul<$name> for f32 {
            type Output = $name;

            fn mul(self, rhs: $name) -> Self::Output {
                rhs.mul(self)
            }
        }

        impl MulAssign<f32> for $name {
            fn mul_assign(&mut self, rhs: f32) {
                *self = *self * rhs
            }
        }

        impl Div<f32> for $name {
            type Output = $name;

            fn div(self, rhs: f32) -> Self::Output {
                $name::new($(self.$field / rhs),*)
            }
        }

        impl Div<$name> for f32 {
            type Output = $name;

            fn div(self, rhs: $name) -> Self::Output {
                rhs.div(self)
            }
        }

        impl DivAssign<f32> for $name {
            fn div_assign(&mut self, rhs: f32) {
                *self = *self / rhs
            }
        }

        impl Neg for $name {
            type Output = $name;

            fn neg(self) -> Self::Output {
                $name::new($(-self.$field),*)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::ZERO
            }
        }

        impl $name {
            pub const ZERO: $name = $name {
                $($field: 0.0),*
            };

            pub const fn new($($field: f32),*) -> Self {
                Self {
                    $($field),*
                }
            }

            pub fn magnitude(&self) -> f32 {
                self.magnitude_squared().sqrt()
            }

            pub fn magnitude_squared(&self) -> f32 {
                self.dot(*self)
            }

            pub fn normalize(&self) -> $name {
                *self / self.magnitude()
            }

            pub fn dot(&self, other: $name) -> f32 {
                0.0 $(+ self.$field * other.$field)*
            }

            #[doc = "Multiplies two vectors component wise"]
            pub fn component_mul(&self, other: $name) -> $name {
                $name::new($(self.$field * other.$field),*)
            }

            #[doc = "Divides two vectors component wise"]
            pub fn component_div(&self, other: $name) -> $name {
                $name::new($(self.$field / other.$field),*)
            }

            pub fn lerp(from: $name, to: $name, t: f32) -> $name {
                (to - from) * t + from
            }
        }
    };
    (getters [$($field: ident),*], [$($($alias: ident),*);*]) => {
        $(
            $(
                #[inline(always)]
                pub fn $alias(&self) -> f32 {
                    self.$field
                }

                paste!{
                    #[inline(always)]
                    pub fn [<$alias _mut>](&mut self) -> &mut f32 {
                        &mut self.$field
                    }
                }
            )*
        )*
    };
}

vector!(
    {
        Vec2, [x, y], [
            x, u;
            y, v
        ], [Vec2]
    }
    {
        Vec3, [x, y, z], [
            x, r, u;
            y, g, v;
            z, b, w
        ], [Vec2, Vec3]
    }
    {
        Vec4, [x, y, z, w], [
            x, r;
            y, g;
            z, b;
            w, a
        ], [Vec2, Vec3, Vec4]
    }
);

swizzles! {
    Vec2
    [Vec2, Vec3, Vec4]
    [
        (x, y),
        (u, v),
    ]
}
swizzles! {
    Vec3
    [Vec2, Vec3, Vec4]
    [
        (x, y, z),
        (r, g, b),
        (u, v, w),
    ]
}
swizzles! {
    Vec4
    [Vec2, Vec3, Vec4]
    [
        (x, y, z, w),
        (r, g, b, a),
    ]
}

impl Vec2 {
    pub const X: Vec2 = Vec2::new(1.0, 0.0);
    pub const Y: Vec2 = Vec2::new(0.0, 1.0);

    pub fn cross(&self, other: Vec2) -> f32 {
        self.x * other.y - other.x * self.y
    }

    pub fn angle_from_origin(&self) -> f32 {
        f32::atan(self.y / self.x)
    }

    pub fn angel_from(&self, other: Self) -> f32 {
        let v = *self - other;
        f32::atan(v.y / v.x)
    }

    pub fn to_array(self) -> [f32; 2] {
        [self.x, self.y]
    }

    pub fn from_array(arr: [f32; 2]) -> Vec2 {
        Vec2::new(arr[0], arr[1])
    }
}

impl Vec3 {
    pub const X: Vec3 = Vec3::new(1.0, 0.0, 0.0);
    pub const Y: Vec3 = Vec3::new(0.0, 1.0, 0.0);
    pub const Z: Vec3 = Vec3::new(0.0, 0.0, 1.0);

    pub fn cross(&self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn to_array(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    pub fn from_array(arr: [f32; 3]) -> Vec3 {
        Vec3::new(arr[0], arr[1], arr[2])
    }

    pub fn from_xy(xy: Vec2, z: f32) -> Vec3 {
        Vec3::new(xy.x, xy.y, z)
    }

    pub fn from_yz(x: f32, yz: Vec2) -> Vec3 {
        Vec3::new(x, yz.x, yz.y)
    }
}

impl Vec4 {
    pub const W: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);
    pub const X: Vec4 = Vec4::new(1.0, 0.0, 0.0, 0.0);
    pub const Y: Vec4 = Vec4::new(0.0, 1.0, 0.0, 0.0);
    pub const Z: Vec4 = Vec4::new(0.0, 0.0, 1.0, 0.0);

    pub fn to_array(self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }

    pub fn from_array(arr: [f32; 4]) -> Vec4 {
        Vec4::new(arr[0], arr[1], arr[2], arr[3])
    }

    pub fn from_xyz(xyz: Vec3, w: f32) -> Vec4 {
        Vec4::new(xyz.x, xyz.y, xyz.z, w)
    }

    pub fn from_yzw(x: f32, yzw: Vec3) -> Vec4 {
        Vec4::new(x, yzw.x, yzw.y, yzw.z)
    }

    pub fn from_xy_zw(xy: Vec2, zw: Vec2) -> Vec4 {
        Vec4::new(xy.x, xy.y, zw.x, zw.y)
    }

    pub fn from_xy(xy: Vec2, z: f32, w: f32) -> Vec4 {
        Vec4::new(xy.x, xy.y, z, w)
    }

    pub fn from_yz(x: f32, yz: Vec2, w: f32) -> Vec4 {
        Vec4::new(x, yz.x, yz.y, w)
    }

    pub fn from_zw(x: f32, y: f32, zw: Vec2) -> Vec4 {
        Vec4::new(x, y, zw.x, zw.y)
    }
}
