//! Constants in this module represent addresses of TIA registers. To be used
//! with the `TIA::read()` and `TIA::write()` methods.

// Write registers:
pub const VSYNC: u16 = 0x00;
pub const VBLANK: u16 = 0x01;
pub const WSYNC: u16 = 0x02;
pub const RSYNC: u16 = 0x03;
pub const NUSIZ0: u16 = 0x04;
pub const NUSIZ1: u16 = 0x05;
pub const COLUP0: u16 = 0x06;
pub const COLUP1: u16 = 0x07;
pub const COLUPF: u16 = 0x08;
pub const COLUBK: u16 = 0x09;
pub const CTRLPF: u16 = 0x0A;
pub const REFP0: u16 = 0x0B;
pub const REFP1: u16 = 0x0C;
pub const PF0: u16 = 0x0D;
pub const PF1: u16 = 0x0E;
pub const PF2: u16 = 0x0F;
pub const RESP0: u16 = 0x10;
pub const RESP1: u16 = 0x11;
pub const RESM0: u16 = 0x12;
pub const RESM1: u16 = 0x13;
// pub const RESBL: u16 = 0x14;
pub const AUDC0: u16 = 0x15;
pub const AUDC1: u16 = 0x16;
pub const AUDF0: u16 = 0x17;
pub const AUDF1: u16 = 0x18;
pub const AUDV0: u16 = 0x19;
pub const AUDV1: u16 = 0x1A;
pub const GRP0: u16 = 0x1B;
pub const GRP1: u16 = 0x1C;
pub const ENAM0: u16 = 0x1D;
pub const ENAM1: u16 = 0x1E;
// pub const ENABL: u16 = 0x1F;
pub const HMP0: u16 = 0x20;
pub const HMP1: u16 = 0x21;
pub const HMM0: u16 = 0x22;
pub const HMM1: u16 = 0x23;
// pub const HMBL: u16 = 0x24;
// pub const VDELP0: u16 = 0x25;
// pub const VDELP1: u16 = 0x26;
// pub const VDELBL: u16 = 0x27;
pub const RESMP0: u16 = 0x28;
pub const RESMP1: u16 = 0x29;
pub const HMOVE: u16 = 0x2A;
pub const HMCLR: u16 = 0x2B;
pub const CXCLR: u16 = 0x2C;

// Read registers:
pub const CXM0P: u16 = 0x00;
pub const CXM1P: u16 = 0x01;
pub const CXP0FB: u16 = 0x02;
pub const CXP1FB: u16 = 0x03;
pub const CXM0FB: u16 = 0x04;
pub const CXM1FB: u16 = 0x05;
pub const CXBLPF: u16 = 0x06;
pub const CXPPMM: u16 = 0x07;
// pub const INPT0: u16 = 0x08;
// pub const INPT1: u16 = 0x09;
// pub const INPT2: u16 = 0x0A;
// pub const INPT3: u16 = 0x0B;
pub const INPT4: u16 = 0x0C;
pub const INPT5: u16 = 0x0D;
