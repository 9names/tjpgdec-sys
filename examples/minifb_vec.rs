use minifb::{Key, Window, WindowOptions};
use tjpgdec_sys::*;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

#[repr(C)]
struct io_dev {
    f: File,         /* File pointer for input function */
    fb_ptr: *mut u8, /* Pointer to the frame buffer for output function */
    wfbuf: usize,    /* Width of the frame buffer [pix] */
}

/// Returns number of bytes read (zero on error)
unsafe extern "C" fn jpeg_file_read(
    jd: *mut JDEC,                 /* Decompression object */
    target_buf: *mut cty::c_uchar, /* Pointer to the read buffer (null to remove data) */
    len: u32,                      /* Number of bytes to read/remove */
) -> u32 {
    let iodev = (*jd).device as *mut io_dev;
    let mut file = &(*iodev).f;
    if !target_buf.is_null() {
        /* Read data from input stream */
        let mut buf = std::slice::from_raw_parts_mut(target_buf, len as usize);
        file.read(&mut buf).unwrap_or(0) as u32
    } else {
        /* Remove data from input stream */
        file.seek(SeekFrom::Current(len as i64)).unwrap_or(0) as u32
    }
}

/// Initial port of output function. non-functional!
unsafe extern "C" fn r_out_func(jd: *mut JDEC, bitmap: *mut cty::c_void, rect: *mut JRECT) -> i32 {
    let iodev = (*jd).device as *mut io_dev;
    let r = *(rect);
    let framebuf = (*iodev).fb_ptr as *mut std::ffi::c_void;
    let (top, bottom, left, right) = (
        r.top as usize,
        r.bottom as usize,
        r.left as usize,
        r.right as usize,
    );

    let dstoffset = 3 * (top * (*iodev).wfbuf + left);
    let bws = 3 * (right - left + 1);
    let bwd = 3 * (*iodev).wfbuf;
    println!("Top {}", top);
    for y in top..bottom {
        std::ptr::copy_nonoverlapping(
            bitmap.add(y * bws),
            framebuf.add(dstoffset + y * bwd),
            bws as usize,
        );
    }
    return 1;
}

fn main() {
    // Hardcode our render buffer for now
    const BUF_WIDTH: usize = 640;
    const BUF_HEIGHT: usize = 480;
    const BUFSIZE: usize = BUF_WIDTH * BUF_HEIGHT;

    // Buffer is 888RGB, so we need 3 bytes per pixel
    const PIXEL_BYTES: usize = 3;

    let mut fb = vec![0 as u32; WIDTH * HEIGHT];
    let mut frame_buffer = vec![0 as cty::c_uchar; BUFSIZE * PIXEL_BYTES];
    // Create our window so we've got somewhere to put our pixels
    let mut window = Window::new(
        "tjpgd demo - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    window.limit_update_rate(Some(std::time::Duration::from_micros(1_000_000 / 60)));

    let mut work_buffer = vec![0 as u8; 4000];

    // Init our C structs
    let mut jdec = JDEC::new();

    /* Initialize input stream */
    let filename_r = "src/tulips.jpg";
    let myfile = File::open(filename_r).expect("no file found");
    let mut dev = io_dev {
        f: myfile,
        fb_ptr: frame_buffer.as_mut_ptr(),
        wfbuf: BUF_WIDTH,
    };

    let res = unsafe {
        jd_prepare(
            &mut jdec as *mut JDEC,
            Some(jpeg_file_read),
            work_buffer.as_mut_ptr() as *mut cty::c_void,
            work_buffer.len() as u32,
            //&mut devid as *mut _ as *mut cty::c_void,
            &mut dev as *mut _ as *mut cty::c_void,
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

    assert!((jdec.width as usize) <= BUF_WIDTH);
    assert!((jdec.height as usize) <= BUF_HEIGHT);

    let res = unsafe { jd_decomp(&mut jdec, Some(r_out_func), 0) }; /* Start to decompress with 1/1 scaling */
    if res == JRESULT_JDR_OK {
        /* Decompression succeeded. You have the decompressed image in the frame buffer here. */
        println!("\rDecompression succeeded.\n");
    } else {
        println!("Failed to decompress. (rc={})\n", res);
    }

    /// tjpgd packs rgb into 3 bytes, convert that into a u32 for minifb
    fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
        let (r, g, b) = (r as u32, g as u32, b as u32);
        (r << 16) | (g << 8) | b
    }
    for (i, d) in fb.iter_mut().enumerate() {
        let fb_offset = i * 3;
        *d = from_u8_rgb(
            frame_buffer[fb_offset],
            frame_buffer[fb_offset + 1],
            frame_buffer[fb_offset + 2],
        );
    }

    // Main event loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer( &fb, WIDTH, HEIGHT)
            .unwrap();
    }
}
