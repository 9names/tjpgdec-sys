use minifb::{Key, Window, WindowOptions};
use tjpgdec_sys::*;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

fn main() {
    let mut fb: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];
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

    let mut work_buffer: [u8; 10000] = [0; 10000];

    // Init our C structs
    let mut jdec = JDEC::new();
    let mut devid = IODEV::new();

    /* Initialize input stream */
    // File names are going to be passed as C strings.
    // To keep us no-std friendly, manually null terminal our &str's
    let filename = "src/tulips.jpg\0";
    let file_mode = "rb\0";
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

    let res = unsafe { jd_decomp(&mut jdec, Some(out_func), 0) }; /* Start to decompress with 1/1 scaling */
    if res == JRESULT_JDR_OK {
        /* Decompression succeeded. You have the decompressed image in the frame buffer here. */
        println!("\rDecompression succeeded.\n");
    } else {
        println!("Failed to decompress. (rc={})\n", res);
    }

    unsafe {
        fclose(devid.fp);
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
        window.update_with_buffer(&fb, WIDTH, HEIGHT).unwrap();
    }
}
