// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Simple Arithmetic Gates

use crate::constraint_system::StandardComposer;
use crate::constraint_system::Variable;
use ark_ff::FftField;

impl<F> StandardComposer<F>
where
    F: FftField,
{
    /// Adds a width-3 add gate to the circuit, linking the addition of the
    /// provided inputs, scaled by the selector coefficients with the output
    /// provided.
    pub fn add_gate(
        &mut self,
        a: Variable,
        b: Variable,
        c: Variable,
        q_l: F,
        q_r: F,
        q_o: F,
        q_c: F,
        pi: Option<F>,
    ) -> Variable {
        self.big_add_gate(a, b, c, None, q_l, q_r, q_o, F::zero(), q_c, pi)
    }

    /// Adds a width-4 add gate to the circuit and it's corresponding
    /// constraint.
    ///
    /// This type of gate is usually used when we need to have
    /// the largest amount of performance and the minimum circuit-size
    /// possible. Since it allows the end-user to set every selector coefficient
    /// as scaling value on the gate eq.
    pub fn big_add_gate(
        &mut self,
        a: Variable,
        b: Variable,
        c: Variable,
        d: Option<Variable>,
        q_l: F,
        q_r: F,
        q_o: F,
        q_4: F,
        q_c: F,
        pi: Option<F>,
    ) -> Variable {
        // Check if advice wire has a value
        let d = match d {
            Some(var) => var,
            None => self.zero_var,
        };

        self.w_l.push(a);
        self.w_r.push(b);
        self.w_o.push(c);
        self.w_4.push(d);

        // For an add gate, q_m is zero
        self.q_m.push(F::zero());

        // Add selector vectors
        self.q_l.push(q_l);
        self.q_r.push(q_r);
        self.q_o.push(q_o);
        self.q_c.push(q_c);
        self.q_4.push(q_4);
        self.q_arith.push(F::one());
        self.q_range.push(F::zero());
        self.q_logic.push(F::zero());
        self.q_fixed_group_add.push(F::zero());
        self.q_variable_group_add.push(F::zero());

        if let Some(pi) = pi {
            assert!(self.public_inputs_sparse_store.insert(self.n, pi).is_none(),"The invariant of already having a PI inserted for this position should never exist");
        }

        self.perm.add_variables_to_map(a, b, c, d, self.n);

        self.n += 1;

        c
    }
    /// Adds a width-3 mul gate to the circuit linking the product of the
    /// provided inputs scaled by the selector coefficient `q_m` with the output
    /// provided scaled by `q_o`.
    ///
    /// Note that this gate requires to provide the actual result of the gate
    /// (output wire) since it will just add a `mul constraint` to the circuit.
    pub fn mul_gate(
        &mut self,
        a: Variable,
        b: Variable,
        c: Variable,
        q_m: F,
        q_o: F,
        q_c: F,
        pi: Option<F>,
    ) -> Variable {
        self.big_mul_gate(a, b, c, None, q_m, q_o, q_c, F::zero(), pi)
    }

    /// Adds a width-4 `big_mul_gate` with the left, right and fourth inputs
    /// and it's scaling factors, computing & returning the output (result)
    /// `Variable` and adding the corresponding mul constraint.
    ///
    /// This type of gate is usually used when we need to have
    /// the largest amount of performance and the minimum circuit-size
    /// possible. Since it allows the end-user to setup all of the selector
    /// coefficients.
    ///
    /// Forces `q_m * (a * b) + q_4 * d + q_c + q_o * c = 0`
    // XXX: Maybe make these tuples instead of individual field?
    pub fn big_mul_gate(
        &mut self,
        a: Variable,
        b: Variable,
        c: Variable,
        d: Option<Variable>,
        q_m: F,
        q_o: F,
        q_c: F,
        q_4: F,
        pi: Option<F>,
    ) -> Variable {
        // Check if advice wire has a value
        let d = match d {
            Some(var) => var,
            None => self.zero_var,
        };

        self.w_l.push(a);
        self.w_r.push(b);
        self.w_o.push(c);
        self.w_4.push(d);

        // For a mul gate q_L and q_R is zero
        self.q_l.push(F::zero());
        self.q_r.push(F::zero());

        // Add selector vectors
        self.q_m.push(q_m);
        self.q_o.push(q_o);
        self.q_c.push(q_c);
        self.q_4.push(q_4);
        self.q_arith.push(F::one());

        self.q_range.push(F::zero());
        self.q_logic.push(F::zero());
        self.q_fixed_group_add.push(F::zero());
        self.q_variable_group_add.push(F::zero());

        if let Some(pi) = pi {
            assert!(
                self.public_inputs_sparse_store.insert(self.n, pi).is_none(),"The invariant of already having a PI inserted for this position should never exist"
            );
        }

        self.perm.add_variables_to_map(a, b, c, d, self.n);

        self.n += 1;

        c
    }

    /// This gates turns on all the selctor polynomials to give users,
    /// in some situations, the ability to use the extra selector for
    /// more variables into additions.
    ///
    /// This type of gate is usually used when we need to have
    /// the largest amount of performance and the minimum circuit-size
    /// possible. Since it allows the end-user to setup all of the selector
    /// coefficients.
    ///
    /// Equation: `(a*b)*q_m + a*q_l + b*q_r + d*q_4 + q_c + PI + q_o * c = 0`.
    /// `d` will be set to zero if not provided.
    ///
    /// ### Returns
    /// `c`
    pub fn big_arith_gate(
        &mut self,
        a: Variable,
        b: Variable,
        c: Variable,
        d: Option<Variable>,
        q_m: F,
        q_l: F,
        q_r: F,
        q_o: F,
        q_c: F,
        q_4: F,
        pi: Option<F>,
    ) -> Variable {
        // Check if advice wire has a value
        let d = match d {
            Some(var) => var,
            None => self.zero_var,
        };

        self.w_l.push(a);
        self.w_r.push(b);
        self.w_o.push(c);
        self.w_4.push(d);

        // Add selector vectors
        self.q_m.push(q_m);
        self.q_o.push(q_o);
        self.q_c.push(q_c);
        self.q_4.push(q_4);
        self.q_l.push(q_l);
        self.q_r.push(q_r);
        self.q_arith.push(F::one());

        self.q_range.push(F::zero());
        self.q_logic.push(F::zero());
        self.q_fixed_group_add.push(F::zero());
        self.q_variable_group_add.push(F::zero());

        if let Some(pi) = pi {
            assert!(
                self.public_inputs_sparse_store.insert(self.n, pi).is_none(),"The invariant of already having a PI inserted for this position should never exist"
            );
        }

        self.perm.add_variables_to_map(a, b, c, d, self.n);

        self.n += 1;

        c
    }

    /// Adds a [`StandardComposer::big_add_gate`] with the left and right
    /// inputs and it's scaling factors, computing & returning the output
    /// (result) [`Variable`], and adding the corresponding addition
    /// constraint.
    ///
    /// This type of gate is usually used when we don't need to have
    /// the largest amount of performance as well as the minimum circuit-size
    /// possible. Since it defaults some of the selector coeffs = 0 in order
    /// to reduce the verbosity and complexity.
    ///
    /// Forces `q_l * w_l + q_r * w_r + q_c + PI = w_o(computed by the gate)`.
    pub fn add(
        &mut self,
        q_l_a: (F, Variable),
        q_r_b: (F, Variable),
        q_c: F,
        pi: Option<F>,
    ) -> Variable {
        self.big_add(q_l_a, q_r_b, None, q_c, pi)
    }

    /// Adds a [`StandardComposer::big_add_gate`] with the left, right and
    /// fourth inputs and it's scaling factors, computing & returning the
    /// output (result) [`Variable`] and adding the corresponding addition
    /// constraint.
    ///
    /// This type of gate is usually used when we don't need to have
    /// the largest amount of performance and the minimum circuit-size
    /// possible. Since it defaults some of the selector coeffs = 0 in order
    /// to reduce the verbosity and complexity.
    ///
    /// Forces `q_l * w_l + q_r * w_r + q_4 * w_4 + q_c + PI = w_o(computed by
    /// the gate)`.
    pub fn big_add(
        &mut self,
        q_l_a: (F, Variable),
        q_r_b: (F, Variable),
        q_4_d: Option<(F, Variable)>,
        q_c: F,
        pi: Option<F>,
    ) -> Variable {
        // Check if advice wire is available
        let (q_4, d) = match q_4_d {
            Some((q_4, var)) => (q_4, var),
            None => (F::zero(), self.zero_var),
        };

        let (q_l, a) = q_l_a;
        let (q_r, b) = q_r_b;

        let q_o = -F::one();

        // Compute the output wire
        let a_eval = self.variables[&a];
        let b_eval = self.variables[&b];
        let d_eval = self.variables[&d];
        let c_eval = (q_l * a_eval)
            + (q_r * b_eval)
            + (q_4 * d_eval)
            + q_c
            + pi.unwrap_or_default();
        let c = self.add_input(c_eval);

        self.big_add_gate(a, b, c, Some(d), q_l, q_r, q_o, q_4, q_c, pi)
    }

    /// Adds a [`StandardComposer::big_mul_gate`] with the left, right
    /// and fourth inputs and it's scaling factors, computing & returning
    /// the output (result) [`Variable`] and adding the corresponding mul
    /// constraint.
    ///
    /// This type of gate is usually used when we don't need to have
    /// the largest amount of performance and the minimum circuit-size
    /// possible. Since it defaults some of the selector coeffs = 0 in order
    /// to reduce the verbosity and complexity.
    ///
    /// Forces `q_m * (w_l * w_r) + w_4 * q_4 + q_c + PI = w_o(computed by the
    /// gate)`.
    ///
    /// `{w_l, w_r, w_4} = {a, b, d}`
    pub fn mul(
        &mut self,
        q_m: F,
        a: Variable,
        b: Variable,
        q_c: F,
        pi: Option<F>,
    ) -> Variable {
        self.big_mul(q_m, a, b, None, q_c, pi)
    }

    /// Adds a width-4 [`StandardComposer::big_mul_gate`] with the left, right
    /// and fourth inputs and it's scaling factors, computing & returning
    /// the output (result) [`Variable`] and adding the corresponding mul
    /// constraint.
    ///
    /// This type of gate is usually used when we don't need to have
    /// the largest amount of performance and the minimum circuit-size
    /// possible. Since it defaults some of the selector coeffs = 0 in order
    /// to reduce the verbosity and complexity.
    ///
    /// Forces `q_m * (w_l * w_r) + w_4 * q_4 + q_c + PI = w_o(computed by the
    /// gate)`.
    ///
    /// `{w_l, w_r, w_4} = {a, b, d}`
    pub fn big_mul(
        &mut self,
        q_m: F,
        a: Variable,
        b: Variable,
        q_4_d: Option<(F, Variable)>,
        q_c: F,
        pi: Option<F>,
    ) -> Variable {
        let q_o = -F::one();

        // Check if advice wire is available
        let (q_4, d) = match q_4_d {
            Some((q_4, var)) => (q_4, var),
            None => (F::zero(), self.zero_var),
        };

        // Compute output wire
        let a_eval = self.variables[&a];
        let b_eval = self.variables[&b];
        let d_eval = self.variables[&d];
        let c_eval = (q_m * a_eval * b_eval)
            + (q_4 * d_eval)
            + q_c
            + pi.unwrap_or_default();
        let c = self.add_input(c_eval);

        self.big_mul_gate(a, b, c, Some(d), q_m, q_o, q_c, q_4, pi)
    }

    /// Adds a [`StandardComposer::big_arith_gate`] with the left, right
    /// , fourth inputs and corresponding coefficients, computing & returning
    /// the output (result) [`Variable`] and adding the corresponding arith
    /// constraint.
    ///
    /// This type of gate is usually used when we don't need to have
    /// the largest amount of performance and the minimum circuit-size
    /// possible, since it defaults set `q_o` to `-1` to reduce the verbosity.
    ///
    /// Equation: `(a*b)*q_m + a*q_l + b*q_r + d*q_4 + q_c + PI = c`
    /// ### Returns
    /// `c`
    pub fn big_arith(
        &mut self,
        q_m: F,
        a: Variable,
        b: Variable,
        q_l: F,
        q_r: F,
        q_4_d: Option<(F, Variable)>,
        q_c: F,
        pi: Option<F>,
    ) -> Variable {
        // check if advice wire is available
        let (q_4, d) = match q_4_d {
            Some((q_4, d)) => (q_4, d),
            None => (F::zero(), self.zero_var),
        };

        // compute output wire
        let a_eval = self.variables[&a];
        let b_eval = self.variables[&b];
        let d_eval = self.variables[&d];

        let c_eval = (q_m * a_eval * b_eval)
            + (q_l * a_eval)
            + (q_r * b_eval)
            + (q_4 * d_eval)
            + q_c
            + pi.unwrap_or_default();

        let c = self.add_input(c_eval);

        self.big_arith_gate(
            a,
            b,
            c,
            Some(d),
            q_m,
            q_l,
            q_r,
            -F::one(),
            q_c,
            q_4,
            pi,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::batch_test;
    use crate::constraint_system::helper::*;
    use ark_bls12_377::Bls12_377;
    use ark_bls12_381::Bls12_381;
    use ark_ec::{PairingEngine, TEModelParameters};
    use ark_ff::{One, Zero};

    fn test_public_inputs<E, P>()
    where
        E: PairingEngine,
        P: TEModelParameters<BaseField = E::Fr>,
    {
        let res = gadget_tester::<E, P>(
            |composer: &mut StandardComposer<E::Fr>| {
                let var_one = composer.add_input(E::Fr::one());
                let should_be_three = composer.big_add(
                    (E::Fr::one(), var_one),
                    (E::Fr::one(), var_one),
                    None,
                    E::Fr::zero(),
                    Some(E::Fr::one()),
                );
                composer.constrain_to_constant(
                    should_be_three,
                    E::Fr::from(3u64),
                    None,
                );
                let should_be_four = composer.big_add(
                    (E::Fr::one(), var_one),
                    (E::Fr::one(), var_one),
                    None,
                    E::Fr::zero(),
                    Some(E::Fr::from(2u64)),
                );
                composer.constrain_to_constant(
                    should_be_four,
                    E::Fr::from(4u64),
                    None,
                );
            },
            200,
        );
        assert!(res.is_ok(), "{:?}", res.err().unwrap());
    }

    fn test_correct_add_mul_gate<E, P>()
    where
        E: PairingEngine,
        P: TEModelParameters<BaseField = E::Fr>,
    {
        let res = gadget_tester::<E, P>(
            |composer: &mut StandardComposer<E::Fr>| {
                // Verify that (4+5+5) * (6+7+7) = 280
                let four = composer.add_input(E::Fr::from(4u64));
                let five = composer.add_input(E::Fr::from(5u64));
                let six = composer.add_input(E::Fr::from(6u64));
                let seven = composer.add_input(E::Fr::from(7u64));

                let fourteen = composer.big_add(
                    (E::Fr::one(), four),
                    (E::Fr::one(), five),
                    Some((E::Fr::one(), five)),
                    E::Fr::zero(),
                    None,
                );

                let twenty = composer.big_add(
                    (E::Fr::one(), six),
                    (E::Fr::one(), seven),
                    Some((E::Fr::one(), seven)),
                    E::Fr::zero(),
                    None,
                );

                // There are quite a few ways to check the equation is correct,
                // depending on your circumstance If we already
                // have the output wire, we can constrain the output of the
                // mul_gate to be equal to it If we do not, we
                // can compute it using the `mul` If the output
                // is public, we can also constrain the output wire of the mul
                // gate to it. This is what this test does
                let output = composer.mul(
                    E::Fr::one(),
                    fourteen,
                    twenty,
                    E::Fr::zero(),
                    None,
                );
                composer.constrain_to_constant(
                    output,
                    E::Fr::from(280u64),
                    None,
                );
            },
            200,
        );
        assert!(res.is_ok(), "{:?}", res.err().unwrap());
    }

    fn test_correct_add_gate<E, P>()
    where
        E: PairingEngine,
        P: TEModelParameters<BaseField = E::Fr>,
    {
        let res = gadget_tester::<E, P>(
            |composer: &mut StandardComposer<E::Fr>| {
                let zero = composer.zero_var();
                let one = composer.add_input(E::Fr::one());

                let c = composer.add(
                    (E::Fr::one(), one),
                    (E::Fr::zero(), zero),
                    E::Fr::from(2u64),
                    None,
                );
                composer.constrain_to_constant(c, E::Fr::from(3u64), None);
            },
            32,
        );
        assert!(res.is_ok(), "{:?}", res.err().unwrap());
    }

    fn test_correct_big_add_mul_gate<E, P>()
    where
        E: PairingEngine,
        P: TEModelParameters<BaseField = E::Fr>,
    {
        let res = gadget_tester::<E, P>(
            |composer: &mut StandardComposer<E::Fr>| {
                // Verify that (4+5+5) * (6+7+7) + (8*9) = 352
                let four = composer.add_input(E::Fr::from(4u64));
                let five = composer.add_input(E::Fr::from(5u64));
                let six = composer.add_input(E::Fr::from(6u64));
                let seven = composer.add_input(E::Fr::from(7u64));
                let nine = composer.add_input(E::Fr::from(9u64));

                let fourteen = composer.big_add(
                    (E::Fr::one(), four),
                    (E::Fr::one(), five),
                    Some((E::Fr::one(), five)),
                    E::Fr::zero(),
                    None,
                );

                let twenty = composer.big_add(
                    (E::Fr::one(), six),
                    (E::Fr::one(), seven),
                    Some((E::Fr::one(), seven)),
                    E::Fr::zero(),
                    None,
                );

                let output = composer.big_mul(
                    E::Fr::one(),
                    fourteen,
                    twenty,
                    Some((E::Fr::from(8u64), nine)),
                    E::Fr::zero(),
                    None,
                );
                composer.constrain_to_constant(
                    output,
                    E::Fr::from(352u64),
                    None,
                );
            },
            200,
        );
        assert!(res.is_ok());
    }

    fn test_correct_big_arith_gate<E, P>()
    where
        E: PairingEngine,
        P: TEModelParameters<BaseField = E::Fr>,
    {
        let res = gadget_tester::<E, P>(
            |composer: &mut StandardComposer<E::Fr>| {
                // Verify that (4*5)*6 + 4*7 + 5*8 + 9*10 + 11 = 289
                let a = composer.add_input(E::Fr::from(4u64));
                let b = composer.add_input(E::Fr::from(5u64));
                let q_m = E::Fr::from(6u64);
                let q_l = E::Fr::from(7u64);
                let q_r = E::Fr::from(8u64);
                let d = composer.add_input(E::Fr::from(9u64));
                let q_4 = E::Fr::from(10u64);
                let q_c = E::Fr::from(11u64);

                let output = composer.big_arith(
                    q_m,
                    a,
                    b,
                    q_l,
                    q_r,
                    Some((q_4, d)),
                    q_c,
                    None,
                );

                composer.constrain_to_constant(
                    output,
                    E::Fr::from(289u64),
                    None,
                );
            },
            200,
        );
        assert!(res.is_ok());
    }

    fn test_incorrect_big_arith_gate<E, P>()
    where
        E: PairingEngine,
        P: TEModelParameters<BaseField = E::Fr>,
    {
        let res = gadget_tester::<E, P>(
            |composer: &mut StandardComposer<E::Fr>| {
                // Verify that (4*5)*6 + 4*7 + 5*8 + 9*12 + 11 != 289
                let a = composer.add_input(E::Fr::from(4u64));
                let b = composer.add_input(E::Fr::from(5u64));
                let q_m = E::Fr::from(6u64);
                let q_l = E::Fr::from(7u64);
                let q_r = E::Fr::from(8u64);
                let d = composer.add_input(E::Fr::from(9u64));
                let q_4 = E::Fr::from(12u64);
                let q_c = E::Fr::from(11u64);

                let output = composer.big_arith(
                    q_m,
                    a,
                    b,
                    q_l,
                    q_r,
                    Some((q_4, d)),
                    q_c,
                    None,
                );

                composer.constrain_to_constant(
                    output,
                    E::Fr::from(289u64),
                    None,
                );
            },
            200,
        );
        assert!(res.is_err());
    }

    fn test_incorrect_add_mul_gate<E, P>()
    where
        E: PairingEngine,
        P: TEModelParameters<BaseField = E::Fr>,
    {
        let res = gadget_tester::<E, P>(
            |composer: &mut StandardComposer<E::Fr>| {
                // Verify that (5+5) * (6+7) != 117
                let five = composer.add_input(E::Fr::from(5u64));
                let six = composer.add_input(E::Fr::from(6u64));
                let seven = composer.add_input(E::Fr::from(7u64));

                let five_plus_five = composer.big_add(
                    (E::Fr::one(), five),
                    (E::Fr::one(), five),
                    None,
                    E::Fr::zero(),
                    None,
                );

                let six_plus_seven = composer.big_add(
                    (E::Fr::one(), six),
                    (E::Fr::one(), seven),
                    None,
                    E::Fr::zero(),
                    None,
                );

                let output = composer.mul(
                    E::Fr::one(),
                    five_plus_five,
                    six_plus_seven,
                    E::Fr::zero(),
                    None,
                );
                composer.constrain_to_constant(
                    output,
                    E::Fr::from(117u64),
                    None,
                );
            },
            200,
        );
        assert!(res.is_err());
    }

    // Bls12-381 tests
    batch_test!(
        [
            test_public_inputs,
            test_correct_add_mul_gate,
            test_correct_add_gate,
            test_correct_big_add_mul_gate,
            test_correct_big_arith_gate,
            test_incorrect_add_mul_gate,
            test_incorrect_big_arith_gate
        ],
        [] => (
            Bls12_381, ark_ed_on_bls12_381::EdwardsParameters
        )
    );

    // Bls12-377 tests
    batch_test!(
        [
            test_public_inputs,
            test_correct_add_mul_gate,
            test_correct_add_gate,
            test_correct_big_add_mul_gate,
            test_correct_big_arith_gate,
            test_incorrect_add_mul_gate,
            test_incorrect_big_arith_gate
        ],
        [] => (
            Bls12_377, ark_ed_on_bls12_377::EdwardsParameters
        )
    );
}
