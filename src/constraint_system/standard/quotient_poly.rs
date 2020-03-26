/// This quotient polynomial can only be used for the standard composer
/// Each composer will need to implement their own method for computing the quotient polynomial
use crate::constraint_system::standard::PreProcessedCircuit;
use crate::constraint_system::widget::{ArithmeticWidget, RangeWidget};

use crate::fft::Evaluations;
use crate::fft::{EvaluationDomain, Polynomial};
use crate::permutation::grand_product_quotient;
use bls12_381::Scalar;
use rayon::prelude::*;

/// Computes the quotient polynomial
pub(crate) fn compute(
    domain: &EvaluationDomain,
    preprocessed_circuit: &PreProcessedCircuit,
    z_poly: &Polynomial,
    witness_polynomials: [&Polynomial; 4],
    public_inputs_poly: &Polynomial,
    (alpha, beta, gamma): &(Scalar, Scalar, Scalar),
) -> Polynomial {
    let w_l_poly = witness_polynomials[0];
    let w_r_poly = witness_polynomials[1];
    let w_o_poly = witness_polynomials[2];
    let w_4_poly = witness_polynomials[3];

    // Compute 4n eval of z(X)
    let domain_4n = EvaluationDomain::new(4 * domain.size()).unwrap();
    let mut z_eval_4n = domain_4n.coset_fft(&z_poly);
    z_eval_4n.push(z_eval_4n[0]);
    z_eval_4n.push(z_eval_4n[1]);
    z_eval_4n.push(z_eval_4n[2]);
    z_eval_4n.push(z_eval_4n[3]);

    let t_1 = compute_circuit_satisfiability_equation(
        domain,
        preprocessed_circuit,
        w_l_poly,
        w_r_poly,
        w_o_poly,
        w_4_poly,
        public_inputs_poly,
    );

    let t_2 = grand_product_quotient::compute_identity_polynomial(
        domain, &alpha, beta, gamma, &z_eval_4n, &w_l_poly, &w_r_poly, &w_o_poly, &w_4_poly,
    );
    let t_3 = grand_product_quotient::compute_copy_polynomial(
        domain,
        &alpha,
        beta,
        gamma,
        &z_eval_4n,
        &w_l_poly,
        &w_r_poly,
        &w_o_poly,
        &w_4_poly,
        &preprocessed_circuit.permutation.left_sigma.polynomial,
        &preprocessed_circuit.permutation.right_sigma.polynomial,
        &preprocessed_circuit.permutation.out_sigma.polynomial,
        &preprocessed_circuit.permutation.fourth_sigma.polynomial,
    );

    let t_4 = grand_product_quotient::compute_is_one_polynomial(domain, z_poly, alpha.square());

    let quotient: Vec<_> = (0..domain_4n.size())
        .into_par_iter()
        .map(|i| {
            let numerator = t_2[i] + t_3[i] + t_4[i];
            let denominator = preprocessed_circuit.v_h_coset_4n()[i];
            t_1[i] + (numerator * denominator.invert().unwrap())
        })
        .collect();

    Polynomial::from_coefficients_vec(domain_4n.coset_ifft(&quotient))
}

// Ensures that the circuit is satisfied
fn compute_circuit_satisfiability_equation(
    domain: &EvaluationDomain,
    preprocessed_circuit: &PreProcessedCircuit,
    wl_poly: &Polynomial,
    wr_poly: &Polynomial,
    wo_poly: &Polynomial,
    w4_poly: &Polynomial,
    pi_poly: &Polynomial,
) -> Evaluations {
    let domain_4n = EvaluationDomain::new(4 * domain.size()).unwrap();

    let pi_eval_4n = domain_4n.coset_fft(pi_poly);
    let wl_eval_4n = domain_4n.coset_fft(&wl_poly);
    let wr_eval_4n = domain_4n.coset_fft(&wr_poly);
    let wo_eval_4n = domain_4n.coset_fft(&wo_poly);
    let mut w4_eval_4n = domain_4n.coset_fft(&w4_poly);
    w4_eval_4n.push(w4_eval_4n[0]);
    w4_eval_4n.push(w4_eval_4n[1]);
    w4_eval_4n.push(w4_eval_4n[2]);
    w4_eval_4n.push(w4_eval_4n[3]);

    let v_h = domain_4n.compute_vanishing_poly_over_coset(domain.size() as u64);

    let t_1: Vec<_> = (0..domain_4n.size())
        .into_par_iter()
        .map(|i| {
            let wl = &wl_eval_4n[i];
            let wr = &wr_eval_4n[i];
            let wo = &wo_eval_4n[i];
            let w4 = &w4_eval_4n[i];
            let w4_next = &w4_eval_4n[i + 4];
            let pi = &pi_eval_4n[i];
            let v_h_i = v_h[i].invert().unwrap();

            let a = preprocessed_circuit
                .arithmetic
                .compute_quotient(i, wl, wr, wo, w4);
            let b = preprocessed_circuit
                .range
                .compute_quotient(i, wl, wr, wo, w4, w4_next);

            (a + b + pi) * v_h_i
        })
        .collect();
    Evaluations::from_vec_and_domain(t_1, domain_4n)
}
