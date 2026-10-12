use super::decision::{Graph, Id, Operation};

#[test]
fn all_boolean_functions_obey_set_operations() -> Result<(), &'static str> {
    let mut graph = Graph::new(1_000_000);
    let x = graph.variable(1)?;
    let y = graph.variable(1)?;
    let mut points = Vec::new();
    for value in 0..4 {
        let a = graph.interval(x, value & 1, value & 1)?;
        let b = graph.interval(y, value >> 1, value >> 1)?;
        points.push(graph.apply(Operation::Intersection, a, b)?);
    }
    let mut functions = Vec::new();
    for mask in 0_u8..16 {
        let mut set = Id::EMPTY;
        for (point, &id) in points.iter().enumerate() {
            if mask & (1 << point) != 0 {
                set = graph.apply(Operation::Union, set, id)?;
            }
        }
        functions.push(set);
    }
    for a in 0..16 {
        for b in 0..16 {
            for operation in [
                Operation::Union,
                Operation::Intersection,
                Operation::Difference,
            ] {
                let result = graph.apply(operation, functions[a], functions[b])?;
                let mask = match operation {
                    Operation::Union => a | b,
                    Operation::Intersection => a & b,
                    Operation::Difference => a & !b,
                };
                assert_eq!(result, functions[mask]);
                for value in 0..4 {
                    assert_eq!(
                        graph.contains(result, &[value & 1, value >> 1])?,
                        mask & (1 << value) != 0
                    );
                }
                if result != Id::EMPTY {
                    let mut values = [0, 0];
                    for (variable, rank) in graph.witness(result)? {
                        values[variable.0 as usize] = rank;
                    }
                    assert!(graph.contains(result, &values)?);
                }
            }
        }
    }
    Ok(())
}

#[test]
fn interval_splits_preserve_u128_endpoints_and_budget() -> Result<(), &'static str> {
    let mut graph = Graph::new(10_000);
    let variable = graph.variable(u128::MAX)?;
    let low = graph.interval(variable, 0, u128::MAX - 1)?;
    let high = graph.interval(variable, u128::MAX, u128::MAX)?;
    assert_eq!(graph.apply(Operation::Union, low, high)?, Id::ALL);
    assert_eq!(graph.apply(Operation::Difference, Id::ALL, low)?, high);
    assert_eq!(graph.witness(high)?, [(variable, u128::MAX)]);
    assert!(graph.work() < 5_000);
    let mut bounded = Graph::new(0);
    assert_eq!(
        bounded.variable(1),
        Err("pattern coverage work budget exceeded")
    );
    Ok(())
}

#[test]
fn non_power_of_two_domains_do_not_invent_values() -> Result<(), &'static str> {
    for maximum in 0..16 {
        let mut graph = Graph::new(1_000_000);
        let variable = graph.variable(maximum)?;
        for lower in 0..=maximum {
            for upper in lower..=maximum {
                let interval = graph.interval(variable, lower, upper)?;
                let outside = graph.apply(Operation::Difference, Id::ALL, interval)?;
                for value in 0..=maximum {
                    assert_eq!(
                        graph.contains(interval, &[value])?,
                        (lower..=upper).contains(&value)
                    );
                    assert_eq!(
                        graph.contains(outside, &[value])?,
                        !(lower..=upper).contains(&value)
                    );
                }
                if outside != Id::EMPTY {
                    let witness = graph.witness(outside)?;
                    assert!(witness.iter().all(|(_, rank)| *rank <= maximum));
                    let value = witness.first().map_or(0, |(_, rank)| *rank);
                    assert!(!(lower..=upper).contains(&value));
                }
            }
        }
    }
    Ok(())
}
