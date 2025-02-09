use std::{env, fs};
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_rom>", args[0]);
    }

    let rom_path = &args[1];

    println!("Rom file: {}", rom_path);

    let rom_data: Vec<u8> = fs::read(rom_path).unwrap();

    for (index, opcode) in rom_data.chunks(2).enumerate() {
        match opcode {
            &[high_byte, low_byte] => {
                print!("{:#012x}: {:02x}{:02x}", 0x200 + (2 * index), high_byte, low_byte);

                let nnn = ((high_byte & 0x0F) as u16) << 8 | (low_byte as u16);
                let x = high_byte & 0x0F;
                let y = low_byte >> 4;
                let kk = low_byte;
                let n = low_byte & 0x0F;

                let description = match(
                    (high_byte >> 4) & 0xF,
                    high_byte & 0xF,
                    (low_byte >> 4) & 0xF,
                    low_byte & 0xF
                ) {
                    (0x0, 0x0, 0x0, 0x0) => {
                        String::from("NOP")
                    }

                    (0x0, 0x0, 0xE, 0x0) => {
                        String::from("CLS")
                    }
                    
                    (0x0, 0x0, 0xE, 0xE) => {
                        String::from("RET")
                    }

                    (0x0, _, _, _) => {
                        format!("SYS {:#012X}", nnn)
                    }

                    (0x1, _, _, _) => {
                        format!("JP {:#012X}", nnn)
                    }

                    (0x2, _, _, _) => {
                        format!("CALL {:012X}", nnn)
                    }

                    (0x3, _, _, _) => {
                        format!("SE V{:01x}, {:#02X}", x, kk)
                    }

                    (0x6, _, _, _) => {
                        format!("LD V{:01x}, {:#02X}", x, kk)
                    }

                    (0x7, _, _, _) => {
                        format!("ADD V{:01x}, {}", x, kk)
                    }

                    (0xA, _, _, _) => {
                        format!("LD I, {:#03X}", nnn)
                    }

                    (0xD, _, _, _) => {
                        format!("DRW V{:01x}, V{:01x}, {:#01X}", x, y, n)
                    }

                    _ => {
                        // eprintln!("Unsupported opcode: {:02x}{:02x}", high_byte, low_byte);

                        String::new()
                    }
                };

                print!(" ; {}", description);

                println!();
            }

            _ => {
                panic!("Opcodes without two bytes not supported!");
            }
        }
    }

    println!();

    Ok(())
}