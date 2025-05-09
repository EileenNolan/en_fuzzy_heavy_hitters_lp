// use scuttlebutt::AesRng;
// use fancy_garbling::{twopac::semihonest, util, Fancy, FancyBinary, FancyArithmetic, BinaryGadgets, BinaryBundle, FancyReveal};
// use scuttlebutt::field::F128b;
// use std::time::Instant;

// /// Inputs to the fancy function: two 128-bit binary bundles
// #[derive(Clone)]
// struct SUMInputs<F> {
//     garbler_wires: BinaryBundle<F>,
//     evaluator_wires: BinaryBundle<F>,
// }

// /// Secure computation: compute sum = x + y (XOR), return MSB only (bit 127)
// fn fancy_sum_is_negative<F>(
//     f: &mut F,
//     wire_inputs: SUMInputs<F::Item>,
// ) -> Result<BinaryBundle<F::Item>, F::Error>
// where
//     F: FancyReveal + Fancy + BinaryGadgets + FancyBinary + FancyArithmetic,
//     F::Item: Clone,
// {
//     let sum = f.bin_addition_no_carry(&wire_inputs.garbler_wires, &wire_inputs.evaluator_wires)?;
    
//     // Extract MSB (bit 127)
//     let msb = extract::<F>(&sum, 127, 1)?;
//     Ok(msb)
// }

// /// Prepare MSB shares: XOR-share the result between garbler and evaluator
// fn get_msb_shares(
//     garbler_input: u128,
//     evaluator_input: u128,
// ) -> Result<(u128, u128), Box<dyn std::error::Error>> {
//     let mut rng = AesRng::new();
//     let gb_inputs = util::u128_to_bits(garbler_input)
//         .into_iter()
//         .map(F128b::from)
//         .collect::<Vec<_>>();
//     let ev_inputs = util::u128_to_bits(evaluator_input)
//         .into_iter()
//         .map(F128b::from)
//         .collect::<Vec<_>>();

//     let mut garbler = semihonest::Garbler::new(&mut rng);
//     let mut evaluator = semihonest::Evaluator::new(&mut rng);

//     let start = Instant::now();

//     // Encode inputs
//     let gb_encoded = garbler.encode_binary(&gb_inputs)?;
//     let ev_encoded = evaluator.encode_binary(&ev_inputs)?;

//     let gb_bundle = BinaryBundle::from(gb_encoded);
//     let ev_bundle = BinaryBundle::from(ev_encoded);

//     // Run secure computation to get MSB
//     let msb_bundle = fancy_sum_is_negative(&mut garbler, SUMInputs {
//         garbler_wires: gb_bundle.clone(),
//         evaluator_wires: ev_bundle.clone(),
//     })?;

//     let output_wire = msb_bundle.get(0).clone(); // 1-bit result

//     // Garbler chooses random mask and masks the MSB
//     let r: u128 = rng.gen::<u128>() & 1; // 1-bit random value
//     let masked_output = garbler.output_clear(&output_wire)?.to_u128() ^ r;

//     let elapsed = start.elapsed();
//     println!("Computation done in {:?}", elapsed);

//     Ok((r, masked_output))
// }

// /// Helper to extract a slice of bits (e.g. extract MSB)
// fn extract<F>(
//     b: &BinaryBundle<F::Item>,
//     start: usize,
//     len: usize,
// ) -> Result<BinaryBundle<F::Item>, F::Error>
// where
//     F: Fancy,
//     F::Item: Clone,
// {
//     Ok(BinaryBundle::from(
//         b.wires()[start..start + len].to_vec(),
//     ))
// }

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Test inputs
//     let x: u128 = 123;
//     let y: u128 = 456;

//     let (r, masked_share) = get_msb_shares(x, y)?;

//     // Reconstruct MSB
//     let computed_msb = r ^ masked_share;

//     // Ground truth MSB from x ^ y
//     let expected = (x ^ y) >> 127;

//     println!("Reconstructed MSB: {}", computed_msb);
//     println!("Expected MSB: {}", expected);

//     assert_eq!(computed_msb, expected);
//     Ok(())
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_msb_share_computation() {
//         // Try a variety of input cases
//         let test_cases = vec![
//             (0u128, 0u128),
//             (1u128 << 127, 0u128),           // MSB set
//             (0u128, 1u128 << 127),           // MSB set
//             (1u128 << 127, 1u128 << 127),    // MSB cleared (XOR)
//             (123456789u128, 987654321u128), // Random example
//             (u128::MAX, u128::MAX),         // XOR = 0
//             (u128::MAX, 0u128),             // MSB set
//         ];

//         for (x, y) in test_cases {
//             let (r, masked_share) = get_msb_shares(x, y).expect("Failed to compute MSB shares");
//             let reconstructed_msb = r ^ masked_share;
//             let expected_msb = ((x ^ y) >> 127) & 1;

//             assert_eq!(
//                 reconstructed_msb, expected_msb,
//                 "Failed on inputs x = {}, y = {}", x, y
//             );
//         }
//     }
// }