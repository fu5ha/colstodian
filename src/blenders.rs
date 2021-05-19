use crate::traits::ColorBlender;

/*
pub struct SampleArea<T> {
    pub prev: T,
    pub first: T,
    pub second: T,
    pub next: T,
}
pub trait InterpolationRange<T>
where
    T: Copy
        + Mul<Output=T>
        + Add<Output=T>
        + Sub<Output=T>
        + Mul<f32, Output=T>
        + Add<f32, Output=T>
        + Sub<f32, Output=T>
{
    fn sample_area(&self, at: f32) -> SampleArea<T>;
}

impl<T> InterpolationRange<T> for RangeInclusive<T>
where
    T: Copy
        + Mul<Output=T>
        + Add<Output=T>
        + Sub<Output=T>
        + Mul<f32, Output=T>
        + Add<f32, Output=T>
        + Sub<f32, Output=T>
{
    fn sample_area(&self, _: f32) -> SampleArea<T> {
        let first = *self.start();
        let second = *self.end();
        SampleArea {
            prev: (first * 2.0) - second,
            first,
            second,
            next: (second * 2.0) - first
        }
    }
}

impl<T, const N: usize> InterpolationRange<T> for [T; N]
where
    T: Copy
        + Mul<Output=T>
        + Add<Output=T>
        + Sub<Output=T>
        + Mul<f32, Output=T>
        + Add<f32, Output=T>
        + Sub<f32, Output=T>
{
    fn sample_area(&self, at: f32) -> SampleArea<T> {
        assert!(at >= 0.0);
        assert!(at < 1.0);
        let first_index = (at * N as f32) as usize;
        let second_index = first_index + 1;
        assert!(second_index < N);
        let first = self[first_index];
        let second = self[second_index];
        let prev = if first_index > 0 {
            self[first_index - 1]
        } else {
            first * 2.0 - second
        };
        let next = if second_index < N - 1 {
            self[second_index + 1]
        } else {
            second * 2.0 - first
        };
        SampleArea { prev, first, second, next }
    }
}
*/

// pub struct Gradient<Range, Blender>

/// Linearly interpolate between `range.start..=range.end` by `factor`.
pub enum Linear {}

impl ColorBlender for Linear 
{
    type Params = ();

    fn blend_with(start: f32, end: f32, factor: f32, _params: Self::Params) -> f32 {
        start + (end - start) * factor
    }
}
/// Interpolate with cubic b-spline interpolation
pub enum BSpline {}

#[derive(Clone, Copy, Debug)]
pub struct BSplineParams {
    pub prev_closer_control_point_weight: f32,
    pub prev_further_control_point_weight: f32,
    pub next_closer_control_point_weight: f32,
    pub next_further_control_point_weight: f32,
}

impl BSplineParams {
    pub fn new(first_control_point_bias: f32, second_control_point_bias: f32) -> BSplineParams {
        // x / y = bias
        // x - y = 1.0
        // -> 
        // y = x - 1
        // y(z - 1) = 1
        // y = 1 / (z - 1)
        // x = z / (z - 1)
        fn calc_weights_from_bias(bias: f32) -> (f32, f32) {
            let bm1 = bias - 1.0;
            (bias / bm1, 1.0 / bm1)
        }

        let (prev_closer_control_point_weight, prev_further_control_point_weight) = calc_weights_from_bias(first_control_point_bias);
        let (next_closer_control_point_weight, next_further_control_point_weight) = calc_weights_from_bias(second_control_point_bias);

        BSplineParams {
            prev_closer_control_point_weight,
            prev_further_control_point_weight,
            next_closer_control_point_weight,
            next_further_control_point_weight,
        }
    }
}

impl Default for BSplineParams {
    fn default() -> Self {
        Self {
            prev_closer_control_point_weight: 2.0,
            prev_further_control_point_weight: 1.0,
            next_closer_control_point_weight: 2.0,
            next_further_control_point_weight: 1.0,
        }
    }
}

impl ColorBlender for BSpline
{
    type Params = BSplineParams;
    
    fn blend_with(start: f32, end: f32, factor: f32, params: Self::Params) -> f32 {
        let prev = params.prev_closer_control_point_weight * start - params.prev_further_control_point_weight * end;
        let next = params.next_closer_control_point_weight * end - params.next_further_control_point_weight * start;

        let t = factor;
        let t2 = t * t;
        let t3 = t * t2;

        ((1.0 - 3.0 * t + 3.0 * t2 - t3) * prev
            + (4.0 - 6.0 * t2 + 3.0 * t3) * start
            + (1.0 + 3.0 * t + 3.0 * t2 - 3.0 * t3) * end
            + t3 * next) / 6.0
    }
}
