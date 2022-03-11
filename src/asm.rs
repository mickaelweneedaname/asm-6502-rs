use super::ast::{get_addresing_mode, Ast};
use super::instruction;
use super::linker;
use super::parser;

pub struct Asm {
    parser: parser::Parser,
    program: Vec<u8>,
    linker: linker::Linker,
    program_counter: u16,
}

impl Asm {
    pub fn new(text: String, origin: u16) -> Self {
        Self {
            parser: parser::Parser::new(text),
            linker: linker::Linker::new(origin),
            program_counter: origin,
            program: Vec::new(),
        }
    }

    pub fn compile(&mut self) -> Vec<u8> {
        let ast = self.parser.parse();
        self.linker.index(&ast);
        self.visit(&ast);
        self.program.clone()
    }

    fn visit(&mut self, ast: &Ast) {
        match ast {
            Ast::Statements(statements) => {
                for statement in statements.iter() {
                    self.visit(statement);
                }
            }
            Ast::Instruction {
                instruction: i,
                args: a,
            } => {
                let instruction = instruction::get_instruction(i, get_addresing_mode(a));
                let arg = self.get_args_bytes(a, instruction);
                self.program.push(instruction.opcode);
                if !arg.is_empty() {
                    self.program.extend_from_slice(&arg);
                }
                self.program_counter += u16::from(instruction.len);
            }
            Ast::Label(_) => {}
            _ => panic!("Asm :: unexpected node :{:?}", ast),
        }
    }

    fn get_args_bytes(
        &self,
        args: &Option<Box<Ast>>,
        instruction: &instruction::Instruction,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        if let Some(args) = args {
            match args.as_ref() {
                Ast::Absolute(address16) => bytes.extend_from_slice(&address16.to_le_bytes()),
                Ast::ZeroPage(address8) => {
                    bytes.push(*address8);
                }
                Ast::AbsoluteIndirect(address16) => {
                    bytes.extend_from_slice(&address16.to_le_bytes())
                }
                Ast::AsoluteIndexed(address16, _) => {
                    bytes.extend_from_slice(&address16.to_le_bytes())
                }
                Ast::ZeroPageIndexed(address8, _) => {
                    bytes.push(*address8);
                }
                Ast::IndirectX(address8) => {
                    bytes.push(*address8);
                }
                Ast::IndirectY(address8) => {
                    bytes.push(*address8);
                }
                Ast::Label(label) => {
                    bytes.extend_from_slice(&self.linker.link(
                        label,
                        instruction,
                        self.program_counter,
                    ));
                }
                Ast::Number8(number) => bytes.push(*number),
                _ => panic!("Asm : unexpected node : {:?}", *args),
            }
        }
        bytes
    }
}
