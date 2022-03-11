use asm_6502;

fn main() {
    println!("#### COMPILE SNAKE");
    let text = std::fs::read_to_string("src/script/snake.asm").unwrap();
    asm_6502::compile(text, 0x600);
}