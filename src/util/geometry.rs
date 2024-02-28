use nalgebra::{ClosedAdd, ClosedMul, ClosedSub, Scalar, Vector3};
use num_traits::Signed;
use std::ops::Range;

pub trait Coordinate:
    Clone + ClosedAdd + ClosedMul + ClosedSub + Copy + PartialEq + PartialOrd + Scalar + Signed
{
    const ZERO: Self;
    const ONE: Self;
}

#[derive(Clone, Copy, Debug)]
pub struct Cuboid<T> {
    base: Vector3<T>,
    size: Vector3<T>,
}

impl<T: Coordinate> Cuboid<T> {
    pub fn new_empty() -> Cuboid<T> {
        Cuboid::new_cube(Vector3::from_element(T::ZERO), T::ZERO)
    }

    pub fn new_unit_cube(base: Vector3<T>) -> Cuboid<T> {
        Cuboid::new_cube(base, T::ONE)
    }

    pub fn new_cube(base: Vector3<T>, side: T) -> Cuboid<T> {
        Cuboid::new(base, Vector3::from_element(side))
    }

    pub fn new(base: Vector3<T>, size: Vector3<T>) -> Cuboid<T> {
        Cuboid { base, size }
    }

    pub fn is_empty(&self) -> bool {
        self.size == Vector3::from_element(T::ZERO)
    }

    pub fn contains(&self, point: Vector3<T>) -> bool {
        let diff = point - self.base;
        diff.x >= T::ZERO
            && diff.x < self.size.x
            && diff.y >= T::ZERO
            && diff.y < self.size.y
            && diff.z >= T::ZERO
            && diff.z < self.size.z
    }

    pub fn side_voxels(&self, direction: Vector3<T>) -> impl Iterator<Item = Vector3<T>>
    where
        Range<T>: Iterator<Item = T>,
    {
        self.assert_is_direction(direction);
        let (du, lu) = if direction.x == T::ZERO {
            (Vector3::new(T::ONE, T::ZERO, T::ZERO), self.size.x)
        } else {
            (Vector3::new(T::ZERO, T::ONE, T::ZERO), self.size.y)
        };
        let (dv, lv) = if direction.z == T::ZERO {
            (Vector3::new(T::ZERO, T::ZERO, T::ONE), self.size.z)
        } else {
            (Vector3::new(T::ZERO, T::ONE, T::ZERO), self.size.y)
        };
        let side_base = self.base
            + Vector3::new(
                if direction.x > T::ZERO {
                    self.size.x - T::ONE
                } else {
                    T::ZERO
                },
                if direction.y > T::ZERO {
                    self.size.y - T::ONE
                } else {
                    T::ZERO
                },
                if direction.z > T::ZERO {
                    self.size.z - T::ONE
                } else {
                    T::ZERO
                },
            );
        (T::ZERO..lu).flat_map(move |u| (T::ZERO..lv).map(move |v| side_base + du * u + dv * v))
    }

    pub fn distance_from_inside(&self, point: Vector3<T>, direction: Vector3<T>) -> T {
        self.assert_is_direction(direction);
        let abs_direction = direction.abs();
        let edge = if direction.sum() > T::ZERO {
            self.base + self.size
        } else {
            self.base - Vector3::from_element(T::ONE)
        };
        let point_on_axis = point.component_mul(&abs_direction).sum();
        let edge_on_axis = edge.component_mul(&abs_direction).sum();
        (point_on_axis - edge_on_axis).abs()
    }

    pub fn extend_in_direction(&self, direction: Vector3<T>) -> Cuboid<T> {
        self.assert_is_direction(direction);
        let base = self.base
            + Vector3::new(
                if direction.x < T::ZERO {
                    -T::ONE
                } else {
                    T::ZERO
                },
                if direction.y < T::ZERO {
                    -T::ONE
                } else {
                    T::ZERO
                },
                if direction.z < T::ZERO {
                    -T::ONE
                } else {
                    T::ZERO
                },
            );
        let size = self.size + direction.abs();
        Cuboid { base, size }
    }

    fn assert_is_direction(&self, _direction: Vector3<T>) {
        // TODO:
    }
}

impl Coordinate for i64 {
    const ZERO: i64 = 0;
    const ONE: i64 = 1;
}

#[allow(dead_code)]
pub fn directions<T: Coordinate>() -> [Vector3<T>; 6] {
    [
        Vector3::new(T::ONE, T::ZERO, T::ZERO),
        Vector3::new(-T::ONE, T::ZERO, T::ZERO),
        Vector3::new(T::ZERO, T::ONE, T::ZERO),
        Vector3::new(T::ZERO, -T::ONE, T::ZERO),
        Vector3::new(T::ZERO, T::ZERO, T::ONE),
        Vector3::new(T::ZERO, T::ZERO, -T::ONE),
    ]
}
