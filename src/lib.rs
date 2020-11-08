#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[allow(dead_code)]
impl JDEC {
    pub fn new() -> JDEC {
        unsafe { JDEC_new() }
    }

}
impl Default for JDEC {
    fn default() -> JDEC {
        JDEC::new()
    }
}

#[allow(dead_code)]
impl IODEV {
    pub fn new() -> IODEV {
        unsafe { IODEV_new() }
    }
}
impl Default for IODEV {
    fn default() -> IODEV {
        IODEV::new()
    }
}

#[allow(dead_code)]
impl JRECT {
    pub fn new() -> JRECT {
        unsafe { JRECT_new() }
    }
}
impl Default for JRECT {
    fn default() -> JRECT {
        JRECT::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decode() {
        // Need callback functions for read and write
        unsafe extern "C" fn _r_out_func(_jd: *mut JDEC, _bitmap: *mut cty::c_void, _rect: *mut JRECT) -> i32{ 
            1 // Pretend we managed to draw successfully
        }
        // Not using rust in-function yet
        extern "C" fn _r_in_func(
            /* Returns number of bytes read (zero on error) */
            _jd: *mut JDEC,           /* Decompression object */
            _buff: *mut cty::c_uchar, /* Pointer to the read buffer (null to remove data) */
            _nbyte: cty::c_int,       /* Number of bytes to read/remove */
        ) -> cty::c_int {
            0
        }
        let mut work_buffer: [u8; 10000] = [0; 10000];

        let mut jdec = JDEC::new();
        let mut devid = IODEV::new();

        let filename = "src/tulips.jpg\0";
        let file_mode = "rb\0";

        /* Initialize input stream */
        devid.fp = unsafe {
            fopen(
                filename.as_ptr() as *const i8,
                file_mode.as_ptr() as *const i8,
            )
        };
        if devid.fp.is_null() {
            panic!("Could not open file");
        }

        let res = unsafe {
            jd_prepare(
                &mut jdec as *mut JDEC,
                Some(in_func),
                work_buffer.as_mut_ptr() as *mut cty::c_void,
                work_buffer.len() as u32,
                &mut devid as *mut _ as *mut cty::c_void,
            )
        };
        if res != JRESULT_JDR_OK {
            panic!("Failed to prepare decode");
        }

        /* It is ready to dcompress and image info is available here */
        println!(
            "Image size is {} x {}. {} bytes of work area is used.",
            jdec.width,
            jdec.height,
            work_buffer.len() as u32 - jdec.sz_pool
        );

        // Hardcode our render buffer for now
        const BUF_WIDTH: usize = 640;
        const BUF_HEIGHT: usize = 480;

        assert!((jdec.width as usize) <= BUF_WIDTH);
        assert!((jdec.height as usize) <= BUF_HEIGHT);
        const BUFSIZE: usize = BUF_WIDTH * BUF_HEIGHT;
        // Buffer is 888RGB, so we need 3 bytes per pixel
        const PIXEL_BYTES: usize = 3;
        let mut frame_buffer: [cty::c_uchar; BUFSIZE * PIXEL_BYTES] = [0; BUFSIZE * PIXEL_BYTES];

        devid.fbuf = frame_buffer.as_mut_ptr() as *mut cty::c_uchar;
        devid.wfbuf = jdec.width as u32;

        let res = unsafe { jd_decomp(&mut jdec, Some(_r_out_func), 0) }; /* Start to decompress with 1/1 scaling */
        if res == JRESULT_JDR_OK {
            /* Decompression succeeded. You have the decompressed image in the frame buffer here. */
            println!("\rDecompression succeeded.\n");
        } else {
            println!("Failed to decompress. (rc={})\n", res);
        }

        unsafe {
            fclose(devid.fp);
        }
    }
}
