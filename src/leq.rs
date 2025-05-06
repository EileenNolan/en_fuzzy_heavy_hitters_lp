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
/// This structure simplifies the API for wiring the circuit.
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

    /// New gadget: computes (x ≤ 0) for a shared number x in two's complement (big‑endian).
    /// Here x is provided by the evaluator’s bundle while the garbler’s bundle holds an extra mask bit.
    /// The implementation uses a simple idea:
    ///   • Compute `is_zero` by AND’ing the negation of every bit of x.
    ///   • Assume that the first bit (big‑endian) is the sign bit.
    ///   • Then, x ≤ 0 if x = 0 or if the sign bit is 1.
    /// Finally, the computed bit is masked (XOR’d with the mask from the garbler).
    fn multi_bin_leq_zero_bundles_shared(
        &mut self,
        x: &BinaryBundle<Self::Item>, // Garbler's bundle: [x bits, mask]
        y: &BinaryBundle<Self::Item>, // Evaluator's bundle: [x bits only]
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
            let mask = &x.wires()[x_start + string_len]; // Last bit of the current chunk.
            let bits = &y.wires()[y_start..y_start + string_len]; // The actual bits of x.
            let mut not_bits = Vec::with_capacity(string_len);
            for b in bits {
                let nb = self.negate(b)?;
                not_bits.push(nb);
            }
            let is_zero = self.and_many(&not_bits)?;
            // Assume big‑endian order: the first bit is the sign bit.
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

/// The garbler's wire exchange method.
fn gb_set_fancy_inputs<F, E>(gb: &mut F, input: &[u16], num_tests: usize) -> EQInputs<F::Item>
where
    F: FancyInput<Item = AllWire, Error = E>,
    E: Debug,
{
    // The garbler encodes its input (which includes a random mask bit appended per test).
    let garbler_wires: BinaryBundle<F::Item> =
        gb.encode_bundle(&input, &vec![2; input.len()])
            .map(BinaryBundle::from)
            .unwrap();
    // The evaluator receives their input labels via OT.
    let evaluator_wires: BinaryBundle<F::Item> =
        gb.bin_receive(input.len() - num_tests).unwrap();

    EQInputs {
        garbler_wires,
        evaluator_wires,
    }
}

/// The evaluator's wire exchange method.
fn ev_set_fancy_inputs<F, E>(ev: &mut F, input: &[u16], num_tests: usize) -> EQInputs<F::Item>
where
    F: FancyInput<Item = AllWire, Error = E>,
    E: Debug,
{
    let nwires = input.len();
    // The evaluator receives the garbler's input labels.
    let garbler_wires: BinaryBundle<F::Item> = ev.bin_receive(nwires + num_tests).unwrap();
    // The evaluator encodes its own input (the actual x bits).
    let evaluator_wires: BinaryBundle<F::Item> =
        ev.encode_bundle(input, &vec![2; nwires])
            .map(BinaryBundle::from)
            .unwrap();

    EQInputs {
        garbler_wires,
        evaluator_wires,
    }
}

/// Fancy LEQ-zero test using garbled circuits.
/// This computes, for each shared input x, (x ≤ 0) where x is interpreted as a signed integer.
/// It calls the new gadget which operates on the two bundles.
fn fancy_leq_zero<F>(
    f: &mut F,
    wire_inputs: EQInputs<F::Item>,
    num_tests: usize,
) -> Result<BinaryBundle<F::Item>, F::Error>
where
    F: FancyReveal + Fancy + BinaryGadgets + FancyBinary + FancyArithmetic,
{
    let leq_bits = f.multi_bin_leq_zero_bundles_shared(
        &wire_inputs.garbler_wires,
        &wire_inputs.evaluator_wires,
        num_tests,
    )?;
    Ok(leq_bits)
}

/// Garbler wrapper for the LEQ-zero test.
pub fn multiple_gb_leq_zero_test<C>(
    rng: &mut AesRng,
    channel: &mut C,
    inputs: &[Vec<u16>],
) -> Vec<bool>
where
    C: AbstractChannel + Clone,
{
    let num_tests = inputs.len();
    let mut results = Vec::with_capacity(num_tests);
    let mut gb = Garbler::<C, AesRng, OtSender, AllWire>::new(channel.clone(), rng.clone())
        .unwrap();

    let masked_inputs = inputs
        .iter()
        .map(|input| {
            let mask = rng.clone().gen_bool();
            results.push(mask);
            [input.as_slice(), &[mask as u16]].concat()
        })
        .collect::<Vec<Vec<u16>>>();

    let wire_inputs = masked_inputs.into_iter().flatten().collect::<Vec<u16>>();
    let wires = gb_set_fancy_inputs(&mut gb, wire_inputs.as_slice(), inputs.len());
    let leq = fancy_leq_zero(&mut gb, wires, num_tests).unwrap();
    gb.outputs(leq.wires()).unwrap();
    channel.flush().unwrap();
    let mut ack = [0u8; 1];
    channel.read_bytes(&mut ack).unwrap();
    results
}

/// Evaluator wrapper for the LEQ-zero test.
pub fn multiple_ev_leq_zero_test<C>(
    rng: &mut AesRng,
    channel: &mut C,
    inputs: &[Vec<u16>],
) -> Vec<bool>
where
    C: AbstractChannel + Clone,
{
    let num_tests = inputs.len();
    let mut ev = Evaluator::<C, AesRng, OtReceiver, AllWire>::new(channel.clone(), rng.clone())
        .unwrap();
    let input_vec = inputs.to_vec().into_iter().flatten().collect::<Vec<u16>>();
    let wires = ev_set_fancy_inputs(&mut ev, &input_vec, num_tests);
    let leq = fancy_leq_zero(&mut ev, wires, num_tests).unwrap();
    let output = ev.outputs(leq.wires()).unwrap().unwrap();
    let results = output.iter().map(|r| *r == 1).collect();

    channel.write_bytes(&[1u8]).unwrap();
    channel.flush().unwrap();

    results
}

#[test]
fn leq_gc() {
    // For testing we use 4-bit signed numbers (in two's complement, big-endian):
    // •  [0,1,1,0] represents 6 (positive)      --> 6 > 0, so test yields false.
    // •  [0,0,0,0] represents 0    (zero)         --> 0 ≤ 0, so test yields true.
    // •  [1,1,1,0] represents -2 (negative)       --> -2 ≤ 0, so test yields true.
    let gb_value = vec![vec![0, 1, 1, 0], vec![0, 0, 0, 0], vec![1, 1, 1, 0]];
    let ev_value = vec![vec![0, 1, 1, 0], vec![0, 0, 0, 0], vec![1, 1, 1, 0]];
    let expected = vec![false, true, true];

    // Create a bidirectional channel between garbler and evaluator.
    let (sender, receiver) = std::os::unix::net::UnixStream::pair().unwrap();
    let (result_sender, result_receiver) = std::sync::mpsc::channel();

    // Spawn the garbler thread.
    let garbler_thread = std::thread::spawn(move || {
        let mut rng_gb = AesRng::new();
        let reader = std::io::BufReader::new(sender.try_clone().unwrap());
        let writer = std::io::BufWriter::new(sender);
        let mut channel = Channel::new(reader, writer);
        let masks = multiple_gb_leq_zero_test(&mut rng_gb, &mut channel, gb_value.as_slice());
        result_sender.send(masks).unwrap();
    });

    // Evaluator runs in the main thread.
    let mut rng_ev = AesRng::new();
    let reader = std::io::BufReader::new(receiver.try_clone().unwrap());
    let writer = std::io::BufWriter::new(receiver);
    let mut channel = Channel::new(reader, writer);
    let ev_results = multiple_ev_leq_zero_test(&mut rng_ev, &mut channel, ev_value.as_slice());
    let gb_masks = result_receiver.recv().unwrap();

    // Ensure the garbler thread has finished.
    garbler_thread.join().unwrap();

    // Check that we obtained the same number of results.
    assert_eq!(
        gb_masks.len(),
        ev_results.len(),
        "Masks and results should have the same length"
    );

    // The true output is given by the XOR of the garbler's mask and the evaluator's output.
    for i in 0..ev_results.len() {
        let unmasked_result = gb_masks[i] ^ ev_results[i];
        assert_eq!(
            unmasked_result as u16,
            expected[i] as u16,
            "Test failed at index {}: expected {} but got {}",
            i,
            expected[i] as u16,
            unmasked_result as u16
        );
    }
}
