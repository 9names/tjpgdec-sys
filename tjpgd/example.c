 /*------------------------------------------------*/
/* TJpgDec Quick Evaluation Program for PCs       */
/*------------------------------------------------*/

#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include "tjpgd.h"


/* User defined device identifier */
typedef struct {
    FILE *fp;               /* File pointer for input function */
    uint8_t *fbuf;          /* Pointer to the frame buffer for output function */
    unsigned int wfbuf;     /* Width of the frame buffer [pix] */
} IODEV;


/*------------------------------*/
/* User defined input funciton  */
/*------------------------------*/

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


/*------------------------------*/
/* User defined output funciton */
/*------------------------------*/

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
    if (rect->left == 0) {
        printf("\r%lu%%", (rect->top << jd->scale) * 100UL / jd->height);
    }

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


/*------------------------------*/
/* Program Main                 */
/*------------------------------*/

int main (int argc, char* argv[])
{
    JRESULT res;      /* Result code of TJpgDec API */
    JDEC jdec;        /* Decompression object */
    void *work = (void*)malloc(3100);  /* Pointer to the work area */
    IODEV devid;      /* User defined device identifier */


    /* Initialize input stream */
    if (argc < 2) return -1;
    devid.fp = fopen(argv[1], "rb");
    if (!devid.fp) return -1;

    /* Prepare to decompress */
    res = jd_prepare(&jdec, in_func, work, 3100, &devid);
    if (res == JDR_OK) {
        /* It is ready to dcompress and image info is available here */
        printf("Image size is %u x %u.\n%u bytes of work ares is used.\n", jdec.width, jdec.height, 3100 - jdec.sz_pool);

        /* Initialize output device */
        devid.fbuf = (uint8_t*)malloc(3 * jdec.width * jdec.height); /* Create frame buffer for output image (assuming RGB888 cfg) */
        devid.wfbuf = jdec.width;

        res = jd_decomp(&jdec, out_func, 0);   /* Start to decompress with 1/1 scaling */
        if (res == JDR_OK) {
            /* Decompression succeeded. You have the decompressed image in the frame buffer here. */
            printf("\rDecompression succeeded.\n");

        } else {
            printf("Failed to decompress. (rc=%d)\n", res);
        }

        free(devid.fbuf);    /* Discard frame buffer */

    } else {
        printf("Failed to prepare. (rc=%d)\n", res);
    }

    free(work);             /* Discard work area */

    fclose(devid.fp);       /* Close the JPEG file */

    return res;
}
