#![allow(non_camel_case_types, non_upper_case_globals)]

use core::slice;
use std::{
    alloc::{self, Layout},
    ptr,
};

use libc::{c_float, c_int, c_void};
use bitreader::BitReader;

mod idct;
mod bitstream;

pub type DynRingFunction = unsafe extern "C" fn(range: sample_t, dynrngdata: *mut c_void) -> sample_t;

pub const A52_CHANNEL: c_int = 0;
pub const A52_MONO: c_int = 1;
pub const A52_STEREO: c_int = 2;
pub const A52_3F: c_int = 3;
pub const A52_2F1R: c_int = 4;
pub const A52_3F1R: c_int = 5;
pub const A52_2F2R: c_int = 6;
pub const A52_3F2R: c_int = 7;
pub const A52_CHANNEL1: c_int = 8;
pub const A52_CHANNEL2: c_int = 9;
pub const A52_DOLBY: c_int = 10;
pub const A52_CHANNEL_MASK: c_int = 15;

pub const A52_LFE: c_int = 16;
pub const A52_ADJUST_LEVEL: c_int = 32;

pub type sample_t = c_float;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct expbap_t {
    pub exp: [u8; 256],
    pub bap: [i8; 256],
}

impl Default for expbap_t {
    fn default() -> Self {
        Self {
            exp: [0; 256],
            bap: [0; 256],
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ba_t {
    pub bai: u8,
    pub deltbae: u8,
    pub deltba: [i8; 50],
}

impl Default for ba_t {
    fn default() -> Self {
        Self {
            bai: 0,
            deltbae: 0,
            deltba: [0; 50],
        }
    }
}

#[repr(C)]
pub struct a52_state_s {
    pub fscod: u8,
    pub halfrate: u8,
    pub acmod: u8,
    pub lfeon: u8,
    pub clev: sample_t,
    pub slev: sample_t,
    pub output: c_int,
    pub level: sample_t,
    pub bias: sample_t,
    pub dynrnge: c_int,
    pub dynrng: sample_t,
    pub dynrngdata: *mut c_void,
    pub dynrngcall:
        Option<DynRingFunction>,
    pub chincpl: u8,
    pub phsflginu: u8,
    pub cplstrtmant: u8,
    pub cplendmant: u8,
    pub cplbndstrc: u32,
    pub cplco: [[sample_t; 18]; 5],
    pub cplstrtbnd: u8,
    pub ncplbnd: u8,
    pub rematflg: u8,
    pub endmant: [u8; 5],
    pub bai: u16,
    pub buffer_start: *mut u32,
    pub lfsr_state: u16,
    pub bits_left: u32,
    pub current_word: u32,
    pub csnroffst: u8,
    pub cplba: ba_t,
    pub ba: [ba_t; 5],
    pub lfeba: ba_t,
    pub cplfleak: u8,
    pub cplsleak: u8,
    pub cpl_expbap: expbap_t,
    pub fbw_expbap: [expbap_t; 5],
    pub lfe_expbap: expbap_t,
    pub samples: *mut sample_t,
    pub downmixed: c_int,
}

pub type a52_state_t = a52_state_s;

impl a52_state_s {
    pub fn new() -> Self {
        Self {
            fscod: 0,
            halfrate: 0,
            acmod: 0,
            lfeon: 0,
            clev: 0.0,
            slev: 0.0,
            output: 0,
            level: 0.0,
            bias: 0.0,
            dynrnge: 0,
            dynrng: 0.0,
            dynrngdata: ptr::null_mut(),
            dynrngcall: None,
            chincpl: 0,
            phsflginu: 0,
            cplstrtmant: 0,
            cplendmant: 0,
            cplbndstrc: 0,
            cplco: [[0.0; 18]; 5],
            cplstrtbnd: 0,
            ncplbnd: 0,
            rematflg: 0,
            endmant: [0; 5],
            bai: 0,
            buffer_start: ptr::null_mut(),
            lfsr_state: 0,
            bits_left: 0,
            current_word: 0,
            csnroffst: 0,
            cplba: ba_t::default(),
            ba: [ba_t::default(); 5],
            lfeba: ba_t::default(),
            cplfleak: 0,
            cplsleak: 0,
            cpl_expbap: expbap_t::default(),
            fbw_expbap: [expbap_t::default(); 5],
            lfe_expbap: expbap_t::default(),
            samples: ptr::null_mut(),
            downmixed: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

static halfrate: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3];

#[no_mangle]
pub unsafe fn a52_init(mm_accel: u32) -> *mut a52_state_t {
    let mut state = a52_state_s::new();

    let layout = match Layout::array::<sample_t>(256 * 12) {
        Ok(layout) => layout,
        Err(_) => return ptr::null_mut(),
    };
    state.samples = alloc::alloc_zeroed(layout) as *mut sample_t;
    if state.samples.is_null() {
        return ptr::null_mut();
    };

    state.downmixed = 1;

    state.lfsr_state = 1;

    a52_imdct_init(mm_accel);

    Box::into_raw(Box::new(state))
}

#[no_mangle]
pub unsafe fn a52_samples(state: *mut a52_state_t) -> *mut sample_t {
    assert!(!state.is_null());

    (*state).samples
}

#[no_mangle]
pub unsafe fn a52_syncinfo(
    buf: *mut u8,
    flags: &mut c_int,
    sample_rate: &mut c_int,
    bit_rate: &mut c_int,
) -> c_int {
    assert!(!buf.is_null());

    let rate = [
        32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384, 448, 512, 576, 640,
    ];
    let lfeon: [u8; 8] = [0x10, 0x10, 0x04, 0x04, 0x04, 0x01, 0x04, 0x01];

    if (*buf.offset(0) != 0x0b) || (*buf.offset(1) != 0x77) {
        return 0;
    }

    if *buf.offset(5) >= 0x60 {
        return 0;
    }
    let half = halfrate[*buf.offset(5) as usize >> 3] as i32;

    /* acmod, dsurmod and lfeon */
    let acmod = *buf.offset(6) as i32 >> 5;
    *flags = if *buf.offset(6) & 0xf8 as u8 == 0x50 {
        A52_DOLBY
    } else {
        acmod
    } | if *buf.offset(6) & lfeon[acmod as usize] != 0 {
        A52_LFE
    } else {
        0
    };

    let frmsizecod: c_int = (*buf.offset(4) & 63) as c_int;
    if frmsizecod >= 38 {
        return 0;
    }
    let bitrate = rate[frmsizecod as usize >> 1] as c_int;
    *bit_rate = (bitrate * 1000) >> half;

    match *buf.offset(4) & 0xc0 {
        0 => {
            *sample_rate = 48000 >> half;
            return 4 * bitrate;
        }
        0x40 => {
            *sample_rate = 44100 >> half;
            return 2 * (320 * bitrate / 147 + (frmsizecod & 1));
        }
        0x80 => {
            *sample_rate = 32000 >> half;
            return 6 * bitrate;
        }
        _ => 0,
    };

    0
}

const LEVEL_PLUS6DB: sample_t = 2.0;
const LEVEL_PLUS3DB: sample_t = 1.4142135623730951;
const LEVEL_3DB: sample_t = 0.7071067811865476;
const LEVEL_45DB: sample_t = 0.5946035575013605;
const LEVEL_6DB: sample_t = 0.5;

const DELTA_BIT_REUSE: u8 = 0;
const DELTA_BIT_NEW: u8 =  1;
const DELTA_BIT_NONE: u8 = 2;
const DELTA_BIT_RESERVED: u8 = 3;

#[no_mangle]
pub unsafe fn a52_frame (state: *mut a52_state_t, buf: *mut u8, flags: *mut c_int,
    level: *mut sample_t, bias: sample_t) -> c_int
{
    let clev: [sample_t; 4] = [LEVEL_3DB, LEVEL_45DB, LEVEL_6DB, LEVEL_45DB];
    let slev: [sample_t; 4] = [LEVEL_3DB, LEVEL_6DB, 0.0, LEVEL_6DB];

    let buf = slice::from_raw_parts_mut(buf, 1024);

    (*state).fscod = buf[4] >> 6;
    (*state).halfrate = halfrate[(buf[5] >> 3) as usize];
    let mut acmod = buf[6] >> 5;
    (*state).acmod = acmod;

    let mut reader = BitReader::new(buf);

    // a52_bitstream_set_ptr (state, buf + 6);
    reader.skip(6 * 8).unwrap();
    // bitstream_get (state, 3);	/* skip acmod we already parsed */
    reader.skip(3).unwrap();

    if (acmod == 2) && (reader.read_u32(2).unwrap() == 2)	/* dsurmod */ {
        acmod = A52_DOLBY as u8;
    }

    if (acmod & 1 != 0) && (acmod != 1) {
        (*state).clev = clev[reader.read_u32 ( 2).unwrap() as usize];	/* cmixlev */
    }

    if acmod & 4 != 0 {
        (*state).slev = slev[reader.read_u8 (2).unwrap() as usize];	/* surmixlev */
    }

    (*state).lfeon = reader.read_u8 ( 1).unwrap();

    (*state).output = a52_downmix_init (acmod as c_int, *flags, level,
                (*state).clev, (*state).slev);
    if (*state).output < 0 {
        return 1;
    }
    if (*state).lfeon != 0 && (*flags & A52_LFE != 0) {
        (*state).output |= A52_LFE;
    }
    *flags = (*state).output;
    /* the 2* compensates for differences in imdct */
    (*state).level = 2.0 * *level;
    (*state).dynrng = (*state).level;
    (*state).bias = bias;
    (*state).dynrnge = 1;
    (*state).dynrngcall = None;
    (*state).cplba.deltbae = DELTA_BIT_NONE;
    (*state).ba[0].deltbae = DELTA_BIT_NONE;
    (*state).ba[1].deltbae = DELTA_BIT_NONE;
    (*state).ba[2].deltbae = DELTA_BIT_NONE;
    (*state).ba[3].deltbae = DELTA_BIT_NONE;
    (*state).ba[4].deltbae = DELTA_BIT_NONE;

    let mut chaninfo = !acmod;
    loop {
        if chaninfo == 0 {
            break;
        }
        reader.skip(5).unwrap();	/* dialnorm */
        if reader.read_bool().unwrap() {	/* compre */
            reader.skip (8).unwrap();	/* compr */
        };
        if reader.read_bool().unwrap() {	/* langcode */
            reader.skip(8).unwrap();	/* langcod */
        };
        if reader.read_bool ().unwrap() {	/* audprodie */
            reader.skip (7).unwrap();	/* mixlevel + roomtyp */
        }
        chaninfo -= 1;
    };

    reader.skip (2).unwrap();		/* copyrightb + origbs */

    if reader.read_bool ().unwrap() {  /* timecod1e */
        reader.skip (14).unwrap();	/* timecod1 */
    };
    if reader.read_bool ().unwrap() {	/* timecod2e */
        reader.skip (14).unwrap();	/* timecod2 */
    };

    if reader.read_bool ().unwrap() {	/* addbsie */
        let mut addbsil = reader.read_i32 ( 6).unwrap();
        loop {
            if addbsil <= 0 {
                break;
            };

            reader.skip ( 8).unwrap();	/* addbsi */
            addbsil -= 1;
        };
    }

    0
}

#[no_mangle]
pub unsafe fn a52_dynrng(state: *mut a52_state_t, call: Option<DynRingFunction>, data: *mut c_void)
{
    assert!(!state.is_null());
    (*state).dynrnge = 0;
    if let Some(call) = Some(call) {
        (*state).dynrnge = 1;
        (*state).dynrngcall = call;
        (*state).dynrngdata = data;
    }
}

#[no_mangle]
pub unsafe fn a52_free(state: *mut a52_state_t) {
    assert!(!state.is_null());

    let state = Box::from_raw(state);
    let layout = Layout::array::<sample_t>(256 * 12);
    match layout {
        Ok(layout) => alloc::dealloc(state.samples as _, layout),
        Err(_) => todo!(),
    };
}

extern "C" {
    fn a52_imdct_init(mm_accel: u32);
    fn a52_downmix_init (input: c_int, flags: c_int, level: *mut sample_t,
        clev: sample_t, slev: sample_t) -> c_int;
}
