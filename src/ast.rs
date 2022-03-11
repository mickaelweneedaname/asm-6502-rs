use super::instruction::Mode;

#[derive(Debug)]
pub enum Ast {
    Statements(Vec<Ast>),
    Instruction {
        instruction: String,
        args: Option<Box<Ast>>,
    },
    Label(String),
    Number8(u8),
    Absolute(u16),
    ZeroPage(u8),
    AbsoluteIndirect(u16),
    AsoluteIndexed(u16, char),
    ZeroPageIndexed(u8, char),
    IndirectX(u8),
    IndirectY(u8),
}

pub fn get_addresing_mode(ast: &Option<Box<Ast>>) -> Mode {
    if let Some(ast) = ast {
        match ast.as_ref() {
            Ast::Absolute(_) => Mode::Absolute,
            Ast::AbsoluteIndirect(_) => Mode::Indirect,
            Ast::IndirectX(_) => Mode::IndirectX,
            Ast::IndirectY(_) => Mode::IndirectY,
            Ast::ZeroPage(_) => Mode::ZeroPage,
            Ast::AsoluteIndexed(_, reg) => match reg {
                'x' | 'X' => Mode::AbsoluteX,
                'y' | 'Y' => Mode::AbsoluteY,
                _ => panic!("get_addressing_mode : Unknown register {reg}"),
            },
            Ast::ZeroPageIndexed(_, reg) => match reg {
                'x' | 'X' => Mode::ZeroPageX,
                'y' | 'Y' => Mode::ZeroPageY,
                _ => panic!("get_addressing_mode : Unknown register {reg}"),
            },
            Ast::Label(_) => Mode::Relative,
            Ast::Number8(_) => Mode::Immediate,
            _ => panic!("get_addressing_mode : Unexpected node : {:?}", ast),
        }
    } else {
        Mode::Implicit
    }
}
