use crate::field::FieldElm;

/// Constructs the Lagrange basis polynomial coefficients for a given index `i`
pub fn lagrange_basis_coeffs(x_values: &[FieldElm], i: usize) -> Vec<FieldElm> {
    let mut coeffs = vec![FieldElm::one()];
    let mut denom = FieldElm::one();

    for (j, x_j) in x_values.iter().enumerate() {
        if i != j {
            let mut new_coeffs = vec![FieldElm::zero(); coeffs.len() + 1];

            for (k, c) in coeffs.iter().enumerate() {
                new_coeffs[k] = new_coeffs[k].add(&c.neg().mul(x_j));
                new_coeffs[k + 1] = new_coeffs[k + 1].add(c);
            }

            coeffs = new_coeffs;
            let diff = FieldElm::from((x_values[i].value + crate::field::MODULUS_64 - x_j.value) % crate::field::MODULUS_64);
            denom = denom.mul(&diff);
        }
    }

    let denom_inv = denom.mod_inverse();
    coeffs.iter().map(|c| c.mul(&denom_inv)).collect()
}

/// Computes Lagrange interpolation coefficients for a given set of points
pub fn lagrange_interpolation_coeffs(x_values: &[FieldElm], y_values: &[FieldElm]) -> Vec<FieldElm> {
    assert_eq!(x_values.len(), y_values.len());

    let mut final_coeffs = vec![FieldElm::zero(); x_values.len()];

    for i in 0..x_values.len() {
        let basis_coeffs = lagrange_basis_coeffs(x_values, i);
        for (j, coeff) in basis_coeffs.iter().enumerate() {
            final_coeffs[j] = final_coeffs[j].add(&coeff.mul(&y_values[i]));
        }
    }

    final_coeffs
}

/// Evaluates a polynomial at a given field element
pub fn evaluate_polynomial(coeffs: &[FieldElm], x: &FieldElm) -> FieldElm {
    let mut result = FieldElm::zero();
    let mut power = FieldElm::one();

    for coeff in coeffs {
        result = result.add(&coeff.mul(&power));
        power = power.mul(x);
    }

    result
}

/// Prints a polynomial in human-readable form
pub fn print_polynomial(coeffs: &[FieldElm], name: &str) {
    print!("{}(x) = ", name);
    let mut first = true;
    for (i, coeff) in coeffs.iter().enumerate().rev() {
        if coeff.value != 0 {
            if !first {
                print!(" + ");
            }
            print!("{}x^{}", coeff.value, i);
            first = false;
        }
    }
    println!();
}