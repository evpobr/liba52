use crate::sample_t;

use std::slice;

use lazy_static::lazy_static;
use libc::c_int;

#[repr(C)]
pub struct complex_s {
    pub real: sample_t,
    pub imag: sample_t,
}

pub type complex_t = complex_s;

lazy_static! {
    static ref ROOTS16: [sample_t; 3] = {
        let mut roots16 = [0.0; 3];
        for i in 0..3 {
            roots16[i] = ((std::f32::consts::PI / 8.0) * (i + 1) as f32).cos();
        };  
        roots16
    };
}

#[no_mangle]
pub unsafe fn ifft2(buf: *mut complex_t) {
    assert!(!buf.is_null());
    let buf = slice::from_raw_parts_mut(buf, 2);

    let r = buf[0].real;
    let i = buf[0].imag;
    buf[0].real += buf[1].real;
    buf[0].imag += buf[1].imag;
    buf[1].real = r - buf[1].real;
    buf[1].imag = i - buf[1].imag;
}

#[no_mangle]
pub unsafe fn ifft4(buf: *mut complex_t) {
    assert!(!buf.is_null());
    let buf = slice::from_raw_parts_mut(buf, 4);

    let tmp1 = buf[0].real + buf[1].real;
    let tmp2 = buf[3].real + buf[2].real;
    let tmp3 = buf[0].imag + buf[1].imag;
    let tmp4 = buf[2].imag + buf[3].imag;
    let tmp5 = buf[0].real - buf[1].real;
    let tmp6 = buf[0].imag - buf[1].imag;
    let tmp7 = buf[2].imag - buf[3].imag;
    let tmp8 = buf[3].real - buf[2].real;

    buf[0].real = tmp1 + tmp2;
    buf[0].imag = tmp3 + tmp4;
    buf[2].real = tmp1 - tmp2;
    buf[2].imag = tmp3 - tmp4;
    buf[1].real = tmp5 + tmp7;
    buf[1].imag = tmp6 + tmp8;
    buf[3].real = tmp5 - tmp7;
    buf[3].imag = tmp6 - tmp8;
}

macro_rules! BUTTERFLY {
    ($a0:expr,$a1:expr,$a2:expr,$a3:expr,$wr:expr,$wi:expr) => {
        tmp5 = $a2.real * $wr + $a2.imag * $wi;
        tmp6 = $a2.imag * $wr - $a2.real * $wi;
        tmp7 = $a3.real * $wr - $a3.imag * $wi;
        tmp8 = $a3.imag * $wr + $a3.real * $wi;
        tmp1 = tmp5 + tmp7;
        tmp2 = tmp6 + tmp8;
        tmp3 = tmp6 - tmp8;
        tmp4 = tmp7 - tmp5;
        $a2.real = $a0.real - tmp1;
        $a2.imag = $a0.imag - tmp2;
        $a3.real = $a1.real - tmp3;
        $a3.imag = $a1.imag - tmp4;
        $a0.real += tmp1;
        $a0.imag += tmp2;
        $a1.real += tmp3;
        $a1.imag += tmp4;
    };
}

macro_rules! BUTTERFLY_ZERO {
    ($a0:expr,$a1:expr,$a2:expr,$a3:expr) => {
        let tmp1 = $a2.real + $a3.real;
        let tmp2 = $a2.imag + $a3.imag;
        let tmp3 = $a2.imag - $a3.imag;
        let tmp4 = $a3.real - $a2.real;
        $a2.real = $a0.real - tmp1;
        $a2.imag = $a0.imag - tmp2;
        $a3.real = $a1.real - tmp3;
        $a3.imag = $a1.imag - tmp4;
        $a0.real += tmp1;
        $a0.imag += tmp2;
        $a1.real += tmp3;
        $a1.imag += tmp4;
    };
}

macro_rules! BUTTERFLY_HALF {
    ($a0:expr,$a1:expr,$a2:expr,$a3:expr,$w:expr) => {
        let tmp5 = ($a2.real + $a2.imag) * $w;
        let tmp6 = ($a2.imag - $a2.real) * $w;
        let tmp7 = ($a3.real - $a3.imag) * $w;
        let tmp8 = ($a3.imag + $a3.real) * $w;
        let tmp1 = tmp5 + tmp7;
        let tmp2 = tmp6 + tmp8;
        let tmp3 = tmp6 - tmp8;
        let tmp4 = tmp7 - tmp5;
        $a2.real = $a0.real - tmp1;
        $a2.imag = $a0.imag - tmp2;
        $a3.real = $a1.real - tmp3;
        $a3.imag = $a1.imag - tmp4;
        $a0.real += tmp1;
        $a0.imag += tmp2;
        $a1.real += tmp3;
        $a1.imag += tmp4;
    };
}

#[no_mangle]
pub unsafe fn ifft8 (buf: *mut complex_t)
{
    assert!(!buf.is_null());
    let buf = slice::from_raw_parts_mut(buf, 8);
    
    ifft4 (buf.as_mut_ptr());
    ifft2 (buf[4..].as_mut_ptr());
    ifft2 (buf[6..].as_mut_ptr());
    BUTTERFLY_ZERO! (buf[0], buf[2], buf[4], buf[6]);
    BUTTERFLY_HALF! (buf[1], buf[3], buf[5], buf[7], ROOTS16[1]);
}

#[no_mangle]
pub unsafe fn ifft16 (buf: *mut complex_t)
{
    assert!(!buf.is_null());
    let buf = slice::from_raw_parts_mut(buf, 8);

    ifft8 (buf.as_mut_ptr());
    ifft4 (buf[8..].as_mut_ptr());
    ifft4 (buf[12..].as_mut_ptr());
    ifft_pass (buf.as_mut_ptr(), ROOTS16.as_ptr().offset(-4), 4);
}

extern "C" {
    fn ifft_pass (buf: *mut complex_t, weight: *const sample_t, n: c_int);
}