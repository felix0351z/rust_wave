use thiserror::Error;

#[derive(Debug, Error)]
pub enum MathError {
    #[error("Both arrays must have the same length")]
    NoEqualLength
}


pub fn array_product(a: &[f32], b: &[f32]) -> Result<Vec<f32>, MathError> {
    if a.len() != b.len() {
        return Err(MathError::NoEqualLength)
    }
    let mut out = Vec::new();
    for (num1, num2) in a.iter().zip(b.iter()) {
        out.push(*num1 * *num2);
    }

    Ok(out)
}

pub fn linspace(x0: f32, xend: f32, n: usize) -> Vec<f32> {
    let mut out = vec![x0;n];

    let delta = (xend - x0) / (n -1) as f32;
    for i in 1..n {
        out[i] = x0 + delta*i as f32;
    }

    out
}

pub(crate) fn transpose(x: &[f32], color: [u8; 3]) -> Vec<u8> {
    let mut out = vec![0u8; x.len()*3+1];
    out[0] = 0u8;

    let mut idx_output = 1;
    for i in 1..x.len() {
        out[idx_output] = (x[i] * color[0] as f32).round() as u8;
        out[idx_output+1] = (x[i] * color[1] as f32).round() as u8;
        out[idx_output+2] = (x[i] * color[2] as f32).round() as u8;
        idx_output += 3;
    }

    out
}

pub fn gaussian_curve(len: usize, std: f32) -> Vec<f32> {
    let mut curve = Vec::with_capacity(len);
    let m = len as f32 - 1.0 ;

    let center = m / 2.0;
    let sigma2 = 2.0 * std * std;

    for i in 0..len {
        let x = i as f32;
        let exponent = -(x - center).powi(2) / sigma2;
        curve.push(f32::exp(exponent));
    }

    curve
}

pub trait Flip {
    fn clone_flip(&self) -> Self;
}

impl<T: Clone + Sized> Flip for Vec<T> {
    fn clone_flip(&self) -> Self {
        Vec::from_iter(self.into_iter().cloned().rev())
    }
}

