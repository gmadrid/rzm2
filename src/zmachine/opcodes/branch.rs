use log::debug;

use super::{Result, Variables, ZOperand, PC};

// Read the jump offset from the PC.
fn read_jump_offset<P>(pc: &mut P) -> (i16, bool)
where
    P: PC,
{
    let first_byte = pc.next_byte();

    let jump_offset = if first_byte & 0b0100_0000 != 0 {
        // One byte only.
        i16::from(first_byte & 0b0011_1111)
    } else {
        let second_byte = pc.next_byte();
        let mut offset: u16 = ((first_byte as u16 & 0b0011_1111) << 8) + second_byte as u16;
        // Check for a negative 14-bit value, and sign extend to 16-bits if necessary.
        if offset & 0b0010_0000_0000_0000 != 0 {
            offset |= 0b1100_0000_0000_0000;
        }

        offset as i16
    };

    let branch_on_truth = !((first_byte & 0b1000_0000) == 0);

    (jump_offset, branch_on_truth)
}

// All of the Z-Machine branch codes use the same logic for everything but the actual test.
// branch() will perform the jump based on the test closure provided.
fn branch<P, F>(pc: &mut P, tst: F) -> Result<()>
where
    F: FnOnce(i16, bool) -> Result<bool>,
    P: PC,
{
    let (offset, branch_on_truth) = read_jump_offset(pc);
    let truth = tst(offset, branch_on_truth)?;

    if branch_on_truth == truth {
        // Branch!
        match offset {
            0 => unimplemented!("ret false"),
            1 => unimplemented!("ret true"),
            o => {
                pc.offset_pc((o - 2) as isize);
            }
        }
    }
    Ok(())
}

// ZSpec: 1OP:128 0x00 jz a ?(label)
pub fn jz<P, V>(pc: &mut P, variables: &mut V, operand: ZOperand) -> Result<()>
where
    P: PC,
    V: Variables,
{
    branch(pc, |offset, branch_on_truth| {
        debug!(
            "jz         {} ?{}(x{:x})",
            operand,
            if branch_on_truth { "" } else { "~" },
            offset
        );

        // TODO: what if this is Omitted?
        Ok(operand.value(variables)? == 0)
    })
}

#[cfg(test)]
mod tests {
    use super::super::super::fixtures::{TestPC, TestVariables};
    use super::*;

    #[test]
    fn test_jump_offset() {
        let mut pc = TestPC::new(0, vec![0b0100_1111, 0]);
        assert_eq!((15, false), read_jump_offset(&mut pc));

        let mut pc = TestPC::new(0, vec![0b1100_1111, 0]);
        assert_eq!((15, true), read_jump_offset(&mut pc));

        let mut pc = TestPC::new(0, vec![0b0001_1000, 0b0000_1111]);
        assert_eq!((0b0001_1000_0000_1111, false), read_jump_offset(&mut pc));

        let mut pc = TestPC::new(0, vec![0b0011_1111, 0b1111_1111]);
        assert_eq!((-1, false), read_jump_offset(&mut pc));

        let mut pc = TestPC::new(0, vec![0b1011_1111, 0b1111_0100]);
        assert_eq!((-12, true), read_jump_offset(&mut pc));
    }

    #[test]
    fn test_branch() {
        let mut pc = TestPC::new(0, vec![0b1100_0110, 0, 0, 0, 0, 0]);
        branch(&mut pc, |_, _| Ok(true));
        assert_eq!(5, pc.current_pc());

        let mut pc = TestPC::new(0, vec![0b0100_0110, 0, 0, 0, 0, 0]);
        branch(&mut pc, |_, _| Ok(true));
        assert_eq!(1, pc.current_pc());

        let mut pc = TestPC::new(0, vec![0b1100_0110, 0, 0, 0, 0, 0]);
        branch(&mut pc, |_, _| Ok(false));
        assert_eq!(1, pc.current_pc());

        let mut pc = TestPC::new(0, vec![0b0100_0110, 0, 0, 0, 0, 0]);
        branch(&mut pc, |_, _| Ok(false));
        assert_eq!(5, pc.current_pc());
    }

    fn branch_pc(offset: u8, size: u8) -> TestPC {
        let mut vec = vec![0; usize::from(size)];
        vec[0] = 0b1100_0000 + offset + 2;
        TestPC::new(0, vec)
    }

    #[test]
    fn test_jz() {
        let mut variables = TestVariables::new();

        let mut pc = branch_pc(8, 10);
        jz(&mut pc, &mut variables, ZOperand::SmallConstant(0)).unwrap();
        assert_eq!(9, pc.current_pc());

        let mut pc = branch_pc(8, 10);
        jz(&mut pc, &mut variables, ZOperand::SmallConstant(3)).unwrap();
        assert_eq!(1, pc.current_pc());
    }
}
