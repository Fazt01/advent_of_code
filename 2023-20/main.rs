use std::collections::HashMap;
use std::io;
use anyhow::{Result, Ok, Context, bail};

#[derive(Clone, Copy, Debug)]
enum Pulse {
    High,
    Low,
}

#[derive(Clone)]
struct PulseTarget {
    target_module: String,
    input_info: ModuleInputInfo,
}

struct PulseEvent {
    pulse: Pulse,
    pulse_target: PulseTarget,
}

#[derive(Copy, Clone)]
enum ModuleInputInfo {
    None,
    Id(usize),
}

struct Puzzle {
    modules: HashMap<String, Module>,
}

struct Module {
    outputs: Vec<PulseTarget>,
    module_type: ModuleType,
}

impl Module {
    fn register_input(&mut self) -> ModuleInputInfo {
        match &mut self.module_type {
            ModuleType::Nand(ref mut nand) => {
                let id = nand.input_lasts.len();
                nand.input_lasts.push(Pulse::Low);
                ModuleInputInfo::Id(id)
            }
            _ => ModuleInputInfo::None
        }
    }

    fn process_pulse(&mut self, pulse: Pulse, info: &ModuleInputInfo) -> Result<Option<Pulse>> {
        let output_pulse = match &mut self.module_type {
            ModuleType::Broadcast => Some(pulse),
            ModuleType::FlipFlop(flip_flop) => {
                if let Pulse::Low = pulse {
                    flip_flop.state = !flip_flop.state;
                    Some(match flip_flop.state {
                        true => Pulse::High,
                        false => Pulse::Low,
                    })
                } else {
                    None
                }
            }
            ModuleType::Nand(nand) => {
                let ModuleInputInfo::Id(id) = info else {
                    bail!("invalid info for nand module")
                };
                nand.input_lasts[*id] = pulse;
                if nand.input_lasts.iter().all(|&x| matches!(x, Pulse::High)) {
                    Some(Pulse::Low)
                } else {
                    Some(Pulse::High)
                }
            }
        };

        Ok(output_pulse)
    }
}

enum ModuleType {
    Broadcast,
    FlipFlop(FlipFlop),
    Nand(Nand),
}

#[derive(Default)]
struct FlipFlop {
    state: bool,
}

#[derive(Default)]
struct Nand {
    input_lasts: Vec<Pulse>,
}

fn main() -> Result<()> {
    let mut puzzle = parse()?;

    let mut pulsed_low: i64 = 0;
    let mut pulsed_high: i64 = 0;
    let button_output = PulseTarget { target_module: "broadcaster".to_owned(), input_info: ModuleInputInfo::None };
    for button_press in 0i64.. {
        let mut current_to_process = vec![PulseEvent {
            pulse: Pulse::Low,
            pulse_target: button_output.clone(),
        }];
        while !current_to_process.is_empty() {
            let mut next_to_process = vec![];
            for current in current_to_process {
                let pulse_target = current.pulse_target;
                match current.pulse {
                    Pulse::High => {
                        // this is NAND with 4 inputs that outputs to rx
                        if pulse_target.target_module == "kh" {
                            if let ModuleInputInfo::Id(id) = pulse_target.input_info {
                                println!("input {} high in button_press {}", id, button_press + 1);
                                // Figure out the lcm outside of code.
                                // Cycles apparently all start at 0.
                            }
                        }
                        pulsed_high += 1;
                    }
                    Pulse::Low => {
                        if pulse_target.target_module == "rx" { // too large, never finishes
                            println!("{}", button_press + 1);
                            break;
                        }
                        pulsed_low += 1;
                    }
                };
                let Some(module) = puzzle.modules.get_mut(&pulse_target.target_module)
                    else {
                        continue;
                    };
                if let Some(output_pulse) = module.process_pulse(current.pulse, &pulse_target.input_info)? {
                    for output in &module.outputs {
                        next_to_process.push(PulseEvent {
                            pulse: output_pulse,
                            pulse_target: output.clone(),
                        })
                    }
                }
            }
            current_to_process = next_to_process;
        }
    }

    println!("{}", pulsed_high * pulsed_low);

    Ok(())
}

fn parse() -> Result<Puzzle> {
    let stdin = io::stdin();
    let mut puzzle = Puzzle {
        modules: Default::default(),
    };
    let mut outputs_map = HashMap::<String, Vec<String>>::new();
    for line in stdin.lines() {
        let line = line?;
        let mut s = line.as_str();
        let module_type = match s.chars().next().context("unexpected empty line")? {
            '%' => {
                s = s.strip_prefix('%').unwrap();
                ModuleType::FlipFlop(FlipFlop::default())
            }
            '&' => {
                s = s.strip_prefix('&').unwrap();
                ModuleType::Nand(Nand::default())
            }
            _ => ModuleType::Broadcast,
        };
        let (name, outputs_str) = s.split_once(" -> ").context("invalid line")?;
        let outputs = outputs_str.split(", ").collect::<Vec<_>>();
        puzzle.modules.insert(name.to_owned(), Module {
            outputs: vec![],
            module_type,
        });
        outputs_map.insert(
            name.to_owned(),
            outputs
                .into_iter()
                .map(<str>::to_owned)
                .collect(),
        );
    }
    for (name, outputs) in outputs_map {
        let mut module_output = Vec::<PulseTarget>::new();
        for output in outputs {
            let output_module = puzzle.modules.get_mut(output.as_str());
            let pulse_target = PulseTarget {
                target_module: output,
                input_info: output_module.map_or(ModuleInputInfo::None, Module::register_input),
            };
            module_output.push(pulse_target)
        }
        puzzle.modules.get_mut(name.as_str()).unwrap().outputs = module_output;
    }
    Ok(puzzle)
}
