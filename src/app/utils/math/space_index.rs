/// Gives index in `pos.len()` dimension array
/// HINT: X -> Y -> Z and W -> H -> D
pub fn get_index(pos: &[usize], dims: &[usize]) -> usize {
	assert!(
		pos.len() >= dims.len() - 1,
		"`pos.len()` should be dimension of array with dimensions `dims.len()` but both equal: [{}, {}]",
		pos.len(), dims.len()
	);

	pos.iter()
		.zip(dims.iter())
		.skip(1)
		.fold(pos[0], |accum, (&p, &d)| d * accum + p)
}

#[cfg(test)]
mod tests {
    use super::get_index;

	#[test]
	fn test() {
		assert_eq!(get_index(&[2, 1], &[4, 5]), 11);
		assert_eq!(get_index(&[2, 3], &[4, 5]), 13);
	}
}