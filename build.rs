extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        // Wrapper includes our includes
        .header("src/wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .use_core()
        .ctypes_prefix("cty")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Set up compile time config for lib
    // #define JD_FORMAT		0   /* Output pixel format 0:RGB888 (3 BYTE/pix), 1:RGB565 (1 WORD/pix) */
    // Output RGB565 instead of RGB888
    let rgb_mode = if cfg!(feature = "RGB565") { "1" } else { "0" };

    //#define	JD_USE_SCALE	1	/* Use descaling feature for output */
    let descaling = if cfg!(feature = "descale") { "1" } else { "0" };

    //#define JD_TBLCLIP		1	/* Use table for saturation (might be a bit faster but increases 1K bytes of code size) */
    let table_sat = if cfg!(feature = "table_sat") {
        "1"
    } else {
        "0"
    };

    // Build the tjpgd static lib as well
    cc::Build::new()
        .shared_flag(false)
        .static_flag(true)
        .define("JD_FORMAT", Some(rgb_mode))
        .define("JD_USE_SCALE", Some(descaling))
        .define("JD_TBLCLIP", Some(table_sat))
        .file("tjpgd/tjpgd.c")
        .file("src/wrapper.c")
        .compile("jpegdec");
}
