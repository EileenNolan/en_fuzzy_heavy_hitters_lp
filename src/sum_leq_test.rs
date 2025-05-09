// //! An example that adds two secret numbers in a binary garbled circuit
// //! using fancy-garbling.
// use fancy_garbling::{
//     AllWire, BinaryBundle, Bundle, BinaryGadgets, Fancy, FancyArithmetic, FancyBinary, FancyInput,
//     FancyReveal,
//     twopac::semihonest::{Evaluator, Garbler},
//     util,
// };

// use ocelot::{ot::AlszReceiver as OtReceiver, ot::AlszSender as OtSender};
// use scuttlebutt::{AbstractChannel, AesRng, Channel};

// use std::fmt::Debug;
// use std::{
//     io::{BufReader, BufWriter},
//     os::unix::net::UnixStream,
// };

// /// Structure that contains both the garbler and evaluator wires.
// struct SUMInputs<F> {
//     pub garbler_wires: BinaryBundle<F>,
//     pub evaluator_wires: BinaryBundle<F>,
// }

// /// Extracts `num_bits` starting at position `start_bit` from a `BinaryBundle`.
// fn extract<F>(
//     bundle: &BinaryBundle<F::Item>,
//     start_bit: usize,
//     num_bits: usize,
// ) -> Result<BinaryBundle<F::Item>, F::Error>
// where
//     F: Fancy,
//     F::Item: Clone,
// {
//     let wires = bundle.wires();
//     let end_bit = start_bit + num_bits;
//     let sliced = wires[start_bit..end_bit].to_vec();
//     Ok(BinaryBundle::from(Bundle::new(sliced)))
// }

// /// **Garbler’s function**: Generates and sends their additive share
// fn gb_sum_additive_msb<C>(rng: &mut AesRng, channel: &mut C, input: u128)
// where
//     C: AbstractChannel + std::clone::Clone,
// {
//     let mut gb =
//         Garbler::<C, AesRng, OtSender, AllWire>::new(channel.clone(), rng.clone()).unwrap();
//     let circuit_wires = gb_set_fancy_inputs(&mut gb, input);
    
//     let (share_garbler, _) = fancy_additive_share_msb::<Garbler<C, AesRng, OtSender, AllWire>>(&mut gb, circuit_wires).unwrap();
    
//     // Send garbler's additive share to evaluator
//     gb.outputs(share_garbler.wires()).unwrap();
// }

// /// **Evaluator’s function**: Receives their additive share
// fn ev_sum_additive_msb<C>(rng: &mut AesRng, channel: &mut C, input: u128) -> u128
// where
//     C: AbstractChannel + std::clone::Clone,
// {
//     let mut ev =
//         Evaluator::<C, AesRng, OtReceiver, AllWire>::new(channel.clone(), rng.clone()).unwrap();
//     let circuit_wires = ev_set_fancy_inputs(&mut ev, input);

//     let (_, share_evaluator) = fancy_additive_share_msb::<Evaluator<C, AesRng, OtReceiver, AllWire>>(&mut ev, circuit_wires).unwrap();
    
//     let bits = ev.outputs(share_evaluator.wires()).unwrap().expect("Evaluator should produce additive share");

//     util::u128_from_bits(&bits)
// }

// /// Garbler's wire exchange function
// fn gb_set_fancy_inputs<F, E>(gb: &mut F, input: u128) -> SUMInputs<F::Item>
// where
//     F: FancyInput<Item = AllWire, Error = E>,
//     E: Debug,
// {
//     let nbits = 128;
//     let garbler_wires: BinaryBundle<F::Item> = gb.bin_encode(input, nbits).unwrap();
//     let evaluator_wires: BinaryBundle<F::Item> = gb.bin_receive(nbits).unwrap();

//     SUMInputs {
//         garbler_wires,
//         evaluator_wires,
//     }
// }

// /// Evaluator's wire exchange function
// fn ev_set_fancy_inputs<F, E>(ev: &mut F, input: u128) -> SUMInputs<F::Item>
// where
//     F: FancyInput<Item = AllWire, Error = E>,
//     E: Debug,
// {
//     let nbits = 128;
//     let garbler_wires: BinaryBundle<F::Item> = ev.bin_receive(nbits).unwrap();
//     let evaluator_wires: BinaryBundle<F::Item> = ev.bin_encode(input, nbits).unwrap();

//     SUMInputs {
//         garbler_wires,
//         evaluator_wires,
//     }
// }

// /// **Secure Additive Secret Sharing of MSB**
// fn fancy_additive_share_msb<F>(
//     f: &mut F,
//     wire_inputs: SUMInputs<F::Item>,
// ) -> Result<(BinaryBundle<F::Item>, BinaryBundle<F::Item>), F::Error>
// where
//     F: FancyReveal + Fancy + BinaryGadgets + FancyBinary + FancyArithmetic,
//     F::Item: Clone, 
// {
//     let sum = f.bin_addition_no_carry(&wire_inputs.garbler_wires, &wire_inputs.evaluator_wires)?;
//     let msb = extract::<F>(&sum, 127, 1)?;

//     //let r_garbler = f.rand_bit()?; 
//     let r_garbler = f.constant(rand::random::<u8>() % 2, 1)?;

//     //let r_evaluator = f.bin_sub(&msb, &r_garbler)?; 
//     let r_evaluator = f.bin_xor(&msb, &r_garbler)?;


//     Ok((BinaryBundle::from(Bundle::new(vec![r_garbler])),
//     BinaryBundle::from(Bundle::new(vec![r_evaluator]))))

// }

// fn sum_in_clear(gb_value: u128, ev_value: u128) -> u128 {
//     gb_value + ev_value
// }

// use clap::Parser;
// #[derive(Parser)]
// struct Cli {
//     gb_value: u128,
//     ev_value: u128,
// }

// /// **Unit Test**
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_additive_secret_share_gc() {
//         let gb_value: u128 = 5;
//         let ev_value: u128 = 6;

//         let (sender, receiver) = UnixStream::pair().unwrap();

//         std::thread::spawn(move || {
//             let rng_gb = AesRng::new();
//             let reader = BufReader::new(sender.try_clone().unwrap());
//             let writer = BufWriter::new(sender);
//             let mut channel = Channel::new(reader, writer);
//             gb_sum_additive_msb(&mut rng_gb.clone(), &mut channel, gb_value);
//         });

//         let rng_ev = AesRng::new();
//         let reader = BufReader::new(receiver.try_clone().unwrap());
//         let writer = BufWriter::new(receiver);
//         let mut channel = Channel::new(reader, writer);

//         let sum = sum_in_clear(gb_value, ev_value);
//         let expected_msb = ((sum as i128) < 0) as u128;

//         let result = ev_sum_additive_msb(&mut rng_ev.clone(), &mut channel, ev_value);

//         assert_eq!(
//             result, expected_msb,
//             "Expected MSB = {}, but got {}",
//             expected_msb, result
//         );
//     }
// }

// /// **Main Function**
// fn main() {
//     let cli = Cli::parse();
//     let gb_value = cli.gb_value;
//     let ev_value = cli.ev_value;

//     let (sender, receiver) = UnixStream::pair().unwrap();

//     std::thread::spawn(move || {
//         let rng_gb = AesRng::new();
//         let reader = BufReader::new(sender.try_clone().unwrap());
//         let writer = BufWriter::new(sender);
//         let mut channel = Channel::new(reader, writer);
//         gb_sum_additive_msb(&mut rng_gb.clone(), &mut channel, gb_value);
//     });

//     let rng_ev = AesRng::new();
//     let reader = BufReader::new(receiver.try_clone().unwrap());
//     let writer = BufWriter::new(receiver);
//     let mut channel = Channel::new(reader, writer);

//     let result = ev_sum_additive_msb(&mut rng_ev.clone(), &mut channel, ev_value);

//     let sum = gb_value.wrapping_add(ev_value);
//     let expected_msb = (sum >> 127) & 1;

//     println!(
//         "Garbled Circuit result is : MSB(SUM({}, {})) = {}",
//         gb_value, ev_value, result
//     );

//     assert_eq!(result, expected_msb, "The result is incorrect. Expected MSB = {expected_msb}");
// }
