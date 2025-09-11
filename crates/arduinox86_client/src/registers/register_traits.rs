/*
    ArduinoX86 Copyright 2022-2025 Daniel Balsom
    https://github.com/dbalsom/arduinoX86

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
*/

pub trait Registers16 {
    fn ax(&self) -> u16;
    fn bx(&self) -> u16;
    fn cx(&self) -> u16;
    fn dx(&self) -> u16;
    fn sp(&self) -> u16;
    fn bp(&self) -> u16;
    fn si(&self) -> u16;
    fn di(&self) -> u16;
    fn cs(&self) -> u16;
    fn ds(&self) -> u16;
    fn es(&self) -> u16;

    fn ss(&self) -> u16;
    fn ip(&self) -> u16;
    fn flags(&self) -> u16;

    fn ax_mut(&mut self) -> &mut u16;
    fn bx_mut(&mut self) -> &mut u16;
    fn cx_mut(&mut self) -> &mut u16;
    fn dx_mut(&mut self) -> &mut u16;
    fn sp_mut(&mut self) -> &mut u16;
    fn bp_mut(&mut self) -> &mut u16;
    fn si_mut(&mut self) -> &mut u16;
    fn di_mut(&mut self) -> &mut u16;
    fn cs_mut(&mut self) -> &mut u16;
    fn ds_mut(&mut self) -> &mut u16;
    fn es_mut(&mut self) -> &mut u16;
    fn ss_mut(&mut self) -> &mut u16;
    fn ip_mut(&mut self) -> &mut u16;
    fn flags_mut(&mut self) -> &mut u16;

    fn set_ax(&mut self, value: u16);
    fn set_bx(&mut self, value: u16);
    fn set_cx(&mut self, value: u16);
    fn set_dx(&mut self, value: u16);
    fn set_sp(&mut self, value: u16);
    fn set_bp(&mut self, value: u16);
    fn set_si(&mut self, value: u16);
    fn set_di(&mut self, value: u16);
    fn set_cs(&mut self, value: u16);
    fn set_ds(&mut self, value: u16);
    fn set_es(&mut self, value: u16);
    fn set_ss(&mut self, value: u16);
    fn set_ip(&mut self, value: u16);
    fn set_flags(&mut self, value: u16);
}

pub trait Registers32 {
    fn cr0(&self) -> u32;
    //fn cr3(&self) -> u32;
    fn dr6(&self) -> u32;
    fn dr7(&self) -> u32;

    fn eax(&self) -> u32;
    fn ebx(&self) -> u32;
    fn ecx(&self) -> u32;
    fn edx(&self) -> u32;
    fn esp(&self) -> u32;
    fn ebp(&self) -> u32;
    fn esi(&self) -> u32;
    fn edi(&self) -> u32;
    fn eip(&self) -> u32;
    fn eflags(&self) -> u32;

    fn cs(&self) -> u16;
    fn ds(&self) -> u16;
    fn es(&self) -> u16;
    fn fs(&self) -> u16;
    fn gs(&self) -> u16;
    fn ss(&self) -> u16;

    fn cr0_mut(&mut self) -> &mut u32;
    //fn cr3_mut(&mut self) -> &mut u32;
    fn dr6_mut(&mut self) -> &mut u32;
    fn dr7_mut(&mut self) -> &mut u32;

    fn eax_mut(&mut self) -> &mut u32;
    fn ebx_mut(&mut self) -> &mut u32;
    fn ecx_mut(&mut self) -> &mut u32;
    fn edx_mut(&mut self) -> &mut u32;
    fn esp_mut(&mut self) -> &mut u32;
    fn ebp_mut(&mut self) -> &mut u32;
    fn esi_mut(&mut self) -> &mut u32;
    fn edi_mut(&mut self) -> &mut u32;
    fn eip_mut(&mut self) -> &mut u32;
    fn eflags_mut(&mut self) -> &mut u32;

    fn cs_mut(&mut self) -> &mut u16;
    fn ds_mut(&mut self) -> &mut u16;
    fn es_mut(&mut self) -> &mut u16;
    fn fs_mut(&mut self) -> &mut u16;
    fn gs_mut(&mut self) -> &mut u16;
    fn ss_mut(&mut self) -> &mut u16;

    fn set_cr0(&mut self, value: u32);
    //fn set_cr3(&mut self, value: u32);
    fn set_dr6(&mut self, value: u32);
    fn set_dr7(&mut self, value: u32);

    fn set_eax(&mut self, value: u32);
    fn set_ebx(&mut self, value: u32);
    fn set_ecx(&mut self, value: u32);
    fn set_edx(&mut self, value: u32);
    fn set_esp(&mut self, value: u32);
    fn set_ebp(&mut self, value: u32);
    fn set_esi(&mut self, value: u32);
    fn set_edi(&mut self, value: u32);

    fn set_cs(&mut self, value: u16);
    fn set_ds(&mut self, value: u16);
    fn set_es(&mut self, value: u16);
    fn set_fs(&mut self, value: u16);
    fn set_gs(&mut self, value: u16);
    fn set_ss(&mut self, value: u16);

    fn set_eip(&mut self, value: u32);
    fn set_eflags(&mut self, value: u32);

    fn normalize_descriptors(&mut self);
}
