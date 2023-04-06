use rand::{Rng, seq::SliceRandom};

/// Structure to randomly choose between elements T with weights
#[derive(Debug, Clone, Copy, Default)]
pub struct Choices<'a, T> {
	pub choices: &'a [T],
	pub weights: Option<&'a [f32]>,
}

impl<'a, T> Choices<'a, T> {
	#[inline]
	pub fn get_random(&self, rng: &mut impl Rng) -> Option<&'a T> { 
		if let Some(weights) = self.weights {
			assert!(weights.len() == self.choices.len());

			let all_weights = weights.iter().sum::<f32>();
			let mut random_weight = rng.gen_range(0.0..all_weights); 
	
			for (choice,weight) in self.choices.iter().zip(weights.iter()) {
				random_weight -= weight;
	
				if random_weight <= 0.0 {
					return Some(choice);
				}
			}
			self.choices.last()
		} else {
			self.choices.choose(rng)
		}
	}

	#[inline]
	pub fn random(&self, mut rng: &mut impl Rng) -> &'a T {
		self.get_random(&mut rng).unwrap()
	}

	pub fn from_vec(vec: &'a Vec<T>) -> Self {
		Choices { 
			choices: vec,
			weights: None,
		}
	}
}

#[macro_export]
macro_rules! choice {
	[$( ($element:expr, $weight:expr) ),* $(,)?] => {
		Choices { choices: &[ $($element,)* ], weights: &[ $($weight,)*] }
	};

	[$( $element:expr ),* $(,)?] => {
		Choices { choices: &[ $($element,)* ], weights: None }
	};
}