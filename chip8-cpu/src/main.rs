struct CPU {
    pc: usize,
    memory: [u8; 0x1000],
    registers: [u8; 16],
    stack: [u16; 16],
    stack_pointer: usize,
}

impl CPU {
    pub fn read_opcode(&self) -> u16 {
        let op_byte1 = self.memory[self.pc] as u16;
        let op_byte2 = self.memory[self.pc + 1] as u16;

        op_byte1 << 8 | op_byte2
    }

    pub fn add_xy(&mut self, x: u8, y: u8) {
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];

        // println!("Addin reg{}({})+reg{}({})", x, arg1, y, arg2);

        let (val, overflow) = arg1.overflowing_add(arg2);
        self.registers[x as usize] = val;

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    pub fn mul_xy(&mut self, x: u8, y: u8) {
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];

        println!("Multiplying reg{}({})*reg{}({})", x, arg1, y, arg2);

        let (val, overflow) = arg1.overflowing_mul(arg2);
        self.registers[x as usize] = val;

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    pub fn run(&mut self) {
        loop {
            let op = self.read_opcode();
            self.pc += 2;

            let c = ((op & 0xF000) >> 12) as u8;
            let x = ((op & 0x0F00) >> 8) as u8;
            let y = ((op & 0x00F0) >> 4) as u8;
            let d = ((op & 0x000F) >> 0) as u8;

            let nnn = op & 0x0FFF;

            match (c, x, y, d) {
                (0, 0, 0, 0) => {
                    return;
                }
                (0, 0, 0xE, 0xE) => self.ret(),
                (0x2, _, _, _) => self.call(nnn),
                (0x8, _, _, 0x4) => self.add_xy(x, y),
                (0x9, _, _, 0x1) => self.mul_xy(x, y),
                _ => todo!("implement op {:04x}", op),
            }
        }
    }

    pub fn call(&mut self, addr: u16) {
        println!("Calling function at 0x{:04x}", addr);
        let sp = self.stack_pointer;
        let stack = &mut self.stack;
        if sp > stack.len() {
            panic!("Stack overflow: Too many recursive functions");
        }
        stack[sp] = self.pc as u16;
        self.stack_pointer += 1;
        self.pc = addr as usize;
    }

    pub fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("Stack underflow: Returning on empty stack");
        }
        self.stack_pointer -= 1;
        self.pc = self.stack[self.stack_pointer] as usize;
    }
}

fn main() {
    let mut cpu = CPU {
        pc: 0,
        memory: [0; 4096],
        registers: [0; 16],
        stack_pointer: 0,
        stack: [0; 16],
    };

    cpu.registers[0] = 5;
    cpu.registers[1] = 2;
    cpu.registers[2] = 10;
    cpu.registers[3] = 10;

    let mem = &mut cpu.memory;
    // mem[0] = 0x80;
    // mem[1] = 0x14;
    // mem[2] = 0x80;
    // mem[3] = 0x24;
    // mem[4] = 0x80;
    // mem[5] = 0x34;
    //
    // cpu.run();

    // assert_eq!(cpu.registers[0], 35);

    let add_twice: [u8; 6] = [0x80, 0x14, 0x90, 0x11, 0x00, 0xEE];

    mem[0x00] = 0x21;
    mem[0x01] = 0x00;
    mem[0x05] = 0x00;

    mem[0x100..0x106].copy_from_slice(&add_twice);
    cpu.registers[0] = 5;

    cpu.run();
    // (5+2)*2=14
    assert_eq!(cpu.registers[0], 14);
    println!("Result in Reg[0] is {}", cpu.registers[0]);
}
