//!Neural Network Layers

use linalg::{Matrix, MatrixSlice, BaseMatrix, BaseMatrixMut};

use learning::toolkit::activ_fn::ActivationFunc;

use rand::thread_rng;
use rand::distributions::Sample;
use rand::distributions::normal::Normal;

use std::fmt::Debug;

/// Trait for neural net layers
pub trait NetLayer : Debug {
	/// The result of propogating data forward through this layer
	fn forward(&self, input: &Matrix<f64>, params: MatrixSlice<f64>) -> Matrix<f64>;

	/// The gradient of the output of this layer with respect to its input
	fn back_input(&self, out_grad: &Matrix<f64>, input: &Matrix<f64>, params: MatrixSlice<f64>) -> Matrix<f64>;
	
	/// The gradient of the output of this layer with respect to its parameters
	fn back_params(&self, out_grad: &Matrix<f64>, input: &Matrix<f64>, params: MatrixSlice<f64>) -> Matrix<f64>;

	/// The default value of the parameters of this layer before training
	fn default_params(&self) -> Vec<f64>;

	/// The shape of the parameters used by this layer
	fn param_shape(&self) -> (usize, usize);

	/// The number of parameters used by this layer
	fn num_params(&self) -> usize {
		let shape = self.param_shape();
		shape.0 * shape.1
	}
}

/// Linear network layer
///
/// Represents a fully connected layer with optional bias term
///
/// The parameters are a matrix of weights of size I x O
/// where O is the dimensionality of the output and I the dimensionality of the input
#[derive(Debug, Clone, Copy)]
pub struct Linear {
	/// The number of dimensions of the input
	input_size: usize,
	/// The number of dimensions of the output
	output_size: usize,
	/// Whether or not to include a bias term
	has_bias: bool,
}

impl Linear {
	/// Construct a new Linear layer
	pub fn new(input_size: usize, output_size: usize) -> Linear {
		Linear {
			input_size: input_size + 1, 
			output_size: output_size,
			has_bias: true
		}
	}

	/// Construct a Linear layer with a bias term
	pub fn without_bias(input_size: usize, output_size: usize) -> Linear {
		Linear {
			input_size: input_size, 
			output_size: output_size,
			has_bias: false
		}
	}
}

impl NetLayer for Linear {
	/// Computes a matrix product
	///
	/// input should have dimensions N x I
	/// where N is the number of samples and I is the dimensionality of the input
	fn forward(&self, input: &Matrix<f64>, params: MatrixSlice<f64>) -> Matrix<f64> {
		if self.has_bias {
			debug_assert_eq!(input.cols()+1, params.rows());
			input.hcat(&Matrix::<f64>::ones(input.rows(), 1)) * &params
		} else {
			debug_assert_eq!(input.cols(), params.rows());
			input * &params
		}
	}

	fn back_input(&self, out_grad: &Matrix<f64>, _: &Matrix<f64>, params: MatrixSlice<f64>) -> Matrix<f64> {
		debug_assert_eq!(out_grad.cols(), params.cols());
		//let gradient = out_grad * &params.into_matrix().transpose();
		if self.has_bias {
			//let columns: Vec<_> = (0..gradient.cols()-1).collect();
			//gradient.select_cols(&columns)
			let rows: Vec<_> = (0..params.rows()-1).collect();
			out_grad * &params.into_matrix().select_rows(&rows).transpose()
		} else {
			//gradient
			out_grad * &params.into_matrix().transpose()
		}
	}
	
	fn back_params(&self, out_grad: &Matrix<f64>, input: &Matrix<f64>, _: MatrixSlice<f64>) -> Matrix<f64> {
		assert_eq!(input.rows(), out_grad.rows());
		if self.has_bias {
			//input.transpose().vcat(&Matrix::<f64>::ones(1, input.rows())) * out_grad
			input.hcat(&Matrix::<f64>::ones(input.rows(), 1)).transpose() * out_grad
		} else {
			input.transpose() * out_grad
		}
	}

	/// Initializes weights using Xavier initialization
	///
	/// weights drawn from gaussian distribution with 0 mean and variance 2/(input_size+output_size)
	fn default_params(&self) -> Vec<f64> {
		let mut distro = Normal::new(0.0, (2.0/(self.input_size+self.output_size) as f64).sqrt());
		let mut rng = thread_rng();

		(0..self.input_size*self.output_size).map(|_| distro.sample(&mut rng))
											 .collect()
	}

	fn param_shape(&self) -> (usize, usize) {
		(self.input_size, self.output_size)
	}
}

impl<T: ActivationFunc + Debug> NetLayer for T {
	/// Applys the activation function to each element of the input
	fn forward(&self, input: &Matrix<f64>, _: MatrixSlice<f64>) -> Matrix<f64> {
		//Matrix::new(input.rows(), input.cols(),
		//	input.iter().map(|&x| T::func(x)).collect::<Vec<_>>());
		input.clone().apply(&T::func)
	}

	fn back_input(&self, out_grad: &Matrix<f64>, input: &Matrix<f64>, _: MatrixSlice<f64>) -> Matrix<f64> {
		let in_grad = Matrix::new(input.rows(), input.cols(),
	        			input.iter().map(|&x| T::func_grad(x)).collect::<Vec<_>>());
		out_grad.elemul(&in_grad)
	}
	
	fn back_params(&self, _: &Matrix<f64>, _: &Matrix<f64>, _: MatrixSlice<f64>) -> Matrix<f64> {
		Matrix::new(0, 0, Vec::new())
	}

	fn default_params(&self) -> Vec<f64> {
		Vec::new()
	}

	fn param_shape(&self) -> (usize, usize) {
		(0, 0)
	}
}