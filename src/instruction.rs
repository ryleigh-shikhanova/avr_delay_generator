pub struct InstructionTiming {
    pub cycles_taken: u8,
    pub cycles_not_taken: u8,
}

pub enum AvrInstruction {
    Nop,
    Ldi { register: u8, value: u8 },
    Dec { register: u8 },
    Brne { label: String },
    Rjmp { label: String },
}

impl AvrInstruction {
    pub fn timing(&self) -> InstructionTiming {
        match self {
            AvrInstruction::Nop => InstructionTiming {
                cycles_taken: 1,
                cycles_not_taken: 1,
            },

            AvrInstruction::Ldi { .. } => InstructionTiming {
                cycles_taken: 1,
                cycles_not_taken: 1,
            },

            AvrInstruction::Dec { .. } => InstructionTiming {
                cycles_taken: 1,
                cycles_not_taken: 1,
            },

            AvrInstruction::Brne { .. } => InstructionTiming {
                cycles_taken: 2,
                cycles_not_taken: 1,
            },

            AvrInstruction::Rjmp { .. } => InstructionTiming {
                cycles_taken: 2,
                cycles_not_taken: 2,
            },
        }
    }
}
