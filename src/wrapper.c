#include "wrapper.h"
#include <string.h>
unsigned int in_func (  /* Returns number of bytes read (zero on error) */
    JDEC* jd,           /* Decompression object */
    uint8_t* buff,      /* Pointer to the read buffer (null to remove data) */
    unsigned int nbyte  /* Number of bytes to read/remove */
)
{
    IODEV *dev = (IODEV*)jd->device;   /* Device identifier for the session (5th argument of jd_prepare function) */


    if (buff) { /* Raad data from imput stream */
        return (unsigned int)fread(buff, 1, nbyte, dev->fp);
    } else {    /* Remove data from input stream */
        return fseek(dev->fp, nbyte, SEEK_CUR) ? 0 : nbyte;
    }
}


int out_func (      /* 1:Ok, 0:Aborted */
    JDEC* jd,       /* Decompression object */
    void* bitmap,   /* Bitmap data to be output */
    JRECT* rect     /* Rectangular region of output image */
)
{
    IODEV *dev = (IODEV*)jd->device;
    uint8_t *src, *dst;
    uint16_t y, bws, bwd;


    /* Put progress indicator */
    // if (rect->left == 0) {
    //     printf("\r%lu%%", (rect->top << jd->scale) * 100UL / jd->height);
    // }

    /* Copy the decompressed RGB rectanglar to the frame buffer (assuming RGB888 cfg) */
    src = (uint8_t*)bitmap;
    dst = dev->fbuf + 3 * (rect->top * dev->wfbuf + rect->left);  /* Left-top of destination rectangular */
    bws = 3 * (rect->right - rect->left + 1);     /* Width of source rectangular [byte] */
    bwd = 3 * dev->wfbuf;                         /* Width of frame buffer [byte] */
    for (y = rect->top; y <= rect->bottom; y++) {
        memcpy(dst, src, bws);   /* Copy a line */
        src += bws; dst += bwd;  /* Next line */
    }

    return 1;    /* Continue to decompress */
}


JDEC JDEC_new(){
    JDEC j = {0};
    return j;
}

JRECT JRECT_new(){
    JRECT j = {0};
    return j;
}

IODEV IODEV_new(){
    IODEV i = {0};
    return i;
}