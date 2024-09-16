use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance, Selector},
    poly::Rotation,
    dev::MockProver,
    pasta::Fp,
};
use std::marker::PhantomData;

// Define the chip for our multiplication circuit
struct MultiplicationChip<F: FieldExt> {
    config: MultiplicationConfig,
    _marker: PhantomData<F>,
}

// Configuration for our multiplication chip
#[derive(Clone, Debug)]
struct MultiplicationConfig {
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,
    selector: Selector,
    instance: Column<Instance>,
}

// Implementation of the multiplication chip
impl<F: FieldExt> MultiplicationChip<F> {
    fn construct(config: MultiplicationConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> MultiplicationConfig {
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();
        let selector = meta.selector();
        let instance = meta.instance_column();

        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(c);
        meta.enable_equality(instance);

        meta.create_gate("multiplication", |meta| {
            let s = meta.query_selector(selector);
            let a = meta.query_advice(a, Rotation::cur());
            let b = meta.query_advice(b, Rotation::cur());
            let c = meta.query_advice(c, Rotation::cur());
            vec![s * (a * b - c)]
        });

        MultiplicationConfig {
            a,
            b,
            c,
            selector,
            instance,
        }
    }

    fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        a: F,
        b: F,
    ) -> Result<AssignedCell<F, F>, Error> {
        layouter.assign_region(
            || "multiplication",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;

                region.assign_advice(|| "a", self.config.a, 0, || Value::known(a))?;
                region.assign_advice(|| "b", self.config.b, 0, || Value::known(b))?;
                region.assign_advice(|| "c", self.config.c, 0, || Value::known(a * b))
            },
        )
    }

    fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        cell: AssignedCell<F, F>,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell.cell(), self.config.instance, row)
    }
}

// Define our circuit
#[derive(Default)]
struct MultiplicationCircuit<F: FieldExt> {
    a: F,
    b: F,
}

impl<F: FieldExt> Circuit<F> for MultiplicationCircuit<F> {
    type Config = MultiplicationConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        MultiplicationChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = MultiplicationChip::construct(config);

        let c = chip.assign(layouter.namespace(|| "assign multiplication"), self.a, self.b)?;
        chip.expose_public(layouter.namespace(|| "expose c"), c, 0)?;

        Ok(())
    }
}

fn main() {
    // Run the test case
    let result = test_multiplication_circuit();
    match result {
        Ok(_) => println!("Test passed successfully!"),
        Err(e) => println!("Test failed: {:?}", e),
    }
}

fn test_multiplication_circuit() -> Result<(), Box<dyn std::error::Error>> {
    // Use Fp as our field
    type F = Fp;

    // Set up the circuit
    let a = F::from(3);
    let b = F::from(4);
    let c = a * b;

    let circuit = MultiplicationCircuit { a, b };

    // Set up the public input (instance)
    let public_inputs = vec![c];

    // Run the mock prover
    let prover = MockProver::run(4, &circuit, vec![public_inputs])?;

    // Verify the circuit
    prover.assert_satisfied();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiplication() {
        assert!(test_multiplication_circuit().is_ok());
    }
}