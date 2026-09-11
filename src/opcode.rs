#[derive(Debug, Clone, Copy, PartialEq, Eq)]

// Repr for specifying the underlying representation of the enum as u8
#[repr(u8)]
pub enum Opcode {
    Stop = 0x00,
    Return = 0x0B,

    Push = 0x01,
    Pop  = 0x06,

    Add  = 0x02,
    Sub  = 0x03,
    Mul  = 0x04,
    Div  = 0x05,


    Store = 0x07,
    Load  = 0x08,

    Jump  = 0x09,
    Jumpi = 0x0A,
    JumpDest = 0x0C,
}

impl TryFrom<u8> for Opcode {
    type Error = String;

    // Self is currently the Opcode enum.
    // Return a Result<SuccessType, ErrorType> corresponding to the OK and Err
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Opcode::Stop),
            0x0B => Ok(Opcode::Return),
            0x01 => Ok(Opcode::Push),
            0x06 => Ok(Opcode::Pop),
            0x02 => Ok(Opcode::Add),
            0x03 => Ok(Opcode::Sub),
            0x04 => Ok(Opcode::Mul),
            0x05 => Ok(Opcode::Div),
            0x07 => Ok(Opcode::Store),
            0x08 => Ok(Opcode::Load),
            0x09 => Ok(Opcode::Jump),
            0x0A => Ok(Opcode::Jumpi),
            0x0C => Ok(Opcode::JumpDest),
            _ => Err(format!("Invalid opcode: 0x{:02X}", value)),
        }
    }
}

impl Opcode {
    pub fn gas_cost(&self) -> u64 {
        match self {
            Opcode::Stop => 0,
            Opcode::Return => 0,
            Opcode::Push => 1,
            Opcode::Pop  => 1,
            Opcode::Sub  => 1,
            Opcode::Add  => 1,
            Opcode::Mul  => 1,
            Opcode::Div  => 1,
            Opcode::Store => 5,
            Opcode::Load  => 3,
            Opcode::Jump  => 2,
            Opcode::Jumpi => 3,
            Opcode::JumpDest => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_opcodes() {
        let cases = [
            (0x00, Opcode::Stop),
            (0x01, Opcode::Push),
            (0x02, Opcode::Add),
            (0x03, Opcode::Sub),
            (0x04, Opcode::Mul),
            (0x05, Opcode::Div),
            (0x06, Opcode::Pop),
            (0x07, Opcode::Store),
            (0x08, Opcode::Load),
            (0x09, Opcode::Jump),
            (0x0A, Opcode::Jumpi),
            (0x0B, Opcode::Return),
            (0x0C, Opcode::JumpDest),
        ];
        for (value, expected) in cases {
            assert_eq!(Opcode::try_from(value), Ok(expected));
        }
    }

    #[test]
    fn test_invalid_opcode() {
        assert!(Opcode::try_from(0x0D).is_err());
        assert!(Opcode::try_from(0xFF).is_err());
    }


    #[test]
    fn test_invalid_opcode_error_message() {
        let err = Opcode::try_from(0x0D).unwrap_err();
        assert_eq!(err, "Invalid opcode: 0x0D");
        
        let err = Opcode::try_from(0xFF).unwrap_err();
        assert_eq!(err, "Invalid opcode: 0xFF");
    }

}