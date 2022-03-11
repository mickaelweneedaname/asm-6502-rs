use super::ast::{get_addresing_mode, Ast};
use super::instruction;

pub struct Linker {
    labels_indexes: std::collections::HashMap<String, u16>,
    program_counter: u16,
}

impl Linker {
    pub fn new(origin: u16) -> Self {
        Self {
            labels_indexes: std::collections::HashMap::new(),
            program_counter: origin,
        }
    }

    pub fn index(&mut self, ast: &Ast) {
        match ast {
            Ast::Statements(statements) => {
                for statement in statements.iter() {
                    self.index(statement);
                }
            }
            Ast::Label(label) => {
                self.labels_indexes
                    .insert(label.clone(), self.program_counter);
            }
            Ast::Instruction {
                instruction: i,
                args: a,
            } => {
                let instruction = instruction::get_instruction(i, get_addresing_mode(a));
                self.program_counter += u16::from(instruction.len);
            }
            _ => {}
        }
    }

    fn get(&self, label: &str) -> u16 {
        *self.labels_indexes.get(label).unwrap()
    }

    pub fn link(
        &self,
        label: &str,
        instruction: &instruction::Instruction,
        asm_program_counter: u16,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        let label_index = self.get(label);
        match instruction.mode {
            instruction::Mode::Relative => {
                let mut offset: u8 =
                    i16::abs(label_index as i16 - asm_program_counter as i16) as u8;
                if label_index < asm_program_counter {
                    offset = offset.wrapping_neg();
                }
                bytes.push((offset - instruction.len) as u8)
            }
            instruction::Mode::Absolute => bytes.extend_from_slice(&label_index.to_le_bytes()),
            _ => {
                panic!(
                    "Asm : calculate_offset Unexpected mode: {:?}",
                    instruction.mode
                )
            }
        }
        bytes
    }
}
