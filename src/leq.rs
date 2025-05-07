// use fancy_garbling::{
//     twopac::semihonest::{Evaluator, Garbler},
//     util, AllWire, BinaryBundle, BundleGadgets, Fancy, FancyArithmetic, FancyBinary, FancyInput,
//     FancyReveal,
// };
// use ocelot::{ot::AlszReceiver as OtReceiver, ot::AlszSender as OtSender};
// use scuttlebutt::{AbstractChannel, AesRng, Channel, SyncChannel};
// use std::fmt::Debug;
// use std::{
//     io::{BufReader, BufWriter},
//     os::unix::net::UnixStream,
// };
// use std::io::{Read, Write};
// use std::time::Instant;
// use fancy_garbling::util::RngExt;
// use ocelot::ot::Sender;
// use rayon::prelude::*;

// /// A structure that contains both the garbler and evaluator wires.
// /// This structure simplifies the API for wiring the circuit.
// struct EQInputs<F> {
//     pub garbler_wires: BinaryBundle<F>,
//     pub evaluator_wires: BinaryBundle<F>,
// }

// /// Extension trait for `FancyBinary` providing gadgets that operate over binary bundles.
// pub trait BinaryGadgets: FancyBinary + BundleGadgets {
//     fn bin_eq_bundles(
//         &mut self,
//         x: &BinaryBundle<Self::Item>,
//         y: &BinaryBundle<Self::Item>,
//     ) -> Result<Self::Item, Self::Error> {
//         let zs = x
//             .wires()
//             .iter()
//             .zip(y.wires().iter())
//             .map(|(x_bit, y_bit)| {
//                 let xy = self.xor(x_bit, y_bit)?;
//                 self.negate(&xy)
//             })
//             .collect::<Result<Vec<Self::Item>, Self::Error>>()?;
//         self.and_many(&zs)
//     }

//     fn bin_eq_bundles_shared(
//         &mut self,
//         x: &BinaryBundle<Self::Item>,
//         y: &BinaryBundle<Self::Item>,
//     ) -> Result<Self::Item, Self::Error> {
//         assert_eq!(
//             x.wires().len(),
//             y.wires().len() + 1,
//             "x must have one more wire than y"
//         );
//         let (x_wires, mask) = x.wires().split_at(x.wires().len() - 1);
//         let mask = &mask[0]; // Last wire is the mask.
//         let eq_result = self.bin_eq_bundles(&BinaryBundle::new(x_wires.to_vec()), y)?;
//         self.xor(&eq_result, mask) // Obscure the output with the mask.
//     }

//     fn add_binary_bundles<F>(
//         f: &mut F,
//         a: &BinaryBundle<F::Item>,
//         b: &BinaryBundle<F::Item>,
//     ) -> Result<BinaryBundle<F::Item>, F::Error>
//     where
//         F: FancyBinary,
//     {
//         // Handles same-length a and b
//         f.addition(a, b)
//     }

//     fn multi_bin_eq_bundles_shared(
//         &mut self,
//         x: &BinaryBundle<Self::Item>,
//         y: &BinaryBundle<Self::Item>,
//         num_tests: usize,
//     ) -> Result<BinaryBundle<Self::Item>, Self::Error> {
//         assert_eq!(
//             x.wires().len(),
//             y.wires().len() + num_tests,
//             "each string in x must have one extra mask bit"
//         );
//         assert_eq!(y.wires().len() % num_tests, 0);
//         let string_len = y.wires().len() / num_tests;
//         let mut results = Vec::with_capacity(num_tests);
//         for i in 0..num_tests {
//             let x_start = i * (string_len + 1);
//             let y_start = i * string_len;
//             let eq_result = self.bin_eq_bundles(
//                 &BinaryBundle::new(x.wires()[x_start..x_start + string_len].to_vec()),
//                 &BinaryBundle::new(y.wires()[y_start..y_start + string_len].to_vec()),
//             )?;
//             let masked_result = self.xor(&eq_result, &x.wires()[x_start + string_len])?;
//             results.push(masked_result);
//         }
//         Ok(BinaryBundle::new(results))
//     }

//     /// New gadget: computes (x ≤ 0) for a shared number x in two's complement (big‑endian).
//     /// Here x is provided by the evaluator’s bundle while the garbler’s bundle holds an extra mask bit.
//     /// The implementation uses a simple idea:
//     ///   • Compute `is_zero` by AND’ing the negation of every bit of x.
//     ///   • Assume that the first bit (big‑endian) is the sign bit.
//     ///   • Then, x ≤ 0 if x = 0 or if the sign bit is 1.
//     /// Finally, the computed bit is masked (XOR’d with the mask from the garbler).
//     fn multi_bin_leq_zero_bundles_shared(
//         &mut self,
//         x: &BinaryBundle<Self::Item>, // Garbler's bundle: [x bits, mask]
//         y: &BinaryBundle<Self::Item>, // Evaluator's bundle: [x bits only]
//         num_tests: usize,
//     ) -> Result<BinaryBundle<Self::Item>, Self::Error> {
//         assert_eq!(
//             x.wires().len(),
//             y.wires().len() + num_tests,
//             "each string in x must have one extra mask bit"
//         );
//         let string_len = y.wires().len() / num_tests;
//         let mut results = Vec::with_capacity(num_tests);
//         for i in 0..num_tests {
//             let x_start = i * (string_len + 1);
//             let y_start = i * string_len;
//             let mask = &x.wires()[x_start + string_len]; // Last bit of the current chunk.
//             let bits = &y.wires()[y_start..y_start + string_len]; // The actual bits of x.
//             let mut not_bits = Vec::with_capacity(string_len);
//             for b in bits {
//                 let nb = self.negate(b)?;
//                 not_bits.push(nb);
//             }
//             let is_zero = self.and_many(&not_bits)?;
//             // Assume big‑endian order: the first bit is the sign bit.
//             let sign_bit = bits[0].clone();
//             let leq = self.or(&is_zero, &sign_bit)?;
//             let masked_leq = self.xor(&leq, mask)?;
//             results.push(masked_leq);
//         }
//         Ok(BinaryBundle::new(results))
//     }
// }

// /// Implement BinaryGadgets for Garbler.
// impl<C, R, S, W> BinaryGadgets
//     for fancy_garbling::twopac::semihonest::Garbler<C, R, S, W>
// where
//     Self: FancyBinary + BundleGadgets,
// {
// }

// /// Implement BinaryGadgets for Evaluator.
// impl<C, R, S, W> BinaryGadgets
//     for fancy_garbling::twopac::semihonest::Evaluator<C, R, S, W>
// where
//     Self: FancyBinary + BundleGadgets,
// {
// }

// /// The garbler's wire exchange method.
// fn gb_set_fancy_inputs<F, E>(gb: &mut F, input: &[u16], num_tests: usize) -> EQInputs<F::Item>
// where
//     F: FancyInput<Item = AllWire, Error = E>,
//     E: Debug,
// {
//     // The garbler encodes its input (which includes a random mask bit appended per test).
//     let garbler_wires: BinaryBundle<F::Item> =
//         gb.encode_bundle(&input, &vec![2; input.len()])
//             .map(BinaryBundle::from)
//             .unwrap();
//     // The evaluator receives their input labels via OT.
//     let evaluator_wires: BinaryBundle<F::Item> =
//         gb.bin_receive(input.len() - num_tests).unwrap();

//     EQInputs {
//         garbler_wires,
//         evaluator_wires,
//     }
// }

// /// The evaluator's wire exchange method.
// fn ev_set_fancy_inputs<F, E>(ev: &mut F, input: &[u16], num_tests: usize) -> EQInputs<F::Item>
// where
//     F: FancyInput<Item = AllWire, Error = E>,
//     E: Debug,
// {
//     let nwires = input.len();
//     // The evaluator receives the garbler's input labels.
//     let garbler_wires: BinaryBundle<F::Item> = ev.bin_receive(nwires + num_tests).unwrap();
//     // The evaluator encodes its own input (the actual x bits).
//     let evaluator_wires: BinaryBundle<F::Item> =
//         ev.encode_bundle(input, &vec![2; nwires])
//             .map(BinaryBundle::from)
//             .unwrap();

//     EQInputs {
//         garbler_wires,
//         evaluator_wires,
//     }
// }

// /// Fancy LEQ-zero test using garbled circuits.
// /// This computes, for each shared input x, (x ≤ 0) where x is interpreted as a signed integer.
// /// It calls the new gadget which operates on the two bundles.
// fn fancy_leq_zero<F>(
//     f: &mut F,
//     wire_inputs: EQInputs<F::Item>,
//     num_tests: usize,
// ) -> Result<BinaryBundle<F::Item>, F::Error>
// where
//     F: FancyReveal + Fancy + BinaryGadgets + FancyBinary + FancyArithmetic,
// {
//     let leq_bits = f.multi_bin_leq_zero_bundles_shared(
//         &wire_inputs.garbler_wires,
//         &wire_inputs.evaluator_wires,
//         num_tests,
//     )?;
//     Ok(leq_bits)
// }

// /// Garbler wrapper for the LEQ-zero test.
// pub fn multiple_gb_leq_zero_test<C>(
//     rng: &mut AesRng,
//     channel: &mut C,
//     inputs: &[Vec<u16>],
// ) -> Vec<bool>
// where
//     C: AbstractChannel + Clone,
// {
//     let num_tests = inputs.len();
//     let mut results = Vec::with_capacity(num_tests);
//     let mut gb = Garbler::<C, AesRng, OtSender, AllWire>::new(channel.clone(), rng.clone())
//         .unwrap();

//     let masked_inputs = inputs
//         .iter()
//         .map(|input| {
//             let mask = rng.clone().gen_bool();
//             results.push(mask);
//             [input.as_slice(), &[mask as u16]].concat()
//         })
//         .collect::<Vec<Vec<u16>>>();

//     let wire_inputs = masked_inputs.into_iter().flatten().collect::<Vec<u16>>();
//     let wires = gb_set_fancy_inputs(&mut gb, wire_inputs.as_slice(), inputs.len());
//     let leq = fancy_leq_zero(&mut gb, wires, num_tests).unwrap();
//     gb.outputs(leq.wires()).unwrap();
//     channel.flush().unwrap();
//     let mut ack = [0u8; 1];
//     channel.read_bytes(&mut ack).unwrap();
//     results
// }

// /// Evaluator wrapper for the LEQ-zero test.
// pub fn multiple_ev_leq_zero_test<C>(
//     rng: &mut AesRng,
//     channel: &mut C,
//     inputs: &[Vec<u16>],
// ) -> Vec<bool>
// where
//     C: AbstractChannel + Clone,
// {
//     let num_tests = inputs.len();
//     let mut ev = Evaluator::<C, AesRng, OtReceiver, AllWire>::new(channel.clone(), rng.clone())
//         .unwrap();
//     let input_vec = inputs.to_vec().into_iter().flatten().collect::<Vec<u16>>();
//     let wires = ev_set_fancy_inputs(&mut ev, &input_vec, num_tests);
//     let leq = fancy_leq_zero(&mut ev, wires, num_tests).unwrap();
//     let output = ev.outputs(leq.wires()).unwrap().unwrap();
//     let results = output.iter().map(|r| *r == 1).collect();

//     channel.write_bytes(&[1u8]).unwrap();
//     channel.flush().unwrap();

//     results
// }

// #[test]
// fn leq_gc() {
//     // For testing we use 4-bit signed numbers (in two's complement, big-endian):
//     // •  [0,1,1,0] represents 6 (positive)      --> 6 > 0, so test yields false.
//     // •  [0,0,0,0] represents 0    (zero)         --> 0 ≤ 0, so test yields true.
//     // •  [1,1,1,0] represents -2 (negative)       --> -2 ≤ 0, so test yields true.
//     let gb_value = vec![vec![0, 1, 1, 0], vec![0, 0, 0, 0], vec![1, 1, 1, 0]];
//     let ev_value = vec![vec![0, 1, 1, 0], vec![0, 0, 0, 0], vec![1, 1, 1, 0]];
//     let expected = vec![false, true, true];

//     // Create a bidirectional channel between garbler and evaluator.
//     let (sender, receiver) = std::os::unix::net::UnixStream::pair().unwrap();
//     let (result_sender, result_receiver) = std::sync::mpsc::channel();

//     // Spawn the garbler thread.
//     let garbler_thread = std::thread::spawn(move || {
//         let mut rng_gb = AesRng::new();
//         let reader = std::io::BufReader::new(sender.try_clone().unwrap());
//         let writer = std::io::BufWriter::new(sender);
//         let mut channel = Channel::new(reader, writer);
//         let masks = multiple_gb_leq_zero_test(&mut rng_gb, &mut channel, gb_value.as_slice());
//         result_sender.send(masks).unwrap();
//     });

//     // Evaluator runs in the main thread.
//     let mut rng_ev = AesRng::new();
//     let reader = std::io::BufReader::new(receiver.try_clone().unwrap());
//     let writer = std::io::BufWriter::new(receiver);
//     let mut channel = Channel::new(reader, writer);
//     let ev_results = multiple_ev_leq_zero_test(&mut rng_ev, &mut channel, ev_value.as_slice());
//     let gb_masks = result_receiver.recv().unwrap();

//     // Ensure the garbler thread has finished.
//     garbler_thread.join().unwrap();

//     // Check that we obtained the same number of results.
//     assert_eq!(
//         gb_masks.len(),
//         ev_results.len(),
//         "Masks and results should have the same length"
//     );

//     // The true output is given by the XOR of the garbler's mask and the evaluator's output.
//     for i in 0..ev_results.len() {
//         let unmasked_result = gb_masks[i] ^ ev_results[i];
//         assert_eq!(
//             unmasked_result as u16,
//             expected[i] as u16,
//             "Test failed at index {}: expected {} but got {}",
//             i,
//             expected[i] as u16,
//             unmasked_result as u16
//         );
//     }
// }


use fancy_garbling::{
    twopac::semihonest::{Evaluator, Garbler},
    util, AllWire, BinaryBundle, BundleGadgets, Fancy, FancyArithmetic, FancyBinary, FancyInput,
    FancyReveal,
};
use ocelot::{ot::AlszReceiver as OtReceiver, ot::AlszSender as OtSender};
use scuttlebutt::{AbstractChannel, AesRng, Channel, SyncChannel};
use std::fmt::Debug;
use std::{
    io::{BufReader, BufWriter},
    os::unix::net::UnixStream,
};
use std::io::{Read, Write};
use std::time::Instant;
use fancy_garbling::util::RngExt;
use ocelot::ot::Sender;
use rayon::prelude::*;

/// A structure that contains both the garbler and evaluator wires.
///
/// For this protocol the two parties supply separate numeric shares.
/// The garbler’s bundle contains, for each test, the numeric bits (first) and then an extra mask bit.
/// The evaluator’s bundle contains only the numeric bits.
struct EQInputs<F> {
    pub garbler_wires: BinaryBundle<F>,
    pub evaluator_wires: BinaryBundle<F>,
}

/// Extension trait for `FancyBinary` providing gadgets that operate over binary bundles.
pub trait BinaryGadgets: FancyBinary + BundleGadgets {
    fn bin_eq_bundles(
        &mut self,
        x: &BinaryBundle<Self::Item>,
        y: &BinaryBundle<Self::Item>,
    ) -> Result<Self::Item, Self::Error> {
        let zs = x
            .wires()
            .iter()
            .zip(y.wires().iter())
            .map(|(x_bit, y_bit)| {
                let xy = self.xor(x_bit, y_bit)?;
                self.negate(&xy)
            })
            .collect::<Result<Vec<Self::Item>, Self::Error>>()?;
        self.and_many(&zs)
    }

    fn bin_eq_bundles_shared(
        &mut self,
        x: &BinaryBundle<Self::Item>,
        y: &BinaryBundle<Self::Item>,
    ) -> Result<Self::Item, Self::Error> {
        assert_eq!(
            x.wires().len(),
            y.wires().len() + 1,
            "x must have one more wire than y"
        );
        let (x_wires, mask) = x.wires().split_at(x.wires().len() - 1);
        let mask = &mask[0]; // Last wire is the mask.
        let eq_result = self.bin_eq_bundles(&BinaryBundle::new(x_wires.to_vec()), y)?;
        self.xor(&eq_result, mask) // Obscure the output with the mask.
    }

    fn add_binary_bundles<FN>(
        f: &mut FN,
        a: &BinaryBundle<FN::Item>,
        b: &BinaryBundle<FN::Item>,
    ) -> Result<BinaryBundle<FN::Item>, FN::Error>
    where
        FN: FancyBinary + Fancy,
    {
        let n = a.wires().len();
        assert_eq!(
            n,
            b.wires().len(),
            "Bundles must be of the same length for addition"
        );

        // We process from the least significant bit to the most significant bit.
        // Since the bits are in big‑endian order, we iterate from the end to the beginning.
        let mut rev = Vec::with_capacity(n);
        //let mut carry = f.constant(false)?; // initialize carry to 0
        let mut carry = f.constant(0, 2)?; // binary 0 with modulus 2
        for i in (0..n).rev() {
            let ai = a.wires()[i].clone();
            let bi = b.wires()[i].clone();
            // Compute sum bit: sum = ai XOR bi XOR carry
            let axb = f.xor(&ai, &bi)?;
            let sum = f.xor(&axb, &carry)?;
            // Compute carry out: carry_out = (ai AND bi) OR (carry AND (ai XOR bi))
            let ab = f.and(&ai, &bi)?;
            let carry_and_axb = f.and(&carry, &axb)?;
            let new_carry = f.or(&ab, &carry_and_axb)?;
            rev.push(sum);
            carry = new_carry;
        }
        // Reverse the result to recover big‑endian order.
        rev.reverse();
        Ok(BinaryBundle::new(rev))
    }
    

    fn multi_bin_eq_bundles_shared(
        &mut self,
        x: &BinaryBundle<Self::Item>,
        y: &BinaryBundle<Self::Item>,
        num_tests: usize,
    ) -> Result<BinaryBundle<Self::Item>, Self::Error> {
        assert_eq!(
            x.wires().len(),
            y.wires().len() + num_tests,
            "each string in x must have one extra mask bit"
        );
        assert_eq!(y.wires().len() % num_tests, 0);
        let string_len = y.wires().len() / num_tests;
        let mut results = Vec::with_capacity(num_tests);
        for i in 0..num_tests {
            let x_start = i * (string_len + 1);
            let y_start = i * string_len;
            let eq_result = self.bin_eq_bundles(
                &BinaryBundle::new(x.wires()[x_start..x_start + string_len].to_vec()),
                &BinaryBundle::new(y.wires()[y_start..y_start + string_len].to_vec()),
            )?;
            let masked_result = self.xor(&eq_result, &x.wires()[x_start + string_len])?;
            results.push(masked_result);
        }
        Ok(BinaryBundle::new(results))
    }

    /// New gadget: computes (x ≤ 0) where x is interpreted as a two's complement number in big‑endian form.
    ///
    /// In this version x is the sum of evaluator’s and garbler’s inputs. The garbler’s
    /// bundle (first argument) contains the sum bits with an extra mask bit appended per test;
    /// the evaluator’s bundle (second argument) contains only the sum bits.
    ///
    /// The idea is:
    ///   • Compute `is_zero` = AND(negation of every bit) for the sum bits.
    ///   • The sign bit is the first bit (big‑endian; 1 indicates a negative number).
    ///   • Then, (sum ≤ 0) if either the number is zero or the sign bit is 1.
    /// Finally, the output is masked by XORing with the mask from the garbler.
    fn multi_bin_leq_zero_bundles_shared(
        &mut self,
        x: &BinaryBundle<Self::Item>, // Garbler's bundle: [sum bits, mask]
        y: &BinaryBundle<Self::Item>, // Evaluator’s bundle: [sum bits only]
        num_tests: usize,
    ) -> Result<BinaryBundle<Self::Item>, Self::Error> {
        assert_eq!(
            x.wires().len(),
            y.wires().len() + num_tests,
            "each string in x must have one extra mask bit"
        );
        let string_len = y.wires().len() / num_tests;
        let mut results = Vec::with_capacity(num_tests);
        for i in 0..num_tests {
            let x_start = i * (string_len + 1);
            let y_start = i * string_len;
            let mask = &x.wires()[x_start + string_len]; // mask bit from garbler.
            let bits = &y.wires()[y_start..y_start + string_len]; // computed sum bits.
            let mut not_bits = Vec::with_capacity(string_len);
            for b in bits {
                let nb = self.negate(b)?;
                not_bits.push(nb);
            }
            let is_zero = self.and_many(&not_bits)?;
            // The sign bit is the first bit (big‑endian): 1 indicates negative.
            let sign_bit = bits[0].clone();
            let leq = self.or(&is_zero, &sign_bit)?;
            let masked_leq = self.xor(&leq, mask)?;
            results.push(masked_leq);
        }
        Ok(BinaryBundle::new(results))
    }
}

/// Implement BinaryGadgets for Garbler.
impl<C, R, S, W> BinaryGadgets
    for fancy_garbling::twopac::semihonest::Garbler<C, R, S, W>
where
    Self: FancyBinary + BundleGadgets,
{
}

/// Implement BinaryGadgets for Evaluator.
impl<C, R, S, W> BinaryGadgets
    for fancy_garbling::twopac::semihonest::Evaluator<C, R, S, W>
where
    Self: FancyBinary + BundleGadgets,
{
}

/// The garbler’s wire exchange method.
///
/// Here the garbler’s input is expected to contain, for each test, first the numeric bits
/// and then an extra mask bit.
fn gb_set_fancy_inputs<F, E>(gb: &mut F, input: &[u16], num_tests: usize) -> EQInputs<F::Item>
where
    F: FancyInput<Item = AllWire, Error = E>,
    E: Debug,
{
    // Encode the entire garbler input (numeric bits + mask bits per test).
    let garbler_wires: BinaryBundle<F::Item> =
        gb.encode_bundle(input, &vec![2; input.len()])
            .map(BinaryBundle::from)
            .unwrap();
    // Compute the number of numeric wires per test:
    let num_numeric = (input.len() / num_tests) - 1;
    // The evaluator’s wires (obtained via OT) are for the numeric bits only.
    let evaluator_wires: BinaryBundle<F::Item> = gb.bin_receive(num_numeric).unwrap();

    EQInputs {
        garbler_wires,
        evaluator_wires,
    }
}

/// The evaluator’s wire exchange method.
///
/// Here the evaluator’s input is expected to contain only the numeric bits.
fn ev_set_fancy_inputs<F, E>(ev: &mut F, input: &[u16], num_tests: usize) -> EQInputs<F::Item>
where
    F: FancyInput<Item = AllWire, Error = E>,
    E: Debug,
{
    let nwires = input.len();
    // The evaluator first receives the garbler’s shared bundle (numeric bits plus mask bits).
    let garbler_wires: BinaryBundle<F::Item> = ev.bin_receive(nwires + num_tests).unwrap();
    let evaluator_wires: BinaryBundle<F::Item> =
        ev.encode_bundle(input, &vec![2; nwires])
            .map(BinaryBundle::from)
            .unwrap();
    EQInputs {
        garbler_wires,
        evaluator_wires,
    }
}

/// Fancy LEQ-zero test on the sum of evaluator and garbler inputs.
/// This function first extracts the numeric (unmasked) shares from the two EQInputs
/// and then adds them (using the provided gadget). It then reattaches the garbler’s mask
/// to form the “shared” sum and finally performs the ≤ 0 check.
///
/// In other words, if the two parties supply x (garbler) and y (evaluator), then
/// this computes (x + y ≤ 0).
fn fancy_leq_zero<F>(
    f: &mut F,
    wire_inputs: EQInputs<F::Item>,
    num_tests: usize,
) -> Result<BinaryBundle<F::Item>, F::Error>
where
    F: FancyReveal + Fancy + BinaryGadgets + FancyBinary + FancyArithmetic,
{
    // The garbler’s bundle contains (for each test): numeric bits followed by one mask bit.
    let total_garbler = wire_inputs.garbler_wires.wires();
    let l_plus_1 = total_garbler.len() / num_tests; // each test has (l numeric bits + 1 mask bit)
    let numeric_len = l_plus_1 - 1;
    // Extract the numeric share and mask bits from the garbler’s wires.
    let mut gb_numeric = Vec::with_capacity(num_tests * numeric_len);
    let mut gb_mask = Vec::with_capacity(num_tests);
    for i in 0..num_tests {
        let start = i * l_plus_1;
        gb_numeric.extend_from_slice(&total_garbler[start..start + numeric_len]);
        gb_mask.push(total_garbler[start + numeric_len].clone());
    }
    let gb_numeric_bundle = BinaryBundle::new(gb_numeric);
    // The evaluator’s wires are already the numeric share.
    let ev_numeric_bundle = wire_inputs.evaluator_wires;
    // Compute the sum of the numeric shares.
    let sum_bundle =
        <F as BinaryGadgets>::add_binary_bundles(&mut *f, &gb_numeric_bundle, &ev_numeric_bundle)?;
    // For the garbler’s view of the sum, append the mask bit (one per test).
    let sum_wires = sum_bundle.wires();
    let mut gb_sum_wires = Vec::with_capacity(num_tests * (numeric_len + 1));
    for i in 0..num_tests {
        let start = i * numeric_len;
        gb_sum_wires.extend_from_slice(&sum_wires[start..start + numeric_len]);
        gb_sum_wires.push(gb_mask[i].clone());
    }
    let gb_sum_bundle = BinaryBundle::new(gb_sum_wires);
    // Now perform the ≤ 0 check on the sum.
    f.multi_bin_leq_zero_bundles_shared(&gb_sum_bundle, &sum_bundle, num_tests)
}

/// Garbler wrapper for the LEQ-zero test on the sum of inputs.
///
/// Here, `gb_inputs` is an array (one per test) of the garbler’s numeric inputs (in bit form).
/// For each test a random mask bit is generated and appended.
pub fn multiple_gb_leq_zero_test<C>(
    rng: &mut AesRng,
    channel: &mut C,
    gb_inputs: &[Vec<u16>],
) -> Vec<bool>
where
    C: AbstractChannel + Clone,
{
    let num_tests = gb_inputs.len();
    let mut mask_results = Vec::with_capacity(num_tests);
    // For each test, append a random mask bit.
    let masked_inputs = gb_inputs
        .iter()
        .map(|input| {
            let mask = rng.gen_bool();
            mask_results.push(mask);
            [input.as_slice(), &[mask as u16]].concat()
        })
        .collect::<Vec<Vec<u16>>>();
    let flat_inputs = masked_inputs.into_iter().flatten().collect::<Vec<u16>>();
    let mut gb = Garbler::<C, AesRng, OtSender, AllWire>::new(channel.clone(), rng.clone())
        .unwrap();
    let wires = gb_set_fancy_inputs(&mut gb, flat_inputs.as_slice(), num_tests);
    // Note that fancy_leq_zero now internally computes the sum and then applies the leq‑0 gadget.
    let leq = fancy_leq_zero(&mut gb, wires, num_tests).unwrap();
    gb.outputs(leq.wires()).unwrap();
    channel.flush().unwrap();
    let mut ack = [0u8; 1];
    channel.read_bytes(&mut ack).unwrap();
    mask_results
}

/// Evaluator wrapper for the LEQ-zero test on the sum of inputs.
///
/// Here, `ev_inputs` is an array (one per test) of the evaluator’s numeric inputs (in bit form).
pub fn multiple_ev_leq_zero_test<C>(
    rng: &mut AesRng,
    channel: &mut C,
    ev_inputs: &[Vec<u16>],
) -> Vec<bool>
where
    C: AbstractChannel + Clone,
{
    let num_tests = ev_inputs.len();
    let flat_inputs = ev_inputs.iter().flatten().cloned().collect::<Vec<u16>>();
    let mut ev = Evaluator::<C, AesRng, OtReceiver, AllWire>::new(channel.clone(), rng.clone())
        .unwrap();
    let wires = ev_set_fancy_inputs(&mut ev, flat_inputs.as_slice(), num_tests);
    let leq = fancy_leq_zero(&mut ev, wires, num_tests).unwrap();
    let output = ev.outputs(leq.wires()).unwrap().unwrap();
    let results = output.iter().map(|r| *r == 1).collect();
    channel.write_bytes(&[1u8]).unwrap();
    channel.flush().unwrap();
    results
}

// #[test]
// fn leq_gc() {
//     // For testing we use 4-bit signed numbers (in two's complement, big‑endian):
//     // •  [0,1,1,0] represents 6 (positive)      --> 6 > 0, so (6 ≤ 0) yields false.
//     // •  [0,0,0,0] represents 0    (zero)         --> 0 ≤ 0 yields true.
//     // •  [1,1,1,0] represents -2 (negative)       --> -2 ≤ 0 yields true.
//     //
//     // In the new shared-input version the sum is computed as:
//     //     sum = garbler_input + evaluator_input.
//     // To recover the original numbers we let the evaluator’s share be 0.
//     let gb_value = vec![
//         vec![0, 1, 1, 0], // 6
//         vec![0, 0, 0, 0], // 0
//         vec![1, 1, 1, 0]  // -2
//     ];
//     let ev_value = vec![
//         vec![0, 0, 0, 0], // 0 share for 6 → sum = 6
//         vec![0, 0, 0, 0], // 0 share for 0 → sum = 0
//         vec![0, 0, 0, 0]  // 0 share for -2 → sum = -2
//     ];
//     // Expected results are based on the sum being 6 (false), 0 (true), -2 (true).
//     let expected = vec![false, true, true];

//     // Create a bidirectional channel between garbler and evaluator.
//     let (sender, receiver) = std::os::unix::net::UnixStream::pair().unwrap();
//     let (result_sender, result_receiver) = std::sync::mpsc::channel();

//     // Spawn the garbler thread.
//     let garbler_thread = std::thread::spawn(move || {
//         let mut rng_gb = AesRng::new();
//         let reader = std::io::BufReader::new(sender.try_clone().unwrap());
//         let writer = std::io::BufWriter::new(sender);
//         let mut channel = Channel::new(reader, writer);
//         // This function internally appends a random mask to each garbler input and
//         // later uses it to obscure the final output.
//         let masks = multiple_gb_leq_zero_test(&mut rng_gb, &mut channel, gb_value.as_slice());
//         result_sender.send(masks).unwrap();
//     });

//     // Evaluator runs in the main thread.
//     let mut rng_ev = AesRng::new();
//     let reader = std::io::BufReader::new(receiver.try_clone().unwrap());
//     let writer = std::io::BufWriter::new(receiver);
//     let mut channel = Channel::new(reader, writer);
//     let ev_results = multiple_ev_leq_zero_test(&mut rng_ev, &mut channel, ev_value.as_slice());
//     let gb_masks = result_receiver.recv().unwrap();

//     // Ensure the garbler thread has finished.
//     garbler_thread.join().unwrap();

//     // The true output is recovered by XOR’ing the garbler’s mask and the evaluator’s output.
//     // (Recall that the garbler initially masked the computed bit.)
//     assert_eq!(
//         gb_masks.len(),
//         ev_results.len(),
//         "Masks and results should have the same length"
//     );
//     for i in 0..ev_results.len() {
//         let unmasked_result = gb_masks[i] ^ ev_results[i];
//         assert_eq!(
//             unmasked_result as u16,
//             expected[i] as u16,
//             "Test failed at index {}: expected {} but got {}",
//             i,
//             expected[i] as u16,
//             unmasked_result as u16
//         );
//     }
// }
