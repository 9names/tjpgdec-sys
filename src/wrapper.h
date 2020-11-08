#include "../tjpgd/tjpgd.h"
#include <stdio.h>
// These were in example.c. Brought in here to help porting

/* User defined device identifier */
typedef struct {
    FILE *fp;               /* File pointer for input function */
    uint8_t *fbuf;          /* Pointer to the frame buffer for output function */
    unsigned int wfbuf;     /* Width of the frame buffer [pix] */
} IODEV;

int out_func (      /* 1:Ok, 0:Aborted */
    JDEC* jd,       /* Decompression object */
    void* bitmap,   /* Bitmap data to be output */
    JRECT* rect     /* Rectangular region of output image */
);

unsigned int in_func (  /* Returns number of bytes read (zero on error) */
    JDEC* jd,           /* Decompression object */
    uint8_t* buff,      /* Pointer to the read buffer (null to remove data) */
    unsigned int nbyte  /* Number of bytes to read/remove */
);

JDEC JDEC_new();
JRECT JRECT_new();
IODEV IODEV_new();

