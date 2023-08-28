/*
    MartyPC
    https://github.com/dbalsom/martypc

    Copyright 2022-2023 Daniel Balsom

    Permission is hereby granted, free of charge, to any person obtaining a
    copy of this software and associated documentation files (the “Software”),
    to deal in the Software without restriction, including without limitation
    the rights to use, copy, modify, merge, publish, distribute, sublicense,
    and/or sell copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER   
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
    FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.

    ---------------------------------------------------------------------------

    cpu_808x::jump.rs

    Implements microcode routines for jumps and calls.
*/

use crate::cpu_808x::*;
use crate::cpu_808x::biu::*;

impl Cpu {

    /// Execute the RELJMP microcode routine, optionally including the jump into the procedure.
    #[inline]
    pub fn reljmp(&mut self, new_ip: u16, jump: bool) {
        if jump {
            self.cycle_i(MC_JUMP);
        }
        //self.biu_suspend_fetch_i(0x0d2);
        self.biu_suspend_fetch();
        self.cycles_i(3, &[0x0d2, 0x0d3, 0x0d4]);
        self.ip = new_ip;
        self.biu_queue_flush(); // 0d5
        self.cycle_i(0x0d5);
    }

    /// Execute the FARCALL microcode routine.
    #[inline]
    pub fn farcall(&mut self, new_cs: u16, new_ip: u16, jump: bool) {
        if jump {
            self.cycle_i(MC_JUMP);
        }
        self.biu_suspend_fetch(); // 0x06B
        self.cycles_i(2, &[0x06c, MC_CORR]);
        // Push return segment to stack
        self.push_u16(self.cs, ReadWriteFlag::Normal);
        self.cs = new_cs;
        self.cycles_i(2, &[0x06e, 0x06f]);
        self.nearcall(new_ip);
    }

    /// Execute the FARCALL2 microcode routine. Called by interrupt procedures.
    #[inline]
    pub fn farcall2(&mut self, new_cs: u16, new_ip: u16) {
        
        self.cycles_i(3, &[MC_JUMP, 0x06c, MC_CORR]);
        // Push return segment to stack
        self.push_u16(self.cs, ReadWriteFlag::Normal);
        self.cs = new_cs;
        self.cycles_i(2, &[0x06e, 0x06f]);
        self.nearcall(new_ip);
    }

    /// Execute the NEARCALL microcode routine.
    #[inline]
    pub fn nearcall(&mut self, new_ip: u16) {
        let ret_ip = self.ip;
        self.cycle_i(MC_JUMP);
        self.ip = new_ip;
        self.biu_queue_flush();
        self.cycles_i(3, &[0x077, 0x078, 0x079]);
        self.push_u16(ret_ip, ReadWriteFlag::RNI);
    }

    /// Execute the FARRET microcode routine, including the jump into the procedure.
    pub fn farret(&mut self, far: bool) {

        self.cycle_i(MC_JUMP);
        self.set_mc_pc(0x0c2);
        self.pop_register16(Register16::IP, ReadWriteFlag::Normal);
        self.biu_suspend_fetch();
        self.cycles_i(2, &[0x0c3, 0x0c4]);

        let far2 = self.i.opcode & 0x08 != 0;
        assert_eq!(far, far2);

        if far {
            self.cycle_i(MC_JUMP);
            self.pop_register16(Register16::CS, ReadWriteFlag::Normal);
            
            self.biu_queue_flush();
            self.cycles_i(2, &[0x0c7, MC_RTN]);
        }
        else {
            self.biu_queue_flush();
            self.cycles_i(2, &[0x0c5, MC_RTN]);
        }

        self.cycles_i(2, &[0x0c7, MC_RTN]);
    }
}