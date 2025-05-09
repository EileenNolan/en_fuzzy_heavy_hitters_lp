
//! An example that adds two secret numbers and subtracts a CONSTANT and outputs MSB bit in a binary garbled circuit
//! using fancy-garbling.

use fancy_garbling::{
    AllWire, BinaryBundle, Bundle, BinaryGadgets, Fancy, FancyArithmetic, FancyBinary, FancyInput,
    FancyReveal,
    twopac::semihonest::{Evaluator, Garbler},
    util,
};
use ocelot::{ot::AlszReceiver as OtReceiver, ot::AlszSender as OtSender};
use scuttlebutt::{AbstractChannel, AesRng, Channel};
use std::fmt::Debug;
use std::{
    io::{BufReader, BufWriter},
    os::unix::net::UnixStream,
};

// The constant to subtract from the garbler's input.
// The protocol computes (garbler_input - CONSTANT) + evaluator_input.
// To test for a "negative" result with small inputs such as 5 and 6,
// CONSTANT must be increased such that (garbler_input - CONSTANT + evaluator_input)
// interpreted in two's complement is negative.
const CONSTANT: u128 = 13; // this will be delta^P

// A structure that contains both the garbler and the evaluator's wires.
// This structure simplifies the API of the garbled circuit.
struct SUMInputs<F> {
    pub garbler_wires: BinaryBundle<F>,
    pub evaluator_wires: BinaryBundle<F>,
}

// Extracts `num_bits` starting at position `start_bit` from a `BinaryBundle`.
// Returns a new `BinaryBundle` containing the extracted bits.
fn extract<F>(
    bundle: &BinaryBundle<F::Item>,
    start_bit: usize,
    num_bits: usize,
) -> Result<BinaryBundle<F::Item>, F::Error>
where
    F: Fancy,
    F::Item: Clone,
{
    let wires = bundle.wires();
    let end_bit = start_bit + num_bits;
    let sliced = wires[start_bit..end_bit].to_vec();
    Ok(BinaryBundle::from(Bundle::new(sliced)))
}

// The garbler's main method.
fn gb_sum<C>(rng: &mut AesRng, channel: &mut C, input: u128)
where
    C: AbstractChannel + std::clone::Clone,
{
    // (1) Create the garbler.
    let mut gb =
        Garbler::<C, AesRng, OtSender, AllWire>::new(channel.clone(), rng.clone()).unwrap();
    // (2) Exchange inputs; note that gb_set_fancy_inputs adjusts the input.
    let circuit_wires = gb_set_fancy_inputs(&mut gb, input);
    // (3) Run the circuit that computes the MSB of (garbler_input - CONSTANT + evaluator_input).
    let is_negative = fancy_sum_is_negative::<Garbler<C, AesRng, OtSender, AllWire>>(&mut gb, circuit_wires)
        .unwrap();
    // (4) Send out the result.
    gb.outputs(is_negative.wires()).unwrap();
}

// The garbler's wire exchange method.
// The garbler's input is adjusted by subtracting CONSTANT before encoding.
fn gb_set_fancy_inputs<F, E>(gb: &mut F, input: u128) -> SUMInputs<F::Item>
where
    F: FancyInput<Item = AllWire, Error = E>,
    E: Debug,
{
    let nbits = 128;
    // Adjust the garbler's input using wrapping subtraction (two's complement arithmetic).
    let adjusted_input: u128 = input.wrapping_sub(CONSTANT);
    let garbler_wires: BinaryBundle<F::Item> = gb.bin_encode(adjusted_input, nbits).unwrap();
    let evaluator_wires: BinaryBundle<F::Item> = gb.bin_receive(nbits).unwrap();

    SUMInputs {
        garbler_wires,
        evaluator_wires,
    }
}

// The evaluator's main method.
fn ev_sum<C>(rng: &mut AesRng, channel: &mut C, input: u128) -> u128
where
    C: AbstractChannel + std::clone::Clone,
{
    // (1) Create the evaluator.
    let mut ev =
        Evaluator::<C, AesRng, OtReceiver, AllWire>::new(channel.clone(), rng.clone()).unwrap();
    // (2) Exchange inputs with the garbler.
    let circuit_wires = ev_set_fancy_inputs(&mut ev, input);
    // (3) Compute the MSB indicating negativity.
    let is_negative = fancy_sum_is_negative::<Evaluator<C, AesRng, OtReceiver, AllWire>>(&mut ev, circuit_wires)
        .unwrap();
    // (4) Reveal the result.
    let bits = ev.outputs(is_negative.wires()).unwrap().expect("evaluator should produce outputs");
    // (5) Convert the revealed bits into a u128 result.
    util::u128_from_bits(&bits)
}

// The evaluator's wire exchange method.
fn ev_set_fancy_inputs<F, E>(ev: &mut F, input: u128) -> SUMInputs<F::Item>
where
    F: FancyInput<Item = AllWire, Error = E>,
    E: Debug,
{
    let nbits = 128;
    let garbler_wires: BinaryBundle<F::Item> = ev.bin_receive(nbits).unwrap();
    let evaluator_wires: BinaryBundle<F::Item> = ev.bin_encode(input, nbits).unwrap();

    SUMInputs {
        garbler_wires,
        evaluator_wires,
    }
}

// Computes the garbled circuit function that adds the inputs and extracts the MSB.
fn fancy_sum_is_negative<F>(
    f: &mut F,
    wire_inputs: SUMInputs<F::Item>,
) -> Result<BinaryBundle<F::Item>, F::Error>
where
    F: FancyReveal + Fancy + BinaryGadgets + FancyBinary + FancyArithmetic,
    F::Item: Clone,
{
    let sum = f.bin_addition_no_carry(&wire_inputs.garbler_wires, &wire_inputs.evaluator_wires)?;
    let msb = extract::<F>(&sum, 127, 1)?;
    Ok(msb)
}

/// Computes the clear-text adjusted sum, i.e. (gb_value - CONSTANT) + ev_value.
fn sum_in_clear(gb_value: u128, ev_value: u128) -> u128 {
    gb_value.wrapping_sub(CONSTANT).wrapping_add(ev_value)
}

use clap::Parser;
#[derive(Parser)]
// Example usage: (if in examples folder)
//
// cargo run --example simple_sum 2 3
//
// Computes whether (garbler_value - CONSTANT + evaluator_value) < 0.
struct Cli {
    /// The garbler's value.
    gb_value: u128,
    /// The evaluator's value.
    ev_value: u128,
}

fn main() {
    let cli = Cli::parse();
    let gb_value: u128 = cli.gb_value;
    let ev_value: u128 = cli.ev_value;

    let (sender, receiver) = UnixStream::pair().unwrap();

    std::thread::spawn(move || {
        let rng_gb = AesRng::new();
        let reader = BufReader::new(sender.try_clone().unwrap());
        let writer = BufWriter::new(sender);
        let mut channel = Channel::new(reader, writer);
        gb_sum(&mut rng_gb.clone(), &mut channel, gb_value);
    });

    let rng_ev = AesRng::new();
    let reader = BufReader::new(receiver.try_clone().unwrap());
    let writer = BufWriter::new(receiver);
    let mut channel = Channel::new(reader, writer);

    let result = ev_sum(&mut rng_ev.clone(), &mut channel, ev_value);

    // Compute the expected clear-text result.
    let sum = gb_value.wrapping_sub(CONSTANT).wrapping_add(ev_value);
    let expected_msb = (sum >> 127) & 1;

    println!(
        "Garbled Circuit result is : MSB((( {} - {} ) + {})) = {}",
        gb_value, CONSTANT, ev_value, result
    );
    println!("Clear computed sum (adjusted) = {}", sum);
    println!("Clear computed sum (interpreted as signed i128) = {}", sum as i128);
    println!("Expected MSB (negative flag): {}", expected_msb);

    assert!(
        result == expected_msb,
        "The garbled circuit result is incorrect. Expected MSB = {}",
        expected_msb
    );
}

// To run test: cargo test test_add_leq_binary_gc -- --nocapture
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, BufWriter};

    #[test]
    fn test_add_leq_binary_gc() {
        let gb_value: u128 = 5;
        let ev_value: u128 = 6;

        println!("--- Running test_add_leq_gc ---");
        println!("Garbler value: {}", gb_value);
        println!("Evaluator value: {}", ev_value);
        println!("Constant: {}", CONSTANT);

        let clear_sum = sum_in_clear(gb_value, ev_value);
        println!("Clear computed sum (adjusted) = {}", clear_sum);
        println!("Clear computed sum (interpreted as i128) = {}", clear_sum as i128);
        let expected_msb = (clear_sum >> 127) & 1;
        println!("Expected MSB (negative flag): {}", expected_msb);

        let (sender, receiver) = UnixStream::pair().unwrap();

        std::thread::spawn(move || {
            let rng_gb = AesRng::new();
            let reader = BufReader::new(sender.try_clone().unwrap());
            let writer = BufWriter::new(sender);
            let mut channel = Channel::new(reader, writer);
            gb_sum(&mut rng_gb.clone(), &mut channel, gb_value);
        });

        let rng_ev = AesRng::new();
        let reader = BufReader::new(receiver.try_clone().unwrap());
        let writer = BufWriter::new(receiver);
        let mut channel = Channel::new(reader, writer);

        let result = ev_sum(&mut rng_ev.clone(), &mut channel, ev_value);
        println!("Garbled Circuit computed MSB: {}", result);

        assert_eq!(
            result, expected_msb,
            "Expected MSB = {}, but got {}",
            expected_msb, result
        );
    }
}
