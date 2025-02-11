pub struct Interpreter {
    pub memory: [u8; 4096],
    pub registers: [u8; 16],
    pub index_register: u16,
    pub program_counter: u16,
}

impl Interpreter {
    pub const DISPLAY_WIDTH: usize = 64;
    pub const DISPLAY_HEIGHT: usize = 32;

    pub const DISPLAY_SIZE: usize = Self::DISPLAY_WIDTH * Self::DISPLAY_HEIGHT / 8;

    pub fn new() -> Self {
        let mut memory = [0; 4096];

        let character_rom = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // Binary '0'
            0x20, 0x60, 0x20, 0x20, 0x70, // Binary '1'
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // Binary '2'
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // Binary '3'
            0x90, 0x90, 0xF0, 0x10, 0x10, // Binary '4'
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // Binary '5'
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // Binary '6'
            0xF0, 0x10, 0x20, 0x40, 0x40, // Binary '7'
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // Binary '8'
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // Binary '9'
            0xF0, 0x90, 0xF0, 0x90, 0x90, // Binary 'A'
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // Binary 'B'
            0xF0, 0x80, 0x80, 0x80, 0xF0, // Binary 'C'
            0xE0, 0x90, 0x90, 0x90, 0xE0, // Binary 'D'
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // Binary 'E'
            0xF0, 0x80, 0xF0, 0x80, 0x80, // Binary 'F'
        ];

        memory[0x00..0x00 + character_rom.len()].copy_from_slice(&character_rom);

        Self {
            memory,
            registers: [0; 16],
            index_register: 0x00,
            program_counter: 0x200, // Program starts at 0x200
        }
    }

    pub fn load_program(&mut self, rom_data: &[u8]) -> Result<(), String> {
        let program_offset: usize = 0x200;

        let available_space = self.memory.len() - program_offset;

        if rom_data.len() > available_space {
            return Err(format!(
                "Program size ({}) exceeds available memory space ({})",
                rom_data.len(),
                available_space
            ));
        }

        self.memory[program_offset..(program_offset + rom_data.len())].copy_from_slice(rom_data);

        Ok(())
    }

    pub fn execute_cycle(&mut self) {
        let pc = self.program_counter as usize;

        let high_byte = self.memory[pc];
        let low_byte = self.memory[pc + 1];

        let opcode = (high_byte as u16) << 8 | (low_byte as u16);

        match(
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            opcode & 0x000F
        ) {
            (0x0, 0x0, 0xE, 0x0) => {
                self.memory[0xF00..0xF00 + Self::DISPLAY_SIZE].fill(0x00);

                self.step_to_next_instruction();
            }

            (0x1, _, _, _) => {
                let new_address = opcode & 0x0FFF;

                self.program_counter = new_address;
            }

            (0x3, _, _, _) => {
                let register_index = ((opcode & 0x0F00) >> 8) as usize;
                let comparison_value = (opcode & 0x00FF) as u8;

                if self.registers[register_index] == comparison_value {
                    self.program_counter += 4;
                } else {
                    self.step_to_next_instruction();
                }
            }

            (0x4, _, _, _) => {
                let register_index = ((opcode & 0x0F00) >> 8) as usize;
                let comparison_value = (opcode & 0x00FF) as u8;

                if self.registers[register_index] != comparison_value {
                    self.program_counter += 4;
                } else {
                    self.step_to_next_instruction();
                }
            }

            (0x5, _, _, _) => {
                let register_index_x = ((opcode & 0x0F00) >> 8) as usize;
                let register_index_y = ((opcode & 0x00F0) >> 4) as usize;

                if self.registers[register_index_x] == self.registers[register_index_y] {
                    self.program_counter += 4;
                } else {
                    self.step_to_next_instruction();
                }
            }

            (0x6, _, _, _) => {
                let register_index = ((opcode & 0x0F00) >> 8) as usize;
                let register_value = (opcode & 0x00FF) as u8;

                self.registers[register_index] = register_value;

                self.step_to_next_instruction();
            }

            (0x7, _, _, _) => {
                let register_index = ((opcode & 0x0F00) >> 8) as usize;
                let register_value = (opcode & 0x00FF) as u8;

                self.registers[register_index] += register_value;

                self.step_to_next_instruction();
            }

            (0x8, _, _, _) => {
                let register_index_x = ((opcode & 0x0F00) >> 8) as usize;
                let register_index_y = ((opcode & 0x00F0) >> 4) as usize;

                match opcode & 0x000F {
                    0 => {
                        self.registers[register_index_x] = self.registers[register_index_y];
                    }

                    1 => {
                        self.registers[register_index_x] |= self.registers[register_index_y];
                    }

                    2 => {
                        self.registers[register_index_x] &= self.registers[register_index_y];
                    }

                    3 => {
                        self.registers[register_index_x] ^= self.registers[register_index_y];
                    }

                    _ => {
                        panic!("Unsupported 0x8000 bit: {:01X}", opcode & 0x000F)
                    }
                }

                self.step_to_next_instruction();
            }

            (0xA, _, _, _) => {
                let address = opcode & 0x0FFF;

                self.index_register = address;

                self.step_to_next_instruction();
            }

            (0xD, _, _, _) => {
                let i = self.index_register as usize;

                let register_index_x = ((opcode & 0x0F00) >> 8) as usize;
                let register_index_y = ((opcode & 0x00F0) >> 4) as usize;

                let vx = self.registers[register_index_x] as usize;
                let vy = self.registers[register_index_y] as usize;

                let nibble = (opcode & 0x000F) as usize;

                self.registers[0xF] = 0;

                for byte in 0..nibble {
                    let row_index = (vy + byte) % 32;

                    let column_index = vx % 64;

                    let value = self.memory[i + byte];

                    let byte_remainder = column_index % 8;

                    let first_byte = value >> byte_remainder;

                    let display_offset = (row_index * 8) + (column_index / 8);

                    self.registers[0xF] |= first_byte & self.memory[0xF00 + display_offset];

                    self.memory[0xF00 + display_offset] ^= first_byte;

                    if byte_remainder > 0 {
                        let second_byte = value << (8 - byte_remainder);

                        if second_byte > 0 {
                            self.registers[0xF] |=
                                second_byte & self.memory[0xF00 + display_offset + 1];

                            self.memory[0xF00 + display_offset + 1] ^= second_byte;
                        }
                    }
                }

                self.step_to_next_instruction();
            }

            _ => {
                panic!("Unsupported opcode: {:04X}", opcode);
            }
        }
    }

    fn step_to_next_instruction(&mut self) {
        self.program_counter += 2;
    }
}

#[cfg(test)]
mod tests {
    use crate::chip8::Interpreter as Chip8Interpreter;

    fn setup_instructions(program_start: u16, opcodes: &[u16]) -> Chip8Interpreter {
        let mut interpreter = Chip8Interpreter::new();

        let program_data: Vec<u8> = opcodes
            .iter()
            .flat_map(|&opcode| opcode.to_be_bytes())
            .collect();

        interpreter.load_program(&program_data).unwrap();

        interpreter.program_counter = program_start;

        return interpreter;
    }

    #[test]
    fn test_loading_program_of_max_size() {
        let mut interpreter = Chip8Interpreter::new();

        interpreter.program_counter = 0x200;

        let rom_data: [u8; 3584] = [0xFF; 3584];

        assert!(interpreter.load_program(&rom_data).is_ok());
    }

    #[test]
    fn test_loading_program_too_large() {
        let mut interpreter = Chip8Interpreter::new();

        interpreter.program_counter = 0x200;

        let rom_data: [u8; 3585] = [0xFF; 3585];

        assert!(interpreter.load_program(&rom_data).is_err());
    }

    #[test]
    fn test_opcode_1nnn_jumps_to_address_nnn() {
        let mut interpreter = setup_instructions(0x200, &[0x1FFF]);

        interpreter.execute_cycle();

        assert_eq!(
            interpreter.program_counter, 0x0FFF,
            "Program counter should contain 0x000!"
        );
    }

    #[test]
    fn test_opcode_3xnn_skip_next_instruction_if_vx_equals_nn() {
        let mut interpreter = setup_instructions(0x200, &[0x3000]);

        interpreter.execute_cycle();

        assert_eq!(
            interpreter.program_counter,
            (0x200 + 0x04),
            "Program counter should contain 0x204!"
        );
    }

    #[test]
    fn test_opcode_3xnn_runs_next_instruction_if_vx_not_equal_nn() {
        let mut interpreter = setup_instructions(0x200, &[0x3001]);

        interpreter.execute_cycle();

        assert_eq!(
            interpreter.program_counter,
            (0x200 + 0x02),
            "Program counter should contain 0x202!"
        );
    }

    #[test]
    fn test_opcode_4xnn_skip_next_instruction_if_vx_not_equal_nn() {
        let mut interpreter = setup_instructions(0x200, &[0x4001]);

        interpreter.execute_cycle();

        assert_eq!(
            interpreter.program_counter,
            (0x200 + 0x04),
            "Program counter should contain 0x204!"
        );
    }

    #[test]
    fn test_opcode_4xnn_runs_next_instruction_if_vx_equal_nn() {
        let mut interpreter = setup_instructions(0x200, &[0x4000]);

        interpreter.execute_cycle();

        assert_eq!(
            interpreter.program_counter,
            (0x200 + 0x02),
            "Program counter should contain 0x202!"
        );
    }

    #[test]
    fn test_opcode_5xy0_skips_next_instruction_if_vx_equals_vy() {
        let mut interpreter = setup_instructions(0x200, &[0x5000]);

        interpreter.execute_cycle();

        assert_eq!(
            interpreter.program_counter,
            (0x200 + 0x04),
            "Program counter should contain 0x204!"
        );
    }

    #[test]
    fn test_opcode_6xnn_sets_vx_to_nn() {
        let mut interpreter = setup_instructions(0x200, &[0x6012]);

        interpreter.execute_cycle();

        assert_eq!(
            interpreter.registers[0], 0x12,
            "Register V0 should contain 0x12!"
        );

        assert_eq!(
            interpreter.program_counter, 0x202,
            "Program counter should advance by 2!"
        );
    }

    #[test]
    fn test_opcode_7xnn_add_nn_to_vx() {
        // TODO: This test assumes Register[x] is 0 by default
        let mut interpreter = setup_instructions(0x200, &[0x7012]);

        interpreter.execute_cycle();

        assert_eq!(
            interpreter.registers[0], 0x12,
            "Register V0 should contain 0x12!"
        );

        assert_eq!(
            interpreter.program_counter, 0x202,
            "Program counter should advance by 2!"
        );
    }

    #[test]
    fn test_opcode_8xy0_sets_vx_to_value_of_vy() {
        let mut interpreter = setup_instructions(0x200, &[0x8010]);

        interpreter.registers[0] = 0x01;
        interpreter.registers[1] = 0x02;

        interpreter.execute_cycle();

        assert_eq!(
            interpreter.registers[0], 0x02,
            "Register V0 should equal the value of V1: 0x02!"
        );

        assert_eq!(
            interpreter.program_counter,
            (0x200 + 0x02),
            "Program counter should contain 0x202!"
        );
    }

    #[test]
    fn test_opcode_8xy1_sets_vx_to_vx_bitwise_or_vy() {
        let mut interpreter = setup_instructions(0x200, &[0x8011]);

        interpreter.registers[0] = 0x01;
        interpreter.registers[1] = 0x02;

        interpreter.execute_cycle();

        assert_eq!(
            interpreter.registers[0], 0x03,
            "Register V0 should equal 0x03!"
        );

        assert_eq!(
            interpreter.program_counter,
            (0x200 + 0x02),
            "Program counter should contain 0x202!"
        );
    }

    #[test]
    fn test_opcode_8xy2_sets_vx_to_vx_bitwise_and_vy() {
        let mut interpreter = setup_instructions(0x200, &[0x8012]);

        interpreter.registers[0] = 0x01;
        interpreter.registers[1] = 0x02;

        interpreter.execute_cycle();

        assert_eq!(
            interpreter.registers[0], 0x00,
            "Register V0 should equal 0x00!"
        );

        assert_eq!(
            interpreter.program_counter,
            (0x200 + 0x02),
            "Program counter should contain 0x202!"
        );
    }

    #[test]
    fn test_opcode_8xy3_sets_vx_to_vx_bitwise_xor_vy() {
        // TODO: This might be unintended opcode 8XY3
        let mut interpreter = setup_instructions(0x200, &[0x8013]);

        interpreter.registers[0] = 0x01;
        interpreter.registers[1] = 0x03;

        interpreter.execute_cycle();

        assert_eq!(
            interpreter.registers[0], 0x02,
            "Register V0 should equal 0x02!"
        );

        assert_eq!(
            interpreter.program_counter,
            (0x200 + 0x02),
            "Program counter should contain 0x202!"
        );
    }

    #[test]
    fn test_opcode_00e0_clear_screen() {
        let mut interpreter = setup_instructions(0x200, &[0x00E0]);

        const DISPLAY_SIZE: usize = 64 * 32 / 8;

        interpreter.memory[0xF00..0xF00 + DISPLAY_SIZE].fill(0x01);

        interpreter.execute_cycle();

        assert_eq!(
            &interpreter.memory[0xF00..0xF00 + DISPLAY_SIZE],
            &[0x00; DISPLAY_SIZE],
            "Display memory is not fully cleared!"
        );

        assert_eq!(
            interpreter.program_counter,
            (0x200 + 0x02),
            "Program counter should contain 0x202!"
        );
    }

    #[test]
    fn test_opcode_annn_set_register_i_to_nnn() {
        let mut interpreter = setup_instructions(0x200, &[0xA111]);

        interpreter.execute_cycle();

        assert_eq!(
            interpreter.index_register, 0x111,
            "Index register should contain 0x111!"
        );

        assert_eq!(
            interpreter.program_counter,
            (0x200 + 0x02),
            "Program counter should contain 0x202!"
        );
    }

    #[test]
    fn test_opcode_dxyn_display_binary_0_sprite() {
        let mut interpreter = setup_instructions(0x200, &[0xD015]);

        interpreter.execute_cycle();

        let screen: [u8; 64 * 32 / 8] = [
            0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x90, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x90, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let ram_screen = &interpreter.memory[0xF00..0xF00 + (64 * 32 / 8)];

        assert_eq!(ram_screen, screen, "Screen does not match!");

        assert_eq!(
            interpreter.program_counter,
            (0x200 + 0x02),
            "Program counter should contain 0x202!"
        );
    }

    #[test]
    fn test_opcode_dxyn_display_binary_0_sprite_offset_x() {
        let mut interpreter = setup_instructions(0x200, &[0x6004, 0xD015]);

        interpreter.execute_cycle();
        interpreter.execute_cycle();

        let screen: [u8; 64 * 32 / 8] = [
            0x0F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let ram_screen = &interpreter.memory[0xF00..0xF00 + (64 * 32 / 8)];

        assert_eq!(ram_screen, screen, "Screen does not match!");
    }

    #[test]
    fn test_opcode_dxyn_display_binary_0_sprite_offset_y() {
        let mut interpreter = setup_instructions(0x200, &[0x6001, 0xD105]);

        interpreter.execute_cycle();
        interpreter.execute_cycle();

        let screen: [u8; 64 * 32 / 8] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x90, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let ram_screen = &interpreter.memory[0xF00..0xF00 + (64 * 32 / 8)];

        assert_eq!(ram_screen, screen, "Screen does not match!");
    }

    // #[test]
    // fn test_opcode_dxyn_display_binary_0_sprite_overlapping() {
    //     let mut interpreter = setup_instructions(0x200, &[0xD015, 0x6001, 0xD015]);

    //     interpreter.execute_cycle();
    //     interpreter.execute_cycle();

    //     let screen: [u8; 64 * 32 / 8] = [
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //     ];

    //     let ram_screen= &interpreter.memory[0xF00..0xF00 + (64 * 32 / 8)];

    //     assert_eq!(ram_screen, screen, "Screen does not match!");
    // }

    #[test]
    fn test_display_binary_0() {
        let interpreter = setup_instructions(0x200, &[]);

        assert_eq!(
            interpreter.memory[0x00..0x00 + 0x05],
            [0xF0, 0x90, 0x90, 0x90, 0xF0],
            "Binary 0 does not match!"
        );
    }

    #[test]
    fn test_display_binary_1() {
        let interpreter = setup_instructions(0x200, &[]);

        assert_eq!(
            interpreter.memory[0x05..0x05 + 0x05],
            [0x20, 0x60, 0x20, 0x20, 0x70],
            "Binary 1 does not match!"
        );
    }

    #[test]
    fn test_display_binary_2() {
        let interpreter = setup_instructions(0x200, &[]);

        assert_eq!(
            interpreter.memory[0x0A..0x0A + 0x05],
            [0xF0, 0x10, 0xF0, 0x80, 0xF0],
            "Binary 2 does not match!"
        );
    }

    #[test]
    fn test_display_binary_3() {
        let interpreter = setup_instructions(0x200, &[]);

        assert_eq!(
            interpreter.memory[0x0F..0x0F + 0x05],
            [0xF0, 0x10, 0xF0, 0x10, 0xF0],
            "Binary 3 does not match!"
        );
    }

    #[test]
    fn test_display_binary_4() {
        let interpreter = setup_instructions(0x200, &[]);

        assert_eq!(
            interpreter.memory[0x14..0x14 + 0x05],
            [0x90, 0x90, 0xF0, 0x10, 0x10],
            "Binary 4 does not match!"
        );
    }

    #[test]
    fn test_display_binary_5() {
        let interpreter = setup_instructions(0x200, &[]);

        assert_eq!(
            interpreter.memory[0x19..0x19 + 0x05],
            [0xF0, 0x80, 0xF0, 0x10, 0xF0],
            "Binary 5 does not match!"
        );
    }

    #[test]
    fn test_display_binary_6() {
        let interpreter = setup_instructions(0x200, &[]);

        assert_eq!(
            interpreter.memory[0x1E..0x1E + 0x05],
            [0xF0, 0x80, 0xF0, 0x90, 0xF0],
            "Binary 6 does not match!"
        );
    }

    #[test]
    fn test_display_binary_7() {
        let interpreter = setup_instructions(0x200, &[]);

        assert_eq!(
            interpreter.memory[0x23..0x23 + 0x05],
            [0xF0, 0x10, 0x20, 0x40, 0x40],
            "Binary 7 does not match!"
        );
    }

    #[test]
    fn test_display_binary_8() {
        let interpreter = setup_instructions(0x200, &[]);

        assert_eq!(
            interpreter.memory[0x28..0x28 + 0x05],
            [0xf0, 0x90, 0xF0, 0x90, 0xF0],
            "Binary 8 does not match!"
        );
    }

    #[test]
    fn test_display_binary_9() {
        let interpreter = setup_instructions(0x200, &[]);

        assert_eq!(
            interpreter.memory[0x2D..0x2D + 0x05],
            [0xF0, 0x90, 0xF0, 0x10, 0xF0],
            "Binary 9 does not match!"
        );
    }

    #[test]
    fn test_display_binary_a() {
        let interpreter = setup_instructions(0x200, &[]);

        assert_eq!(
            interpreter.memory[0x32..0x32 + 0x05],
            [0xF0, 0x90, 0xF0, 0x90, 0x90],
            "Binary A does not match!"
        );
    }

    #[test]
    fn test_display_binary_b() {
        let interpreter = setup_instructions(0x200, &[]);

        assert_eq!(
            interpreter.memory[0x37..0x37 + 0x05],
            [0xE0, 0x90, 0xE0, 0x90, 0xE0],
            "Binary B does not match!"
        );
    }

    #[test]
    fn test_display_binary_c() {
        let interpreter = setup_instructions(0x200, &[]);

        assert_eq!(
            interpreter.memory[0x3C..0x3C + 0x05],
            [0xF0, 0x80, 0x80, 0x80, 0xF0],
            "Binary C does not match!"
        );
    }

    #[test]
    fn test_display_binary_d() {
        let interpreter = setup_instructions(0x200, &[]);

        assert_eq!(
            interpreter.memory[0x41..0x41 + 0x05],
            [0xE0, 0x90, 0x90, 0x90, 0xE0],
            "Binary D does not match!"
        );
    }

    #[test]
    fn test_display_binary_e() {
        let interpreter = setup_instructions(0x200, &[]);

        assert_eq!(
            interpreter.memory[0x46..0x46 + 0x05],
            [0xF0, 0x80, 0xF0, 0x80, 0xF0],
            "Binary E does not match!"
        );
    }

    #[test]
    fn test_display_binary_f() {
        let interpreter = setup_instructions(0x200, &[]);

        assert_eq!(
            interpreter.memory[0x4B..0x4B + 0x05],
            [0xF0, 0x80, 0xF0, 0x80, 0x80],
            "Binary F does not match!"
        );
    }
}
