use core::{num, slice};

use bitreader::BitReader;

use crate::a52_state_t;

fn swab32(x: u32) -> u32 {
    u32::from_be(x)
}

#[no_mangle]
pub unsafe fn a52_bitstream_set_ptr (state: *mut a52_state_t, buf: *mut u8)
{
    let align = buf as isize & 3;

    (*state).buffer_start = (buf as isize - align) as _;
    (*state).bits_left = 0;
    bitstream_get (state, (align * 8) as u32);
}

#[no_mangle]
pub unsafe fn bitstream_fill_current (state: *mut a52_state_t)
{
    let tmp = *(*state).buffer_start;
    (*state).buffer_start = (*state).buffer_start.offset(1);
    (*state).current_word = swab32 (tmp);
}

extern "C" {
    fn bitstream_get(state: *mut a52_state_t, num_bits: u32) -> u32;
}