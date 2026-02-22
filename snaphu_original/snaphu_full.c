/* snaphu.h */
#ifndef SNAPHU_H
#define SNAPHU_H
/*************************************************************************

  snaphu header file
  Written by Curtis W. Chen
  Copyright 2002 Board of Trustees, Leland Stanford Jr. University
  Please see the supporting documentation for terms of use.
  No warranty.

*************************************************************************/

#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <signal.h>
#include <limits.h>
#include <float.h>
#include <string.h>
#include <ctype.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <time.h>
#include <sys/time.h>
#include <sys/resource.h>
#include <assert.h>

/**********************/
/* defined constants  */
/**********************/

#define PROGRAMNAME          "snaphu"
#define VERSION              "2.0.7"
#define BUGREPORTEMAIL       "snaphu@gmail.com"
#ifdef PI
#undef PI
#endif
#define PI                   3.14159265358979323846
#define TWOPI                6.28318530717958647692
#define SQRTHALF             0.70710678118654752440
#define MAXSTRLEN            512
#define MAXTMPSTRLEN         1024
#define MAXLINELEN           2048
#define TRUE                 1
#define FALSE                0
#define LARGESHORT           32000
#define LARGEINT             2000000000
#define LARGEFLOAT           1.0e35
#define VERYFAR              LARGEINT
#define GROUNDROW            -2
#define GROUNDCOL            -2
#define BOUNDARYROW          -4
#define BOUNDARYCOL          -4
#define MAXGROUPBASE         LARGEINT
#define ONTREE               -1
#define INBUCKET             -2
#define NOTINBUCKET          -3
#define PRUNED               -4
#define MASKED               -5
#define BOUNDARYPTR          -6
#define BOUNDARYCANDIDATE    -7
#define BOUNDARYLEVEL        LARGEINT
#define INTERIORLEVEL        (BOUNDARYLEVEL-1)
#define MINBOUNDARYSIZE      100
#define POSINCR              0
#define NEGINCR              1
#define NOCOSTSHELF          -LARGESHORT
#define MINSCALARCOST        1
#define INITARRSIZE          500
#define NEWNODEBAGSTEP       500
#define CANDIDATEBAGSTEP     500
#define NEGBUCKETFRACTION    1.0
#define POSBUCKETFRACTION    1.0
#define CLIPFACTOR           0.6666666667
#define NSOURCELISTMEMINCR   1024
#define NLISTMEMINCR         1024
#define DEF_OUTFILE          "snaphu.out"
#define DEF_SYSCONFFILE      ""     /* "/usr/local/snaphu/snaphu.conf" */
#define DEF_WEIGHTFILE       ""     /* "snaphu.weight" */
#define DEF_AMPFILE          ""     /* "snaphu.amp" */
#define DEF_AMPFILE2         ""     /* "snaphu.amp" */
#define DEF_MAGFILE          ""     /* "snaphu.mag" */
#define DEF_CORRFILE         ""     /* "snaphu.corr" */
#define DEF_ESTFILE          ""     /* "snaphu.est" */
#define DEF_COSTINFILE       ""
#define DEF_BYTEMASKFILE     ""
#define DEF_DOTILEMASKFILE   ""
#define DEF_INITFILE         ""
#define DEF_FLOWFILE         ""
#define DEF_EIFILE           ""
#define DEF_ROWCOSTFILE      ""
#define DEF_COLCOSTFILE      ""
#define DEF_MSTROWCOSTFILE   ""
#define DEF_MSTCOLCOSTFILE   ""
#define DEF_MSTCOSTSFILE     ""
#define DEF_CORRDUMPFILE     ""
#define DEF_RAWCORRDUMPFILE  ""
#define DEF_CONNCOMPFILE     ""
#define DEF_COSTOUTFILE      ""
#define DEF_LOGFILE          ""
#define MAXITERATION         5000
#define NEGSHORTRANGE        SHRT_MIN
#define POSSHORTRANGE        SHRT_MAX
#define MAXRES               SCHAR_MAX
#define MINRES               SCHAR_MIN
#define PROBCOSTP            (-99.999)
#define NULLFILE             "/dev/null"
#define DEF_ERRORSTREAM      stderr
#define DEF_OUTPUTSTREAM     stdout
#define DEF_VERBOSESTREAM    NULL
#define DEF_COUNTERSTREAM    NULL
#define DEF_INITONLY         FALSE
#define DEF_INITMETHOD       MSTINIT
#define DEF_UNWRAPPED        FALSE
#define DEF_REGROWCONNCOMPS  FALSE
#define DEF_EVAL             FALSE
#define DEF_WEIGHT           1
#define DEF_COSTMODE         TOPO
#define DEF_VERBOSE          FALSE
#define DEF_AMPLITUDE        TRUE
#define AUTOCALCSTATMAX      0
#define MAXNSHORTCYCLE       8192
#define USEMAXCYCLEFRACTION  (-123)
#define COMPLEX_DATA         1         /* file format */
#define FLOAT_DATA           2         /* file format */
#define ALT_LINE_DATA        3         /* file format */
#define ALT_SAMPLE_DATA      4         /* file format */
#define TILEINITFILEFORMAT   ALT_LINE_DATA
#define TILEINITFILEROOT     "snaphu_tileinit_"
#define ABNORMAL_EXIT        1         /* exit code */
#define NORMAL_EXIT          0         /* exit code */
#define DUMP_PATH            "/tmp/"   /* default location for writing dumps */
#define NARMS                8         /* number of arms for Despeckle() */
#define ARMLEN               5         /* length of arms for Despeckle() */
#define KEDGE                5         /* length of edge detection window */
#define ARCUBOUND            200       /* capacities for cs2 */
#define MSTINIT              1         /* initialization method */
#define MCFINIT              2         /* initialization method */
#define BIGGESTDZRHOMAX      10000.0
#define SECONDSPERPIXEL      0.000001  /* for delay between thread creations */
#define MAXTHREADS           64
#define TMPTILEDIRROOT       "snaphu_tiles_"
#define TILEDIRMODE          511
#define TMPTILEROOT          "tmptile_"
#define TMPTILECOSTSUFFIX    "cost_"
#define TMPTILEOUTFORMAT     ALT_LINE_DATA
#define REGIONSUFFIX         "_regions"
#define LOGFILEROOT          "tmptilelog_"
#define RIGHT                1
#define DOWN                 2
#define LEFT                 3
#define UP                   4
#define TILEDPSICOLFACTOR    0.8
#define TILEOVRLPWARNTHRESH  400
#define ZEROCOSTARC          -LARGEINT
#define PINGPONG             2
#define SINGLEANTTRANSMIT    1
#define NOSTATCOSTS          0
#define TOPO                 1
#define DEFO                 2
#define SMOOTH               3
#define CONNCOMPOUTTYPEUCHAR 1
#define CONNCOMPOUTTYPEUINT  4


/* SAR and geometry parameter defaults */

#define DEF_ORBITRADIUS      7153000.0
#define DEF_ALTITUDE         0.0
#define DEF_EARTHRADIUS      6378000.0
#define DEF_BASELINE         150.0
#define DEF_BASELINEANGLE    (1.25*PI)
#define DEF_BPERP            0
#define DEF_TRANSMITMODE     PINGPONG
#define DEF_NLOOKSRANGE      1
#define DEF_NLOOKSAZ         5
#define DEF_NLOOKSOTHER      1
#define DEF_NCORRLOOKS       23.8
#define DEF_NCORRLOOKSRANGE  3
#define DEF_NCORRLOOKSAZ     15
#define DEF_NEARRANGE        831000.0
#define DEF_DR               8.0
#define DEF_DA               20.0
#define DEF_RANGERES         10.0
#define DEF_AZRES            6.0
#define DEF_LAMBDA           0.0565647


/* scattering model defaults */

#define DEF_KDS              0.02
#define DEF_SPECULAREXP      8.0
#define DEF_DZRCRITFACTOR    2.0
#define DEF_SHADOW           FALSE
#define DEF_DZEIMIN          -4.0
#define DEF_LAYWIDTH         16
#define DEF_LAYMINEI         1.25
#define DEF_SLOPERATIOFACTOR 1.18
#define DEF_SIGSQEI          100.0


/* decorrelation model parameters */

#define DEF_DRHO             0.005
#define DEF_RHOSCONST1       1.3
#define DEF_RHOSCONST2       0.14
#define DEF_CSTD1            0.4
#define DEF_CSTD2            0.35
#define DEF_CSTD3            0.06
#define DEF_DEFAULTCORR      0.01
#define DEF_RHOMINFACTOR     1.3


/* pdf model parameters */

#define DEF_DZLAYPEAK        -2.0
#define DEF_AZDZFACTOR       0.99
#define DEF_DZEIFACTOR       4.0
#define DEF_DZEIWEIGHT       0.5
#define DEF_DZLAYFACTOR      1.0
#define DEF_LAYCONST         0.9
#define DEF_LAYFALLOFFCONST  2.0
#define DEF_SIGSQSHORTMIN    1
#define DEF_SIGSQLAYFACTOR   0.1


/* deformation mode parameters */

#define DEF_DEFOAZDZFACTOR   1.0
#define DEF_DEFOTHRESHFACTOR 1.2
#define DEF_DEFOMAX          1.2
#define DEF_SIGSQCORR        0.05
#define DEF_DEFOLAYCONST     0.9


/* algorithm parameters */

#define DEF_FLIPPHASESIGN    FALSE
#define DEF_ONETILEREOPT     FALSE
#define DEF_RMTILEINIT       TRUE
#define DEF_MAXFLOW          4
#define DEF_KROWEI           65
#define DEF_KCOLEI           257
#define DEF_KPARDPSI         7
#define DEF_KPERPDPSI        7
#define DEF_THRESHOLD        0.001
#define DEF_INITDZR          2048.0
#define DEF_INITDZSTEP       100.0
#define DEF_MAXCOST          1000.0
#define DEF_COSTSCALE        100.0
#define DEF_COSTSCALEAMBIGHT 80.0
#define DEF_DNOMINCANGLE     0.01
#define DEF_SRCROW           -1
#define DEF_SRCCOL           -1
#define DEF_P                PROBCOSTP
#define DEF_BIDIRLPN         TRUE
#define DEF_NSHORTCYCLE      200
#define DEF_MAXNEWNODECONST  0.0008
#define DEF_MAXCYCLEFRACTION 0.00001
#define DEF_NCONNNODEMIN     0
#define DEF_MAXNFLOWCYCLES   USEMAXCYCLEFRACTION
#define DEF_INITMAXFLOW      9999
#define INITMAXCOSTINCR      200
#define NOSTATINITMAXFLOW    15
#define DEF_ARCMAXFLOWCONST  3
#define DEF_DUMPALL          FALSE
#define DUMP_INITFILE        "snaphu.init"
#define DUMP_FLOWFILE        "snaphu.flow"
#define DUMP_EIFILE          "snaphu.ei"
#define DUMP_ROWCOSTFILE     "snaphu.rowcost"
#define DUMP_COLCOSTFILE     "snaphu.colcost"
#define DUMP_MSTROWCOSTFILE  "snaphu.mstrowcost"
#define DUMP_MSTCOLCOSTFILE  "snaphu.mstcolcost"
#define DUMP_MSTCOSTSFILE    "snaphu.mstcosts"
#define DUMP_CORRDUMPFILE    "snaphu.corr"
#define DUMP_RAWCORRDUMPFILE "snaphu.rawcorr"
#define INCRCOSTFILEPOS      "snaphu.incrcostpos"
#define INCRCOSTFILENEG      "snaphu.incrcostneg"
#define DEF_CS2SCALEFACTOR   8
#define DEF_NMAJORPRUNE      LARGEINT
#define DEF_PRUNECOSTTHRESH  LARGEINT
#define DEF_EDGEMASKTOP      0
#define DEF_EDGEMASKBOT      0
#define DEF_EDGEMASKLEFT     0
#define DEF_EDGEMASKRIGHT    0
#define CONNCOMPMEMINCR      1024


/* default tile parameters */

#define DEF_NTILEROW         1
#define DEF_NTILECOL         1
#define DEF_ROWOVRLP         0
#define DEF_COLOVRLP         0
#define DEF_PIECEFIRSTROW    1
#define DEF_PIECEFIRSTCOL    1
#define DEF_PIECENROW        0
#define DEF_PIECENCOL        0
#define DEF_TILECOSTTHRESH   500
#define DEF_MINREGIONSIZE    100
#define DEF_NTHREADS         1
#define DEF_SCNDRYARCFLOWMAX 8
#define DEF_TILEEDGEWEIGHT   2.5
#define DEF_TILEDIR          ""
#define DEF_ASSEMBLEONLY     FALSE
#define DEF_RMTMPTILE        TRUE


/* default connected component parameters */
#define DEF_MINCONNCOMPFRAC  0.01
#define DEF_CONNCOMPTHRESH   300
#define DEF_MAXNCOMPS        32
#define DEF_CONNCOMPOUTTYPE  CONNCOMPOUTTYPEUCHAR


/* default file formats */

#define DEF_INFILEFORMAT              COMPLEX_DATA
#define DEF_UNWRAPPEDINFILEFORMAT     ALT_LINE_DATA
#define DEF_MAGFILEFORMAT             FLOAT_DATA
#define DEF_OUTFILEFORMAT             ALT_LINE_DATA
#define DEF_CORRFILEFORMAT            ALT_LINE_DATA
#define DEF_ESTFILEFORMAT             ALT_LINE_DATA
#define DEF_AMPFILEFORMAT             ALT_SAMPLE_DATA

/* command-line usage help strings */

#define OPTIONSHELPFULL\
 "usage:  snaphu [options] infile linelength [options]\n"\
 "options:\n"\
 "  -t              use topography mode costs (default)\n"\
 "  -d              use deformation mode costs\n"\
 "  -s              use smooth-solution mode costs\n"\
 "  -C <confstr>    parse argument string as config line as from conf file\n"\
 "  -f <filename>   read configuration parameters from file\n"\
 "  -o <filename>   write output to file\n"\
 "  -a <filename>   read amplitude data from file\n"\
 "  -A <filename>   read power data from file\n"\
 "  -m <filename>   read interferogram magnitude data from file\n"\
 "  -M <filename>   read byte mask data from file\n"\
 "  -c <filename>   read correlation data from file\n"\
 "  -e <filename>   read coarse unwrapped-phase estimate from file\n"\
 "  -w <filename>   read scalar weights from file\n"\
 "  -b <decimal>    perpendicular baseline (meters, topo mode only)\n"\
 "  -p <decimal>    Lp-norm parameter p\n"\
 "  -i              do initialization and exit\n"\
 "  -n              do not use statistical costs (with -p or -i)\n"\
 "  -u              infile is already unwrapped; initialization not needed\n"\
 "  -q              quantify cost of unwrapped input file then exit\n"\
 "  -g <filename>   grow connected components mask and write to file\n"\
 "  -G <filename>   grow connected components mask for unwrapped input\n"\
 "  -S              single-tile reoptimization after multi-tile init\n"\
 "  -k              keep temporary tile outputs\n"\
 "  -l <filename>   log runtime parameters to file\n"\
 "  -v              give verbose output\n"\
 "  --mst           use MST algorithm for initialization (default)\n"\
 "  --mcf           use MCF algorithm for initialization\n"\
 "  --aa <filename1> <filename2>    read amplitude from next two files\n"\
 "  --AA <filename1> <filename2>    read power from next two files\n"\
 "  --costinfile <filename>         read statistical costs from file\n"\
 "  --costoutfile <filename>        write statistical costs to file\n"\
 "  --tile <nrow> <ncol> <rowovrlp> <colovrlp>  unwrap as nrow x ncol tiles\n"\
 "  --nproc <integer>               number of processors used in tile mode\n"\
 "  --tiledir <dirname>             use specified directory for tiles\n"\
 "  --assemble                      assemble unwrapped tiles in tiledir\n"\
 "  --piece <firstrow> <firstcol> <nrow> <ncol>  unwrap subset of image\n" \
 "  --debug, --dumpall              dump all intermediate data arrays\n"\
 "  --copyright, --info             print copyright and bug report info\n"\
 "  -h, --help                      print this help text\n"\
 "\n"

#define OPTIONSHELPBRIEF\
 "usage:  snaphu [options] infile linelength [options]\n"\
 "most common options:\n"\
 "  -t              use topography mode costs (default)\n"\
 "  -d              use deformation mode costs\n"\
 "  -s              use smooth-solution mode costs\n"\
 "  -C <confstr>    parse argument string as config line as from conf file\n"\
 "  -f <filename>   read configuration parameters from file\n"\
 "  -o <filename>   write output to file\n"\
 "  -a <filename>   read amplitude data from file\n"\
 "  -c <filename>   read correlation data from file\n"\
 "  -M <filename>   read byte mask data from file\n"\
 "  -b <decimal>    perpendicular baseline (meters)\n"\
 "  -i              do initialization and exit\n"\
 "  -S              single-tile reoptimization after multi-tile init\n"\
 "  -l <filename>   log runtime parameters to file\n"\
 "  -u              infile is already unwrapped; initialization not needed\n"\
 "  -v              give verbose output\n"\
 "  --mst           use MST algorithm for initialization (default)\n"\
 "  --mcf           use MCF algorithm for initialization\n"\
 "  --tile <nrow> <ncol> <rowovrlp> <colovrlp>  unwrap as nrow x ncol tiles\n"\
 "  --nproc <integer>               number of processors used in tile mode\n"\
 "\n"\
 "type snaphu -h for a complete list of options\n"\
 "\n"

#define COPYRIGHT\
 "Written by Curtis W. Chen\n"\
 "Copyright 2002,2017 Board of Trustees, Leland Stanford Jr. University\n"\
 "\n"\
 "Except as noted below, permission to use, copy, modify, and\n"\
 "distribute, this software and its documentation for any purpose is\n"\
 "hereby granted without fee, provided that the above copyright notice\n"\
 "appear in all copies and that both that copyright notice and this\n"\
 "permission notice appear in supporting documentation, and that the\n"\
 "name of the copyright holders be used in advertising or publicity\n"\
 "pertaining to distribution of the software with specific, written\n"\
 "prior permission, and that no fee is charged for further distribution\n"\
 "of this software, or any modifications thereof.  The copyright holder\n"\
 "makes no representations about the suitability of this software for\n"\
 "any purpose.  It is provided \"as is\" without express or implied\n"\
 "warranty.\n"\
 "\n"\
 "THE COPYRIGHT HOLDER DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS\n"\
 "SOFTWARE, INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND\n"\
 "FITNESS, IN NO EVENT SHALL THE COPYRIGHT HOLDERS BE LIABLE FOR ANY\n"\
 "SPECIAL, INDIRECT OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER\n"\
 "RESULTING FROM LOSS OF USE, DATA, PROFITS, QPA OR GPA, WHETHER IN AN\n"\
 "ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT\n"\
 "OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.\n"\
 "\n"\
 "The parts of this software derived from the CS2 minimum cost flow\n"\
 "solver written by A. V. Goldberg and B. Cherkassky are governed by the\n"\
 "terms of the copyright holder of that software.  Permission has been\n"\
 "granted to use and distrubute that software for strictly noncommercial\n"\
 "purposes as part of this package, provided that the following\n"\
 "copyright notice from the original distribution and URL accompany the\n"\
 "software:\n"\
 "\n"\
 "  COPYRIGHT C 1995 IG Systems, Inc.  Permission to use for\n"\
 "  evaluation purposes is granted provided that proper\n"\
 "  acknowledgments are given.  For a commercial licence, contact\n"\
 "  igsys@eclipse.net (http://www.igsystems.com/cs2).\n"\
 "\n"\
 "  This software comes with NO WARRANTY, expressed or implied. By way\n"\
 "  of example, but not limitation, we make no representations of\n"\
 "  warranties of merchantability or fitness for any particular\n"\
 "  purpose or that the use of the software components or\n"\
 "  documentation will not infringe any patents, copyrights,\n"\
 "  trademarks, or other rights.\n"\
 "\n"\
 "\n"\
 "Please send snaphu bug reports to " BUGREPORTEMAIL "\n"\
 "\n"


/********************/
/* type definitions */
/********************/

/** node data structure */
typedef struct nodeST{
  int row,col;                  /** row, col of this node */
  struct nodeST *next;          /** ptr to next node in thread or bucket */
  struct nodeST *prev;          /** ptr to previous node in thread or bucket */
  struct nodeST *pred;          /** parent node in tree */
  unsigned int level;           /** tree level */
  int group;                    /** for marking label */
  int incost,outcost;           /** costs to, from root of tree */
}nodeT;


/** boundary neighbor structure */
typedef struct neighborST{
  nodeT *neighbor;              /** neighbor node pointer */
  int arcrow;                   /** row of arc to neighbor */
  int arccol;                   /** col of arc to neighbor */
  int arcdir;                   /** direction of arc to neighbor */
}neighborT;


/** boundary data structure */
typedef struct boundaryST{
  nodeT node[1];                /** ground node pointed to by this boundary */
  neighborT* neighborlist;      /** list of neighbors of common boundary */
  nodeT **boundarylist;         /** list of nodes covered by common boundary */
  long nneighbor;               /** number of neighbor nodes of boundary */
  long nboundary;               /** number of nodes covered by boundary */
}boundaryT;


/** arc cost data structure */
typedef struct costST{
  short offset;                 /** offset of wrapped phase gradient from 0 */
  short sigsq;                  /** variance due to decorrelation */
  short dzmax;                  /** largest discontinuity on shelf */
  short laycost;                /** cost of layover discontinuity shelf */
}costT;


/** arc cost data structure for smooth costs */
typedef struct smoothcostST{
  short offset;                 /** offset of wrapped phase gradient from 0 */
  short sigsq;                  /** variance due to decorrelation */
}smoothcostT;


/** arc cost data structure for bidirectional scalar costs */
typedef struct bidircostST{
  short posweight;              /** weight for positive flows */
  short negweight;              /** weight for negative flows */
}bidircostT;


/** incremental cost data structure */
typedef struct incrcostST{
  short poscost;                /** cost for positive flow increment */
  short negcost;                /** cost for negative flow increment */
}incrcostT;


/** arc candidate data structure */
typedef struct candidateST{
  nodeT *from, *to;             /** endpoints of candidate arc */
  long violation;               /** magnitude of arc violation */
  int arcrow,arccol;            /** indexes into arc arrays */
  signed char arcdir;           /** direction of arc (1=fwd, -1=rev) */
}candidateT;


/** bucket data structure */
typedef struct bucketST{
  long size;                    /** number of buckets in list */
  long curr;                    /** current bucket index */
  long maxind;                  /** maximum bucket index */
  long minind;                  /** smallest (possibly negative) bucket index */
  nodeT **bucket;               /** array of first nodes in each bucket */
  nodeT **bucketbase;           /** real base of bucket array */
  signed char wrapped;          /** flag denoting wrapped circular buckets */
}bucketT;


/** secondary arc data structure */
typedef struct scndryarcST{
  int arcrow;                   /** row of arc in secondary network array */
  int arccol;                   /** col of arc in secondary network array */
  nodeT *from;                  /** secondary node at tail of arc */
  nodeT *to;                    /** secondary node at head of arc */
  signed char fromdir;          /** direction from which arc enters head */
}scndryarcT;


/** supplementary data structure for secondary nodes */
typedef struct nodesuppST{
  int row;                      /** row of node in primary network problem */
  int col;                      /** col of node in primary network problem */
  nodeT **neighbornodes;        /** pointers to neighboring secondary nodes */
  scndryarcT **outarcs;         /** pointers to secondary arcs to neighbors */
  int noutarcs;                 /** number of arcs from this node */
}nodesuppT;


/** run-time parameter data structure */
typedef struct paramST{

  /* SAR and geometry parameters */
  double orbitradius;     /** radius of platform orbit (meters) */
  double altitude;        /** SAR altitude (meters) */
  double earthradius;     /** radius of earth (meters) */
  double bperp;           /** nominal perpendiuclar baseline (meters) */
  signed char transmitmode; /** transmit mode (PINGPONG or SINGLEANTTRANSMIT) */
  double baseline;        /** baseline length (meters, always postive) */
  double baselineangle;   /** baseline angle above horizontal (rad) */
  long nlooksrange;       /** number of looks in range for input data */
  long nlooksaz;          /** number of looks in azimuth for input data */
  long nlooksother;       /** number of nonspatial looks for input data */
  double ncorrlooks;      /** number of independent looks in correlation est */
  long ncorrlooksrange;   /** number of looks in range for correlation */
  long ncorrlooksaz;      /** number of looks in azimuth for correlation */
  double nearrange;       /** slant range to near part of swath (meters) */
  double dr;              /** range bin spacing (meters) */
  double da;              /** azimuth bin spacing (meters) */
  double rangeres;        /** range resolution (meters) */
  double azres;           /** azimuth resolution (meters) */
  double lambda;          /** wavelength (meters) */

  /* scattering model parameters */
  double kds;             /** ratio of diffuse to specular scattering */
  double specularexp;     /** power specular scattering component */
  double dzrcritfactor;   /** fudge factor for linearizing scattering model */
  signed char shadow;     /** allow discontinuities from shadowing */
  double dzeimin;         /** lower limit for backslopes (if shadow = FALSE) */
  long laywidth;          /** width of window for summing layover brightness */
  double layminei;        /** threshold brightness for assuming layover */
  double sloperatiofactor;/** fudge factor for linearized scattering slopes */
  double sigsqei;         /** variance (dz, meters) due to uncertainty in EI */

  /* decorrelation model parameters */
  double drho;            /** step size of correlation-slope lookup table */
  double rhosconst1,rhosconst2;/** for calculating rho0 in biased rho */
  double cstd1,cstd2,cstd3;/** for calculating correlation power given nlooks */
  double defaultcorr;     /** default correlation if no correlation file */
  double rhominfactor;    /** threshold for setting unbiased correlation to 0 */

  /* pdf model parameters */
  double dzlaypeak;       /** range pdf peak for no discontinuity when bright */
  double azdzfactor;      /** fraction of dz in azimuth vs. rnage */
  double dzeifactor;      /** nonlayover dz scale factor */
  double dzeiweight;      /** weight to give dz expected from intensity */
  double dzlayfactor;     /** layover regime dz scale factor */
  double layconst;        /** normalized constant pdf of layover edge */
  double layfalloffconst; /** factor of sigsq for layover cost increase */
  long sigsqshortmin;     /** min short value for costT variance */
  double sigsqlayfactor;  /** fration of ambiguityheight^2 for layover sigma */

  /* deformation mode parameters */
  double defoazdzfactor;  /** scale for azimuth ledge in defo cost function */
  double defothreshfactor;/** factor of rho0 for discontinuity threshold */
  double defomax;         /** max discontinuity (cycles) from deformation */
  double sigsqcorr;       /** variance in measured correlation */
  double defolayconst;    /** layconst for deformation mode */

  /* algorithm parameters */
  signed char eval;       /** evaluate unwrapped input file if TRUE */
  signed char unwrapped;  /** input file is unwrapped if TRUE */
  signed char regrowconncomps;/** grow connected components and exit if TRUE */
  signed char initonly;   /** exit after initialization if TRUE */
  signed char initmethod; /** MST or MCF initialization */
  signed char costmode;   /** statistical cost mode */
  signed char dumpall;    /** dump intermediate files */
  signed char verbose;    /** print verbose output */
  signed char amplitude;  /** intensity data is amplitude, not power */
  signed char havemagnitude; /** flag: create correlation from other inputs */
  signed char flipphasesign; /** flag: flip phase and flow array signs */
  signed char onetilereopt;  /** flag: reoptimize full input after tile init */
  signed char rmtileinit; /** flag to remove temporary tile unw init soln */
  long initmaxflow;       /** maximum flow for initialization */
  long arcmaxflowconst;   /** units of flow past dzmax to use for initmaxflow */
  long maxflow;           /** max flow for tree solve looping */
  long krowei, kcolei;    /** size of boxcar averaging window for mean ei */
  long kpardpsi;          /** length of boxcar for mean wrapped gradient */
  long kperpdpsi;         /** width of boxcar for mean wrapped gradient */
  double threshold;       /** thershold for numerical dzrcrit calculation */
  double initdzr;         /** initial dzr for numerical dzrcrit calc. (m) */
  double initdzstep;      /** initial stepsize for spatial decor slope calc. */
  double maxcost;         /** min and max float values for cost arrays */
  double costscale;       /** scale factor for discretizing to integer costs */
  double costscaleambight;/** ambiguity height for auto costs caling */
  double dnomincangle;    /** step size for range-varying param lookup table */
  long srcrow,srccol;     /** source node location */
  double p;               /** power for Lp-norm solution (less than 0 is MAP) */
  signed char bidirlpn;   /** use bidirectional Lp costs if TRUE */
  long nshortcycle;       /** number of points for one cycle in short int dz */
  double maxnewnodeconst; /** number of nodes added to tree on each iteration */
  long maxnflowcycles;    /** max number of cycles to consider nflow done */
  double maxcyclefraction;/** ratio of max cycles to pixels */
  long nconnnodemin;      /** min number of nodes to keep in connected set */
  long cs2scalefactor;    /** scale factor for cs2 initialization (eg, 3-30) */
  long nmajorprune;       /** number of major iterations between tree pruning */
  long prunecostthresh;   /** cost threshold for pruning */
  long edgemasktop;       /** number of pixels to mask at top edge of input */
  long edgemaskbot;       /** number of pixels to mask at bottom edge */
  long edgemaskleft;      /** number of pixels to mask at left edge */
  long edgemaskright;     /** number of pixels to mask at right edge */
  long parentpid;         /** process identification number of parent */

  /* tiling parameters */
  long ntilerow;          /** number of tiles in azimuth */
  long ntilecol;          /** number of tiles in range */
  long rowovrlp;          /** pixels of overlap between row tiles */
  long colovrlp;          /** pixels of overlap between column tiles */
  long piecefirstrow;     /** first row (indexed from 1) for piece mode */
  long piecefirstcol;     /** first column (indexed from 1) for piece mode */
  long piecenrow;         /** number of rows for piece mode */
  long piecencol;         /** number of cols for piece mode */
  long tilecostthresh;    /** maximum cost within single reliable tile region */
  long minregionsize;     /** minimum number of pixels in a region */
  long nthreads;          /** number of parallel processes to run */
  long scndryarcflowmax;  /** max flow increment for which to keep cost data */
  double tileedgeweight;  /** weight applied to tile-edge secondary arc costs */
  signed char assembleonly; /** flag for assemble-only (no unwrap) mode */
  signed char rmtmptile;  /** flag for removing temporary tile files */
  char tiledir[MAXSTRLEN];/** directory for temporary tile files */

  /* connected component parameters */
  double minconncompfrac; /** min fraction of pixels in connected component */
  long conncompthresh;    /** cost threshold for connected component */
  long maxncomps;         /** max number of connected components */
  int conncompouttype;    /** flag for type of connected component output file */

}paramT;


/** input file name data structure */
typedef struct infileST{
  char infile[MAXSTRLEN];             /** input interferogram */
  char magfile[MAXSTRLEN];            /** interferogram magnitude (optional) */
  char ampfile[MAXSTRLEN];            /** image amplitude or power file */
  char ampfile2[MAXSTRLEN];           /** second amplitude or power file */
  char weightfile[MAXSTRLEN];         /** arc weights */
  char corrfile[MAXSTRLEN];           /** correlation file */
  char estfile[MAXSTRLEN];            /** unwrapped estimate */
  char costinfile[MAXSTRLEN];         /** file from which cost data is read */
  char bytemaskfile[MAXSTRLEN];       /** signed char valid pixel mask */
  char dotilemaskfile[MAXSTRLEN];     /** signed char tile unwrap mask file */
  signed char infileformat;           /** input file format */
  signed char unwrappedinfileformat;  /** input file format if unwrapped */
  signed char magfileformat;          /** interferogram magnitude file format */
  signed char corrfileformat;         /** correlation file format */
  signed char weightfileformat;       /** weight file format */
  signed char ampfileformat;          /** amplitude file format */
  signed char estfileformat;          /** unwrapped-estimate file format */
}infileT;


/** output file name data structure */
typedef struct outfileST{
  char outfile[MAXSTRLEN];            /** unwrapped output */
  char initfile[MAXSTRLEN];           /** unwrapped initialization */
  char flowfile[MAXSTRLEN];           /** flows of unwrapped solution */
  char eifile[MAXSTRLEN];             /** despckled, normalized intensity */
  char rowcostfile[MAXSTRLEN];        /** statistical azimuth cost array */
  char colcostfile[MAXSTRLEN];        /** statistical range cost array */
  char mstrowcostfile[MAXSTRLEN];     /** scalar initialization azimuth costs */
  char mstcolcostfile[MAXSTRLEN];     /** scalar initialization range costs */
  char mstcostsfile[MAXSTRLEN];       /** scalar initialization costs (all) */
  char corrdumpfile[MAXSTRLEN];       /** correlation coefficient magnitude */
  char rawcorrdumpfile[MAXSTRLEN];    /** correlation coefficient magnitude */
  char conncompfile[MAXSTRLEN];       /** connected component map or mask */
  char costoutfile[MAXSTRLEN];        /** file to which cost data is written */
  char logfile[MAXSTRLEN];            /** file to which parmeters are logged */
  signed char outfileformat;          /** output file format */
}outfileT;


/** tile parameter data structure */
typedef struct tileparamST{
  long firstcol;          /** first column of tile to process (index from 0) */
  long ncol;              /** number of columns in tile to process */
  long firstrow;          /** first row of tile to process (index from 0) */
  long nrow;              /** number of rows in tile to process */
}tileparamT;


/** connected component size structure */
typedef struct conncompsizeST{
  unsigned int tilenum;               /** tile index */
  unsigned int icomptile;             /** conn comp index in tile */
  unsigned int icompfull;             /** conn comp index in full array */
  long npix;                          /** number of pixels in conn comp */
}conncompsizeT;


/** type for total cost of solution (may overflow long) */
typedef double totalcostT;
#define INITTOTALCOST LARGEFLOAT



/***********************/
/* function prototypes */
/***********************/

/* functions in snaphu_tile.c */

/**
 * Sets up tile parameters and output file names for the current tile.
 */
int SetupTile(long nlines, long linelen, paramT *params,
              tileparamT *tileparams, outfileT *outfiles,
              outfileT *tileoutfiles, long tilerow, long tilecol);
/**
 * Read the tile mask if a file name is specified in the infiles structure,
 * otherwise return an array of all ones.
 */
signed char **SetUpDoTileMask(infileT *infiles, long ntilerow, long ntilecol);
/**
 * Grows contiguous regions demarcated by arcs whose residual costs are
 * less than some threshold.  Numbers the regions sequentially from 0.
 */
int GrowRegions(void **costs, short **flows, long nrow, long ncol,
                incrcostT **incrcosts, outfileT *outfiles,
                tileparamT *tileparams, paramT *params);
/**
 * Grows contiguous regions demarcated by arcs whose residual costs are
 * less than some threshold.  Numbers the regions sequentially from 1.
 * Writes out byte file of connected component mask, with 0 for any pixels
 * not assigned to a component.
 */
int GrowConnCompsMask(void **costs, short **flows, long nrow, long ncol,
                      incrcostT **incrcosts, outfileT *outfiles,
                      paramT *params);
/**
 * TODO: comment
 */
int AssembleTiles(outfileT *outfiles, paramT *params,
                  long nlines, long linelen);


/* functions in snaphu_solver.c */

/**
 * TODO: comment
 */
int SetGridNetworkFunctionPointers(void);
/**
 * TODO: comment
 */
int SetNonGridNetworkFunctionPointers(void);
/**
 * Solves the nonlinear network optimization problem.
 */
long TreeSolve(nodeT **nodes, nodesuppT **nodesupp, nodeT *ground,
               nodeT *source, candidateT **candidatelistptr,
               candidateT **candidatebagptr, long *candidatelistsizeptr,
               long *candidatebagsizeptr, bucketT *bkts, short **flows,
               void **costs, incrcostT **incrcosts, nodeT ***apexes,
               signed char **iscandidate, long ngroundarcs, long nflow,
               float **mag, float **wrappedphase, char *outfile,
               long nnoderow, int *nnodesperrow, long narcrow,
               int *narcsperrow, long nrow, long ncol,
               outfileT *outfiles, long nconnected, paramT *params);
/**
 * TODO: comment
 */
int InitNetwork(short **flows, long *ngroundarcsptr, long *ncycleptr,
                long *nflowdoneptr, long *mostflowptr, long *nflowptr,
                long *candidatebagsizeptr, candidateT **candidatebagptr,
                long *candidatelistsizeptr, candidateT **candidatelistptr,
                signed char ***iscandidateptr, nodeT ****apexesptr,
                bucketT **bktsptr, long *iincrcostfileptr,
                incrcostT ***incrcostsptr, nodeT ***nodesptr, nodeT *ground,
                long *nnoderowptr, int **nnodesperrowptr, long *narcrowptr,
                int **narcsperrowptr, long nrow, long ncol,
                signed char *notfirstloopptr, totalcostT *totalcostptr,
                paramT *params);
/**
 * TODO: comment
 */
long SetupTreeSolveNetwork(nodeT **nodes, nodeT *ground, nodeT ***apexes,
                           signed char **iscandidate, long nnoderow,
                           int *nnodesperrow, long narcrow, int *narcsperrow,
                           long nrow, long ncol);
/**
 * Check magnitude masking to make sure we have at least one pixel
 * that is not masked; return 0 on success, or 1 if all pixels masked.
 */
signed char CheckMagMasking(float **mag, long nrow, long ncol);
/**
 * Set group numbers of nodes to MASKED if they are surrounded by
 * zero-magnitude pixels, 0 otherwise.
 */
int MaskNodes(long nrow, long ncol, nodeT **nodes, nodeT *ground,
              float **mag);
/**
 * Return maximum flow magnitude that does not traverse an arc adjacent
 * to a masked interferogram pixel.
 */
long MaxNonMaskFlow(short **flows, float **mag, long nrow, long ncol);
/**
 * TODO: comment
 */
int InitNodeNums(long nrow, long ncol, nodeT **nodes, nodeT *ground);
/**
 * TODO: comment
 */
int InitNodes(long nrow, long ncol, nodeT **nodes, nodeT *ground);
/**
 * TODO: comment
 */
void BucketInsert(nodeT *node, long ind, bucketT *bkts);
/**
 * TODO: comment
 */
void BucketRemove(nodeT *node, long ind, bucketT *bkts);
/**
 * TODO: comment
 */
nodeT *ClosestNode(bucketT *bkts);
/**
 * Create a list of node pointers to be sources for each set of
 * connected pixels (not disconnected by masking).  Return the number
 * of sources (ie, the number of connected sets of pixels).
 */
long SelectSources(nodeT **nodes, float **mag, nodeT *ground, long nflow,
                   short **flows, long ngroundarcs,
                   long nrow, long ncol, paramT *params,
                   nodeT ***sourcelistptr, long **nconnectedarrptr);
/**
 * Updates the incremental cost for an arc.
 */
long ReCalcCost(void **costs, incrcostT **incrcosts, long flow,
                long arcrow, long arccol, long nflow, long nrow,
                paramT *params);
/**
 * Calculates the costs for positive and negative dflow flow increment
 * if there is zero flow on the arc.
 */
int SetupIncrFlowCosts(void **costs, incrcostT **incrcosts, short **flows,
                       long nflow, long nrow, long narcrow,
                       int *narcsperrow, paramT *params);
/**
 * Computes the total cost of the flow array and prints it out.  Pass nrow
 * and ncol if in grid mode (primary network), or pass nrow=ntiles and
 * ncol=0 for nongrid mode (secondary network).
 */
totalcostT EvaluateTotalCost(void **costs, short **flows, long nrow, long ncol,
                             int *narcsperrow,paramT *params);
/**
 * Initializes the flow on a the network using minimum spanning tree
 * algorithm.
 */
int MSTInitFlows(float **wrappedphase, short ***flowsptr,
                 short **mstcosts, long nrow, long ncol,
                 nodeT ***nodes, nodeT *ground, long maxflow);
/**
 * Initializes the flow on a the network using minimum cost flow
 * algorithm.
 */
int MCFInitFlows(float **wrappedphase, short ***flowsptr, short **mstcosts,
                 long nrow, long ncol, long cs2scalefactor);


/**
 * Builds cost arrays for arcs based on interferogram intensity
 * and correlation, depending on options and passed parameters.
 */
int BuildCostArrays(void ***costsptr, short ***mstcostsptr,
                    float **mag, float **wrappedphase,
                    float **unwrappedest, long linelen, long nlines,
                    long nrow, long ncol, paramT *params,
                    tileparamT *tileparams, infileT *infiles,
                    outfileT *outfiles);
/**
 * Calculates topography arc distance given an array of cost data structures.
 */
void CalcCostTopo(void **costs, long flow, long arcrow, long arccol,
                  long nflow, long nrow, paramT *params,
                  long *poscostptr, long *negcostptr);
/**
 * Calculates deformation arc distance given an array of cost data structures.
 */
void CalcCostDefo(void **costs, long flow, long arcrow, long arccol,
                  long nflow, long nrow, paramT *params,
                  long *poscostptr, long *negcostptr);
/**
 * Calculates smooth-solution arc distance given an array of smoothcost
 *  data structures.
 */
void CalcCostSmooth(void **costs, long flow, long arcrow, long arccol,
                    long nflow, long nrow, paramT *params,
                    long *poscostptr, long *negcostptr);
/**
 * Calculates the L0 arc distance given an array of short integer weights.
 */
void CalcCostL0(void **costs, long flow, long arcrow, long arccol,
                long nflow, long nrow, paramT *params,
                long *poscostptr, long *negcostptr);
/**
 * Calculates the L1 arc cost given an array of bidirectional cost weights.
 */
void CalcCostL1(void **costs, long flow, long arcrow, long arccol,
                long nflow, long nrow, paramT *params,
                long *poscostptr, long *negcostptr);
/**
 * Calculates the L2 arc distance given an array of short integer weights.
 */
void CalcCostL2(void **costs, long flow, long arcrow, long arccol,
                long nflow, long nrow, paramT *params,
                long *poscostptr, long *negcostptr);
/**
 * Calculates the Lp arc distance given an array of short integer weights.
 */
void CalcCostLP(void **costs, long flow, long arcrow, long arccol,
                long nflow, long nrow, paramT *params,
                long *poscostptr, long *negcostptr);
/**
 * Calculates the L0 arc cost given an array of bidirectional cost weights.
 */
void CalcCostL0BiDir(void **costs, long flow, long arcrow, long arccol,
                     long nflow, long nrow, paramT *params,
                     long *poscostptr, long *negcostptr);
/**
 * Calculates the L1 arc cost given an array of bidirectional cost weights.
 */
void CalcCostL1BiDir(void **costs, long flow, long arcrow, long arccol,
                     long nflow, long nrow, paramT *params,
                     long *poscostptr, long *negcostptr);
/**
 * Calculates the L2 arc cost given an array of bidirectional cost weights.
 */
void CalcCostL2BiDir(void **costs, long flow, long arcrow, long arccol,
                     long nflow, long nrow, paramT *params,
                     long *poscostptr, long *negcostptr);
/**
 * Calculates the Lp arc cost given an array of bidirectional cost weights.
 */
void CalcCostLPBiDir(void **costs, long flow, long arcrow, long arccol,
                     long nflow, long nrow, paramT *params,
                     long *poscostptr, long *negcostptr);
/**
 * Calculates the arc cost given an array of long integer cost lookup tables.
 *
 * The cost array for each arc gives the cost for +/-flowmax units of
 * flow around the flow value with minimum cost, which is not
 * necessarily flow == 0.  The offset between the flow value with
 * minimum cost and flow == 0 is given by arroffset = costarr[0].
 * Positive flow values k for k = 1 to flowmax relative to this min
 * cost flow value are in costarr[k].  Negative flow values k relative
 * to the min cost flow from k = -1 to -flowmax costarr[flowmax-k].
 * costarr[2*flowmax+1] contains a scaling factor for extrapolating
 * beyond the ends of the cost table, assuming quadratically (with an offset)
 * increasing cost (subject to rounding and scaling).
 *
 * As of summer 2019, the rationale for how seconeary costs are
 * extrapolated beyond the end of the table has been lost to time, but
 * the logic at least does give a self-consistent cost function that
 * is continuous at +/-flowmax and quadratically increases beyond,
 * albeit not necessarily with a starting slope that has an easily
 * intuitive basis.
 */
void CalcCostNonGrid(void **costs, long flow, long arcrow, long arccol,
                     long nflow, long nrow, paramT *params,
                     long *poscostptr, long *negcostptr);
/**
 * Calculates topography arc cost given an array of cost data structures.
 */
long EvalCostTopo(void **costs, short **flows, long arcrow, long arccol,
                  long nrow, paramT *params);
/**
 * Calculates deformation arc cost given an array of cost data structures.
 */
long EvalCostDefo(void **costs, short **flows, long arcrow, long arccol,
                  long nrow, paramT *params);
/**
 * Calculates smooth-solution arc cost given an array of
 * smoothcost data structures.
 */
long EvalCostSmooth(void **costs, short **flows, long arcrow, long arccol,
                    long nrow, paramT *params);
/**
 * Calculates the L0 arc cost given an array of cost data structures.
 */
long EvalCostL0(void **costs, short **flows, long arcrow, long arccol,
                long nrow, paramT *params);
/**
 * Calculates the L1 arc cost given an array of cost data structures.
 */
long EvalCostL1(void **costs, short **flows, long arcrow, long arccol,
                long nrow, paramT *params);
/**
 * Calculates the L2 arc cost given an array of cost data structures.
 */
long EvalCostL2(void **costs, short **flows, long arcrow, long arccol,
                long nrow, paramT *params);
/**
 * Calculates the Lp arc cost given an array of cost data structures.
 */
long EvalCostLP(void **costs, short **flows, long arcrow, long arccol,
                long nrow, paramT *params);
/**
 * Calculates the L0 arc cost given an array of bidirectional cost structures.
 */
long EvalCostL0BiDir(void **costs, short **flows, long arcrow, long arccol,
                     long nrow, paramT *params);
/**
 * Calculates the L1 arc cost given an array of bidirectional cost structures.
 */
long EvalCostL1BiDir(void **costs, short **flows, long arcrow, long arccol,
                     long nrow, paramT *params);
/**
 * Calculates the L1 arc cost given an array of bidirectional cost structures.
 */
long EvalCostL2BiDir(void **costs, short **flows, long arcrow, long arccol,
                     long nrow, paramT *params);
/**
 * Calculates the Lp arc cost given an array of bidirectional cost structures.
 */
long EvalCostLPBiDir(void **costs, short **flows, long arcrow, long arccol,
                     long nrow, paramT *params);
/**
 * Calculates the arc cost given an array of long integer cost lookup tables.
 */
long EvalCostNonGrid(void **costs, short **flows, long arcrow, long arccol,
                     long nrow, paramT *params);


/* functions in snaphu_util.c */

/**
 * Sets the value of a signed character based on the string argument passed.
 *
 * Returns TRUE if the string was not a valid value, FALSE otherwise.
 */
signed char SetBooleanSignedChar(signed char *boolptr, char *str);
/**
 * Makes sure the passed float array is properly wrapped into the [0,2pi)
 * interval.
 */
int WrapPhase(float **wrappedphase, long nrow, long ncol);
/**
 * Computes an array of wrapped phase differences in range (across rows).
 * Input wrapped phase array should be in radians.  Output is in cycles.
 */
int CalcWrappedRangeDiffs(float **dpsi, float **avgdpsi, float **wrappedphase,
                          long kperpdpsi, long kpardpsi,
                          long nrow, long ncol);
/**
 * Computes an array of wrapped phase differences in range (across rows).
 * Input wrapped phase array should be in radians. Output is in cycles.
 */
int CalcWrappedAzDiffs(float **dpsi, float **avgdpsi, float **wrappedphase,
                       long kperpdpsi, long kpardpsi, long nrow, long ncol);
/**
 * Computes the cycle array of a phase 2D phase array. Input arrays
 * should be type float ** and signed char ** with memory pre-allocated.
 * Numbers of rows and columns in phase array should be passed.
 * Residue array will then have size nrow-1 x ncol-1.  Residues will
 * always be -1, 0, or 1 if wrapped phase is passed in.
 */
int CycleResidue(float **phase, signed char **residue,
                 int nrow, int ncol);
/**
 * Compute residue of node from wrapped phase.  Assumes that memory
 * exists and that row and col are in bounds of the 2-D array of
 * wrapped phase values passed.
 */
int NodeResidue(float **wphase, long row, long col);
/**
 * Calculates flow based on unwrapped phase data in a 2D array.
 * Allocates memory for row and column flow arrays.
 */
int CalcFlow(float **phase, short ***flowsptr, long nrow, long ncol);
/**
 * This function takes row and column flow information and integrates
 * wrapped phase to create an unwrapped phase field.  The unwrapped
 * phase field will be the same size as the wrapped field.  The array
 * rowflow should have size N-1xM and colflow size NxM-1 where the
 * phase fields are NxM.  Output is saved to a file.
 */
int IntegratePhase(float **psi, float **phi, short **flows,
                   long nrow, long ncol);
/**
 * Given an unwrapped phase array, parse the data and find the flows.
 * Assumes only integer numbers of cycles have been added to get the
 * unwrapped phase from the wrapped pase.  Gets memory and writes
 * wrapped phase to passed pointer.  Assumes flows fit into short ints.
 */
float **ExtractFlow(float **unwrappedphase, short ***flowsptr,
                    long nrow, long ncol);
/**
 * Flips the sign of all values in a passed array if the flip flag is set.
 * Otherwise, does nothing.
 */
int FlipPhaseArraySign(float **arr, paramT *params, long nrow, long ncol);
/**
 * Flips the sign of all values in a row-by-column array if the flip
 * flip flag is set.  Otherwise, does nothing.
 */
int FlipFlowArraySign(short **arr, paramT *params, long nrow, long ncol);
/**
 * Allocates memory for 2D array.
 *
 * Dynamically allocates memory and returns pointer of
 * type void ** for an array of size nrow x ncol.
 * First index is row number: array[row][col]
 * size is size of array element, psize is size of pointer
 * to array element (eg sizeof(float *)).
 */
void **Get2DMem(int nrow, int ncol, int psize, size_t size);
/**
 * Allocates memory for 2D array.  The array will have 2*nrow-1 rows.
 * The first nrow-1 rows will have ncol columns, and the rest will
 * have ncol-1 columns.
 */
void **Get2DRowColMem(long nrow, long ncol, int psize, size_t size);
/**
 * Allocates memory for 2D array.  The array will have 2*nrow-1 rows.
 * The first nrow-1 rows will have ncol columns, and the rest will
 * have ncol-1 columns.  Memory is initialized to zero.
 */
void **Get2DRowColZeroMem(long nrow, long ncol, int psize, size_t size);

/**
 * Has same functionality as malloc(), but exits if out of memory.
 */
void *MAlloc(size_t size);
/**
 *
 * Allocates zero-initialized memory for an array of elements.
 *
 * This function has the same behavior as `calloc`, except that it does
 * not return on allocation failure. If memory cannot be allocated, an
 * error message is written to standard error and the process terminates.
 *
 * Parameters:
 * - `nitems`: Number of elements to allocate.
 * - `size`:   Size in bytes of each element.
 *
 * Returns:
 * - A pointer to zero-initialized memory on success.
 * - `NULL` if `size` is zero.
 *
 * Ownership:
 * The caller owns the returned memory and must release it using `free`.
 *
 * Safety notes for FFI:
 * - This function never returns on out-of-memory conditions.
 * - It does not set `errno`.
 * - Callers must not assume recoverable failure semantics.
 */
void *CAlloc(size_t nitems, size_t size);
/**
 * Has same functionality as realloc(), but exits if out of memory.
 */
void *ReAlloc(void *ptr, size_t size);
/**
 * This function frees the dynamically allocated memory for a 2D
 * array.  Pass in a pointer to a pointer cast to a void **.
 * The function assumes the array is of the form arr[rows][cols]
 * so that nrow is the number of elements in the pointer array.
 */
int Free2DArray(void **array, unsigned int nrow);
/**
 * Sets all entries of a 2D array of shorts to the given value.  Assumes
 * that memory is already allocated.
 */
int Set2DShortArray(short **arr, long nrow, long ncol, long value);
/**
 * Given a 2D floating point array, returns FALSE if any elements are NaN
 * or infinite, and TRUE otherwise.
 */
signed char ValidDataArray(float **arr, long nrow, long ncol);
/**
 * Given a 2D floating point array, returns FALSE if any elements are
 * NaN or infinite or negative, and TRUE otherwise.
 */
signed char NonNegDataArray(float **arr, long nrow, long ncol);
/**
 * This function takes a double and returns a nonzero value if
 * the arguemnt is finite (not NaN and not infinite), and zero otherwise.
 * Different implementations are given here since not all machines have
 * these functions available.
 */
signed char IsFinite(double d);
/**
 * Rounds a floating point number to the nearest integer.
 *
 * The function takes a float and returns a long.
 */
long LRound(double a);
/**
 * Return the lesser of two long integers as a long.
 */
long LMin(long a, long b);
/**
 * Clips the input long integer so that it is no less than minval and
 * no greater than maxval, and returns a long integer.
 */
long LClip(long a, long minval, long maxval);
/**
 * Returns the maximum of the absolute values of element in a
 * two-dimensional short array.  The number of rows and columns
 * should be passed in.
 */
long Short2DRowColAbsMax(short **arr, long nrow, long ncol);
/**
 * Given an array of floats, interpolates at the specified noninteger
 * index.  Returns first or last array value if index is out of bounds.
 */
float LinInterp1D(float *arr, double index, long nelem);
/**
 * Given a 2-D array of floats, interpolates at the specified noninteger
 * indices.  Returns first or last array values if index is out of bounds.
 */
float LinInterp2D(float **arr, double rowind, double colind ,
                  long nrow, long ncol);
/**
 * Filters magnitude/power data with adaptive geometric filter to get rid of
 * speckle.  Allocates 2D memory for ei.  Does not square before averaging.
 */
int Despeckle(float **mag, float ***ei, long nrow, long ncol);
/**
 * Returns pointer to 2D array where passed array is in center and
 * edges are padded by mirror reflections.  If the pad dimensions are
 * too large for the array size, a pointer to the original array is
 * returned.
 */
float **MirrorPad(float **array1, long nrow, long ncol, long krow, long kcol);
/**
 * Takes in a 2-D array, convolves with boxcar filter of size specified.
 * Uses a recursion technique (but the function does not actually call
 * itself recursively) to compute the result, so there may be roundoff
 * errors.
 */
int BoxCarAvg(float **avgarr, float **padarr, long nrow, long ncol,
              long krow, long kcol);
/**
 * Just like strncpy(), but terminates string by putting a null
 * character at the end (may overwrite last character).
 */
char *StrNCopy(char *dest, const char *src, size_t n);
/**
 * Given a wrapped phase array and an unwrapped phase array, subtracts
 * the unwrapped data elementwise from the wrapped data and stores
 * the result, rewrapped to [0,2pi), in the wrapped array.
 */
int FlattenWrappedPhase(float **wrappedphase, float **unwrappedest,
                        long nrow, long ncol);
/**
 * Adds the values of two 2-D arrays elementwise.
 */
int Add2DFloatArrays(float **arr1, float **arr2, long nrow, long ncol);
/**
 * Uses strtod to convert a string to a double, but also does error checking.
 * If any part of the string is not converted, the function does not make
 * the assignment and returns TRUE.  Otherwise, returns FALSE.
 */
int StringToDouble(char *str, double *d);
/**
 * Uses strtol to convert a string to a base-10 long, but also does error
 * checking.  If any part of the string is not converted, the function does
 * not make the assignment and returns TRUE.  Otherwise, returns FALSE.
 */
int StringToLong(char *str, long *l);
/**
 * Traps common signals that by default cause the program to abort.
 * Sets (pointer to function) Handler as the signal handler for all.
 * Note that SIGKILL usually cannot be caught.  No return value.
 */
int CatchSignals(void (*SigHandler)(int));
/**
 * Set the global variable dumpresults_global to TRUE if SIGINT or SIGHUP
 * signals recieved.  Also sets requestedstop_global if SIGINT signal
 * received.  This function should only be called via signal() when
 * a signal is caught.
 */
void SetDump(int signum);
/**
 * Signal handler that sends a KILL signal to all processes in the group
 * so that children exit when parent exits.
 */
void KillChildrenExit(int signum);
/**
 * Signal handler that prints message about the signal received, then exits.
 */
void SignalExit(int signum);
/**
 * Compares two long integers.  For use with qsort().
 */
int LongCompare(const void *c1, const void *c2);

/* functions in snaphu_io.c */

/**
 * Sets all parameters to their initial default values.
 */
int SetDefaults(infileT *infiles, outfileT *outfiles, paramT *params);
/**
 * Parses command line inputs passed to main().
 */
int ProcessArgs(int argc, char *argv[], infileT *infiles, outfileT *outfiles,
                long *ncolptr, paramT *params);
/**
 * Checks all parameters to make sure they are valid.  This is just a boring
 * function with lots of checks in it.
 */
int CheckParams(infileT *infiles, outfileT *outfiles,
                long linelen, long nlines, paramT *params);
/**
 * Read in parameter values from a file, overriding existing parameters.
 */
int ReadConfigFile(char *conffile, infileT *infiles, outfileT *outfiles,
                   long *ncolptr, paramT *params);
/**
 * Writes a text log file of configuration parameters and other
 * information.  The log file is in a format compatible to be used as
 * a configuration file.
 */
int WriteConfigLogFile(int argc, char *argv[], infileT *infiles,
                       outfileT *outfiles, long linelen, paramT *params);
/**
 * Gets the number of lines of data in the input file based on the file
 * size.
 */
long GetNLines(infileT *infiles, long linelen, paramT *params);
/**
 * Writes the unwrapped phase to the output file specified, in the
 * format given in the parameter structure.
 */
int WriteOutputFile(float **mag, float **unwrappedphase, char *outfile,
                    outfileT *outfiles, long nrow, long ncol);
/**
 * Write data in a two dimensional array to a file.  Data elements are
 * have the number of bytes specified by size (use sizeof() when
 * calling this function.
 */
int Write2DArray(void **array, char *filename, long nrow, long ncol,
                 size_t size);
/**
 * Write data in a 2-D row-and-column array to a file.  Data elements
 * have the number of bytes specified by size (use sizeof() when
 * calling this function.  The format of the array is nrow-1 rows
 * of ncol elements, followed by nrow rows of ncol-1 elements each.
 */
int Write2DRowColArray(void **array, char *filename, long nrow,
                       long ncol, size_t size);
/**
 * Reads the input file specified on the command line.
 */
int ReadInputFile(infileT *infiles, float ***magptr, float ***wrappedphaseptr,
                  short ***flowsptr, long linelen, long nlines,
                  paramT *params, tileparamT *tileparams);
/**
 * Reads the interferogram magnitude in the specfied file if it exists.
 * Memory for the magnitude array should already have been allocated by
 * ReadInputFile().
 */
int ReadMagnitude(float **mag, infileT *infiles, long linelen, long nlines,
                  tileparamT *tileparams);
/**
 * Read signed byte mask value; set magnitude to zero where byte mask
 * is zero or where pixel is close enough to edge as defined by
 * edgemask parameters; leave magnitude unchanged otherwise.
 */
int ReadByteMask(float **mag, infileT *infiles, long linelen, long nlines,
                 tileparamT *tileparams, paramT *params);
/**
 * Reads the unwrapped-phase estimate from a file (assumes file name exists).
 */
int ReadUnwrappedEstimateFile(float ***unwrappedestptr, infileT *infiles,
                              long linelen, long nlines,
                              paramT *params, tileparamT *tileparams);
/**
 * Read in weights form rowcol format file of short ints.
 */
int ReadWeightsFile(short ***weightsptr,char *weightfile,
                    long linelen, long nlines, tileparamT *tileparams);
/**
 * Reads the intensity information from specified file(s).  If possilbe,
 * sets arrays for average power and individual powers of single-pass
 * SAR images.
 */
int ReadIntensity(float ***pwrptr, float ***pwr1ptr, float ***pwr2ptr,
                  infileT *infiles, long linelen, long nlines,
                  paramT *params, tileparamT *tileparams);
/**
 * Reads the correlation information from specified file.
 */
int ReadCorrelation(float ***corrptr, infileT *infiles,
                    long linelen, long nlines, tileparamT *tileparams);
/**
 * Read in the data from a file containing magnitude and phase
 * data.  File should have one line of magnitude data, one line
 * of phase data, another line of magnitude data, etc.
 * ncol refers to the number of complex elements in one line of
 * data.
 */
int ReadAltLineFile(float ***mag, float ***phase, char *alfile,
                    long linelen, long nlines, tileparamT *tileparams);
/**
 * Read only the phase data from a file containing magnitude and phase
 * data.  File should have one line of magnitude data, one line
 * of phase data, another line of magnitude data, etc.
 * ncol refers to the number of complex elements in one line of
 * data.
 */
int ReadAltLineFilePhase(float ***phase, char *alfile,
                         long linelen, long nlines, tileparamT *tileparams);
/**
 * Reads file of complex floats of the form real,imag,real,imag...
 * ncol is the number of complex samples (half the number of real
 * floats per line).  Ensures that phase values are in the range
 * [0,2pi).
 */
int ReadComplexFile(float ***mag, float ***phase, char *rifile,
                    long linelen, long nlines, tileparamT *tileparams);
/**
 * Reads file of real data of size elsize.  Assumes the native byte order
 * of the platform.
 */
int Read2DArray(void ***arr, char *infile, long linelen, long nlines,
                tileparamT *tileparams, size_t elptrsize, size_t elsize);
/**
 * Reads file of real alternating floats from separate images.  Format is
 * real0A, real0B, real1A, real1B, real2A, real2B,...
 * ncol is the number of samples in each image (note the number of
 * floats per line in the specified file).
 */
int ReadAltSampFile(float ***arr1, float ***arr2, char *infile,
                     long linelen, long nlines, tileparamT *tileparams);
/**
 * Gets memory and reads single array from a file.  Array should be in the
 * file line by line starting with the row array (size nrow-1 x ncol) and
 * followed by the column array (size nrow x ncol-1).  Both arrays
 * are placed into the passed array as they were in the file.
 */
int Read2DRowColFile(void ***arr, char *filename, long linelen, long nlines,
                     tileparamT *tileparams, size_t size);
/**
 * Similar to Read2DRowColFile(), except reads only row (horizontal) data
 * at specified locations.  tileparams->nrow is treated as the number of
 * rows of data to be read from the RowCol file, not the number of
 * equivalent rows in the orginal pixel file (whose arcs are represented
 * in the RowCol file).
 */
int Read2DRowColFileRows(void ***arr, char *filename, long linelen,
                         long nlines, tileparamT *tileparams, size_t size);
/**
 * Sets names of output files so that the program will dump intermediate
 * arrays.  Only sets names if they are not set already.
 */
int SetDumpAll(outfileT *outfiles, paramT *params);
/**
 * Sets the default stream pointers (global variables).
 */
int SetStreamPointers(void);
/**
 * Set the global stream pointer sp2 to be stdout if the verbose flag
 * is set in the parameter data type.
 */
int SetVerboseOut(paramT *params);
/**
 * Dumps incremental cost arrays, creating file names for them.
 */
int DumpIncrCostFiles(incrcostT **incrcosts, long iincrcostfile,
                      long nflow, long nrow, long ncol);
/**
 * Create a temporary directory for tile files in directory of output file.
 * Save directory name in buffer in paramT structure.
 */
int MakeTileDir(paramT *params, outfileT *outfiles);
/**
 * Given a filename, separates it into path and base filename.  Output
 * buffers should be at least MAXSTRLEN characters, and filename buffer
 * should be no more than MAXSTRLEN characters.  The output path
 * has a trailing "/" character.
 */
int ParseFilename(char *filename, char *path, char *basename);
/**
 * Set name of temporary tile-mode output assuming nominal output file
 * name is in string passed.  Write new name in string memory pointed
 * to by input.
 */
int SetTileInitOutfile(char *outfile, long pid);
/**
 * Sets parameters for each tile and calls UnwrapTile() to do the
 * unwrapping.
 */
int Unwrap(infileT *infiles, outfileT *outfiles, paramT *params,
           long linelen, long nlines);
/**
 * This is the main phase unwrapping function for a single tile.
 */
int UnwrapTile(infileT *infiles, outfileT *outfiles, paramT *params,
                tileparamT *tileparams, long nlines, long linelen);
/* functions in snaphu_cs2.c  */

/** SolveCS2 - formerly `main()` */
void SolveCS2(signed char **residue, short **mstcosts, long nrow, long ncol,
              long cs2scalefactor, short ***flowsptr);


/** Entrypoint (`main()`)
 *
 * This function is meant to be used with a wrapper function, and it operates exactly like SNAPHU's main CLI interface.
 */
int run_main(int argc, char **argv);
/* end of snaphu.h */

/*************************************************************************

  This code is derived from cs2 v3.7
  Written by Andrew V. Goldberg and Boris Cherkassky
  Modifications for use in snaphu by Curtis W. Chen

  Header for cs2 minimum cost flow solver.  This file is included with
  a #include from snaphu_cs2.c.

  The cs2 code is used here with permission for strictly noncommerical
  use.  The original cs2 source code can be downloaded from

    <http://www.igsystems.com/cs2>

  The original cs2 copyright is stated as follows:

    COPYRIGHT C 1995 IG Systems, Inc.  Permission to use for
    evaluation purposes is granted provided that proper
    acknowledgments are given.  For a commercial licence, contact
    igsys@eclipse.net.

    This software comes with NO WARRANTY, expressed or implied. By way
    of example, but not limitation, we make no representations of
    warranties of merchantability or fitness for any particular
    purpose or that the use of the software components or
    documentation will not infringe any patents, copyrights,
    trademarks, or other rights.

  Copyright 2002 Board of Trustees, Leland Stanford Jr. University

*************************************************************************/

/* defs.h */

/** Excess of the node */
typedef long excess_t;

/** Arc */
typedef
   struct arc_st
{
   short            r_cap;           /** residual capasity */
   short            cost;            /** cost  of the arc*/
   struct node_st   *head;           /** head node */
   struct arc_st    *sister;         /** opposite arc */
}
  arc;

typedef  /** node */
   struct node_st
{
   arc              *first;           /** first outgoing arc */
   arc              *current;         /** current outgoing arc */
   arc              *suspended;
   double           price;            /** distance from a sink */
   struct node_st   *q_next;          /** next node in push queue */
   struct node_st   *b_next;          /** next node in bucket-list */
   struct node_st   *b_prev;          /** previous node in bucket-list */
   long             rank;             /** bucket number */
   excess_t         excess;           /** excess of the node */
   signed char      inp;              /** temporary number of input arcs */
} node;

typedef /** bucket */
   struct bucket_st
{
   node             *p_first;         /** 1st node with positive excess
                                         or simply 1st node in the buket */
} bucket;

#endif /* SNAPHU_H */

/* end snaphu.h */
/* snaphu.c */
/*************************************************************************

  snaphu main source file
  Written by Curtis W. Chen
  Copyright 2002 Board of Trustees, Leland Stanford Jr. University
  Please see the supporting documentation for terms of use.
  No warranty.

*************************************************************************/

/*******************************************/
/* global (external) variable declarations */
/*******************************************/

/* flags used for signal handling */
extern char dumpresults_global;
extern char requestedstop_global;

/* ouput stream pointers */
/* sp0=error messages, sp1=status output, sp2=verbose, sp3=verbose counter */
extern FILE *sp0, *sp1, *sp2, *sp3;

/* node pointer for marking arc not on tree in apex array */
/* this should be treat as a constant */
extern nodeT NONTREEARC[1];

/* pointers to functions which calculate arc costs */
extern void (*CalcCost)(void **, long, long, long, long, long,
                        paramT *, long *, long *);
extern long (*EvalCost)(void **, short **, long, long, long, paramT *);
/* global (external) variable definitions */

/* flags used for signal handling */
char dumpresults_global = FALSE;
char requestedstop_global = FALSE;

/* ouput stream pointers */
/* sp0=error messages, sp1=status output, sp2=verbose, sp3=verbose counter */
FILE *sp0 = NULL;
FILE *sp1 = NULL;
FILE *sp2 = NULL;
FILE *sp3 = NULL;

/* node pointer for marking arc not on tree in apex array */
/* this should be treated as a constant */
nodeT NONTREEARC[1];

/* pointers to functions which calculate arc costs */
void (*CalcCost)(void **, long, long, long, long, long,
                 paramT *, long *, long *) = NULL;
long (*EvalCost)(void **, short **, long, long, long, paramT *) = NULL;

int StartTimers(time_t *tstart, double *cputimestart);
int DisplayElapsedTime(time_t tstart, double cputimestart);
/***************************/
/* main program for snaphu */
/***************************/

int run_main(int argc, char **argv) {

  /* variable declarations */
  infileT infiles[1];
  outfileT outfiles[1];
  paramT params[1];
  time_t tstart;
  double cputimestart;
  long linelen, nlines;


  /* get current wall clock and CPU time */
  StartTimers(&tstart,&cputimestart);

  /* set output stream pointers (may be reset after inputs parsed) */
  SetStreamPointers();

  /* print greeting */
  fprintf(sp1,"\n%s v%s\n",PROGRAMNAME,VERSION);

  /* set default parameters */
  SetDefaults(infiles,outfiles,params);
  ReadConfigFile(DEF_SYSCONFFILE,infiles,outfiles,&linelen,params);

  /* parse the command line inputs */
  ProcessArgs(argc,argv,infiles,outfiles,&linelen,params);

  /* set verbose output if specified */
  SetVerboseOut(params);

  /* set names of dump files if necessary */
  SetDumpAll(outfiles,params);

  /* get number of lines in file */
  nlines=GetNLines(infiles,linelen,params);

  /* check validity of parameters */
  CheckParams(infiles,outfiles,linelen,nlines,params);

  /* log the runtime parameters */
  WriteConfigLogFile(argc,argv,infiles,outfiles,linelen,params);

  /* unwrap, forming tiles and reassembling if necessary */
  Unwrap(infiles,outfiles,params,linelen,nlines);

  /* finish up */
  fprintf(sp1,"Program %s done\n",PROGRAMNAME);
  DisplayElapsedTime(tstart,cputimestart);
  exit(NORMAL_EXIT);

} /* end of main() */



/* function: ChildResetStreamPointers()
 * -----------------------------------
 * Reset the global stream pointers for a child.  Streams equal to stdout
 * are directed to a log file, and errors are written to the screen.
 */
int ChildResetStreamPointers(pid_t pid, long tilerow, long tilecol,
                             paramT *params){

  FILE *logfp;
  char logfile[MAXSTRLEN], cwd[MAXSTRLEN];

  fflush(NULL);
  sprintf(logfile,"%s/%s%ld_%ld",params->tiledir,LOGFILEROOT,tilerow,tilecol);
  if((logfp=fopen(logfile,"w"))==NULL){
    fflush(NULL);
    fprintf(sp0,"Unable to open log file %s\nAbort\n",logfile);
    exit(ABNORMAL_EXIT);
  }
  fprintf(logfp,"%s (pid %ld): unwrapping tile at row %ld, column %ld\n\n",
          PROGRAMNAME,(long )pid,tilerow,tilecol);
  if(getcwd(cwd,MAXSTRLEN)!=NULL){
    fprintf(logfp,"Current working directory is %s\n",cwd);
  }
  if(sp2==stdout || sp2==stderr){
    sp2=logfp;
  }
  if(sp1==stdout || sp1==stderr){
    sp1=logfp;
  }
  if(sp0==stdout || sp0==stderr){
    sp0=logfp;
  }
  if(sp3!=stdout && sp3!=stderr && sp3!=stdin && sp3!=NULL){
    fclose(sp3);
  }
  if((sp3=fopen(NULLFILE,"w"))==NULL){
    fflush(NULL);
    fprintf(sp0,"Unable to open null file %s\n",NULLFILE);
    exit(ABNORMAL_EXIT);
  }
  return(0);
}

/* function: Unwrap()
 * ------------------
 * Sets parameters for each tile and calls UnwrapTile() to do the
 * unwrapping.
 */
int Unwrap(infileT *infiles, outfileT *outfiles, paramT *params,
           long linelen, long nlines){

  long optiter, noptiter;
  long nexttilerow, nexttilecol, ntilerow, ntilecol, nthreads, nchildren;
  long sleepinterval;
  tileparamT tileparams[1];
  infileT iterinfiles[1];
  outfileT iteroutfiles[1];
  outfileT tileoutfiles[1];
  paramT iterparams[1];
  char tileinitfile[MAXSTRLEN];
  pid_t pid;
  int childstatus;
  double tilecputimestart;
  time_t tiletstart;
  signed char **dotilemask;


  /* initialize structure stack memory to zero for extra robustness */
  memset(tileparams,0,sizeof(tileparamT));
  memset(iterinfiles,0,sizeof(infileT));
  memset(iteroutfiles,0,sizeof(outfileT));
  memset(tileoutfiles,0,sizeof(outfileT));
  memset(iterparams,0,sizeof(paramT));
  memset(tileinitfile,0,MAXSTRLEN);

  /* see if we need to do single-tile reoptimization and set up if so */
  if(params->onetilereopt){
    noptiter=2;
  }else{
    noptiter=1;
  }

  /* iterate if necessary for single-tile reoptimization */
  for(optiter=0;optiter<noptiter;optiter++){

    /* initialize input and output file structures for this iteration */
    memcpy(iterinfiles,infiles,sizeof(infileT));
    memcpy(iteroutfiles,outfiles,sizeof(outfileT));
    memcpy(iterparams,params,sizeof(paramT));

    /* set up for iteration if doing tile init and one-tile reoptimization*/
    if(optiter==0){

      /* first iteration: see if there will be another iteration */
      if(noptiter>1){

        /* set up to write tile-mode unwrapped result to temporary file */
        SetTileInitOutfile(iteroutfiles->outfile,iterparams->parentpid);
        StrNCopy(tileinitfile,iteroutfiles->outfile,MAXSTRLEN);
        iteroutfiles->outfileformat=TILEINITFILEFORMAT;
        fprintf(sp1,"Starting first-round tile-mode unwrapping\n");

      }

    }else if(optiter==1){

      /* second iteration */
      /* set up to read unwrapped tile-mode result as single tile */
      StrNCopy(iterinfiles->infile,tileinitfile,MAXSTRLEN);
      iterinfiles->unwrappedinfileformat=TILEINITFILEFORMAT;
      iterparams->unwrapped=TRUE;
      iterparams->ntilerow=1;
      iterparams->ntilecol=1;
      iterparams->rowovrlp=0;
      iterparams->colovrlp=0;
      fprintf(sp1,"Starting second-round single-tile unwrapping\n");

    }else{
      fprintf(sp0,"ERROR: illegal optiter value in Unwrap()\n");
      exit(ABNORMAL_EXIT);
    }

    /* set up for unwrapping */
    ntilerow=iterparams->ntilerow;
    ntilecol=iterparams->ntilecol;
    nthreads=iterparams->nthreads;
    dumpresults_global=FALSE;
    requestedstop_global=FALSE;

    /* do the unwrapping */
    if(ntilerow==1 && ntilecol==1){

      /* only single tile */

      /* do the unwrapping */
      tileparams->firstrow=iterparams->piecefirstrow;
      tileparams->firstcol=iterparams->piecefirstcol;
      tileparams->nrow=iterparams->piecenrow;
      tileparams->ncol=iterparams->piecencol;
      UnwrapTile(iterinfiles,iteroutfiles,iterparams,tileparams,nlines,linelen);

    }else{

      /* don't unwrap if in assemble-only mode */
      if(!iterparams->assembleonly){

        /* set up mask for which tiles should be unwrapped */
        dotilemask=SetUpDoTileMask(iterinfiles,ntilerow,ntilecol);

        /* make a temporary directory into which tile files will be written */
        MakeTileDir(iterparams,iteroutfiles);

        /* different code for parallel or nonparallel operation */
        if(nthreads>1){

          /* parallel code */

          /* initialize */
          nexttilerow=0;
          nexttilecol=0;
          nchildren=0;
          sleepinterval=(long )ceil(nlines*linelen
                                    /((double )(ntilerow*ntilecol))
                                    *SECONDSPERPIXEL);

          /* trap signals so children get killed if parent dies */
          CatchSignals(KillChildrenExit);

          /* loop until we're done unwrapping */
          while(TRUE){

            /* unwrap next tile if there are free processors and tiles left */
            if(nchildren<nthreads && nexttilerow<ntilerow){

              /* see if next tile needs to be unwrapped */
              if(dotilemask[nexttilerow][nexttilecol]){

                /* wait to make sure file i/o, threads, and OS are synched */
                sleep(sleepinterval);

                /* fork to create new process */
                fflush(NULL);
                pid=fork();

              }else{

                /* tile did not need unwrapping, so set pid to parent pid */
                pid=iterparams->parentpid;

              }

              /* see if parent or child (or error) */
              if(pid<0){

                /* parent kills children and exits if there was a fork error */
                fflush(NULL);
                fprintf(sp0,"Error while forking\nAbort\n");
                kill(0,SIGKILL);
                exit(ABNORMAL_EXIT);

              }else if(pid==0){

                /* child executes this code after fork */

                /* reset signal handlers so that children exit nicely */
                CatchSignals(SignalExit);

                /* start timers for this tile */
                StartTimers(&tiletstart,&tilecputimestart);

                /* set up tile parameters */
                pid=getpid();
                fprintf(sp1,
                        "Unwrapping tile at row %ld, column %ld (pid %ld)\n",
                        nexttilerow,nexttilecol,(long )pid);
                SetupTile(nlines,linelen,iterparams,tileparams,
                          iteroutfiles,tileoutfiles,
                          nexttilerow,nexttilecol);

                /* reset stream pointers for logging */
                ChildResetStreamPointers(pid,nexttilerow,nexttilecol,
                                         iterparams);

                /* unwrap the tile */
                UnwrapTile(iterinfiles,tileoutfiles,iterparams,tileparams,
                           nlines,linelen);

                /* log elapsed time */
                DisplayElapsedTime(tiletstart,tilecputimestart);

                /* child exits when done unwrapping */
                exit(NORMAL_EXIT);

              }

              /* parent executes this code after fork */

              /* increment tile counters */
              if(++nexttilecol==ntilecol){
                nexttilecol=0;
                nexttilerow++;
              }

              /* increment counter of running child processes */
              if(pid!=iterparams->parentpid){
                nchildren++;
              }

            }else{

              /* wait for a child to finish (only parent gets here) */
              pid=wait(&childstatus);

              /* make sure child exited cleanly */
              if(!(WIFEXITED(childstatus)) || (WEXITSTATUS(childstatus))!=0){
                fflush(NULL);
                fprintf(sp0,"Unexpected or abnormal exit of child process %ld\n"
                        "Abort\n",(long )pid);
                signal(SIGTERM,SIG_IGN);
                kill(0,SIGTERM);
                exit(ABNORMAL_EXIT);
              }

              /* we're done if there are no more active children */
              /* shouldn't really need this sleep(), but be extra sure child */
              /*   outputs are really flushed and written to disk by OS */
              if(--nchildren==0){
                sleep(sleepinterval);
                break;
              }

            } /* end if free processor and tiles remaining */
          } /* end while loop */

          /* return signal handlers to default behavior */
          CatchSignals(SIG_DFL);

        }else{

          /* nonparallel code */

          /* loop over all tiles */
          for(nexttilerow=0;nexttilerow<ntilerow;nexttilerow++){
            for(nexttilecol=0;nexttilecol<ntilecol;nexttilecol++){
              if(dotilemask[nexttilerow][nexttilecol]){

                /* set up tile parameters */
                fprintf(sp1,"Unwrapping tile at row %ld, column %ld\n",
                        nexttilerow,nexttilecol);
                SetupTile(nlines,linelen,iterparams,tileparams,
                          iteroutfiles,tileoutfiles,
                          nexttilerow,nexttilecol);

                /* unwrap the tile */
                UnwrapTile(iterinfiles,tileoutfiles,iterparams,tileparams,
                           nlines,linelen);

              }
            }
          }

        } /* end if nthreads>1 */

        /* free tile mask memory */
        Free2DArray((void **)dotilemask,ntilerow);

      } /* end if !iterparams->assembleonly */

      /* reassemble tiles */
      AssembleTiles(iteroutfiles,iterparams,nlines,linelen);

    } /* end if multiple tiles */

    /* remove temporary tile file if desired at end of second iteration */
    if(iterparams->rmtileinit && optiter>0){
      unlink(tileinitfile);
    }

  } /* end of optiter loop */

  /* done */
  return(0);

} /* end of Unwrap() */


/* function: UnwrapTile()
 * ----------------------
 * This is the main phase unwrapping function for a single tile.
 */
int UnwrapTile(infileT *infiles, outfileT *outfiles, paramT *params,
               tileparamT *tileparams,  long nlines, long linelen){

  /* variable declarations */
  long nrow, ncol, nnoderow, narcrow, n, ngroundarcs, iincrcostfile;
  long nflow, ncycle, mostflow, nflowdone;
  long candidatelistsize, candidatebagsize;
  long isource, nsource;
  long nnondecreasedcostiter;
  long *nconnectedarr;
  int *nnodesperrow, *narcsperrow;
  short **flows, **mstcosts;
  float **wrappedphase, **unwrappedphase, **mag, **unwrappedest;
  incrcostT **incrcosts;
  void **costs;
  totalcostT totalcost, oldtotalcost, mintotalcost;
  nodeT **sourcelist;
  nodeT *source, ***apexes;
  nodeT **nodes, ground[1];
  candidateT *candidatebag, *candidatelist;
  signed char **iscandidate;
  signed char notfirstloop, allmasked;
  bucketT *bkts;


  /* get size of tile */
  nrow=tileparams->nrow;
  ncol=tileparams->ncol;

  /* read input file (memory allocated by read function) */
  ReadInputFile(infiles,&mag,&wrappedphase,&flows,linelen,nlines,
                params,tileparams);

  /* read interferogram magnitude if specified separately */
  ReadMagnitude(mag,infiles,linelen,nlines,tileparams);

  /* read mask file and apply to magnitude */
  ReadByteMask(mag,infiles,linelen,nlines,tileparams,params);

  /* make sure we have at least one pixel that is not masked */
  allmasked=CheckMagMasking(mag,nrow,ncol);

  /* read the coarse unwrapped estimate, if provided */
  unwrappedest=NULL;
  if(strlen(infiles->estfile)){
    ReadUnwrappedEstimateFile(&unwrappedest,infiles,linelen,nlines,
                              params,tileparams);

    /* subtract the estimate from the wrapped phase (and re-wrap) */
    FlattenWrappedPhase(wrappedphase,unwrappedest,nrow,ncol);

  }

  /* build the cost arrays */
  BuildCostArrays(&costs,&mstcosts,mag,wrappedphase,unwrappedest,
                  linelen,nlines,nrow,ncol,params,tileparams,infiles,outfiles);

  /* if in quantify-only mode, evaluate cost of unwrapped input then return */
  if(params->eval){
    mostflow=Short2DRowColAbsMax(flows,nrow,ncol);
    fprintf(sp1,"Maximum flow on network: %ld\n",mostflow);
    totalcost=EvaluateTotalCost(costs,flows,nrow,ncol,NULL,params);
    fprintf(sp1,"Total solution cost: %.9g\n",(double )totalcost);
    Free2DArray((void **)costs,2*nrow-1);
    Free2DArray((void **)mag,nrow);
    Free2DArray((void **)wrappedphase,nrow);
    Free2DArray((void **)flows,2*nrow-1);
    return(1);
  }

  /* set network function pointers for grid network */
  SetGridNetworkFunctionPointers();

  /* initialize the flows (find simple unwrapping to get a feasible flow) */
  unwrappedphase=NULL;
  nodes=NULL;
  if(!params->unwrapped){

    /* see which initialization method to use */
    if(params->initmethod==MSTINIT){

      /* use minimum spanning tree (MST) algorithm */
      MSTInitFlows(wrappedphase,&flows,mstcosts,nrow,ncol,
                   &nodes,ground,params->initmaxflow);

    }else if(params->initmethod==MCFINIT){

      /* use minimum cost flow (MCF) algorithm */
      MCFInitFlows(wrappedphase,&flows,mstcosts,nrow,ncol,
                   params->cs2scalefactor);

    }else{
      fflush(NULL);
      fprintf(sp0,"Illegal initialization method\nAbort\n");
      exit(ABNORMAL_EXIT);
    }

    /* integrate the phase and write out if necessary */
    if(params->initonly || strlen(outfiles->initfile)){
      fprintf(sp1,"Integrating phase\n");
      unwrappedphase=(float **)Get2DMem(nrow,ncol,
                                        sizeof(float *),sizeof(float));
      IntegratePhase(wrappedphase,unwrappedphase,flows,nrow,ncol);
      if(unwrappedest!=NULL){
        Add2DFloatArrays(unwrappedphase,unwrappedest,nrow,ncol);
      }
      FlipPhaseArraySign(unwrappedphase,params,nrow,ncol);

      /* return if called in init only; otherwise, free memory and continue */
      if(params->initonly){
        fprintf(sp1,"Writing output to file %s\n",outfiles->outfile);
        WriteOutputFile(mag,unwrappedphase,outfiles->outfile,outfiles,
                        nrow,ncol);
        Free2DArray((void **)mag,nrow);
        Free2DArray((void **)wrappedphase,nrow);
        Free2DArray((void **)unwrappedphase,nrow);
        if(nodes!=NULL){
          Free2DArray((void **)nodes,nrow-1);
        }
        Free2DArray((void **)flows,2*nrow-1);
        return(1);
      }else{
        fprintf(sp2,"Writing initialization to file %s\n",outfiles->initfile);
        WriteOutputFile(mag,unwrappedphase,outfiles->initfile,outfiles,
                        nrow,ncol);
        Free2DArray((void **)unwrappedphase,nrow);
      }
    }
  }

  /* initialize network variables */
  InitNetwork(flows,&ngroundarcs,&ncycle,&nflowdone,&mostflow,&nflow,
              &candidatebagsize,&candidatebag,&candidatelistsize,
              &candidatelist,&iscandidate,&apexes,&bkts,&iincrcostfile,
              &incrcosts,&nodes,ground,&nnoderow,&nnodesperrow,&narcrow,
              &narcsperrow,nrow,ncol,&notfirstloop,&totalcost,params);
  oldtotalcost=totalcost;
  mintotalcost=totalcost;
  nnondecreasedcostiter=0;

  /* regrow regions with -G parameter */
  if(params->regrowconncomps){

    /* free up some memory */
    Free2DArray((void **)apexes,2*nrow-1);
    Free2DArray((void **)iscandidate,2*nrow-1);
    Free2DArray((void **)nodes,nrow-1);
    free(candidatebag);
    free(candidatelist);
    free(bkts->bucketbase);

    /* grow connected components */
    GrowConnCompsMask(costs,flows,nrow,ncol,incrcosts,outfiles,params);

    /* free up remaining memory and return */
    Free2DArray((void **)incrcosts,2*nrow-1);
    Free2DArray((void **)costs,2*nrow-1);
    Free2DArray((void **)mag,nrow);
    Free2DArray((void **)wrappedphase,nrow);
    Free2DArray((void **)flows,2*nrow-1);
    free(nnodesperrow);
    free(narcsperrow);
    return(1);
  }

  /* mask zero-magnitude nodes so they are not considered in optimization */
  MaskNodes(nrow,ncol,nodes,ground,mag);

  /* if we have a single tile, trap signals for dumping results */
  if(params->ntilerow==1 && params->ntilecol==1){
    signal(SIGINT,SetDump);
    signal(SIGHUP,SetDump);
  }

  /* main loop: loop over flow increments and sources */
  if(!allmasked){
    fprintf(sp1,"Running nonlinear network flow optimizer\n");
    fprintf(sp1,"Maximum flow on network: %ld\n",mostflow);
    fprintf(sp2,"Number of nodes in network: %ld\n",(nrow-1)*(ncol-1)+1);
    while(TRUE){

      fprintf(sp1,"Flow increment: %ld  (Total improvements: %ld)\n",
              nflow,ncycle);

      /* set up the incremental (residual) cost arrays */
      SetupIncrFlowCosts(costs,incrcosts,flows,nflow,nrow,narcrow,narcsperrow,
                         params);
      if(params->dumpall && params->ntilerow==1 && params->ntilecol==1){
        DumpIncrCostFiles(incrcosts,++iincrcostfile,nflow,nrow,ncol);
      }

      /* set the tree root (equivalent to source of shortest path problem) */
      sourcelist=NULL;
      nconnectedarr=NULL;
      nsource=SelectSources(nodes,mag,ground,nflow,flows,ngroundarcs,
                            nrow,ncol,params,&sourcelist,&nconnectedarr);

      /* set up network variables for tree solver */
      SetupTreeSolveNetwork(nodes,ground,apexes,iscandidate,
                            nnoderow,nnodesperrow,narcrow,narcsperrow,
                            nrow,ncol);

      /* loop over sources */
      n=0;
      for(isource=0;isource<nsource;isource++){

        /* set source */
        source=sourcelist[isource];

        /* show status if verbose */
        if(source->row==GROUNDROW){
          fprintf(sp3,"Source %ld: (edge ground)\n",isource);
        }else{
          fprintf(sp3,"Source %ld: row, col = %d, %d\n",
                  isource,source->row,source->col);
        }

        /* run the solver, and increment nflowdone if no cycles are found */
        n+=TreeSolve(nodes,NULL,ground,source,
                     &candidatelist,&candidatebag,
                     &candidatelistsize,&candidatebagsize,
                     bkts,flows,costs,incrcosts,apexes,iscandidate,
                     ngroundarcs,nflow,mag,wrappedphase,outfiles->outfile,
                     nnoderow,nnodesperrow,narcrow,narcsperrow,nrow,ncol,
                     outfiles,nconnectedarr[isource],params);
      }

      /* free temporary memory */
      free(sourcelist);
      free(nconnectedarr);

      /* evaluate and save the total cost (skip if first loop through nflow) */
      fprintf(sp2,"Current solution cost: %.16g\n",
              (double )EvaluateTotalCost(costs,flows,nrow,ncol,NULL,params));
      fflush(NULL);
      if(notfirstloop){
        oldtotalcost=totalcost;
        totalcost=EvaluateTotalCost(costs,flows,nrow,ncol,NULL,params);
        if(totalcost<mintotalcost){
          mintotalcost=totalcost;
        }
        if(totalcost>oldtotalcost || (n>0 && totalcost==oldtotalcost)){
          fflush(NULL);
          fprintf(sp1,"Caution: Unexpected increase in total cost\n");
        }
        if(totalcost > mintotalcost){
          nnondecreasedcostiter++;
        }else{
          nnondecreasedcostiter=0;
        }
      }

      /* consider this flow increment done if not too many neg cycles found */
      ncycle+=n;
      if(n<=params->maxnflowcycles){
        nflowdone++;
      }else{
        nflowdone=1;
      }

      /* find maximum flow on network, excluding arcs affected by masking */
      mostflow=MaxNonMaskFlow(flows,mag,nrow,ncol);
      if(nnondecreasedcostiter>=2*mostflow){
        fflush(NULL);
        fprintf(sp0,"WARNING: No overall cost reduction for too many iterations."
                "  Breaking loop\n");
        break;
      }

      /* break if we're done with all flow increments or problem is convex */
      if(nflowdone>=params->maxflow || nflowdone>=mostflow || params->p>=1.0){
        break;
      }

      /* update flow increment */
      nflow++;
      if(nflow>params->maxflow || nflow>mostflow){
        nflow=1;
        notfirstloop=TRUE;
      }
      fprintf(sp2,"Maximum valid flow on network: %ld\n",mostflow);

      /* dump flow arrays if necessary */
      if(strlen(outfiles->flowfile)){
        FlipFlowArraySign(flows,params,nrow,ncol);
        Write2DRowColArray((void **)flows,outfiles->flowfile,nrow,ncol,
                           sizeof(short));
        FlipFlowArraySign(flows,params,nrow,ncol);
      }

    } /* end loop until no more neg cycles */
  } /* end if all pixels masked */

  /* if we have single tile, return signal handlers to default behavior */
  if(params->ntilerow==1 && params->ntilecol==1){
    signal(SIGINT,SIG_DFL);
    signal(SIGHUP,SIG_DFL);
  }

  /* free some memory */
  Free2DArray((void **)apexes,2*nrow-1);
  Free2DArray((void **)iscandidate,2*nrow-1);
  Free2DArray((void **)nodes,nrow-1);
  free(candidatebag);
  free(candidatelist);
  free(bkts->bucketbase);

  /* grow connected component mask */
  if(strlen(outfiles->conncompfile)){
    GrowConnCompsMask(costs,flows,nrow,ncol,incrcosts,outfiles,params);
  }

  /* grow regions for tiling */
  if(params->ntilerow!=1 || params->ntilecol!=1){
    GrowRegions(costs,flows,nrow,ncol,incrcosts,outfiles,tileparams,params);
  }

  /* free some more memory */
  Free2DArray((void **)incrcosts,2*nrow-1);

  /* evaluate and display the maximum flow and total cost */
  totalcost=EvaluateTotalCost(costs,flows,nrow,ncol,NULL,params);
  fprintf(sp1,"Maximum flow on network: %ld\n",mostflow);
  fprintf(sp1,"Total solution cost: %.9g\n",(double )totalcost);

  /* integrate the wrapped phase using the solution flow */
  fprintf(sp1,"Integrating phase\n");
  unwrappedphase=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
  IntegratePhase(wrappedphase,unwrappedphase,flows,nrow,ncol);

  /* reinsert the coarse estimate, if it was given */
  if(unwrappedest!=NULL){
    Add2DFloatArrays(unwrappedphase,unwrappedest,nrow,ncol);
  }

  /* flip the sign of the unwrapped phase array if it was flipped initially, */
  FlipPhaseArraySign(unwrappedphase,params,nrow,ncol);


  /* write the unwrapped output */
  fprintf(sp1,"Writing output to file %s\n",outfiles->outfile);
  WriteOutputFile(mag,unwrappedphase,outfiles->outfile,outfiles,
                  nrow,ncol);

  /* free remaining memory and return */
  Free2DArray((void **)costs,2*nrow-1);
  Free2DArray((void **)mag,nrow);
  Free2DArray((void **)wrappedphase,nrow);
  Free2DArray((void **)unwrappedphase,nrow);
  Free2DArray((void **)flows,2*nrow-1);
  free(nnodesperrow);
  free(narcsperrow);
  return(0);

} /* end of UnwrapTile() */

/* end snaphu.c */

/* snaphu_cost.c */
/*************************************************************************

  snaphu statistical cost model source file
  Written by Curtis W. Chen
  Copyright 2002 Board of Trustees, Leland Stanford Jr. University
  Please see the supporting documentation for terms of use.
  No warranty.

*************************************************************************/

/* ouput stream pointers */
/* sp0=error messages, sp1=status output, sp2=verbose, sp3=verbose counter */
extern FILE *sp0, *sp1, *sp2, *sp3;
/* pointers to functions which calculate arc costs */
extern void (*CalcCost)(void **, long, long, long, long, long,
                        paramT *, long *, long *);
extern long (*EvalCost)(void **, short **, long, long, long, paramT *);
/* static (local) function prototypes */
static
void **BuildStatCostsTopo(float **wrappedphase, float **mag,
                          float **unwrappedest, float **pwr,
                          float **corr, short **rowweight, short **colweight,
                          long nrow, long ncol, tileparamT *tileparams,
                          outfileT *outfiles, paramT *params);
static
void **BuildStatCostsDefo(float **wrappedphase, float **mag,
                          float **unwrappedest, float **corr,
                          short **rowweight, short **colweight,
                          long nrow, long ncol, tileparamT *tileparams,
                          outfileT *outfiles, paramT *params);
static
void **BuildStatCostsSmooth(float **wrappedphase, float **mag,
                            float **unwrappedest, float **corr,
                            short **rowweight, short **colweight,
                            long nrow, long ncol, tileparamT *tileparams,
                            outfileT *outfiles, paramT *params);
static
void MaskCost(costT *costptr);
static
void MaskSmoothCost(smoothcostT *smoothcostptr);
static
int MaskPrespecifiedArcCosts(void **costsptr, short **weights,
                             long nrow, long ncol, paramT *params);
static
int GetIntensityAndCorrelation(float **mag, float **wrappedphase,
                               float ***pwrptr, float ***corrptr,
                               infileT *infiles, long linelen, long nlines,
                               long nrow, long ncol, outfileT *outfiles,
                               paramT *params, tileparamT *tileparams);
static
int RemoveMean(float **ei, long nrow, long ncol,
               long krowei, long kcolei);
static
float *BuildDZRCritLookupTable(double *nominc0ptr, double *dnomincptr,
                               long *tablesizeptr, tileparamT *tileparams,
                               paramT *params);
static
double SolveDZRCrit(double sinnomincangle, double cosnomincangle,
                    paramT *params, double threshold);
static
int SolveEIModelParams(double *slope1ptr, double *slope2ptr,
                       double *const1ptr, double *const2ptr,
                       double dzrcrit, double dzr0, double sinnomincangle,
                       double cosnomincangle, paramT *params);
static
double EIofDZR(double dzr, double sinnomincangle, double cosnomincangle,
               paramT *params);
static
float **BuildDZRhoMaxLookupTable(double nominc0, double dnominc,
                                 long nominctablesize, double rhomin,
                                 double drho, long nrho, paramT *params);
static
double CalcDZRhoMax(double rho, double nominc, paramT *params,
                    double threshold);
static
int CalcInitMaxFlow(paramT *params, void **costs, long nrow, long ncol);



/**  function: BuildCostArrays()
 * ---------------------------
 * Builds cost arrays for arcs based on interferogram intensity
 * and correlation, depending on options and passed parameters.
 */
int BuildCostArrays(void ***costsptr, short ***mstcostsptr,
                    float **mag, float **wrappedphase,
                    float **unwrappedest, long linelen, long nlines,
                    long nrow, long ncol, paramT *params,
                    tileparamT *tileparams, infileT *infiles,
                    outfileT *outfiles){

  long row, col, maxcol, tempcost;
  long poscost, negcost, costtypesize;
  float **pwr, **corr;
  short **weights, **rowweight, **colweight, **scalarcosts;
  bidircostT **bidircosts;
  void **costs, **rowcost, **colcost;
  void (*CalcStatCost)(void **, long, long, long, long, long,
                       paramT *, long *, long *);



  /* initializations to silence compiler warnings */
  costtypesize=0;
  bidircosts=NULL;
  scalarcosts=NULL;

  /* set global pointers to functions for calculating and evaluating costs */
  if(params->p<0){
    if(params->costmode==TOPO){
      CalcCost=CalcCostTopo;
      EvalCost=EvalCostTopo;
    }else if(params->costmode==DEFO){
      CalcCost=CalcCostDefo;
      EvalCost=EvalCostDefo;
    }else if(params->costmode==SMOOTH){
      CalcCost=CalcCostSmooth;
      EvalCost=EvalCostSmooth;
    }
  }else{
    if(params->bidirlpn){
      if(params->p==0){
        CalcCost=CalcCostL0BiDir;
        EvalCost=EvalCostL0BiDir;
      }else if(params->p==1){
        CalcCost=CalcCostL1BiDir;
        EvalCost=EvalCostL1BiDir;
      }else if(params->p==2){
        CalcCost=CalcCostL2BiDir;
        EvalCost=EvalCostL2BiDir;
      }else{
        CalcCost=CalcCostLPBiDir;
        EvalCost=EvalCostLPBiDir;
      }
    }else{
      if(params->p==0){
        CalcCost=CalcCostL0;
        EvalCost=EvalCostL0;
      }else if(params->p==1){
        CalcCost=CalcCostL1;
        EvalCost=EvalCostL1;
      }else if(params->p==2){
        CalcCost=CalcCostL2;
        EvalCost=EvalCostL2;
      }else{
        CalcCost=CalcCostLP;
        EvalCost=EvalCostLP;
      }
    }
  }

  /* read weights */
  weights=NULL;
  ReadWeightsFile(&weights,infiles->weightfile,linelen,nlines,tileparams);
  rowweight=weights;
  colweight=&weights[nrow-1];

  /* set weights to zero for arcs adjacent to zero-magnitude pixels */
  if(mag!=NULL){
    for(row=0;row<nrow;row++){
      for(col=0;col<ncol;col++){
        if(mag[row][col]==0){
          if(row>0){
            rowweight[row-1][col]=0;
          }
          if(row<nrow-1){
            rowweight[row][col]=0;
          }
          if(col>0){
            colweight[row][col-1]=0;
          }
          if(col<ncol-1){
            colweight[row][col]=0;
          }
        }
      }
    }
  }

  /* if we're only initializing and we don't want statistical weights */
  if(params->initonly && params->costmode==NOSTATCOSTS){
    *mstcostsptr=weights;
    return(0);
  }

  /* size of the data type for holding cost data depends on cost mode */
  if(params->costmode==TOPO){
    costtypesize=sizeof(costT);
  }else if(params->costmode==DEFO){
    costtypesize=sizeof(costT);
  }else if(params->costmode==SMOOTH){
    costtypesize=sizeof(smoothcostT);
  }

  /* build or read the statistical cost arrays unless we were told not to */
  if(strlen(infiles->costinfile)){

    /* read cost info from file */
    fprintf(sp1,"Reading cost information from file %s\n",infiles->costinfile);
    costs=NULL;
    Read2DRowColFile((void ***)&costs,infiles->costinfile,
                     linelen,nlines,tileparams,costtypesize);
    (*costsptr)=costs;

    /* weights of arcs next to masked pixels are set to zero */
    /* make sure corresponding costs are nulled when costs are read from */
    /*   file rather than internally generated since read costs are not */
    /*   multiplied by weights */
    MaskPrespecifiedArcCosts(costs,weights,nrow,ncol,params);

  }else if(params->costmode!=NOSTATCOSTS){

    /* get intensity and correlation info */
    /* correlation generated from interferogram and amplitude if not given */
    GetIntensityAndCorrelation(mag,wrappedphase,&pwr,&corr,infiles,
                               linelen,nlines,nrow,ncol,outfiles,
                               params,tileparams);

    /* call specific functions for building cost array and */
    /* set global pointers to functions for calculating and evaluating costs */
    if(params->costmode==TOPO){
      fprintf(sp1,"Calculating topography-mode cost parameters\n");
      costs=BuildStatCostsTopo(wrappedphase,mag,unwrappedest,pwr,corr,
                               rowweight,colweight,nrow,ncol,tileparams,
                               outfiles,params);
    }else if(params->costmode==DEFO){
      fprintf(sp1,"Calculating deformation-mode cost parameters\n");
      costs=BuildStatCostsDefo(wrappedphase,mag,unwrappedest,corr,
                               rowweight,colweight,nrow,ncol,tileparams,
                               outfiles,params);
    }else if(params->costmode==SMOOTH){
      fprintf(sp1,"Calculating smooth-solution cost parameters\n");
      costs=BuildStatCostsSmooth(wrappedphase,mag,unwrappedest,corr,
                                 rowweight,colweight,nrow,ncol,tileparams,
                                 outfiles,params);
    }else{
      costs=NULL;
      fflush(NULL);
      fprintf(sp0,"unrecognized cost mode\n");
      exit(ABNORMAL_EXIT);
    }
    (*costsptr)=costs;


  }/* end if(params->costmode!=NOSTATCOSTS) */

  /* set array subpointers and temporary cost-calculation function pointer */
  if(params->costmode==TOPO){
    rowcost=costs;
    colcost=(void **)&(((costT **)costs)[nrow-1]);
    CalcStatCost=CalcCostTopo;
  }else if(params->costmode==DEFO){
    rowcost=costs;
    colcost=(void **)&(((costT **)costs)[nrow-1]);
    CalcStatCost=CalcCostDefo;
  }else if(params->costmode==SMOOTH){
    rowcost=costs;
    colcost=(void **)&(((smoothcostT **)costs)[nrow-1]);
    CalcStatCost=CalcCostSmooth;
  }else{
    rowcost=NULL;
    colcost=NULL;
    CalcStatCost=NULL;
    fflush(NULL);
    fprintf(sp0,"unrecognized cost mode\n");
    exit(ABNORMAL_EXIT);
  }


  /* dump statistical cost arrays */
  if(strlen(infiles->costinfile) || params->costmode!=NOSTATCOSTS){
    if(strlen(outfiles->costoutfile)){
      Write2DRowColArray((void **)costs,outfiles->costoutfile,
                        nrow,ncol,costtypesize);
    }else{
      if(strlen(outfiles->rowcostfile)){
        Write2DArray((void **)rowcost,outfiles->rowcostfile,
                     nrow-1,ncol,costtypesize);
      }
      if(strlen(outfiles->colcostfile)){
        Write2DArray((void **)colcost,outfiles->colcostfile,
                     nrow,ncol-1,costtypesize);
      }
    }
  }

  /* get memory for scalar costs if in Lp mode */
  if(params->p>=0){
    if(params->bidirlpn){
      bidircosts=(bidircostT **)Get2DRowColMem(nrow,ncol,sizeof(bidircostT *),
                                               sizeof(bidircostT));
      (*costsptr)=(void **)bidircosts;
    }else{
      scalarcosts=(short **)Get2DRowColMem(nrow,ncol,sizeof(short *),
                                           sizeof(short));
      (*costsptr)=(void **)scalarcosts;
    }
  }

  /* now, set scalar costs for MST initialization or optimization if needed */
  if(params->costmode==NOSTATCOSTS){

    /* if in no-statistical-costs mode, copy weights to scalarcosts array */
    if(!params->initonly){
      for(row=0;row<2*nrow-1;row++){
        if(row<nrow-1){
          maxcol=ncol;
        }else{
          maxcol=ncol-1;
        }
        for(col=0;col<maxcol;col++){
          if(params->bidirlpn){
            bidircosts[row][col].posweight=weights[row][col];
            bidircosts[row][col].negweight=weights[row][col];
          }else{
            scalarcosts[row][col]=weights[row][col];
          }
        }
      }
    }

    /* unless input is already unwrapped, use weights memory for mstcosts */
    if(!params->unwrapped){
      (*mstcostsptr)=weights;
    }else{
      Free2DArray((void **)weights,2*nrow-1);
      (*mstcostsptr)=NULL;
    }

  }else if(!params->unwrapped || params->p>=0){

    /* if we got here, we had statistical costs and we need scalar weights */
    /*   from them for MST initialization or for Lp optimization */
    for(row=0;row<2*nrow-1;row++){
      if(row<nrow-1){
        maxcol=ncol;
      }else{
        maxcol=ncol-1;
      }
      for(col=0;col<maxcol;col++){

        /* calculate incremental costs for flow=0, nflow=1 */
        CalcStatCost((void **)costs,0,row,col,1,nrow,params,
                     &poscost,&negcost);

        /* take smaller of positive and negative incremental cost */
        if(poscost<negcost){
          tempcost=poscost;
        }else{
          tempcost=negcost;
        }

        /* clip scalar cost so it is between 1 and params->maxcost */
        /* note: weights used for MST algorithm will not be zero along */
        /*   masked edges since they are clipped to 1, but MST is run */
        /*   once on entire network, not just non-masked regions */
        weights[row][col]=LClip(tempcost,MINSCALARCOST,params->maxcost);

        /* assign Lp costs if in Lp mode */
        /* let scalar cost be zero if costs in both directions are zero */
        if(params->p>=0){
          if(params->bidirlpn){
            bidircosts[row][col].posweight=LClip(poscost,0,params->maxcost);
            bidircosts[row][col].negweight=LClip(negcost,0,params->maxcost);
          }else{
            scalarcosts[row][col]=weights[row][col];
            if(poscost==0 && negcost==0){
              scalarcosts[row][col]=0;
            }
          }
        }
      }
    }

    /* set costs for corner arcs to prevent ambiguous flows */
    weights[nrow-1][0]=LARGESHORT;
    weights[nrow-1][ncol-2]=LARGESHORT;
    weights[2*nrow-2][0]=LARGESHORT;
    weights[2*nrow-2][ncol-2]=LARGESHORT;
    if(params->p>=0){
      if(params->bidirlpn){
        bidircosts[nrow-1][0].posweight=LARGESHORT;
        bidircosts[nrow-1][0].negweight=LARGESHORT;
        bidircosts[nrow-1][ncol-2].posweight=LARGESHORT;
        bidircosts[nrow-1][ncol-2].negweight=LARGESHORT;
        bidircosts[2*nrow-2][0].posweight=LARGESHORT;
        bidircosts[2*nrow-2][0].negweight=LARGESHORT;
        bidircosts[2*nrow-2][ncol-2].posweight=LARGESHORT;
        bidircosts[2*nrow-2][ncol-2].negweight=LARGESHORT;
      }else{
        scalarcosts[nrow-1][0]=LARGESHORT;
        scalarcosts[nrow-1][ncol-2]=LARGESHORT;
        scalarcosts[2*nrow-2][0]=LARGESHORT;
        scalarcosts[2*nrow-2][ncol-2]=LARGESHORT;
      }
    }

    /* dump mst initialization costs */
    if(strlen(outfiles->mstrowcostfile)){
      Write2DArray((void **)rowweight,outfiles->mstrowcostfile,
                   nrow-1,ncol,sizeof(short));
    }
    if(strlen(outfiles->mstcolcostfile)){
      Write2DArray((void **)colweight,outfiles->mstcolcostfile,
                   nrow,ncol-1,sizeof(short));
    }
    if(strlen(outfiles->mstcostsfile)){
      Write2DRowColArray((void **)rowweight,outfiles->mstcostsfile,
                         nrow,ncol,sizeof(short));
    }

    /* unless input is unwrapped, calculate initialization max flow */
    if(params->initmaxflow==AUTOCALCSTATMAX && !params->unwrapped){
      CalcInitMaxFlow(params,(void **)costs,nrow,ncol);
    }

    /* free costs memory if in init-only or Lp mode */
    if(params->initonly || params->p>=0){
      Free2DArray((void **)costs,2*nrow-1);
    }

    /* use memory allocated for weights arrays for mstcosts if needed */
    if(!params->unwrapped){
      (*mstcostsptr)=weights;
    }else{
      Free2DArray((void **)weights,2*nrow-1);
    }

  }else{
    Free2DArray((void **)weights,2*nrow-1);
  }

  /* done */
  return(0);

}


/* function: BuildStatCostsTopo()
 * ------------------------------
 * Builds statistical cost arrays for topography mode.
 */
static
void **BuildStatCostsTopo(float **wrappedphase, float **mag,
                          float **unwrappedest, float **pwr,
                          float **corr, short **rowweight, short **colweight,
                          long nrow, long ncol, tileparamT *tileparams,
                          outfileT *outfiles, paramT *params){

  long row, col, iei, nrho, nominctablesize;
  long kperpdpsi, kpardpsi, sigsqshortmin;
  double a, re, dr, slantrange, nearrange, nominc0, dnominc;
  double nomincangle, nomincind, sinnomincangle, cosnomincangle, bperp;
  double baseline, baselineangle, lambda, lookangle;
  double dzlay, dzei, dzr0, dzrcrit, dzeimin, dphilaypeak, dzrhomax;
  double azdzfactor, dzeifactor, dzeiweight, dzlayfactor;
  double avgei, eicrit, layminei, laywidth, slope1, const1, slope2, const2;
  double rho, rho0, rhomin, drho, rhopow;
  double sigsqrho, sigsqrhoconst, sigsqei, sigsqlay;
  double glay, costscale, ambiguityheight, ztoshort, ztoshortsq;
  double nshortcycle, midrangeambight;
  float **ei, **dpsi, **avgdpsi, *dzrcrittable, **dzrhomaxtable;
  costT **costs, **rowcost, **colcost;
  signed char noshadow, nolayover;


  /* get memory and set cost array pointers */
  costs=(costT **)Get2DRowColMem(nrow,ncol,sizeof(costT *),sizeof(costT));
  rowcost=(costT **)costs;
  colcost=(costT **)&costs[nrow-1];

  /* set up */
  rho0=(params->rhosconst1)/(params->ncorrlooks)+(params->rhosconst2);
  rhomin=params->rhominfactor*rho0;
  rhopow=2*(params->cstd1)+(params->cstd2)*log(params->ncorrlooks)
    +(params->cstd3)*(params->ncorrlooks);
  sigsqshortmin=params->sigsqshortmin;
  kperpdpsi=params->kperpdpsi;
  kpardpsi=params->kpardpsi;
  dr=params->dr;
  nearrange=params->nearrange+dr*tileparams->firstcol;
  drho=params->drho;
  nrho=(long )floor((1-rhomin)/drho)+1;
  nshortcycle=params->nshortcycle;
  layminei=params->layminei;
  laywidth=params->laywidth;
  azdzfactor=params->azdzfactor;
  dzeifactor=params->dzeifactor;
  dzeiweight=params->dzeiweight;
  dzeimin=params->dzeimin;
  dzlayfactor=params->dzlayfactor;
  sigsqei=params->sigsqei;
  lambda=params->lambda;
  noshadow=!(params->shadow);
  a=params->orbitradius;
  re=params->earthradius;

  /* despeckle the interferogram intensity */
  fprintf(sp2,"Despeckling intensity image\n");
  ei=NULL;
  Despeckle(pwr,&ei,nrow,ncol);
  Free2DArray((void **)pwr,nrow);

  /* remove large-area average intensity */
  fprintf(sp2,"Normalizing intensity\n");
  RemoveMean(ei,nrow,ncol,params->krowei,params->kcolei);

  /* dump normalized, despeckled intensity */
  if(strlen(outfiles->eifile)){
    Write2DArray((void **)ei,outfiles->eifile,nrow,ncol,sizeof(float));
  }

  /* compute some midswath parameters */
  slantrange=nearrange+ncol/2*dr;
  sinnomincangle=sin(acos((a*a-slantrange*slantrange-re*re)
                          /(2*slantrange*re)));
  lookangle=asin(re/a*sinnomincangle);

  /* see if we were passed bperp rather than baseline and baselineangle */
  if(params->bperp){
    if(params->bperp>0){
      params->baselineangle=lookangle;
    }else{
      params->baselineangle=lookangle+PI;
    }
    params->baseline=fabs(params->bperp);
  }

  /* the baseline should be halved if we are in single antenna transmit mode */
  if(params->transmitmode==SINGLEANTTRANSMIT){
    params->baseline/=2.0;
  }
  baseline=params->baseline;
  baselineangle=params->baselineangle;

  /* build lookup table for dzrcrit vs incidence angle */
  dzrcrittable=BuildDZRCritLookupTable(&nominc0,&dnominc,&nominctablesize,
                                       tileparams,params);

  /* build lookup table for dzrhomax vs incidence angle */
  dzrhomaxtable=BuildDZRhoMaxLookupTable(nominc0,dnominc,nominctablesize,
                                         rhomin,drho,nrho,params);

  /* set cost autoscale factor based on midswath parameters */
  bperp=baseline*cos(lookangle-baselineangle);
  midrangeambight=fabs(lambda*slantrange*sinnomincangle/(2*bperp));
  costscale=params->costscale*fabs(params->costscaleambight/midrangeambight);
  glay=-costscale*log(params->layconst);

  /* get memory for wrapped difference arrays */
  dpsi=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
  avgdpsi=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));

  /* build array of mean wrapped phase differences in range */
  /* simple average of phase differences is biased, but mean phase */
  /*   differences usually near zero, so don't bother with complex average */
  fprintf(sp2,"Building range cost arrays\n");
  CalcWrappedRangeDiffs(dpsi,avgdpsi,wrappedphase,kperpdpsi,kpardpsi,
                        nrow,ncol);

  /* build colcost array (range slopes) */
  /* loop over range */
  for(col=0;col<ncol-1;col++){

    /* compute range dependent parameters */
    slantrange=nearrange+col*dr;
    cosnomincangle=(a*a-slantrange*slantrange-re*re)/(2*slantrange*re);
    nomincangle=acos(cosnomincangle);
    sinnomincangle=sin(nomincangle);
    lookangle=asin(re/a*sinnomincangle);
    dzr0=-dr*cosnomincangle;
    bperp=baseline*cos(lookangle-baselineangle);
    ambiguityheight=-(lambda*slantrange*sinnomincangle)/(2*bperp);
    sigsqrhoconst=2.0*ambiguityheight*ambiguityheight/12.0;
    ztoshort=nshortcycle/ambiguityheight;
    ztoshortsq=ztoshort*ztoshort;
    sigsqlay=ambiguityheight*ambiguityheight*params->sigsqlayfactor;

    /* interpolate scattering model parameters */
    nomincind=(nomincangle-nominc0)/dnominc;
    dzrcrit=LinInterp1D(dzrcrittable,nomincind,nominctablesize);
    SolveEIModelParams(&slope1,&slope2,&const1,&const2,dzrcrit,dzr0,
                       sinnomincangle,cosnomincangle,params);
    eicrit=(dzrcrit-const1)/slope1;
    dphilaypeak=params->dzlaypeak/ambiguityheight;

    /* loop over azimuth */
    for(row=0;row<nrow;row++){

      /* see if we have a masked pixel */
      if(colweight[row][col]==0){

        /* masked pixel */
        MaskCost(&colcost[row][col]);

      }else{

        /* topography-mode costs */

        /* calculate variance due to decorrelation */
        /* factor of 2 in sigsqrhoconst for pdf convolution */
        rho=corr[row][col];
        if(rho<rhomin){
          rho=0;
        }
        sigsqrho=sigsqrhoconst*pow(1-rho,rhopow);

        /* calculate dz expected from EI if no layover */
        if(ei[row][col]>eicrit){
          dzei=(slope2*ei[row][col]+const2)*dzeifactor;
        }else{
          dzei=(slope1*ei[row][col]+const1)*dzeifactor;
        }
        if(noshadow && dzei<dzeimin){
          dzei=dzeimin;
        }

        /* calculate dz expected from EI if layover exists */
        dzlay=0;
        if(ei[row][col]>layminei){
          for(iei=0;iei<laywidth;iei++){
            if(ei[row][col+iei]>eicrit){
              dzlay+=slope2*ei[row][col+iei]+const2;
            }else{
              dzlay+=slope1*ei[row][col+iei]+const1;
            }
            if(col+iei>ncol-2){
              break;
            }
          }
        }
        if(dzlay){
          dzlay=(dzlay+iei*(-2.0*dzr0))*dzlayfactor;
        }

        /* set maximum dz based on unbiased correlation and layover max */
        if(rho>0){
          dzrhomax=LinInterp2D(dzrhomaxtable,nomincind,(rho-rhomin)/drho,
                               nominctablesize,nrho);
          if(dzrhomax<dzlay){
            dzlay=dzrhomax;
          }
        }

        /* set cost parameters in terms of flow, represented as shorts */
        nolayover=TRUE;
        if(dzlay){
          if(rho>0){
            colcost[row][col].offset=nshortcycle*
              (dpsi[row][col]-0.5*(avgdpsi[row][col]+dphilaypeak));
          }else{
            colcost[row][col].offset=nshortcycle*
              (dpsi[row][col]-0.25*avgdpsi[row][col]-0.75*dphilaypeak);
          }
          colcost[row][col].sigsq=(sigsqrho+sigsqei+sigsqlay)*ztoshortsq
            /(costscale*colweight[row][col]);
          if(colcost[row][col].sigsq<sigsqshortmin){
            colcost[row][col].sigsq=sigsqshortmin;
          }
          colcost[row][col].dzmax=dzlay*ztoshort;
          colcost[row][col].laycost=colweight[row][col]*glay;
          if(labs(colcost[row][col].dzmax)
             >floor(sqrt(colcost[row][col].laycost*colcost[row][col].sigsq))){
            nolayover=FALSE;
          }
        }
        if(nolayover){
          colcost[row][col].sigsq=(sigsqrho+sigsqei)*ztoshortsq
            /(costscale*colweight[row][col]);
          if(colcost[row][col].sigsq<sigsqshortmin){
            colcost[row][col].sigsq=sigsqshortmin;
          }
          if(rho>0){
            colcost[row][col].offset=ztoshort*
              (ambiguityheight*(dpsi[row][col]-0.5*avgdpsi[row][col])
               -0.5*dzeiweight*dzei);
          }else{
            colcost[row][col].offset=ztoshort*
              (ambiguityheight*(dpsi[row][col]-0.25*avgdpsi[row][col])
               -0.75*dzeiweight*dzei);
          }
          colcost[row][col].laycost=NOCOSTSHELF;
          colcost[row][col].dzmax=LARGESHORT;
        }

        /* shift PDF to account for flattening by coarse unwrapped estimate */
        if(unwrappedest!=NULL){
          colcost[row][col].offset+=(nshortcycle/TWOPI*
                                     (unwrappedest[row][col+1]
                                      -unwrappedest[row][col]));
        }

      }
    }
  } /* end of range gradient cost calculation */

  /* reset layover constant for row (azimuth) costs */
  glay+=(-costscale*log(azdzfactor));

  /* build array of mean wrapped phase differences in azimuth */
  /* biased, but not much, so don't bother with complex averaging */
  fprintf(sp2,"Building azimuth cost arrays\n");
  CalcWrappedAzDiffs(dpsi,avgdpsi,wrappedphase,kperpdpsi,kpardpsi,
                     nrow,ncol);

  /* build rowcost array */
  /* for the rowcost array, there is symmetry between positive and */
  /*   negative flows, so we average ei[][] and corr[][] values in azimuth */
  /* loop over range */
  for(col=0;col<ncol;col++){

    /* compute range dependent parameters */
    slantrange=nearrange+col*dr;
    cosnomincangle=(a*a-slantrange*slantrange-re*re)/(2*slantrange*re);
    nomincangle=acos(cosnomincangle);
    sinnomincangle=sin(nomincangle);
    lookangle=asin(re/a*sinnomincangle);
    dzr0=-dr*cosnomincangle;
    bperp=baseline*cos(lookangle-baselineangle);
    ambiguityheight=-lambda*slantrange*sinnomincangle/(2*bperp);
    sigsqrhoconst=2.0*ambiguityheight*ambiguityheight/12.0;
    ztoshort=nshortcycle/ambiguityheight;
    ztoshortsq=ztoshort*ztoshort;
    sigsqlay=ambiguityheight*ambiguityheight*params->sigsqlayfactor;

    /* interpolate scattering model parameters */
    nomincind=(nomincangle-nominc0)/dnominc;
    dzrcrit=LinInterp1D(dzrcrittable,nomincind,nominctablesize);
    SolveEIModelParams(&slope1,&slope2,&const1,&const2,dzrcrit,dzr0,
                       sinnomincangle,cosnomincangle,params);
    eicrit=(dzrcrit-const1)/slope1;
    dphilaypeak=params->dzlaypeak/ambiguityheight;

    /* loop over azimuth */
    for(row=0;row<nrow-1;row++){

      /* see if we have a masked pixel */
      if(rowweight[row][col]==0){

        /* masked pixel */
        MaskCost(&rowcost[row][col]);

      }else{

        /* topography-mode costs */

        /* variance due to decorrelation */
        /* get correlation and clip small values because of estimator bias */
        rho=(corr[row][col]+corr[row+1][col])/2.0;
        if(rho<rhomin){
          rho=0;
        }
        sigsqrho=sigsqrhoconst*pow(1-rho,rhopow);

        /* if no layover, the expected dz for azimuth will always be 0 */
        dzei=0;

        /* calculate dz expected from EI if layover exists */
        dzlay=0;
        avgei=(ei[row][col]+ei[row+1][col])/2.0;
        if(avgei>layminei){
          for(iei=0;iei<laywidth;iei++){
            avgei=(ei[row][col+iei]+ei[row+1][col+iei])/2.0;
            if(avgei>eicrit){
              dzlay+=slope2*avgei+const2;
            }else{
              dzlay+=slope1*avgei+const1;
            }
            if(col+iei>ncol-2){
              break;
            }
          }
        }
        if(dzlay){
          dzlay=(dzlay+iei*(-2.0*dzr0))*dzlayfactor;
        }

        /* set maximum dz based on correlation max and layover max */
        if(rho>0){
          dzrhomax=LinInterp2D(dzrhomaxtable,nomincind,(rho-rhomin)/drho,
                               nominctablesize,nrho);
          if(dzrhomax<dzlay){
            dzlay=dzrhomax;
          }
        }

        /* set cost parameters in terms of flow, represented as shorts */
        if(rho>0){
          rowcost[row][col].offset=nshortcycle*
            (dpsi[row][col]-avgdpsi[row][col]);
        }else{
          rowcost[row][col].offset=nshortcycle*
            (dpsi[row][col]-0.5*avgdpsi[row][col]);
        }
        nolayover=TRUE;
        if(dzlay){
          rowcost[row][col].sigsq=(sigsqrho+sigsqei+sigsqlay)*ztoshortsq
            /(costscale*rowweight[row][col]);
          if(rowcost[row][col].sigsq<sigsqshortmin){
            rowcost[row][col].sigsq=sigsqshortmin;
          }
          rowcost[row][col].dzmax=fabs(dzlay*ztoshort);
          rowcost[row][col].laycost=rowweight[row][col]*glay;
          if(labs(rowcost[row][col].dzmax)
             >floor(sqrt(rowcost[row][col].laycost*rowcost[row][col].sigsq))){
            nolayover=FALSE;
          }
        }
        if(nolayover){
          rowcost[row][col].sigsq=(sigsqrho+sigsqei)*ztoshortsq
            /(costscale*rowweight[row][col]);
          if(rowcost[row][col].sigsq<sigsqshortmin){
            rowcost[row][col].sigsq=sigsqshortmin;
          }
          rowcost[row][col].laycost=NOCOSTSHELF;
          rowcost[row][col].dzmax=LARGESHORT;
        }

        /* shift PDF to account for flattening by coarse unwrapped estimate */
        if(unwrappedest!=NULL){
          rowcost[row][col].offset+=(nshortcycle/TWOPI*
                                     (unwrappedest[row+1][col]
                                      -unwrappedest[row][col]));
        }

      }
    }
  }  /* end of azimuth gradient cost calculation */

  /* free temporary arrays */
  Free2DArray((void **)corr,nrow);
  Free2DArray((void **)dpsi,nrow);
  Free2DArray((void **)avgdpsi,nrow);
  Free2DArray((void **)ei,nrow);
  free(dzrcrittable);
  Free2DArray((void **)dzrhomaxtable,nominctablesize);

  /* return pointer to costs arrays */
  return((void **)costs);

}


/* function: BuildStatCostsDefo()
 * ------------------------------
 * Builds statistical cost arrays for deformation mode.
 */
static
void **BuildStatCostsDefo(float **wrappedphase, float **mag,
                          float **unwrappedest, float **corr,
                          short **rowweight, short **colweight,
                          long nrow, long ncol, tileparamT *tileparams,
                          outfileT *outfiles, paramT *params){

  long row, col;
  long kperpdpsi, kpardpsi, sigsqshortmin, defomax;
  double rho, rho0, rhopow;
  double defocorrthresh, sigsqcorr, sigsqrho, sigsqrhoconst;
  double glay, costscale;
  double nshortcycle, nshortcyclesq;
  float **dpsi, **avgdpsi;
  costT **costs, **rowcost, **colcost;

  /* get memory and set cost array pointers */
  costs=(costT **)Get2DRowColMem(nrow,ncol,sizeof(costT *),sizeof(costT));
  rowcost=(costT **)costs;
  colcost=(costT **)&costs[nrow-1];

  /* set up */
  rho0=(params->rhosconst1)/(params->ncorrlooks)+(params->rhosconst2);
  defocorrthresh=params->defothreshfactor*rho0;
  rhopow=2*(params->cstd1)+(params->cstd2)*log(params->ncorrlooks)
    +(params->cstd3)*(params->ncorrlooks);
  sigsqrhoconst=2.0/12.0;
  sigsqcorr=params->sigsqcorr;
  sigsqshortmin=params->sigsqshortmin;
  kperpdpsi=params->kperpdpsi;
  kpardpsi=params->kpardpsi;
  costscale=params->costscale;
  nshortcycle=params->nshortcycle;
  nshortcyclesq=nshortcycle*nshortcycle;
  glay=-costscale*log(params->defolayconst);
  defomax=(long )ceil(params->defomax*nshortcycle);

  /* get memory for wrapped difference arrays */
  dpsi=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
  avgdpsi=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));

  /* build array of mean wrapped phase differences in range */
  /* simple average of phase differences is biased, but mean phase */
  /*   differences usually near zero, so don't bother with complex average */
  fprintf(sp2,"Building range cost arrays\n");
  CalcWrappedRangeDiffs(dpsi,avgdpsi,wrappedphase,kperpdpsi,kpardpsi,
                        nrow,ncol);

  /* build colcost array (range slopes) */
  for(col=0;col<ncol-1;col++){
    for(row=0;row<nrow;row++){

      /* see if we have a masked pixel */
      if(colweight[row][col]==0){

        /* masked pixel */
        MaskCost(&colcost[row][col]);

      }else{

        /* deformation-mode costs */

        /* calculate variance due to decorrelation */
        /* need symmetry for range if deformation */
        rho=(corr[row][col]+corr[row][col+1])/2.0;
        if(rho<defocorrthresh){
          rho=0;
        }
        sigsqrho=(sigsqrhoconst*pow(1-rho,rhopow)+sigsqcorr)*nshortcyclesq;

        /* set cost paramaters in terms of flow, represented as shorts */
        if(rho>0){
          colcost[row][col].offset=nshortcycle*
            (dpsi[row][col]-avgdpsi[row][col]);
        }else{
          colcost[row][col].offset=nshortcycle*
            (dpsi[row][col]-0.5*avgdpsi[row][col]);
        }
        colcost[row][col].sigsq=sigsqrho/(costscale*colweight[row][col]);
        if(colcost[row][col].sigsq<sigsqshortmin){
          colcost[row][col].sigsq=sigsqshortmin;
        }
        if(rho<defocorrthresh){
          colcost[row][col].dzmax=defomax;
          colcost[row][col].laycost=colweight[row][col]*glay;
          if(colcost[row][col].dzmax<floor(sqrt(colcost[row][col].laycost
                                                *colcost[row][col].sigsq))){
            colcost[row][col].laycost=NOCOSTSHELF;
            colcost[row][col].dzmax=LARGESHORT;
          }
        }else{
          colcost[row][col].laycost=NOCOSTSHELF;
          colcost[row][col].dzmax=LARGESHORT;
        }
      }

      /* shift PDF to account for flattening by coarse unwrapped estimate */
      if(unwrappedest!=NULL){
        colcost[row][col].offset+=(nshortcycle/TWOPI*
                                   (unwrappedest[row][col+1]
                                    -unwrappedest[row][col]));
      }
    }
  }  /* end of range gradient cost calculation */

  /* build array of mean wrapped phase differences in azimuth */
  /* biased, but not much, so don't bother with complex averaging */
  fprintf(sp2,"Building azimuth cost arrays\n");
  CalcWrappedAzDiffs(dpsi,avgdpsi,wrappedphase,kperpdpsi,kpardpsi,
                     nrow,ncol);

  /* build rowcost array */
  for(col=0;col<ncol;col++){
    for(row=0;row<nrow-1;row++){

      /* see if we have a masked pixel */
      if(rowweight[row][col]==0){

        /* masked pixel */
        MaskCost(&rowcost[row][col]);

      }else{

        /* deformation-mode costs */

        /* variance due to decorrelation */
        /* get correlation and clip small values because of estimator bias */
        rho=(corr[row][col]+corr[row+1][col])/2.0;
        if(rho<defocorrthresh){
          rho=0;
        }
        sigsqrho=(sigsqrhoconst*pow(1-rho,rhopow)+sigsqcorr)*nshortcyclesq;

        /* set cost paramaters in terms of flow, represented as shorts */
        if(rho>0){
          rowcost[row][col].offset=nshortcycle*
            (dpsi[row][col]-avgdpsi[row][col]);
        }else{
          rowcost[row][col].offset=nshortcycle*
            (dpsi[row][col]-0.5*avgdpsi[row][col]);
        }
        rowcost[row][col].sigsq=sigsqrho/(costscale*rowweight[row][col]);
        if(rowcost[row][col].sigsq<sigsqshortmin){
          rowcost[row][col].sigsq=sigsqshortmin;
        }
        if(rho<defocorrthresh){
          rowcost[row][col].dzmax=defomax;
          rowcost[row][col].laycost=rowweight[row][col]*glay;
          if(rowcost[row][col].dzmax<floor(sqrt(rowcost[row][col].laycost
                                                *rowcost[row][col].sigsq))){
            rowcost[row][col].laycost=NOCOSTSHELF;
            rowcost[row][col].dzmax=LARGESHORT;
          }
        }else{
          rowcost[row][col].laycost=NOCOSTSHELF;
          rowcost[row][col].dzmax=LARGESHORT;
        }
      }

      /* shift PDF to account for flattening by coarse unwrapped estimate */
      if(unwrappedest!=NULL){
        rowcost[row][col].offset+=(nshortcycle/TWOPI*
                                   (unwrappedest[row+1][col]
                                    -unwrappedest[row][col]));
      }
    }
  } /* end of azimuth cost calculation */

  /* free temporary arrays */
  Free2DArray((void **)corr,nrow);
  Free2DArray((void **)dpsi,nrow);
  Free2DArray((void **)avgdpsi,nrow);

  /* return pointer to costs arrays */
  return((void **)costs);

}


/* function: BuildStatCostsSmooth()
 * --------------------------------
 * Builds statistical cost arrays for smooth-solution mode.
 */
static
void **BuildStatCostsSmooth(float **wrappedphase, float **mag,
                            float **unwrappedest, float **corr,
                            short **rowweight, short **colweight,
                            long nrow, long ncol, tileparamT *tileparams,
                            outfileT *outfiles, paramT *params){

  long row, col;
  long kperpdpsi, kpardpsi, sigsqshortmin;
  double rho, rho0, rhopow;
  double defocorrthresh, sigsqcorr, sigsqrho, sigsqrhoconst;
  double costscale;
  double nshortcycle, nshortcyclesq;
  float **dpsi, **avgdpsi;
  smoothcostT **costs, **rowcost, **colcost;

  /* get memory and set cost array pointers */
  costs=(smoothcostT **)Get2DRowColMem(nrow,ncol,sizeof(smoothcostT *),
                                       sizeof(smoothcostT));
  rowcost=(smoothcostT **)costs;
  colcost=(smoothcostT **)&costs[nrow-1];

  /* set up */
  rho0=(params->rhosconst1)/(params->ncorrlooks)+(params->rhosconst2);
  defocorrthresh=params->defothreshfactor*rho0;
  rhopow=2*(params->cstd1)+(params->cstd2)*log(params->ncorrlooks)
    +(params->cstd3)*(params->ncorrlooks);
  sigsqrhoconst=2.0/12.0;
  sigsqcorr=params->sigsqcorr;
  sigsqshortmin=params->sigsqshortmin;
  kperpdpsi=params->kperpdpsi;
  kpardpsi=params->kpardpsi;
  costscale=params->costscale;
  nshortcycle=params->nshortcycle;
  nshortcyclesq=nshortcycle*nshortcycle;

  /* get memory for wrapped difference arrays */
  dpsi=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
  avgdpsi=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));

  /* build array of mean wrapped phase differences in range */
  /* simple average of phase differences is biased, but mean phase */
  /*   differences usually near zero, so don't bother with complex average */
  fprintf(sp2,"Building range cost arrays\n");
  CalcWrappedRangeDiffs(dpsi,avgdpsi,wrappedphase,kperpdpsi,kpardpsi,
                        nrow,ncol);

  /* build colcost array (range slopes) */
  for(col=0;col<ncol-1;col++){
    for(row=0;row<nrow;row++){

      /* see if we have a masked pixel */
      if(colweight[row][col]==0){

        /* masked pixel */
        MaskSmoothCost(&colcost[row][col]);

      }else{

        /* smooth-mode costs */

        /* calculate variance due to decorrelation */
        /* need symmetry for range if deformation */
        rho=(corr[row][col]+corr[row][col+1])/2.0;
        if(rho<defocorrthresh){
          rho=0;
        }
        sigsqrho=(sigsqrhoconst*pow(1-rho,rhopow)+sigsqcorr)*nshortcyclesq;

        /* set cost paramaters in terms of flow, represented as shorts */
        if(rho>0){
          colcost[row][col].offset=nshortcycle*
            (dpsi[row][col]-avgdpsi[row][col]);
        }else{
          colcost[row][col].offset=nshortcycle*
            (dpsi[row][col]-0.5*avgdpsi[row][col]);
        }
        colcost[row][col].sigsq=sigsqrho/(costscale*colweight[row][col]);
        if(colcost[row][col].sigsq<sigsqshortmin){
          colcost[row][col].sigsq=sigsqshortmin;
        }
      }

      /* shift PDF to account for flattening by coarse unwrapped estimate */
      if(unwrappedest!=NULL){
        colcost[row][col].offset+=(nshortcycle/TWOPI*
                                   (unwrappedest[row][col+1]
                                    -unwrappedest[row][col]));
      }
    }
  }  /* end of range gradient cost calculation */

  /* build array of mean wrapped phase differences in azimuth */
  /* biased, but not much, so don't bother with complex averaging */
  fprintf(sp2,"Building azimuth cost arrays\n");
  CalcWrappedAzDiffs(dpsi,avgdpsi,wrappedphase,kperpdpsi,kpardpsi,
                     nrow,ncol);

  /* build rowcost array */
  for(col=0;col<ncol;col++){
    for(row=0;row<nrow-1;row++){

      /* see if we have a masked pixel */
      if(rowweight[row][col]==0){

        /* masked pixel */
        MaskSmoothCost(&rowcost[row][col]);

      }else{

        /* smooth-mode costs */

        /* variance due to decorrelation */
        /* get correlation and clip small values because of estimator bias */
        rho=(corr[row][col]+corr[row+1][col])/2.0;
        if(rho<defocorrthresh){
          rho=0;
        }
        sigsqrho=(sigsqrhoconst*pow(1-rho,rhopow)+sigsqcorr)*nshortcyclesq;

        /* set cost paramaters in terms of flow, represented as shorts */
        if(rho>0){
          rowcost[row][col].offset=nshortcycle*
            (dpsi[row][col]-avgdpsi[row][col]);
        }else{
          rowcost[row][col].offset=nshortcycle*
            (dpsi[row][col]-0.5*avgdpsi[row][col]);
        }
        rowcost[row][col].sigsq=sigsqrho/(costscale*rowweight[row][col]);
        if(rowcost[row][col].sigsq<sigsqshortmin){
          rowcost[row][col].sigsq=sigsqshortmin;
        }
      }

      /* shift PDF to account for flattening by coarse unwrapped estimate */
      if(unwrappedest!=NULL){
        rowcost[row][col].offset+=(nshortcycle/TWOPI*
                                   (unwrappedest[row+1][col]
                                    -unwrappedest[row][col]));
      }
    }
  } /* end of azimuth cost calculation */

  /* free temporary arrays */
  Free2DArray((void **)corr,nrow);
  Free2DArray((void **)dpsi,nrow);
  Free2DArray((void **)avgdpsi,nrow);

  /* return pointer to costs arrays */
  return((void **)costs);

}


/* function: MaskCost()
 * --------------------
 * Set values of costT structure pointed to by input pointer to give zero
 * cost, as for arcs next to masked pixels.
 */
static
void MaskCost(costT *costptr){

  /* set to special values */
  costptr->laycost=0;
  costptr->offset=LARGESHORT/2;
  costptr->dzmax=LARGESHORT;
  costptr->sigsq=LARGESHORT;

}


/* function: MaskSmoothCost()
 * --------------------------
 * Set values of smoothcostT structure pointed to by input pointer to give zero
 * cost, as for arcs next to masked pixels.
 */
static
void MaskSmoothCost(smoothcostT *smoothcostptr){

  /* set to special values */
  smoothcostptr->offset=LARGESHORT/2;
  smoothcostptr->sigsq=LARGESHORT;

}


/* function: MaskPrespecifiedArcCosts()
 * ------------------------------------
 * Loop over grid arcs and set costs to null if corresponding weights
 * are null.
 */
static
int MaskPrespecifiedArcCosts(void **costsptr, short **weights,
                             long nrow, long ncol, paramT *params){

  long row, col, maxcol;
  costT **costs;
  smoothcostT **smoothcosts;


  /* set up pointers */
  costs=NULL;
  smoothcosts=NULL;
  if(params->costmode==TOPO || params->costmode==DEFO){
    costs=(costT **)costsptr;
  }else if(params->costmode==SMOOTH){
    smoothcosts=(smoothcostT **)costsptr;
  }else{
    fprintf(sp0,"illegal cost mode in MaskPrespecifiedArcCosts()\n");
    exit(ABNORMAL_EXIT);
  }

  /* loop over all arcs */
  for(row=0;row<2*nrow-1;row++){
    if(row<nrow-1){
      maxcol=ncol;
    }else{
      maxcol=ncol-1;
    }
    for(col=0;col<maxcol;col++){
      if(weights[row][col]==0){
        if(costs!=NULL){
          MaskCost(&costs[row][col]);
        }
        if(smoothcosts!=NULL){
          MaskSmoothCost(&smoothcosts[row][col]);
        }
      }
    }
  }

  /* done */
  return(0);

}


/* function: GetIntensityAndCorrelation()
 * --------------------------------------
 * Reads amplitude and correlation info from files if specified.  If ampfile
 * not given, uses interferogram magnitude.  If correlation file not given,
 * generates correlatin info from interferogram and amplitude.
 */
static
int GetIntensityAndCorrelation(float **mag, float **wrappedphase,
                               float ***pwrptr, float ***corrptr,
                               infileT *infiles, long linelen, long nlines,
                               long nrow, long ncol, outfileT *outfiles,
                               paramT *params, tileparamT *tileparams){

  long row, col, krowcorr, kcolcorr, iclipped;
  float **pwr, **corr;
  float **realcomp, **imagcomp;
  float **padreal, **padimag, **avgreal, **avgimag;
  float **pwr1, **pwr2, **padpwr1, **padpwr2, **avgpwr1, **avgpwr2;
  double rho0, rhomin, biaseddefaultcorr;

  /* initialize */
  pwr=NULL;
  corr=NULL;
  pwr1=NULL;
  pwr2=NULL;

  /* read intensity, if specified */
  if(strlen(infiles->ampfile)){
    ReadIntensity(&pwr,&pwr1,&pwr2,infiles,linelen,nlines,params,tileparams);
  }else{
    if(params->costmode==TOPO){
      fprintf(sp1,"No brightness file specified.  ");
      fprintf(sp1,"Using interferogram magnitude as intensity\n");
    }
    pwr=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
    for(row=0;row<nrow;row++){
      for(col=0;col<ncol;col++){
        pwr[row][col]=mag[row][col];
      }
    }
  }

  /* read corrfile, if specified */
  if(strlen(infiles->corrfile)){
    ReadCorrelation(&corr,infiles,linelen,nlines,tileparams);
  }else if(pwr1!=NULL && pwr2!=NULL && params->havemagnitude){

    /* generate the correlation info from the interferogram and amplitude */
    fprintf(sp1,"Generating correlation from interferogram and intensity\n");

    /* get the correct number of looks, and make sure its odd */
    krowcorr=1+2*floor(params->ncorrlooksaz/(double )params->nlooksaz/2);
    kcolcorr=1+2*floor(params->ncorrlooksrange/(double )params->nlooksrange/2);

    /* calculate equivalent number of independent looks */
    params->ncorrlooks=(kcolcorr*(params->dr/params->rangeres))
      *(krowcorr*(params->da/params->azres))*params->nlooksother;
    fprintf(sp1,"   (%.1f equivalent independent looks)\n",
            params->ncorrlooks);

    /* get real and imaginary parts of interferogram */
    realcomp=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
    imagcomp=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
    for(row=0;row<nrow;row++){
      for(col=0;col<ncol;col++){
        realcomp[row][col]=mag[row][col]*cos(wrappedphase[row][col]);
        imagcomp[row][col]=mag[row][col]*sin(wrappedphase[row][col]);
      }
    }

    /* do complex spatial averaging on the interferogram */
    padreal=MirrorPad(realcomp,nrow,ncol,(krowcorr-1)/2,(kcolcorr-1)/2);
    padimag=MirrorPad(imagcomp,nrow,ncol,(krowcorr-1)/2,(kcolcorr-1)/2);
    if(padreal==realcomp || padimag==imagcomp){
      fflush(NULL);
      fprintf(sp0,"Correlation averaging box too large for input array size\n"
              "Abort\n");
      exit(ABNORMAL_EXIT);
    }
    avgreal=realcomp;
    BoxCarAvg(avgreal,padreal,nrow,ncol,krowcorr,kcolcorr);
    avgimag=imagcomp;
    BoxCarAvg(avgimag,padimag,nrow,ncol,krowcorr,kcolcorr);
    Free2DArray((void **)padreal,nrow);
    Free2DArray((void **)padimag,nrow);

    /* spatially average individual SAR power images */
    padpwr1=MirrorPad(pwr1,nrow,ncol,(krowcorr-1)/2,(kcolcorr-1)/2);
    avgpwr1=pwr1;
    BoxCarAvg(avgpwr1,padpwr1,nrow,ncol,krowcorr,kcolcorr);
    padpwr2=MirrorPad(pwr2,nrow,ncol,(krowcorr-1)/2,(kcolcorr-1)/2);
    avgpwr2=pwr2;
    BoxCarAvg(avgpwr2,padpwr2,nrow,ncol,krowcorr,kcolcorr);
    Free2DArray((void **)padpwr1,nrow);
    Free2DArray((void **)padpwr2,nrow);

    /* build correlation data */
    corr=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
    for(row=0;row<nrow;row++){
      for(col=0;col<ncol;col++){
        if(avgpwr1[row][col]<=0 || avgpwr2[row][col]<=0){
          corr[row][col]=0.0;
        }else{
          corr[row][col]=sqrt((avgreal[row][col]*avgreal[row][col]
                               +avgimag[row][col]*avgimag[row][col])
                              /(avgpwr1[row][col]*avgpwr2[row][col]));
        }
      }
    }

    /* free temporary arrays */
    Free2DArray((void **)avgreal,nrow);
    Free2DArray((void **)avgimag,nrow);
    Free2DArray((void **)avgpwr1,nrow);
    Free2DArray((void **)avgpwr2,nrow);
    pwr1=NULL;
    pwr2=NULL;

  }else{

    /* no file specified: set corr to default value */
    /* find biased default correlation using */
    /* inverse of unbias method used by BuildCostArrays() */
    corr=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
    fprintf(sp1,"No correlation file specified.  Assuming correlation = %g\n",
           params->defaultcorr);
    rho0=(params->rhosconst1)/(params->ncorrlooks)+(params->rhosconst2);
    rhomin=params->rhominfactor*rho0;
    if(params->defaultcorr>rhomin){
      biaseddefaultcorr=params->defaultcorr;
    }else{
      biaseddefaultcorr=0.0;
    }
    for(row=0;row<nrow;row++){
      for(col=0;col<ncol;col++){
        corr[row][col]=biaseddefaultcorr;
      }
    }
  }

  /* dump correlation data if necessary */
  if(strlen(outfiles->rawcorrdumpfile)){
    Write2DArray((void **)corr,outfiles->rawcorrdumpfile,
                 nrow,ncol,sizeof(float));
  }

  /* check correlation data validity */
  iclipped=0;
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      if(!IsFinite(corr[row][col])){
        fflush(NULL);
        fprintf(sp0,"NaN or infinity found in correlation data\nAbort\n");
        exit(ABNORMAL_EXIT);
      }else if(corr[row][col]>1.0){
        if(corr[row][col]>1.001){
          iclipped++;               /* don't warn for minor numerical errors */
        }
        corr[row][col]=1.0;
      }else if(corr[row][col]<0.0){
        if(corr[row][col]<-0.001){
          iclipped++;               /* don't warn for minor numerical errors */
        }
        corr[row][col]=0.0;
      }
    }
  }
  if(iclipped){
    fflush(NULL);
    fprintf(sp0,"WARNING: %ld illegal correlation values clipped to [0,1]\n",
            iclipped);
  }

  /* dump correlation data if necessary */
  if(strlen(outfiles->corrdumpfile)){
    Write2DArray((void **)corr,outfiles->corrdumpfile,
                 nrow,ncol,sizeof(float));
  }

  /* free memory and set output pointers */
  if(pwr1!=NULL){
    Free2DArray((void **)pwr1,nrow);
  }
  if(pwr2!=NULL){
    Free2DArray((void **)pwr2,nrow);
  }
  if(params->costmode==DEFO && pwr!=NULL){
    Free2DArray((void **)pwr,nrow);
    pwr=NULL;
  }
  *pwrptr=pwr;
  *corrptr=corr;

  /* done */
  return(0);

}


/* function: RemoveMean()
 * -------------------------
 * Divides intensity by average over sliding window.
 */
static
int RemoveMean(float **ei, long nrow, long ncol,
               long krowei, long kcolei){

  float **avgei, **padei;
  long row, col;

  /* make sure krowei, kcolei are odd */
  if(!(krowei % 2)){
    krowei++;
  }
  if(!(kcolei % 2)){
    kcolei++;
  }

  /* get memory */
  avgei=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));

  /* pad ei in new array */
  padei=MirrorPad(ei,nrow,ncol,(krowei-1)/2,(kcolei-1)/2);
  if(padei==ei){
    fflush(NULL);
    fprintf(sp0,"Intensity-normalization averaging box too large "
            "for input array size\nAbort\n");
    exit(ABNORMAL_EXIT);
  }

  /* calculate average ei by using sliding window */
  BoxCarAvg(avgei,padei,nrow,ncol,krowei,kcolei);

  /* divide ei by avgei */
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      if(avgei){
        ei[row][col]/=(avgei[row][col]);
      }
    }
  }

  /* free memory */
  Free2DArray((void **)padei,nrow+krowei-1);
  Free2DArray((void **)avgei,nrow);
  return(0);

}


/* function: BuildDZRCritLookupTable()
 * -----------------------------------
 * Builds a 1-D lookup table of dzrcrit values indexed by incidence angle
 * (in rad).
 */
static
float *BuildDZRCritLookupTable(double *nominc0ptr, double *dnomincptr,
                               long *tablesizeptr, tileparamT *tileparams,
                               paramT *params){

  long tablesize, k;
  double nominc, nominc0, nomincmax, dnominc;
  double a, re, slantrange;
  float *dzrcrittable;

  /* compute nominal spherical earth incidence angle for near and far range */
  a=params->orbitradius;
  re=params->earthradius;
  slantrange=params->nearrange+params->dr*tileparams->firstcol;
  nominc0=acos((a*a-slantrange*slantrange-re*re)/(2*slantrange*re));
  slantrange+=params->dr*tileparams->ncol;
  nomincmax=acos((a*a-slantrange*slantrange-re*re)/(2*slantrange*re));
  if(!IsFinite(nominc0) || !IsFinite(nomincmax)){
    fflush(NULL);
    fprintf(sp0,"Geometry error detected.  "
            "Check altitude, near range, and earth radius parameters\n"
            "Abort\n");
    exit(ABNORMAL_EXIT);
  }

  /* build lookup table */
  dnominc=params->dnomincangle;
  tablesize=(long )floor((nomincmax-nominc0)/dnominc)+1;
  dzrcrittable=MAlloc(tablesize*sizeof(float));
  nominc=nominc0;
  for(k=0;k<tablesize;k++){
    dzrcrittable[k]=(float )SolveDZRCrit(sin(nominc),cos(nominc),params,
                                 params->threshold);
    nominc+=dnominc;
    if(nominc>PI/2.0){
      nominc-=dnominc;
    }
  }

  /* set return variables */
  (*nominc0ptr)=nominc;
  (*dnomincptr)=dnominc;
  (*tablesizeptr)=tablesize;
  return(dzrcrittable);

}


/* function: SolveDZRCrit()
 * ------------------------
 * Numerically solve for the transition point of the linearized scattering
 * model.
 */
static
double SolveDZRCrit(double sinnomincangle, double cosnomincangle,
                    paramT *params, double threshold){

  double residual, thetai, kds, n, dr, dzr, dx;
  double costhetai, cos2thetai, step;
  double dzrcritfactor, diffuse, specular;
  long i;

  /* get parameters */
  kds=params->kds;
  n=params->specularexp;
  dr=params->dr;
  dzrcritfactor=params->dzrcritfactor;

  /* solve for critical incidence angle */
  thetai=PI/4;
  step=PI/4-1e-6;
  i=0;
  while(TRUE){
    if((cos2thetai=cos(2*thetai))<0){
      cos2thetai=0;
    }
    diffuse=dzrcritfactor*kds*cos(thetai);
    specular=pow(cos2thetai,n);
    if(fabs(residual=diffuse-specular)<threshold*diffuse){
      break;
    }
    if(residual<0){
      thetai+=step;
    }else{
      thetai-=step;
    }
    step/=2.0;
    if(++i>MAXITERATION){
      fflush(NULL);
      fprintf(sp0,"Couldn't find critical incidence angle ");
      fprintf(sp0,"(check scattering parameters)\nAbort\n");
      exit(ABNORMAL_EXIT);
    }
  }

  /* solve for critical height change */
  costhetai=cos(thetai);
  dzr=params->initdzr;
  step=dzr+dr*cosnomincangle-1e-2;
  i=0;
  while(TRUE){
    dx=(dr+dzr*cosnomincangle)/sinnomincangle;
    if(fabs(residual=costhetai-(dzr*sinnomincangle+dx*cosnomincangle)
            /sqrt(dzr*dzr+dx*dx))
       <threshold*costhetai){
      return(dzr);
    }
    if(residual<0){
      dzr-=step;
    }else{
      dzr+=step;
    }
    step/=2.0;
    if(++i>MAXITERATION){
      fflush(NULL);
      fprintf(sp0,"Couldn't find critical slope ");
      fprintf(sp0,"(check geometry parameters)\nAbort\n");
      exit(ABNORMAL_EXIT);
    }
  }
}


/* function: SolveEIModelParams()
 * ------------------------------
 * Calculates parameters for linearized model of EI vs. range slope
 * relationship.
 */
int SolveEIModelParams(double *slope1ptr, double *slope2ptr,
                       double *const1ptr, double *const2ptr,
                       double dzrcrit, double dzr0, double sinnomincangle,
                       double cosnomincangle, paramT *params){

  double slope1, slope2, const1, const2, sloperatio;
  double dzr3, ei3;

  /* set up */
  sloperatio=params->kds*params->sloperatiofactor;

  /* find normalized intensity at 15(dzrcrit-dzr0)+dzr0 */
  dzr3=15.0*(dzrcrit-dzr0)+dzr0;
  ei3=EIofDZR(dzr3,sinnomincangle,cosnomincangle,params)
    /EIofDZR(0,sinnomincangle,cosnomincangle,params);

  /* calculate parameters */
  const1=dzr0;
  slope2=(sloperatio*(dzrcrit-const1)-dzrcrit+dzr3)/ei3;
  slope1=slope2/sloperatio;
  const2=dzr3-slope2*ei3;

  /* set return values */
  *slope1ptr=slope1;
  *slope2ptr=slope2;
  *const1ptr=const1;
  *const2ptr=const2;
  return(0);

}


/* function: EIofDZR()
 * -------------------
 * Calculates expected value of intensity with arbitrary units for given
 * parameters.  Assumes azimuth slope is zero.
 */
static
double EIofDZR(double dzr, double sinnomincangle, double cosnomincangle,
               paramT *params){

  double dr, da, dx, kds, n, dzr0, projarea;
  double costhetai, cos2thetai, sigma0;

  dr=params->dr;
  da=params->da;
  dx=dr/sinnomincangle+dzr*cosnomincangle/sinnomincangle;
  kds=params->kds;
  n=params->specularexp;
  dzr0=-dr*cosnomincangle;
  projarea=da*fabs((dzr-dzr0)/sinnomincangle);
  costhetai=projarea/sqrt(dzr*dzr*da*da + da*da*dx*dx);
  if(costhetai>SQRTHALF){
    cos2thetai=2*costhetai*costhetai-1;
    sigma0=kds*costhetai+pow(cos2thetai,n);
  }else{
    sigma0=kds*costhetai;
  }
  return(sigma0*projarea);

}


/* function: BuildDZRhoMaxLookupTable()
 * ------------------------------------
 * Builds a 2-D lookup table of dzrhomax values vs nominal incidence angle
 * (rad) and correlation.
 */
static
float **BuildDZRhoMaxLookupTable(double nominc0, double dnominc,
                                 long nominctablesize, double rhomin,
                                 double drho, long nrho, paramT *params){

  long krho, knominc;
  double nominc, rho;
  float **dzrhomaxtable;

  dzrhomaxtable=(float **)Get2DMem(nominctablesize,nrho,
                                   sizeof(float *),sizeof(float));
  nominc=nominc0;
  for(knominc=0;knominc<nominctablesize;knominc++){
    rho=rhomin;
    for(krho=0;krho<nrho;krho++){
      dzrhomaxtable[knominc][krho]=(float )CalcDZRhoMax(rho,nominc,params,
                                                        params->threshold);
      rho+=drho;
    }
    nominc+=dnominc;
  }
  return(dzrhomaxtable);

}


/* function: CalcDZRhoMax()
 * ------------------------
 * Calculates the maximum slope (in range) for the given unbiased correlation
 * using spatial decorrelation as an upper limit (Zebker & Villasenor,
 * 1992).
 */
static
double CalcDZRhoMax(double rho, double nominc, paramT *params,
                    double threshold){

  long i;
  double dx, dr, dz, dzstep, rhos, sintheta, costheta, numerator;
  double a, re, bperp, slantrange, lookangle;
  double costhetairsq, rhosfactor, residual;


  /* set up */
  i=0;
  dr=params->dr;
  costheta=cos(nominc);
  sintheta=sin(nominc);
  dzstep=params->initdzstep;
  a=params->orbitradius;
  re=params->earthradius;
  lookangle=asin(re/a*sintheta);
  bperp=params->baseline*cos(lookangle-params->baselineangle);
  slantrange=sqrt(a*a+re*re-2*a*re*cos(nominc-lookangle));
  rhosfactor=2.0*fabs(bperp)*(params->rangeres)/((params->lambda)*slantrange);

  /* take care of the extremes */
  if(rho>=1.0){
    return(-dr*costheta);
  }else if(rho<=0){
    return(LARGEFLOAT);
  }

  /* start with slope for unity correlation, step slope upwards */
  dz=-dr*costheta;
  rhos=1.0;
  while(rhos>rho){
    dz+=dzstep;
    dx=(dr+dz*costheta)/sintheta;
    numerator=dz*sintheta+dx*costheta;
    costhetairsq=numerator*numerator/(dz*dz+dx*dx);
    rhos=1-rhosfactor*sqrt(costhetairsq/(1-costhetairsq));
    if(rhos<0){
      rhos=0;
    }
    if(dz>BIGGESTDZRHOMAX){
      return(BIGGESTDZRHOMAX);
    }
  }

  /* now iteratively decrease step size and narrow in on correct slope */
  while(fabs(residual=rhos-rho)>threshold*rho){
    dzstep/=2.0;
    if(residual<0){
      dz-=dzstep;
    }else{
      dz+=dzstep;
    }
    dx=(dr+dz*costheta)/sintheta;
    numerator=dz*sintheta+dx*costheta;
    costhetairsq=numerator*numerator/(dz*dz+dx*dx);
    rhos=1-rhosfactor*sqrt(costhetairsq/(1-costhetairsq));
    if(rhos<0){
      rhos=0;
    }
    if(++i>MAXITERATION){
      fflush(NULL);
      fprintf(sp0,"Couldn't find slope for correlation of %f\n",rho);
      fprintf(sp0,"(check geometry and spatial decorrelation parameters)\n");
      fprintf(sp0,"Abort\n");
      exit(ABNORMAL_EXIT);
    }
  }

  return(dz);
}


/* function: CalcCostTopo()
 * ------------------------
 * Calculates topography arc distance given an array of cost data structures.
 */
void CalcCostTopo(void **costs, long flow, long arcrow, long arccol,
                  long nflow, long nrow, paramT *params,
                  long *poscostptr, long *negcostptr){

  long idz1, idz2pos, idz2neg, cost1, nflowsq, poscost, negcost;
  long nshortcycle, layfalloffconst;
  long offset, sigsq, laycost, dzmax;
  costT *cost;


  /* get arc info */
  cost=&((costT **)(costs))[arcrow][arccol];
  offset=cost->offset;
  sigsq=cost->sigsq;
  dzmax=cost->dzmax;
  laycost=cost->laycost;

  /* just return 0 if we have zero cost arc */
  if(sigsq==LARGESHORT){
    (*poscostptr)=0;
    (*negcostptr)=0;
    return;
  }

  /* compute argument to cost function */
  nshortcycle=params->nshortcycle;
  layfalloffconst=params->layfalloffconst;
  if(arcrow<nrow-1){

    /* row cost: dz symmetric with respect to origin */
    idz1=labs(flow*nshortcycle+offset);
    idz2pos=labs((flow+nflow)*nshortcycle+offset);
    idz2neg=labs((flow-nflow)*nshortcycle+offset);

  }else{

    /* column cost: non-symmetric dz */
    /* dzmax will only be < 0 if we have a column arc */
    if(dzmax<0){
      dzmax*=-1;
      idz1=-(flow*nshortcycle+offset);
      idz2pos=-((flow+nflow)*nshortcycle+offset);
      idz2neg=-((flow-nflow)*nshortcycle+offset);
    }else{
      idz1=flow*nshortcycle+offset;
      idz2pos=(flow+nflow)*nshortcycle+offset;
      idz2neg=(flow-nflow)*nshortcycle+offset;
    }

  }

  /* calculate cost1 */
  if(idz1>dzmax){
    idz1-=dzmax;
    cost1=(idz1*idz1)/(layfalloffconst*sigsq)+laycost;
  }else{
    cost1=(idz1*idz1)/sigsq;
    if(laycost!=NOCOSTSHELF && idz1>0 && cost1>laycost){
      cost1=laycost;
    }
  }

  /* calculate positive cost increment */
  if(idz2pos>dzmax){
    idz2pos-=dzmax;
    poscost=(idz2pos*idz2pos)/(layfalloffconst*sigsq)
      +laycost-cost1;
  }else{
    poscost=(idz2pos*idz2pos)/sigsq;
    if(laycost!=NOCOSTSHELF && idz2pos>0 && poscost>laycost){
      poscost=laycost-cost1;
    }else{
      poscost-=cost1;
    }
  }

  /* calculate negative cost increment */
  if(idz2neg>dzmax){
    idz2neg-=dzmax;
    negcost=(idz2neg*idz2neg)/(layfalloffconst*sigsq)
      +laycost-cost1;
  }else{
    negcost=(idz2neg*idz2neg)/sigsq;
    if(laycost!=NOCOSTSHELF && idz2neg>0 && negcost>laycost){
      negcost=laycost-cost1;
    }else{
      negcost-=cost1;
    }
  }

  /* scale costs for this nflow */
  nflowsq=nflow*nflow;
  if(poscost>0){
    *poscostptr=(long )ceil((double )poscost/nflowsq);
  }else{
    *poscostptr=(long )floor((double )poscost/nflowsq);
  }
  if(negcost>0){
    *negcostptr=(long )ceil((double )negcost/nflowsq);
  }else{
    *negcostptr=(long )floor((double )negcost/nflowsq);
  }

  /* done */
  return;

}


/* function: CalcCostDefo()
 * ------------------------
 * Calculates deformation arc distance given an array of cost data structures.
 */
void CalcCostDefo(void **costs, long flow, long arcrow, long arccol,
                  long nflow, long nrow, paramT *params,
                  long *poscostptr, long *negcostptr){

  long idz1, idz2pos, idz2neg, cost1, nflowsq, poscost, negcost;
  long nshortcycle, layfalloffconst;
  costT *cost;


  /* get arc info */
  cost=&((costT **)(costs))[arcrow][arccol];

  /* just return 0 if we have zero cost arc */
  if(cost->sigsq==LARGESHORT){
    (*poscostptr)=0;
    (*negcostptr)=0;
    return;
  }

  /* compute argument to cost function */
  nshortcycle=params->nshortcycle;
  layfalloffconst=params->layfalloffconst;
  idz1=labs(flow*nshortcycle+cost->offset);
  idz2pos=labs((flow+nflow)*nshortcycle+cost->offset);
  idz2neg=labs((flow-nflow)*nshortcycle+cost->offset);

  /* calculate cost1 */
  if(idz1>cost->dzmax){
    idz1-=cost->dzmax;
    cost1=(idz1*idz1)/(layfalloffconst*(cost->sigsq))+cost->laycost;
  }else{
    cost1=(idz1*idz1)/cost->sigsq;
    if(cost->laycost!=NOCOSTSHELF && cost1>cost->laycost){
      cost1=cost->laycost;
    }
  }

  /* calculate positive cost increment */
  if(idz2pos>cost->dzmax){
    idz2pos-=cost->dzmax;
    poscost=(idz2pos*idz2pos)/(layfalloffconst*(cost->sigsq))
      +cost->laycost-cost1;
  }else{
    poscost=(idz2pos*idz2pos)/cost->sigsq;
    if(cost->laycost!=NOCOSTSHELF && poscost>cost->laycost){
      poscost=cost->laycost-cost1;
    }else{
      poscost-=cost1;
    }
  }

  /* calculate negative cost increment */
  if(idz2neg>cost->dzmax){
    idz2neg-=cost->dzmax;
    negcost=(idz2neg*idz2neg)/(layfalloffconst*(cost->sigsq))
      +cost->laycost-cost1;
  }else{
    negcost=(idz2neg*idz2neg)/cost->sigsq;
    if(cost->laycost!=NOCOSTSHELF && negcost>cost->laycost){
      negcost=cost->laycost-cost1;
    }else{
      negcost-=cost1;
    }
  }

  /* scale costs for this nflow */
  nflowsq=nflow*nflow;
  if(poscost>0){
    *poscostptr=(long )ceil((double )poscost/nflowsq);
  }else{
    *poscostptr=(long )floor((double )poscost/nflowsq);
  }
  if(negcost>0){
    *negcostptr=(long )ceil((double )negcost/nflowsq);
  }else{
    *negcostptr=(long )floor((double )negcost/nflowsq);
  }

  /* done */
  return;

}


/* function: CalcCostSmooth()
 * --------------------------
 * Calculates smooth-solution arc distance given an array of smoothcost
 *  data structures.
 */
void CalcCostSmooth(void **costs, long flow, long arcrow, long arccol,
                    long nflow, long nrow, paramT *params,
                    long *poscostptr, long *negcostptr){

  long idz1, idz2pos, idz2neg, cost1, nflowsq, poscost, negcost;
  long nshortcycle;
  smoothcostT *cost;


  /* get arc info */
  cost=&((smoothcostT **)(costs))[arcrow][arccol];

  /* just return 0 if we have zero cost arc */
  if(cost->sigsq==LARGESHORT){
    (*poscostptr)=0;
    (*negcostptr)=0;
    return;
  }

  /* compute argument to cost function */
  nshortcycle=params->nshortcycle;
  idz1=labs(flow*nshortcycle+cost->offset);
  idz2pos=labs((flow+nflow)*nshortcycle+cost->offset);
  idz2neg=labs((flow-nflow)*nshortcycle+cost->offset);

  /* calculate cost1 */
  cost1=(idz1*idz1)/cost->sigsq;

  /* calculate positive cost increment */
  poscost=(idz2pos*idz2pos)/cost->sigsq-cost1;

  /* calculate negative cost increment */
  negcost=(idz2neg*idz2neg)/cost->sigsq-cost1;

  /* scale costs for this nflow */
  nflowsq=nflow*nflow;
  if(poscost>0){
    *poscostptr=(long )ceil((double )poscost/nflowsq);
  }else{
    *poscostptr=(long )floor((double )poscost/nflowsq);
  }
  if(negcost>0){
    *negcostptr=(long )ceil((double )negcost/nflowsq);
  }else{
    *negcostptr=(long )floor((double )negcost/nflowsq);
  }

  /* done */
  return;

}


/* function: CalcCostL0()
 * ----------------------
 * Calculates the L0 arc distance given an array of short integer weights.
 */
void CalcCostL0(void **costs, long flow, long arcrow, long arccol,
                long nflow, long nrow, paramT *params,
                long *poscostptr, long *negcostptr){

  /* L0-norm */
  if(flow){
    if(flow+nflow){
      *poscostptr=0;
    }else{
      *poscostptr=-((short **)costs)[arcrow][arccol];
    }
    if(flow-nflow){
      *negcostptr=0;
    }else{
      *negcostptr=-((short **)costs)[arcrow][arccol];
    }
  }else{
    *poscostptr=((short **)costs)[arcrow][arccol];
    *negcostptr=((short **)costs)[arcrow][arccol];
  }

  /* done */
  return;

}


/* function: CalcCostL1()
 * ----------------------
 * Calculates the L1 arc distance given an array of short integer weights.
 */
void CalcCostL1(void **costs, long flow, long arcrow, long arccol,
                long nflow, long nrow, paramT *params,
                long *poscostptr, long *negcostptr){

  /* L1-norm */
  *poscostptr=((short **)costs)[arcrow][arccol]*(labs(flow+nflow)-labs(flow));
  *negcostptr=((short **)costs)[arcrow][arccol]*(labs(flow-nflow)-labs(flow));

  /* done */
  return;

}


/* function: CalcCostL2()
 * ----------------------
 * Calculates the L2 arc distance given an array of short integer weights.
 */
void CalcCostL2(void **costs, long flow, long arcrow, long arccol,
                long nflow, long nrow, paramT *params,
                long *poscostptr, long *negcostptr){

  long flow2, flowsq;

  /* L2-norm */
  flowsq=flow*flow;
  flow2=flow+nflow;
  *poscostptr=((short **)costs)[arcrow][arccol]*(flow2*flow2-flowsq);
  flow2=flow-nflow;
  *negcostptr=((short **)costs)[arcrow][arccol]*(flow2*flow2-flowsq);

  /* done */
  return;
}


/* function: CalcCostLP()
 * ----------------------
 * Calculates the Lp arc distance given an array of short integer weights.
 */
void CalcCostLP(void **costs, long flow, long arcrow, long arccol,
                long nflow, long nrow, paramT *params,
                long *poscostptr, long *negcostptr){

  double p;
  short flow2;

  /* Lp-norm */
  flow2=flow+nflow;
  p=params->p;
  *poscostptr=LRound(((short **)costs)[arcrow][arccol]*
                     (pow(labs(flow2),p)-pow(labs(flow),p)));
  flow2=flow-nflow;
  *negcostptr=LRound(((short **)costs)[arcrow][arccol]*
                     (pow(labs(flow2),p)-pow(labs(flow),p)));

  /* done */
  return;
}


/* function: CalcCostL0BiDir()
 * ---------------------------
 * Calculates the L0 arc cost given an array of bidirectional cost weights.
 */
void CalcCostL0BiDir(void **costs, long flow, long arcrow, long arccol,
                     long nflow, long nrow, paramT *params,
                     long *poscostptr, long *negcostptr){

  long newflow, cost0;

  /* L0-norm */
  if(flow>0){
    cost0=((bidircostT **)costs)[arcrow][arccol].posweight;
  }else if(flow<0){
    cost0=((bidircostT **)costs)[arcrow][arccol].negweight;
  }else{
    cost0=0;
  }
  newflow=flow+nflow;
  if(newflow>0){
    *poscostptr=((bidircostT **)costs)[arcrow][arccol].posweight-cost0;
  }else if(newflow<0){
    *poscostptr=((bidircostT **)costs)[arcrow][arccol].negweight-cost0;
  }else{
    *poscostptr=-cost0;
  }
  newflow=flow-nflow;
  if(newflow>0){
    *negcostptr=((bidircostT **)costs)[arcrow][arccol].posweight-cost0;
  }else if(newflow<0){
    *negcostptr=((bidircostT **)costs)[arcrow][arccol].negweight-cost0;
  }else{
    *negcostptr=-cost0;
  }

  /* done */
  return;
}


/* function: CalcCostL1BiDir()
 * ---------------------------
 * Calculates the L1 arc cost given an array of bidirectional cost weights.
 */
void CalcCostL1BiDir(void **costs, long flow, long arcrow, long arccol,
                     long nflow, long nrow, paramT *params,
                     long *poscostptr, long *negcostptr){

  long newflow, cost0;

  /* L1-norm */
  if(flow>0){
    cost0=((bidircostT **)costs)[arcrow][arccol].posweight*flow;
  }else{
    cost0=-((bidircostT **)costs)[arcrow][arccol].negweight*flow;
  }
  newflow=flow+nflow;
  if(newflow>0){
    *poscostptr=(((bidircostT **)costs)[arcrow][arccol].posweight*newflow
                 -cost0);
  }else{
    *poscostptr=(-((bidircostT **)costs)[arcrow][arccol].negweight*newflow
                 -cost0);
  }
  newflow=flow-nflow;
  if(newflow>0){
    *negcostptr=(((bidircostT **)costs)[arcrow][arccol].posweight*newflow
                 -cost0);
  }else{
    *negcostptr=(-((bidircostT **)costs)[arcrow][arccol].negweight*newflow
                 -cost0);
  }

  /* done */
  return;

}


/* function: CalcCostL2BiDir()
 * ---------------------------
 * Calculates the L2 arc cost given an array of bidirectional cost weights.
 */
void CalcCostL2BiDir(void **costs, long flow, long arcrow, long arccol,
                     long nflow, long nrow, paramT *params,
                     long *poscostptr, long *negcostptr){

  long newflow, cost0;

  /* L2-norm */
  if(flow>0){
    cost0=((bidircostT **)costs)[arcrow][arccol].posweight*flow*flow;
  }else{
    cost0=((bidircostT **)costs)[arcrow][arccol].negweight*flow*flow;
  }
  newflow=flow+nflow;
  if(newflow>0){
    *poscostptr=(((bidircostT **)costs)[arcrow][arccol].posweight
                 *newflow*newflow-cost0);
  }else{
    *poscostptr=(((bidircostT **)costs)[arcrow][arccol].negweight
                 *newflow*newflow-cost0);
  }
  newflow=flow-nflow;
  if(newflow>0){
    *negcostptr=(((bidircostT **)costs)[arcrow][arccol].posweight
                 *newflow*newflow-cost0);
  }else{
    *negcostptr=(((bidircostT **)costs)[arcrow][arccol].negweight
                 *newflow*newflow-cost0);
  }

  /* done */
  return;
}


/* function: CalcCostLPBiDir()
 * ---------------------------
 * Calculates the Lp arc cost given an array of bidirectional cost weights.
 */
void CalcCostLPBiDir(void **costs, long flow, long arcrow, long arccol,
                     long nflow, long nrow, paramT *params,
                     long *poscostptr, long *negcostptr){

  long newflow;
  double p, cost0;

  /* Lp-norm */
  p=params->p;
  if(flow>0){
    cost0=((bidircostT **)costs)[arcrow][arccol].posweight*pow(flow,p);
  }else{
    cost0=((bidircostT **)costs)[arcrow][arccol].negweight*pow(-flow,p);
  }
  newflow=flow+nflow;
  if(newflow>0){
    *poscostptr=LRound(((bidircostT **)costs)[arcrow][arccol].posweight
                       *pow(newflow,p)-cost0);
  }else{
    *poscostptr=LRound(((bidircostT **)costs)[arcrow][arccol].negweight
                       *pow(newflow,p)-cost0);
  }
  newflow=flow-nflow;
  if(newflow>0){
    *negcostptr=LRound(((bidircostT **)costs)[arcrow][arccol].posweight
                       *pow(newflow,p)-cost0);
  }else{
    *negcostptr=LRound(((bidircostT **)costs)[arcrow][arccol].negweight
                       *pow(newflow,p)-cost0);
  }

  /* done */
  return;
}


/* function: CalcCostNonGrid()
 * ---------------------------
 * Calculates the arc cost given an array of long integer cost lookup tables.
 *
 * The cost array for each arc gives the cost for +/-flowmax units of
 * flow around the flow value with minimum cost, which is not
 * necessarily flow == 0.  The offset between the flow value with
 * minimum cost and flow == 0 is given by arroffset = costarr[0].
 * Positive flow values k for k = 1 to flowmax relative to this min
 * cost flow value are in costarr[k].  Negative flow values k relative
 * to the min cost flow from k = -1 to -flowmax costarr[flowmax-k].
 * costarr[2*flowmax+1] contains a scaling factor for extrapolating
 * beyond the ends of the cost table, assuming quadratically (with an offset)
 * increasing cost (subject to rounding and scaling).
 *
 * As of summer 2019, the rationale for how seconeary costs are
 * extrapolated beyond the end of the table has been lost to time, but
 * the logic at least does give a self-consistent cost function that
 * is continuous at +/-flowmax and quadratically increases beyond,
 * albeit not necessarily with a starting slope that has an easily
 * intuitive basis.
 */
void CalcCostNonGrid(void **costs, long flow, long arcrow, long arccol,
                     long nflow, long nrow, paramT *params,
                     long *poscostptr, long *negcostptr){

  long xflow, flowmax, poscost, negcost, nflowsq, arroffset, sumsigsqinv;
  long abscost0;
  long *costarr;
  double c1;


  /* set up */
  flowmax=params->scndryarcflowmax;
  costarr=((long ***)costs)[arcrow][arccol];
  arroffset=costarr[0];
  sumsigsqinv=costarr[2*flowmax+1];

  /* return zero costs if this is a zero cost arc */
  if(sumsigsqinv==ZEROCOSTARC){
    *poscostptr=0;
    *negcostptr=0;
    return;
  }

  /* compute cost of current flow */
  xflow=flow+arroffset;
  if(xflow>flowmax){
    c1=costarr[flowmax]/(double )flowmax-sumsigsqinv*flowmax;
    abscost0=(sumsigsqinv*xflow+LRound(c1))*xflow;
  }else if(xflow<-flowmax){
    c1=costarr[2*flowmax]/(double )flowmax-sumsigsqinv*flowmax;
    abscost0=(sumsigsqinv*xflow+LRound(c1))*xflow;
  }else{
    if(xflow>0){
      abscost0=costarr[xflow];
    }else if(xflow<0){
      abscost0=costarr[flowmax-xflow];
    }else{
      abscost0=0;
    }
  }

  /* compute costs of positive and negative flow increments */
  xflow=flow+arroffset+nflow;
  if(xflow>flowmax){
    c1=costarr[flowmax]/(double )flowmax-sumsigsqinv*flowmax;
    poscost=((sumsigsqinv*xflow+LRound(c1))*xflow)-abscost0;
  }else if(xflow<-flowmax){
    c1=costarr[2*flowmax]/(double )flowmax-sumsigsqinv*flowmax;
    poscost=((sumsigsqinv*xflow+LRound(c1))*xflow)-abscost0;
  }else{
    if(xflow>0){
      poscost=costarr[xflow]-abscost0;
    }else if(xflow<0){
      poscost=costarr[flowmax-xflow]-abscost0;
    }else{
      poscost=-abscost0;
    }
  }
  xflow=flow+arroffset-nflow;
  if(xflow>flowmax){
    c1=costarr[flowmax]/(double )flowmax-sumsigsqinv*flowmax;
    negcost=((sumsigsqinv*xflow+LRound(c1))*xflow)-abscost0;
  }else if(xflow<-flowmax){
    c1=costarr[2*flowmax]/(double )flowmax-sumsigsqinv*flowmax;
    negcost=((sumsigsqinv*xflow+LRound(c1))*xflow)-abscost0;
  }else{
    if(xflow>0){
      negcost=costarr[xflow]-abscost0;
    }else if(xflow<0){
      negcost=costarr[flowmax-xflow]-abscost0;
    }else{
      negcost=-abscost0;
    }
  }

  /* scale for this flow increment and set output values */
  nflowsq=nflow*nflow;
  if(poscost>0){
    *poscostptr=(long )ceil((double )poscost/nflowsq);
  }else{
    *poscostptr=(long )floor((double )poscost/nflowsq);
  }
  if(negcost>0){
    *negcostptr=(long )ceil((double )negcost/nflowsq);
  }else{
    *negcostptr=(long )floor((double )negcost/nflowsq);
  }

  /* done */
  return;

}


/* function: EvalCostTopo()
 * ------------------------
 * Calculates topography arc cost given an array of cost data structures.
 */
long EvalCostTopo(void **costs, short **flows, long arcrow, long arccol,
                  long nrow, paramT *params){

  long idz1, cost1, dzmax;
  costT *cost;

  /* get arc info */
  cost=&((costT **)(costs))[arcrow][arccol];

  /* just return 0 if we have zero cost arc */
  if(cost->sigsq==LARGESHORT){
    return(0);
  }

  /* compute argument to cost function */
  if(arcrow<nrow-1){

    /* row cost: dz symmetric with respect to origin */
    idz1=labs(flows[arcrow][arccol]*(params->nshortcycle)+cost->offset);
    dzmax=cost->dzmax;

  }else{

    /* column cost: non-symmetric dz */
    idz1=flows[arcrow][arccol]*(params->nshortcycle)+cost->offset;
    if((dzmax=cost->dzmax)<0){
      idz1*=-1;
      dzmax*=-1;
    }

  }

  /* calculate and return cost */
  if(idz1>dzmax){
    idz1-=dzmax;
    cost1=(idz1*idz1)/((params->layfalloffconst)*(cost->sigsq))+cost->laycost;
  }else{
    cost1=(idz1*idz1)/cost->sigsq;
    if(cost->laycost!=NOCOSTSHELF && idz1>0 && cost1>cost->laycost){
      cost1=cost->laycost;
    }
  }
  return(cost1);
}


/* function: EvalCostDefo()
 * ------------------------
 * Calculates deformation arc cost given an array of cost data structures.
 */
long EvalCostDefo(void **costs, short **flows, long arcrow, long arccol,
                  long nrow, paramT *params){

  long idz1, cost1;
  costT *cost;

  /* get arc info */
  cost=&((costT **)(costs))[arcrow][arccol];

  /* just return 0 if we have zero cost arc */
  if(cost->sigsq==LARGESHORT){
    return(0);
  }

  /* compute argument to cost function */
  idz1=labs(flows[arcrow][arccol]*(params->nshortcycle)+cost->offset);

  /* calculate and return cost */
  if(idz1>cost->dzmax){
    idz1-=cost->dzmax;
    cost1=(idz1*idz1)/((params->layfalloffconst)*(cost->sigsq))+cost->laycost;
  }else{
    cost1=(idz1*idz1)/cost->sigsq;
    if(cost->laycost!=NOCOSTSHELF && cost1>cost->laycost){
      cost1=cost->laycost;
    }
  }
  return(cost1);
}


/* function: EvalCostSmooth()
 * --------------------------
 * Calculates smooth-solution arc cost given an array of
 * smoothcost data structures.
 */
long EvalCostSmooth(void **costs, short **flows, long arcrow, long arccol,
                    long nrow, paramT *params){

  long idz1;
  smoothcostT *cost;

  /* get arc info */
  cost=&((smoothcostT **)(costs))[arcrow][arccol];

  /* just return 0 if we have zero cost arc */
  if(cost->sigsq==LARGESHORT){
    return(0);
  }

  /* compute argument to cost function */
  idz1=labs(flows[arcrow][arccol]*(params->nshortcycle)+cost->offset);

  /* calculate and return cost */
  return((idz1*idz1)/cost->sigsq);

}


/* function: EvalCostL0()
 * ----------------------
 * Calculates the L0 arc cost given an array of cost data structures.
 */
long EvalCostL0(void **costs, short **flows, long arcrow, long arccol,
                long nrow, paramT *params){

  /* L0-norm */
  if(flows[arcrow][arccol]){
    return((long)((short **)costs)[arcrow][arccol]);
  }else{
    return(0);
  }
}


/* function: EvalCostL1()
 * ----------------------
 * Calculates the L1 arc cost given an array of cost data structures.
 */
long EvalCostL1(void **costs, short **flows, long arcrow, long arccol,
                long nrow, paramT *params){

  /* L1-norm */
  return((long )((((short **)costs)[arcrow][arccol])
                 *labs(flows[arcrow][arccol])));
}


/* function: EvalCostL2()
 * ----------------------
 * Calculates the L2 arc cost given an array of cost data structures.
 */
long EvalCostL2(void **costs, short **flows, long arcrow, long arccol,
                long nrow, paramT *params){

  /* L2-norm */
  return((long )((((short **)costs)[arcrow][arccol])
                 *(flows[arcrow][arccol]*flows[arcrow][arccol])));
}


/* function: EvalCostLP()
 * ----------------------
 * Calculates the Lp arc cost given an array of cost data structures.
 */
long EvalCostLP(void **costs, short **flows, long arcrow, long arccol,
                long nrow, paramT *params){

  /* Lp-norm */
  return(LRound((((short **)costs)[arcrow][arccol]) *
                pow(labs(flows[arcrow][arccol]),params->p)));
}


/* function: EvalCostL0BiDir()
 * ---------------------------
 * Calculates the L0 arc cost given an array of bidirectional cost structures.
 */
long EvalCostL0BiDir(void **costs, short **flows, long arcrow, long arccol,
                     long nrow, paramT *params){

  /* L0-norm */
  if(flows[arcrow][arccol]>0){
    return((long )((bidircostT **)costs)[arcrow][arccol].posweight);
  }else if(flows[arcrow][arccol]<0){
    return((long )((bidircostT **)costs)[arcrow][arccol].negweight);
  }else{
    return(0);
  }
}


/* function: EvalCostL1BiDir()
 * ---------------------------
 * Calculates the L1 arc cost given an array of bidirectional cost structures.
 */
long EvalCostL1BiDir(void **costs, short **flows, long arcrow, long arccol,
                     long nrow, paramT *params){

  /* L1-norm */
  if(flows[arcrow][arccol]>0){
    return((long )((((bidircostT **)costs)[arcrow][arccol].posweight)
                   *(flows[arcrow][arccol])));
  }else{
    return((long )((((bidircostT **)costs)[arcrow][arccol].negweight)
                   *(-flows[arcrow][arccol])));
  }
}


/* function: EvalCostL2BiDir()
 * ---------------------------
 * Calculates the L2 arc cost given an array of bidirectional cost structures.
 */
long EvalCostL2BiDir(void **costs, short **flows, long arcrow, long arccol,
                     long nrow, paramT *params){

  /* L2-norm */
  if(flows[arcrow][arccol]>0){
    return((long )((((bidircostT **)costs)[arcrow][arccol].posweight)
                   *(flows[arcrow][arccol]*flows[arcrow][arccol])));
  }else{
    return((long )((((bidircostT **)costs)[arcrow][arccol].negweight)
                   *(flows[arcrow][arccol]*flows[arcrow][arccol])));
  }
}


/* function: EvalCostLPBiDir()
 * ---------------------------
 * Calculates the Lp arc cost given an array of bidirectional cost structures.
 */
long EvalCostLPBiDir(void **costs, short **flows, long arcrow, long arccol,
                     long nrow, paramT *params){

  /* Lp-norm */
  if(flows[arcrow][arccol]>0){
    return(LRound((((bidircostT **)costs)[arcrow][arccol].posweight)
                  *pow(flows[arcrow][arccol],params->p)));
  }else{
    return(LRound((((bidircostT **)costs)[arcrow][arccol].posweight)
                  *pow(-flows[arcrow][arccol],params->p)));
  }
}


/* function: EvalCostNonGrid()
 * ---------------------------
 * Calculates the arc cost given an array of long integer cost lookup tables.
 */
long EvalCostNonGrid(void **costs, short **flows, long arcrow, long arccol,
                     long nrow, paramT *params){

  long flow, xflow, flowmax, arroffset, sumsigsqinv;
  long *costarr;
  double c1;

  /* set up */
  flow=flows[arcrow][arccol];
  flowmax=params->scndryarcflowmax;
  costarr=((long ***)costs)[arcrow][arccol];
  arroffset=costarr[0];
  sumsigsqinv=costarr[2*flowmax+1];

  /* return zero costs if this is a zero cost arc */
  if(sumsigsqinv==ZEROCOSTARC){
    return(0);
  }

  /* compute cost of current flow */
  xflow=flow+arroffset;
  if(xflow>flowmax){
    c1=costarr[flowmax]/(double )flowmax-sumsigsqinv*flowmax;
    return((sumsigsqinv*xflow+LRound(c1))*xflow);
  }else if(xflow<-flowmax){
    c1=costarr[2*flowmax]/(double )flowmax-sumsigsqinv*flowmax;
    return((sumsigsqinv*xflow+LRound(c1))*xflow);
  }else{
    if(xflow>0){
      return(costarr[xflow]);
    }else if(xflow<0){
      return(costarr[flowmax-xflow]);
    }else{
      return(0);
    }
  }
}


/* function: CalcInitMaxFlow()
 * ---------------------------
 * Calculates the maximum flow magnitude to allow in the initialization
 * by examining the dzmax members of arc statistical cost data structures.
 */
static
int CalcInitMaxFlow(paramT *params, void **costs, long nrow, long ncol){

  long row, col, maxcol, initmaxflow, arcmaxflow;

  if(params->initmaxflow<=0){
    if(params->costmode==NOSTATCOSTS){
      params->initmaxflow=NOSTATINITMAXFLOW;
    }else{
      if(params->costmode==TOPO || params->costmode==DEFO){
        initmaxflow=0;
        for(row=0;row<2*nrow-1;row++){
          if(row<nrow-1){
            maxcol=ncol;
          }else{
            maxcol=ncol-1;
          }
          for(col=0;col<maxcol;col++){
            if(((costT **)costs)[row][col].dzmax!=LARGESHORT){
              arcmaxflow=ceil(labs((long )((costT **)costs)[row][col].dzmax)/
                              (double )(params->nshortcycle)
                              +params->arcmaxflowconst);
              if(arcmaxflow>initmaxflow){
                initmaxflow=arcmaxflow;
              }
            }
          }
        }
        params->initmaxflow=initmaxflow;
      }else{
        params->initmaxflow=DEF_INITMAXFLOW;
      }
    }
  }
  return(0);
}

/* end snaphu_cost.c */
/* snaphu_cs2.c */
/***********************************************************************

  This code is derived from cs2 v3.7
  Written by Andrew V. Goldberg and Boris Cherkassky
  Modifications for use in snaphu by Curtis W. Chen

  The cs2 code is used here with permission for strictly noncommerical
  use.  The original cs2 source code can be downloaded from

    <http://www.igsystems.com/cs2>

  The original cs2 copyright is stated as follows:

    COPYRIGHT C 1995 IG Systems, Inc.  Permission to use for
    evaluation purposes is granted provided that proper
    acknowledgments are given.  For a commercial licence, contact
    igsys@eclipse.net.

    This software comes with NO WARRANTY, expressed or implied. By way
    of example, but not limitation, we make no representations of
    warranties of merchantability or fitness for any particular
    purpose or that the use of the software components or
    documentation will not infringe any patents, copyrights,
    trademarks, or other rights.

  Copyright 2002 Board of Trustees, Leland Stanford Jr. University

*************************************************************************/

/* min-cost flow */
/* successive approximation algorithm */
/* Copyright C IG Systems, igsys@eclipse.com */
/* any use except for evaluation purposes requires a licence */

/* parser changed to take input from passed data */
/* main() changed to callable function */
/* outputs parsed as flow */
/* functions made static */
/* MAX and MIN macros renamed GREATEROF and LESSEROF */

#ifndef NO_CS2

/************************************** constants  &  parameters ********/





/* ouput stream pointers */
/* sp0=error messages, sp1=status output, sp2=verbose, sp3=verbose counter */
extern FILE *sp0, *sp1, *sp2, *sp3;
/* for measuring time */

/* definitions of types: node & arc */

#define PRICE_MAX           1e30
#define BIGGEST_FLOW        LARGESHORT

/* parser for getting  DIMACS format input and transforming the
   data to the internal representation */

/* snaphu_cs2parse.c */
/*************************************************************************

  This code is derived from cs2 v3.7
  Written by Andrew V. Goldberg and Boris Cherkassky
  Modifications for use in snaphu by Curtis W. Chen

  Parser for cs2 minimum cost flow solver.  Originally written to read
  DIMACS format (text) input files.  Modified to parse passed data
  from snaphu.  This file is included with a #include from
  snaphu_cs2.c.

  The cs2 code is used here with permission for strictly noncommerical
  use.  The original cs2 source code can be downloaded from

    <http://www.igsystems.com/cs2>

  The original cs2 copyright is stated as follows:

    COPYRIGHT C 1995 IG Systems, Inc.  Permission to use for
    evaluation purposes is granted provided that proper
    acknowledgments are given.  For a commercial licence, contact
    igsys@eclipse.net.

    This software comes with NO WARRANTY, expressed or implied. By way
    of example, but not limitation, we make no representations of
    warranties of merchantability or fitness for any particular
    purpose or that the use of the software components or
    documentation will not infringe any patents, copyrights,
    trademarks, or other rights.

  Copyright 2002 Board of Trustees, Leland Stanford Jr. University

*************************************************************************/

int cs2mcfparse(
  /* parameters passed to set up network */
  signed char    **residue,             /* 2D array of residues */
  short   **rowcost,             /* 2D array of row arc costs */
  short   **colcost,             /* 2D array of col arc costs */
  long    nNrow,                 /* number of nodes per row */
  long    nNcol,                 /* number of nodes per column */

  /* these parameters are output */
  long    *n_ad,                 /* address of the number of nodes */
  long    *m_ad,                 /* address of the number of arcs */
  node    **nodes_ad,            /* address of the array of nodes */
  arc     **arcs_ad,             /* address of the array of arcs */
  long    *node_min_ad,          /* address of the minimal node */
  double  *m_c_ad,               /* maximal arc cost */
  short    **cap_ad             /* array of capacities (changed to short) */
  )
{


#define ABS( x ) ( (x) >= 0 ) ? (x) : -(x)

/* variables added for unwrapping parse */
unsigned int row, col, dir;
unsigned long narcs, nnodes, nodectr, arcctr, nresidues;
long cumsupply, temp;


long inf_cap = 0;
long    n,                      /* internal number of nodes */
        node_min,               /* minimal no of node  */
        node_max,               /* maximal no of nodes */
       *arc_first,              /* internal array for holding
                                     - node degree
                                     - position of the first outgoing arc */
       *arc_tail,               /* internal array: tails of the arcs */
        /* temporary variables carrying no of nodes */
        head, tail, i;

long    m,                      /* internal number of arcs */
        /* temporary variables carrying no of arcs */
        last, arc_num, arc_new_num;

node    *nodes,                 /* pointers to the node structure */
        *head_p,
        *ndp,
        *in,
        *jn;

arc     *arcs,                  /* pointers to the arc structure */
        *arc_current,
        *arc_new,
        *arc_tmp;

long    excess,                 /* supply/demand of the node */
        low,                    /* lowest flow through the arc */
        acap;                    /* capacity */

long    cost;                   /* arc cost */


double  dcost,                  /* arc cost in double mode */
        m_c;                    /* maximal arc cost */

short    *cap;                   /* array of capacities (changed to short) */

double  total_p,                /* total supply */
        total_n,                /* total demand */
        cap_out,                /* sum of outgoing capacities */
        cap_in;                 /* sum of incoming capacities */

long    no_lines=0,             /* no of current input line */
  /*    no_plines=0, */           /* no of problem-lines */
  /*    no_nlines=0, */            /* no of node lines */
        no_alines=0,            /* no of arc-lines */
        pos_current=0;          /* 2*no_alines */

 int    /* k, */                     /* temporary */
        err_no;                 /* no of detected error */

/* -------------- error numbers & error messages ---------------- */
#define EN1   0
#define EN2   1
#define EN3   2
#define EN4   3
#define EN6   4
#define EN10  5
#define EN7   6
#define EN8   7
#define EN9   8
#define EN11  9
#define EN12 10
#define EN13 11
#define EN14 12
#define EN16 13
#define EN15 14
#define EN17 15
#define EN18 16
#define EN21 17
#define EN19 18
#define EN20 19
#define EN22 20

static char *err_message[] =
  {
/* 0*/    "more than one problem line",
/* 1*/    "wrong number of parameters in the problem line",
/* 2*/    "it is not a Min-cost problem line",
/* 3*/    "bad value of a parameter in the problem line",
/* 4*/    "can't obtain enough memory to solve this problem",
/* 5*/    "",
/* 6*/    "can't read problem name",
/* 7*/    "problem description must be before node description",
/* 8*/    "wrong capacity bounds",
/* 9*/    "wrong number of parameters in the node line",
/*10*/    "wrong value of parameters in the node line",
/*11*/    "unbalanced problem",
/*12*/    "node descriptions must be before arc descriptions",
/*13*/    "too many arcs in the input",
/*14*/    "wrong number of parameters in the arc line",
/*15*/    "wrong value of parameters in the arc line",
/*16*/    "unknown line type in the input",
/*17*/    "read error",
/*18*/    "not enough arcs in the input",
/*19*/    "warning: capacities too big - excess overflow possible",
/*20*/    "can't read anything from the input file",
/*21*/    "warning: infinite capacity replaced by BIGGEST_FLOW"
  };
/* --------------------------------------------------------------- */



/* set up */
nnodes=nNrow*nNcol+1;                           /* add one for ground node */
narcs=2*((nNrow+1)*nNcol+nNrow*(nNcol+1));  /* 2x for two directional arcs */
cumsupply=0;
nresidues=0;

/* get memory (formerly case 'p' in DIMACS file read) */
fprintf(sp2,"Setting up data structures for cs2 MCF solver\n");
n=nnodes;
m=narcs;
if ( n <= 0  || m <= 0 )
  /*wrong value of no of arcs or nodes*/
  { err_no = EN4; goto error; }

/* allocating memory for  'nodes', 'arcs'  and internal arrays */
nodes    = (node*) CAlloc ( n+2, sizeof(node) );
arcs     = (arc*)  CAlloc ( 2*m+1, sizeof(arc) );
cap      = (short*) CAlloc ( 2*m,   sizeof(short) ); /* changed to short */
arc_tail = (long*) CAlloc ( 2*m,   sizeof(long) );
arc_first= (long*) CAlloc ( n+2, sizeof(long) );
/* arc_first [ 0 .. n+1 ] = 0 - initialized by calloc */

for ( in = nodes; in <= nodes + n; in ++ )
  in -> excess = 0;

if ( nodes == NULL || arcs == NULL ||
     arc_first == NULL || arc_tail == NULL )
  /* memory is not allocated */
  { err_no = EN6; goto error; }

/* setting pointer to the first arc */
arc_current = arcs;
node_max = 0;
node_min = n;
m_c      = 0;
total_p = total_n = 0;

for ( ndp = nodes; ndp < nodes + n; ndp ++ )
  ndp -> excess = 0;

/* end of former case 'p' */


/* load supply/demand info into arrays (case 'n' in former loop) */
for(col=0; col<nNcol; col++){
  for(row=0; row<nNrow; row++){
    if(residue[row][col]){
      i=(col*nNrow + row + 1);
      excess=residue[row][col];
      ( nodes + i ) -> excess = excess;
      if ( excess > 0 ) total_p += (double)excess;
      if ( excess < 0 ) total_n -= (double)excess;
      nresidues++;
      cumsupply+=residue[row][col];
    }
  }
}

/* give ground node excess of -cumsupply */
( nodes + nnodes ) -> excess = -cumsupply;
if (cumsupply < 0) total_p -= (double)cumsupply;
if (cumsupply > 0) total_n += (double)cumsupply;

/* load arc info into arrays (case 'a' in former loop) */
low=0;
acap=ARCUBOUND;

/* horizontal (row) direction arcs first */
for(arcctr=1;arcctr<=2*nNrow*nNcol+nNrow+nNcol;arcctr++){
  if(arcctr<=nNrow*(nNcol+1)){
    /* row (horizontal) arcs first */
    nodectr=arcctr;
    if(nodectr<=nNrow*nNcol){
      tail=nodectr;
    }else{
      tail=nnodes;
    }
    if(nodectr<=nNrow){
      head=nnodes;
    }else{
      head=nodectr-nNrow;
    }
    cost=rowcost[((nodectr-1) % nNrow)][(int )((nodectr-1)/nNrow)];
  }else{
    /* column (vertical) arcs */
    nodectr=arcctr-nNrow*(nNcol+1);
    if(nodectr % (nNrow+1)==0){
      tail=nnodes;
    }else{
      tail=(int )(nodectr-ceil(nodectr/(nNrow+1.0))+1);
    }
    if(nodectr % (nNrow+1)==1){
      head=nnodes;
    }else{
      head=(int )(nodectr-ceil(nodectr/(nNrow+1.0)));
    }
    cost=colcost[((nodectr-1) % (nNrow+1))][(int )((nodectr-1)/(nNrow+1))];
  }

  if ( tail < 0  ||  tail > n  ||
       head < 0  ||  head > n
       )
    /* wrong value of nodes */
    { err_no = EN17; goto error; }

  if ( acap < 0 ) {
    acap = BIGGEST_FLOW;
    if (!inf_cap) {
      inf_cap = 1;
      fprintf ( sp0, "\ncs2 solver: %s\n", err_message[21] );
    }
  }

  if ( low < 0 || low > acap )
    { err_no = EN9; goto error; }

  for(dir=0;dir<=1;dir++){
    if(dir){
      /* switch head and tail and loop for two directional arcs */
      temp=tail;
      tail=head;
      head=temp;
    }

    /* no of arcs incident to node i is placed in arc_first[i+1] */
    arc_first[tail + 1] ++;
    arc_first[head + 1] ++;
    in    = nodes + tail;
    jn    = nodes + head;
    dcost = (double)cost;

    /* storing information about the arc */
    arc_tail[pos_current]        = tail;
    arc_tail[pos_current+1]      = head;
    arc_current       -> head    = jn;
    arc_current       -> r_cap   = acap - low;
    cap[pos_current]             = acap;
    arc_current       -> cost    = dcost;
    arc_current       -> sister  = arc_current + 1;
    ( arc_current + 1 ) -> head    = nodes + tail;
    ( arc_current + 1 ) -> r_cap   = 0;
    cap[pos_current+1]           = 0;
    ( arc_current + 1 ) -> cost    = -dcost;
    ( arc_current + 1 ) -> sister  = arc_current;

    in -> excess -= low;
    jn -> excess += low;

    /* searching for minimum and maximum node */
    if ( head < node_min ) node_min = head;
    if ( tail < node_min ) node_min = tail;
    if ( head > node_max ) node_max = head;
    if ( tail > node_max ) node_max = tail;

    if ( dcost < 0 ) dcost = -dcost;
    if ( dcost > m_c && acap > 0 ) m_c = dcost;

    no_alines   ++;
    arc_current += 2;
    pos_current += 2;

  }/* end of for loop over arc direction */
}/* end of for loop over arcss */


/* ----- all is red  or  error while reading ----- */

if ( ABS( total_p - total_n ) > 0.5 ) /* unbalanced problem */
  { err_no = EN13; goto error; }

/********** ordering arcs - linear time algorithm ***********/

/* first arc from the first node */
( nodes + node_min ) -> first = arcs;

/* before below loop arc_first[i+1] is the number of arcs outgoing from i;
   after this loop arc_first[i] is the position of the first
   outgoing from node i arcs after they would be ordered;
   this value is transformed to pointer and written to node.first[i]
   */

for ( i = node_min + 1; i <= node_max + 1; i ++ )
  {
    arc_first[i]          += arc_first[i-1];
    ( nodes + i ) -> first = arcs + arc_first[i];
  }


for ( i = node_min; i < node_max; i ++ ) /* scanning all the nodes
                                            exept the last*/
  {

    last = ( ( nodes + i + 1 ) -> first ) - arcs;
                             /* arcs outgoing from i must be cited
                              from position arc_first[i] to the position
                              equal to initial value of arc_first[i+1]-1  */

    for ( arc_num = arc_first[i]; arc_num < last; arc_num ++ )
      { tail = arc_tail[arc_num];

        while ( tail != i )
          /* the arc no  arc_num  is not in place because arc cited here
             must go out from i;
             we'll put it to its place and continue this process
             until an arc in this position would go out from i */

          { arc_new_num  = arc_first[tail];
            arc_current  = arcs + arc_num;
            arc_new      = arcs + arc_new_num;

            /* arc_current must be cited in the position arc_new
               swapping these arcs:                                 */

            head_p               = arc_new -> head;
            arc_new -> head      = arc_current -> head;
            arc_current -> head  = head_p;

            acap                 = cap[arc_new_num];
            cap[arc_new_num]     = cap[arc_num];
            cap[arc_num]         = acap;

            acap                 = arc_new -> r_cap;
            arc_new -> r_cap     = arc_current -> r_cap;
            arc_current -> r_cap = acap;

            dcost                = arc_new -> cost;
            arc_new -> cost      = arc_current -> cost;
            arc_current -> cost  = dcost;

            if ( arc_new != arc_current -> sister )
              {
                arc_tmp                = arc_new -> sister;
                arc_new  -> sister     = arc_current -> sister;
                arc_current -> sister  = arc_tmp;

                ( arc_current -> sister ) -> sister = arc_current;
                ( arc_new     -> sister ) -> sister = arc_new;
              }

            arc_tail[arc_num] = arc_tail[arc_new_num];
            arc_tail[arc_new_num] = tail;

            /* we increase arc_first[tail]  */
            arc_first[tail] ++ ;

            tail = arc_tail[arc_num];
          }
      }
    /* all arcs outgoing from  i  are in place */
  }

/* -----------------------  arcs are ordered  ------------------------- */

/*------------ testing network for possible excess overflow ---------*/

for ( ndp = nodes + node_min; ndp <= nodes + node_max; ndp ++ )
{
   cap_in  =   ( ndp -> excess );
   cap_out = - ( ndp -> excess );
   for ( arc_current = ndp -> first; arc_current != (ndp+1) -> first;
         arc_current ++ )
      {
        arc_num = arc_current - arcs;
        if ( cap[arc_num] > 0 ) cap_out += cap[arc_num];
        if ( cap[arc_num] == 0 )
          cap_in += cap[( arc_current -> sister )-arcs];
      }

   /*
   if (cap_in > BIGGEST_FLOW || cap_out > BIGGEST_FLOW)
     {
       fprintf ( sp0, "\ncs2 solver: %s\n", err_message[EN20] );
       break;
     }
   */
}

/* ----------- assigning output values ------------*/
*m_ad = m;
*n_ad = node_max - node_min + 1;
*node_min_ad = node_min;
*nodes_ad = nodes + node_min;
*arcs_ad = arcs;
*m_c_ad  = m_c;
*cap_ad   = cap;

/* free internal memory */
free ( arc_first ); free ( arc_tail );

/* Thanks God! All is done! */
return (0);

/* ---------------------------------- */
 error:  /* error found reading input */

fprintf ( sp0, "\ncs2 solver: line %ld of input - %s\n",
         no_lines, err_message[err_no] );

exit (ABNORMAL_EXIT);

/* this is a needless return statement so the compiler doesn't complain */
return(1);

}
/* --------------------   end of parser  -------------------*/
/* end snaphu_cs2parse.c */


#define N_NODE( i ) ( ( (i) == NULL ) ? -1 : ( (i) - ndp + nmin ) )
#define N_ARC( a ) ( ( (a) == NULL )? -1 : (a) - arp )


#define UNFEASIBLE          2
#define ALLOCATION_FAULT    5
#define PRICE_OFL           6

/* parameters */

#define UPDT_FREQ      0.4
#define UPDT_FREQ_S    30

#define SCALE_DEFAULT  12.0

/* PRICE_OUT_START may not be less than 1 */
#define PRICE_OUT_START 1

#define CUT_OFF_POWER    0.44
#define CUT_OFF_COEF     1.5
#define CUT_OFF_POWER2   0.75
#define CUT_OFF_COEF2    1
#define CUT_OFF_GAP      0.8
#define CUT_OFF_MIN      12
#define CUT_OFF_INCREASE 4

/*
#define TIME_FOR_PRICE_IN    5
*/
#define TIME_FOR_PRICE_IN1    2
#define TIME_FOR_PRICE_IN2    4
#define TIME_FOR_PRICE_IN3    6

#define EMPTY_PUSH_COEF      1.0
/*
#define MAX_CYCLES_CANCELLED 10
#define START_CYCLE_CANCEL   3
*/
#define MAX_CYCLES_CANCELLED 0
#define START_CYCLE_CANCEL   100
/************************************************ shared macros *******/

#define GREATEROF( x, y ) ( ( (x) > (y) ) ?  x : y )
#define LESSEROF( x, y ) ( ( (x) < (y) ) ? x : y )

#define OPEN( a )   ( a -> r_cap > 0 )
#define CLOSED( a )   ( a -> r_cap <= 0 )
#define REDUCED_COST( i, j, a ) ( (i->price) + dn*(a->cost) - (j->price) )
#define FEASIBLE( i, j, a )     ( (i->price) + dn*(a->cost) < (j->price) )
#define ADMISSIBLE( i, j, a )   ( OPEN(a) && FEASIBLE( i, j, a ) )


#define INCREASE_FLOW( i, j, a, df )\
{\
   (i) -> excess            -= df;\
   (j) -> excess            += df;\
   (a)            -> r_cap  -= df;\
  ((a) -> sister) -> r_cap  += df;\
}\

/*---------------------------------- macros for excess queue */

#define RESET_EXCESS_Q \
{\
   for ( ; excq_first != NULL; excq_first = excq_last )\
     {\
        excq_last            = excq_first -> q_next;\
        excq_first -> q_next = sentinel_node;\
     }\
}

#define OUT_OF_EXCESS_Q( i )  ( i -> q_next == sentinel_node )

#define EMPTY_EXCESS_Q    ( excq_first == NULL )
#define NONEMPTY_EXCESS_Q ( excq_first != NULL )

#define INSERT_TO_EXCESS_Q( i )\
{\
   if ( NONEMPTY_EXCESS_Q )\
     excq_last -> q_next = i;\
   else\
     excq_first  = i;\
\
   i -> q_next = NULL;\
   excq_last   = i;\
}

#define INSERT_TO_FRONT_EXCESS_Q( i )\
{\
   if ( EMPTY_EXCESS_Q )\
     excq_last = i;\
\
   i -> q_next = excq_first;\
   excq_first  = i;\
}

#define REMOVE_FROM_EXCESS_Q( i )\
{\
   i           = excq_first;\
   excq_first  = i -> q_next;\
   i -> q_next = sentinel_node;\
}

/*---------------------------------- excess queue as a stack */

#define EMPTY_STACKQ      EMPTY_EXCESS_Q
#define NONEMPTY_STACKQ   NONEMPTY_EXCESS_Q

#define RESET_STACKQ  RESET_EXCESS_Q

#define STACKQ_PUSH( i )\
{\
   i -> q_next = excq_first;\
   excq_first  = i;\
}

#define STACKQ_POP( i ) REMOVE_FROM_EXCESS_Q( i )

/*------------------------------------ macros for buckets */

node dnd, *dnode;

#define RESET_BUCKET( b )  ( b -> p_first ) = dnode;

#define INSERT_TO_BUCKET( i, b )\
{\
i -> b_next                  = ( b -> p_first );\
( b -> p_first ) -> b_prev   = i;\
( b -> p_first )             = i;\
}

#define NONEMPTY_BUCKET( b ) ( ( b -> p_first ) != dnode )

#define GET_FROM_BUCKET( i, b )\
{\
i    = ( b -> p_first );\
( b -> p_first ) = i -> b_next;\
}

#define REMOVE_FROM_BUCKET( i, b )\
{\
if ( i == ( b -> p_first ) )\
       ( b -> p_first ) = i -> b_next;\
  else\
    {\
       ( i -> b_prev ) -> b_next = i -> b_next;\
       ( i -> b_next ) -> b_prev = i -> b_prev;\
    }\
}

/*------------------------------------------- misc macros */

#define UPDATE_CUT_OFF \
{\
   if (n_bad_pricein + n_bad_relabel == 0) \
     {\
        cut_off_factor = CUT_OFF_COEF2 * pow ( (double)n, CUT_OFF_POWER2 );\
        cut_off_factor = GREATEROF ( cut_off_factor, CUT_OFF_MIN );\
        cut_off        = cut_off_factor * epsilon;\
        cut_on         = cut_off * CUT_OFF_GAP;\
      }\
     else\
       {\
        cut_off_factor *= CUT_OFF_INCREASE;\
        cut_off        = cut_off_factor * epsilon;\
        cut_on         = cut_off * CUT_OFF_GAP;\
        }\
}

#define TIME_FOR_UPDATE \
( n_rel > n * UPDT_FREQ + n_src * UPDT_FREQ_S )

#define FOR_ALL_NODES_i        for ( i = nodes; i != sentinel_node; i ++ )

#define FOR_ALL_ARCS_a_FROM_i \
for ( a = i -> first, a_stop = ( i + 1 ) -> suspended; a != a_stop; a ++ )

#define FOR_ALL_CURRENT_ARCS_a_FROM_i \
for ( a = i -> current, a_stop = ( i + 1 ) -> suspended; a != a_stop; a ++ )

#define WHITE 0
#define GREY  1
#define BLACK 2

arc     *sa, *sb;
long    d_cap;

#define EXCHANGE( a, b )\
{\
if ( a != b )\
  {\
     sa = a -> sister;\
     sb = b -> sister;\
\
     d_arc.r_cap = a -> r_cap;\
     d_arc.cost  = a -> cost;\
     d_arc.head  = a -> head;\
\
     a -> r_cap  = b -> r_cap;\
     a -> cost   = b -> cost;\
     a -> head   = b -> head;\
\
     b -> r_cap  = d_arc.r_cap;\
     b -> cost   = d_arc.cost;\
     b -> head   = d_arc.head;\
\
     if ( a != sb )\
       {\
          b -> sister = sa;\
          a -> sister = sb;\
          sa -> sister = b;\
          sb -> sister = a;\
        }\
\
     d_cap       = cap[a-arcs];\
     cap[a-arcs] = cap[b-arcs];\
     cap[b-arcs] = d_cap;\
  }\
}

#define SUSPENDED( i, a ) ( a < i -> first )



long n_push      =0,
     n_relabel   =0,
     n_discharge =0,
     n_refine    =0,
     n_update    =0,
     n_scan      =0,
     n_prscan    =0,
     n_prscan1   =0,
     n_prscan2   =0,
     n_bad_pricein = 0,
     n_bad_relabel = 0,
     n_prefine   =0;

long   n,                    /* number of nodes */
       m;                    /* number of arcs */

short   *cap;                 /* array containig capacities */

node   *nodes,               /* array of nodes */
       *sentinel_node,       /* next after last */
       *excq_first,          /* first node in push-queue */
       *excq_last;           /* last node in push-queue */

arc    *arcs,                /* array of arcs */
       *sentinel_arc;        /* next after last */

bucket *buckets,             /* array of buckets */
       *l_bucket;            /* last bucket */
long   linf;                 /* number of l_bucket + 1 */
double dlinf;                /* copy of linf in double mode */

int time_for_price_in;
double epsilon,              /* optimality bound */
       low_bound,            /* lowest bound for epsilon */
       price_min,            /* lowest bound for prices */
       f_scale,              /* scale factor */
       dn,                   /* cost multiplier - number of nodes  + 1 */
       mmc,                  /* multiplied maximal cost */
       cut_off_factor,       /* multiplier to produce cut_on and cut_off
                                from n and epsilon */
       cut_on,               /* the bound for returning suspended arcs */
       cut_off;              /* the bound for suspending arcs */

double total_excess;         /* total excess */

long   n_rel,                /* number of relabels from last price update */
       n_ref,                /* current number of refines */
       n_src;                /* current number of nodes with excess */

int   flag_price = 0,        /* if = 1 - signal to start price-in ASAP -
                                maybe there is infeasibility because of
                                susoended arcs */
      flag_updt = 0;         /* if = 1 - update failed some sources are
                                unreachable: either the problem is
                                unfeasible or you have to return
                                suspended arcs */

long  empty_push_bound;      /* maximal possible number of zero pushes
                                during one discharge */

int   snc_max;               /* maximal number of cycles cancelled
                                during price refine */

arc   d_arc;                 /* dummy arc - for technical reasons */

node  d_node,                /* dummy node - for technical reasons */
      *dummy_node;           /* the address of d_node */

/************************************************ abnormal finish **********/

static void err_end ( int cc )
{
fprintf ( sp0, "\ncs2 solver: Error %d ", cc );
if(cc==ALLOCATION_FAULT){
  fprintf(sp0,"(allocation fault)\n");
}else if(cc==UNFEASIBLE){
  fprintf(sp0,"(problem infeasible)\n");
}else if(cc==PRICE_OFL){
  fprintf(sp0,"(price overflow)\n");
}

/*
2 - problem is unfeasible
5 - allocation fault
6 - price overflow
*/

exit(ABNORMAL_EXIT);
/* exit ( cc ); */
}

/************************************************* initialization **********/

static void cs_init (
long    n_p,        /* number of nodes */
long    m_p,        /* number of arcs */
node    *nodes_p,   /* array of nodes */
arc     *arcs_p,    /* array of arcs */
long    f_sc,       /* scaling factor */
double  max_c,      /* maximal cost */
short    *cap_p     /* array of capacities (changed to short by CWC) */
)
{
node   *i;          /* current node */
/*arc    *a;  */        /* current arc */
bucket *b;          /* current bucket */

n             = n_p;
nodes         = nodes_p;
sentinel_node = nodes + n;

m    = m_p;
arcs = arcs_p;
sentinel_arc  = arcs + m;

cap = cap_p;

f_scale = f_sc;

low_bound = 1.00001;

 dn = (double) n ;
 /*
for ( a = arcs ; a != sentinel_arc ; a ++ )
  a -> cost *= dn;
 */

mmc = max_c * dn;

linf   = n * f_scale + 2;
dlinf  = (double)linf;

buckets = (bucket*) CAlloc ( linf, sizeof (bucket) );
if ( buckets == NULL )
   err_end ( ALLOCATION_FAULT );

l_bucket = buckets + linf;

dnode = &dnd;

for ( b = buckets; b != l_bucket; b ++ )
   RESET_BUCKET ( b );

epsilon = mmc;
if ( epsilon < 1 )
  epsilon = 1;

price_min = - PRICE_MAX;

FOR_ALL_NODES_i
  {
    i -> price  = 0;
    i -> suspended = i -> first;
    i -> q_next = sentinel_node;
  }

sentinel_node -> first = sentinel_node -> suspended = sentinel_arc;

cut_off_factor = CUT_OFF_COEF * pow ( (double)n, CUT_OFF_POWER );

cut_off_factor = GREATEROF ( cut_off_factor, CUT_OFF_MIN );

n_ref = 0;

flag_price = 0;

dummy_node = &d_node;

excq_first = NULL;

empty_push_bound = n * EMPTY_PUSH_COEF;

} /* end of initialization */

/********************************************** up_node_scan *************/

static void up_node_scan (
node *i                      /* node for scanning */
)
{
node   *j;                     /* opposite node */
arc    *a,                     /* ( i, j ) */
       *a_stop,                /* first arc from the next node */
       *ra;                    /* ( j, i ) */
bucket *b_old,                 /* old bucket contained j */
       *b_new;                 /* new bucket for j */
long   i_rank,
       j_rank,                 /* ranks of nodes */
       j_new_rank;
double rc,                     /* reduced cost of (j,i) */
       dr;                     /* rank difference */

n_scan ++;

i_rank = i -> rank;

FOR_ALL_ARCS_a_FROM_i
  {

    ra = a -> sister;

    if ( OPEN ( ra ) )
      {
        j = a -> head;
        j_rank = j -> rank;

        if ( j_rank > i_rank )
          {
            if ( ( rc = REDUCED_COST ( j, i, ra ) ) < 0 )
                j_new_rank = i_rank;
            else
              {
                dr = rc / epsilon;
                j_new_rank = ( dr < dlinf ) ? i_rank + (long)dr + 1
                                            : linf;
              }

            if ( j_rank > j_new_rank )
              {
                j -> rank = j_new_rank;
                j -> current = ra;

                if ( j_rank < linf )
                  {
                    b_old = buckets + j_rank;
                    REMOVE_FROM_BUCKET ( j, b_old )
                  }

                b_new = buckets + j_new_rank;
                INSERT_TO_BUCKET ( j, b_new )
              }
          }
      }
  } /* end of scanning arcs */

i -> price -= i_rank * epsilon;
i -> rank = -1;
}


/*************************************************** price_update *******/

static void  price_update ()

{

register node   *i;

double remain;                 /* total excess of unscanned nodes with
                                  positive excess */
bucket *b;                     /* current bucket */
double dp;                     /* amount to be subtracted from prices */

n_update ++;

FOR_ALL_NODES_i
  {

    if ( i -> excess < 0 )
      {
        INSERT_TO_BUCKET ( i, buckets );
        i -> rank = 0;
      }
    else
      {
        i -> rank = linf;
      }
  }

remain = total_excess;
if ( remain < 0.5 ) return;

/* main loop */

for ( b = buckets; b != l_bucket; b ++ )
  {

    while ( NONEMPTY_BUCKET ( b ) )
       {
         GET_FROM_BUCKET ( i, b )

         up_node_scan ( i );

         if ( i -> excess > 0 )
           {
             remain -= (double)(i -> excess);
             if ( remain <= 0  ) break;
           }

       } /* end of scanning the bucket */

    if ( remain <= 0  ) break;
  } /* end of scanning buckets */

if ( remain > 0.5 ) flag_updt = 1;

/* finishup */
/* changing prices for nodes which were not scanned during main loop */

dp = ( b - buckets ) * epsilon;

FOR_ALL_NODES_i
  {

    if ( i -> rank >= 0 )
    {
      if ( i -> rank < linf )
        REMOVE_FROM_BUCKET ( i, (buckets + i -> rank) );

      if ( i -> price > price_min )
        i -> price -= dp;
    }
  }

} /* end of price_update */



/****************************************************** relabel *********/

static int relabel (
register node *i         /* node for relabelling */
)
{
register arc    *a,       /* current arc from  i  */
                *a_stop,  /* first arc from the next node */
                *a_max;   /* arc  which provides maximum price */
register double p_max,    /* current maximal price */
                i_price,  /* price of node  i */
                dp;       /* current arc partial residual cost */

p_max = price_min;
i_price = i -> price;

for (
      a = i -> current + 1, a_stop = ( i + 1 ) -> suspended;
      a != a_stop;
      a ++
    )
  {
    if ( OPEN ( a )
         &&
         ( ( dp = ( ( a -> head ) -> price ) - dn*( a -> cost ) ) > p_max )
       )
      {
        if ( i_price < dp )
          {
            i -> current = a;
            return ( 1 );
          }

        p_max = dp;
        a_max = a;
      }
  } /* 1/2 arcs are scanned */


for (
      a = i -> first, a_stop = ( i -> current ) + 1;
      a != a_stop;
      a ++
    )
  {
    if ( OPEN ( a )
         &&
         ( ( dp = ( ( a -> head ) -> price ) - dn*( a -> cost ) ) > p_max )
       )
      {
        if ( i_price < dp )
          {
            i -> current = a;
            return ( 1 );
          }

        p_max = dp;
        a_max = a;
      }
  } /* 2/2 arcs are scanned */

/* finishup */

if ( p_max != price_min )
  {
    i -> price   = p_max - epsilon;
    i -> current = a_max;
  }
else
  { /* node can't be relabelled */
    if ( i -> suspended == i -> first )
      {
        if ( i -> excess == 0 )
          {
            i -> price = price_min;
          }
        else
          {
            if ( n_ref == 1 )
              {
                err_end ( UNFEASIBLE );
              }
            else
              {
                err_end ( PRICE_OFL );
              }
          }
      }
    else /* node can't be relabelled because of suspended arcs */
      {
        flag_price = 1;
      }
   }


n_relabel ++;
n_rel ++;

return ( 0 );

} /* end of relabel */


/***************************************************** discharge *********/


static void discharge (
register node *i         /* node to be discharged */
)
{

register arc  *a;       /* an arc from  i  */

arc  *b,                /* an arc from j */
     *ra;               /* reversed arc (j,i) */
register node *j;       /* head of  a  */
register long df;       /* amoumt of flow to be pushed through  a  */
excess_t j_exc;             /* former excess of  j  */

int  empty_push;        /* number of unsuccessful attempts to push flow
                           out of  i. If it is too big - it is time for
                           global update */

n_discharge ++;
empty_push = 0;

a = i -> current;
j = a -> head;

if ( !ADMISSIBLE ( i, j, a ) )
  {
    relabel ( i );
    a = i -> current;
    j = a -> head;
  }

while ( 1 )
{
  j_exc = j -> excess;

  if ( j_exc >= 0 )
    {
      b = j -> current;
      if ( ADMISSIBLE ( j, b -> head, b ) || relabel ( j ) )
        { /* exit from j exists */

          df = LESSEROF ( i -> excess, a -> r_cap );
          if (j_exc == 0) n_src++;
          INCREASE_FLOW ( i, j, a, df )
n_push ++;

          if ( OUT_OF_EXCESS_Q ( j ) )
            {
              INSERT_TO_EXCESS_Q ( j );
            }
        }
      else
        {
          /* push back */
          ra = a -> sister;
          df = LESSEROF ( j -> excess, ra -> r_cap );
          if ( df > 0 )
            {
              INCREASE_FLOW ( j, i, ra, df );
              if (j->excess == 0) n_src--;
n_push ++;
            }

          if ( empty_push ++ >= empty_push_bound )
            {
              flag_price = 1;
              return;
            }
        }
    }
  else /* j_exc < 0 */
    {
      df = LESSEROF ( i -> excess, a -> r_cap );
      INCREASE_FLOW ( i, j, a, df )
n_push ++;

      if ( j -> excess >= 0 )
        {
          if ( j -> excess > 0 )
            {
              n_src++;
              relabel ( j );
              INSERT_TO_EXCESS_Q ( j );
            }
          total_excess += j_exc;
        }
      else
        total_excess -= df;

    }

  if (i -> excess <= 0)
    n_src--;
  if ( i -> excess <= 0 || flag_price ) break;

  relabel ( i );

  a = i -> current;
  j = a -> head;
}

i -> current = a;
} /* end of discharge */

/***************************************************** price_in *******/

static int price_in ()

{
node     *i,                   /* current node */
         *j;

arc      *a,                   /* current arc from i */
         *a_stop,              /* first arc from the next node */
         *b,                   /* arc to be exchanged with suspended */
         *ra,                  /* opposite to  a  */
         *rb;                  /* opposite to  b  */

double   rc;                   /* reduced cost */

int      n_in_bad,             /* number of priced_in arcs with
                                  negative reduced cost */
         bad_found;            /* if 1 we are at the second scan
                                  if 0 we are at the first scan */

excess_t  i_exc,                /* excess of  i  */
          df;                   /* an amount to increase flow */


bad_found = 0;
n_in_bad = 0;

 restart:

FOR_ALL_NODES_i
  {
    for ( a = ( i -> first ) - 1, a_stop = ( i -> suspended ) - 1;
    a != a_stop; a -- )
      {
        rc = REDUCED_COST ( i, a -> head, a );

            if ( (rc < 0) && ( a -> r_cap > 0) )
              { /* bad case */
                if ( bad_found == 0 )
                  {
                    bad_found = 1;
                    UPDATE_CUT_OFF;
                    goto restart;

                  }
                df = a -> r_cap;
                INCREASE_FLOW ( i, a -> head, a, df );

                ra = a -> sister;
                j  = a -> head;

                b = -- ( i -> first );
                EXCHANGE ( a, b );

                if ( SUSPENDED ( j, ra ) )
                  {
                    rb = -- ( j -> first );
                    EXCHANGE ( ra, rb );
                  }

                    n_in_bad ++;
              }
            else
            if ( ( rc < cut_on ) && ( rc > -cut_on ) )
              {
                b = -- ( i -> first );
                EXCHANGE ( a, b );
              }
      }
  }

if ( n_in_bad != 0 )
  {
    n_bad_pricein ++;

    /* recalculating excess queue */

    total_excess = 0;
    n_src=0;
    RESET_EXCESS_Q;

      FOR_ALL_NODES_i
        {
          i -> current = i -> first;
          i_exc = i -> excess;
          if ( i_exc > 0 )
            { /* i  is a source */
              total_excess += i_exc;
              n_src++;
              INSERT_TO_EXCESS_Q ( i );
            }
        }

    INSERT_TO_EXCESS_Q ( dummy_node );
  }

if (time_for_price_in == TIME_FOR_PRICE_IN2)
  time_for_price_in = TIME_FOR_PRICE_IN3;

if (time_for_price_in == TIME_FOR_PRICE_IN1)
  time_for_price_in = TIME_FOR_PRICE_IN2;

return ( n_in_bad );

} /* end of price_in */

/************************************************** refine **************/

static void refine ()

{
node     *i;      /* current node */
excess_t i_exc;   /* excess of  i  */

/* long   np, nr, ns; */  /* variables for additional print */

int    pr_in_int;   /* current number of updates between price_in */

/*
np = n_push;
nr = n_relabel;
ns = n_scan;
*/

n_refine ++;
n_ref ++;
n_rel = 0;
pr_in_int = 0;

/* initialize */

total_excess = 0;
n_src=0;
RESET_EXCESS_Q

time_for_price_in = TIME_FOR_PRICE_IN1;

FOR_ALL_NODES_i
  {
    i -> current = i -> first;
    i_exc = i -> excess;
    if ( i_exc > 0 )
      { /* i  is a source */
        total_excess += i_exc;
        n_src++;
        INSERT_TO_EXCESS_Q ( i )
      }
  }


if ( total_excess <= 0 ) return;

/* main loop */

while ( 1 )
  {
    if ( EMPTY_EXCESS_Q )
      {
        if ( n_ref > PRICE_OUT_START )
          {
            price_in ();
          }

        if ( EMPTY_EXCESS_Q ) break;
      }

    REMOVE_FROM_EXCESS_Q ( i );

    /* push all excess out of i */

    if ( i -> excess > 0 )
     {
       discharge ( i );

       if ( TIME_FOR_UPDATE || flag_price )
         {
           if ( i -> excess > 0 )
             {
               INSERT_TO_EXCESS_Q ( i );
             }

           if ( flag_price && ( n_ref > PRICE_OUT_START ) )
             {
               pr_in_int = 0;
               price_in ();
               flag_price = 0;
             }

           price_update();

           while ( flag_updt )
             {
               if ( n_ref == 1 )
                 {
                   err_end ( UNFEASIBLE );
                 }
               else
                 {
                   flag_updt = 0;
                   UPDATE_CUT_OFF;
                   n_bad_relabel++;

                   pr_in_int = 0;
                   price_in ();

                   price_update ();
                 }
             }

           n_rel = 0;

           if ( n_ref > PRICE_OUT_START &&
               (pr_in_int ++ > time_for_price_in)
               )
             {
               pr_in_int = 0;
               price_in ();
             }

         } /* time for update */
     }
  } /* end of main loop */

return;

} /*----- end of refine */


/*************************************************** price_refine **********/

static int price_refine ()

{

node   *i,              /* current node */
       *j,              /* opposite node */
       *ir,             /* nodes for passing over the negative cycle */
       *is;
arc    *a,              /* arc (i,j) */
       *a_stop,         /* first arc from the next node */
       *ar;

long   bmax;            /* number of farest nonempty bucket */
long   i_rank,          /* rank of node i */
       j_rank,          /* rank of node j */
       j_new_rank;      /* new rank of node j */
bucket *b,              /* current bucket */
       *b_old,          /* old and new buckets of current node */
       *b_new;
double rc,              /* reduced cost of a */
       dr,              /* ranks difference */
       dp;
int    cc;              /* return code: 1 - flow is epsilon optimal
                                        0 - refine is needed        */
long   df;              /* cycle capacity */

int    nnc,             /* number of negative cycles cancelled during
                           one iteration */
       snc;             /* total number of negative cycle cancelled */

n_prefine ++;

cc=1;
snc=0;

snc_max = ( n_ref >= START_CYCLE_CANCEL )
          ? MAX_CYCLES_CANCELLED
          : 0;

/* main loop */

while ( 1 )
{ /* while negative cycle is found or eps-optimal solution is constructed */

nnc=0;

FOR_ALL_NODES_i
  {
    i -> rank    = 0;
    i -> inp     = WHITE;
    i -> current = i -> first;
  }

RESET_STACKQ

FOR_ALL_NODES_i
  {
    if ( i -> inp == BLACK ) continue;

    i -> b_next = NULL;

    /* deapth first search */
    while ( 1 )
      {
        i -> inp = GREY;

        /* scanning arcs from node i starting from current */
        FOR_ALL_CURRENT_ARCS_a_FROM_i
          {
            if ( OPEN ( a ) )
              {
                j = a -> head;
                if ( REDUCED_COST ( i, j, a ) < 0 )
                  {
                    if ( j -> inp == WHITE )
                      { /* fresh node  - step forward */
                        i -> current = a;
                        j -> b_next  = i;
                        i = j;
                        a = j -> current;
                        a_stop = (j+1) -> suspended;
                        break;
                      }

                    if ( j -> inp == GREY )
                      { /* cycle detected */
                        cc = 0;
                        nnc++;

                        i -> current = a;
                        is = ir = i;
                        df = BIGGEST_FLOW;

                        while ( 1 )
                          {
                            ar = ir -> current;
                            if ( ar -> r_cap <= df )
                              {
                                df = ar -> r_cap;
                                is = ir;
                              }
                            if ( ir == j ) break;
                            ir = ir -> b_next;
                          }


                        ir = i;

                        while ( 1 )
                          {
                            ar = ir -> current;
                            INCREASE_FLOW( ir, ar -> head, ar, df)

                            if ( ir == j ) break;
                            ir = ir -> b_next;
                          }


                        if ( is != i )
                          {
                            for ( ir = i; ir != is; ir = ir -> b_next )
                              ir -> inp = WHITE;

                            i = is;
                            a = (is -> current) + 1;
                            a_stop = (is+1) -> suspended;
                            break;
                          }

                      }
                  }
                /* if j-color is BLACK - continue search from i */
              }
          } /* all arcs from i are scanned */

        if ( a == a_stop )
          {
            /* step back */
            i -> inp = BLACK;
n_prscan1++;
            j = i -> b_next;
            STACKQ_PUSH ( i );

            if ( j == NULL ) break;
            i = j;
            i -> current ++;
          }

      } /* end of deapth first search */
  } /* all nodes are scanned */

/* no negative cycle */
/* computing longest paths with eps-precision */


snc += nnc;

if ( snc<snc_max ) cc = 1;

if ( cc == 0 ) break;

bmax = 0;

while ( NONEMPTY_STACKQ )
  {
n_prscan2++;
    STACKQ_POP ( i );
    i_rank = i -> rank;
    FOR_ALL_ARCS_a_FROM_i
      {
        if ( OPEN ( a ) )
          {
            j  = a -> head;
            rc = REDUCED_COST ( i, j, a );


            if ( rc < 0 ) /* admissible arc */
              {
                dr = ( - rc - 0.5 ) / epsilon;
                if (( j_rank = dr + i_rank ) < dlinf )
                  {
                    if ( j_rank > j -> rank )
                      j -> rank = j_rank;
                  }
              }
          }
      } /* all arcs from i are scanned */

    if ( i_rank > 0 )
      {
        if ( i_rank > bmax ) bmax = i_rank;
        b = buckets + i_rank;
        INSERT_TO_BUCKET ( i, b )
      }
  } /* end of while-cycle: all nodes are scanned
           - longest distancess are computed */


if ( bmax == 0 ) /* preflow is eps-optimal */
  { break; }

for ( b = buckets + bmax; b != buckets; b -- )
  {
    i_rank = b - buckets;
    dp     = (double)i_rank * epsilon;

    while ( NONEMPTY_BUCKET( b ) )
      {
        GET_FROM_BUCKET ( i, b );

        n_prscan++;
        FOR_ALL_ARCS_a_FROM_i
          {
            if ( OPEN ( a ) )
              {
                j = a -> head;
                j_rank = j -> rank;
                if ( j_rank < i_rank )
                  {
                    rc = REDUCED_COST ( i, j, a );

                    if ( rc < 0 )
                        j_new_rank = i_rank;
                    else
                      {
                        dr = rc / epsilon;
                        j_new_rank = ( dr < dlinf ) ? i_rank - ( (long)dr + 1 )
                                                    : 0;
                      }
                    if ( j_rank < j_new_rank )
                      {
                        if ( cc == 1 )
                          {
                            j -> rank = j_new_rank;

                            if ( j_rank > 0 )
                              {
                                b_old = buckets + j_rank;
                                REMOVE_FROM_BUCKET ( j, b_old )
                                }

                            b_new = buckets + j_new_rank;
                            INSERT_TO_BUCKET ( j, b_new )
                          }
                        else
                          {
                           df = a -> r_cap;
                            INCREASE_FLOW ( i, j, a, df )
                          }
                      }
                  }
              } /* end if opened arc */
          } /* all arcs are scanned */

            i -> price -= dp;

      } /* end of while-cycle: the bucket is scanned */
  } /* end of for-cycle: all buckets are scanned */

if ( cc == 0 ) break;

} /* end of main loop */

/* finish: */

/* if refine needed - saturate non-epsilon-optimal arcs */

if ( cc == 0 )
{
FOR_ALL_NODES_i
  {
    FOR_ALL_ARCS_a_FROM_i
      {
        if ( REDUCED_COST ( i, a -> head, a ) < -epsilon )
          {
            if ( ( df = a -> r_cap ) > 0 )
              {
                INCREASE_FLOW ( i, a -> head, a, df )
              }
          }

      }
  }
}


/*neg_cyc();*/

return ( cc );

} /* end of price_refine */



void compute_prices ()

{

node   *i,              /* current node */
       *j;              /* opposite node */
arc    *a,              /* arc (i,j) */
       *a_stop;         /* first arc from the next node */

long   bmax;            /* number of farest nonempty bucket */
long   i_rank,          /* rank of node i */
       j_rank,          /* rank of node j */
       j_new_rank;      /* new rank of node j */
bucket *b,              /* current bucket */
       *b_old,          /* old and new buckets of current node */
       *b_new;
double rc,              /* reduced cost of a */
       dr,              /* ranks difference */
       dp;
int    cc;              /* return code: 1 - flow is epsilon optimal
                                        0 - refine is needed        */


n_prefine ++;

cc=1;

/* main loop */

while ( 1 )
{ /* while negative cycle is found or eps-optimal solution is constructed */


FOR_ALL_NODES_i
  {
    i -> rank    = 0;
    i -> inp     = WHITE;
    i -> current = i -> first;
  }

RESET_STACKQ

FOR_ALL_NODES_i
  {
    if ( i -> inp == BLACK ) continue;

    i -> b_next = NULL;

    /* deapth first search */
    while ( 1 )
      {
        i -> inp = GREY;

        /* scanning arcs from node i */
        FOR_ALL_ARCS_a_FROM_i
          {
            if ( OPEN ( a ) )
              {
                j = a -> head;
                if ( REDUCED_COST ( i, j, a ) < 0 )
                  {
                    if ( j -> inp == WHITE )
                      { /* fresh node  - step forward */
                        i -> current = a;
                        j -> b_next  = i;
                        i = j;
                        a = j -> current;
                        a_stop = (j+1) -> suspended;
                        break;
                      }

                    if ( j -> inp == GREY )
                      { /* cycle detected; should not happen */
                        cc = 0;
                      }
                  }
                /* if j-color is BLACK - continue search from i */
              }
          } /* all arcs from i are scanned */

        if ( a == a_stop )
          {
            /* step back */
            i -> inp = BLACK;
            n_prscan1++;
            j = i -> b_next;
            STACKQ_PUSH ( i );

            if ( j == NULL ) break;
            i = j;
            i -> current ++;
          }

      } /* end of deapth first search */
  } /* all nodes are scanned */

/* no negative cycle */
/* computing longest paths */

if ( cc == 0 ) break;

bmax = 0;

while ( NONEMPTY_STACKQ )
  {
    n_prscan2++;
    STACKQ_POP ( i );
    i_rank = i -> rank;
    FOR_ALL_ARCS_a_FROM_i
      {
        if ( OPEN ( a ) )
          {
            j  = a -> head;
            rc = REDUCED_COST ( i, j, a );


            if ( rc < 0 ) /* admissible arc */
              {
                dr = - rc;
                if (( j_rank = dr + i_rank ) < dlinf )
                  {
                    if ( j_rank > j -> rank )
                      j -> rank = j_rank;
                  }
              }
          }
      } /* all arcs from i are scanned */

    if ( i_rank > 0 )
      {
        if ( i_rank > bmax ) bmax = i_rank;
        b = buckets + i_rank;
        INSERT_TO_BUCKET ( i, b )
      }
  } /* end of while-cycle: all nodes are scanned
           - longest distancess are computed */


if ( bmax == 0 )
  { break; }

for ( b = buckets + bmax; b != buckets; b -- )
  {
    i_rank = b - buckets;
    dp     = (double) i_rank;

    while ( NONEMPTY_BUCKET( b ) )
      {
        GET_FROM_BUCKET ( i, b )

          n_prscan++;
        FOR_ALL_ARCS_a_FROM_i
          {
            if ( OPEN ( a ) )
              {
                j = a -> head;
                j_rank = j -> rank;
                if ( j_rank < i_rank )
                  {
                    rc = REDUCED_COST ( i, j, a );

                    if ( rc < 0 )
                        j_new_rank = i_rank;
                    else
                      {
                        dr = rc;
                        j_new_rank = ( dr < dlinf ) ? i_rank - ( (long)dr + 1 )
                                                    : 0;
                      }
                    if ( j_rank < j_new_rank )
                      {
                        if ( cc == 1 )
                          {
                            j -> rank = j_new_rank;

                            if ( j_rank > 0 )
                              {
                                b_old = buckets + j_rank;
                                REMOVE_FROM_BUCKET ( j, b_old )
                                }

                            b_new = buckets + j_new_rank;
                            INSERT_TO_BUCKET ( j, b_new )
                          }
                      }
                  }
              } /* end if opened arc */
          } /* all arcs are scanned */

            i -> price -= dp;

      } /* end of while-cycle: the bucket is scanned */
  } /* end of for-cycle: all buckets are scanned */

if ( cc == 0 ) break;

} /* end of main loop */

} /* end of compute_prices */


/***************************************************** price_out ************/

static void price_out ()

{
node     *i;                /* current node */

arc      *a,                /* current arc from i */
         *a_stop,           /* first arc from the next node */
         *b;                /* arc to be exchanged with suspended */

double   n_cut_off,         /* -cut_off */
         rc;                /* reduced cost */

n_cut_off = - cut_off;

FOR_ALL_NODES_i
  {
    FOR_ALL_ARCS_a_FROM_i
      {
        rc = REDUCED_COST ( i, a -> head, a );

        if (((rc > cut_off) && (CLOSED(a -> sister)))
             ||
             ((rc < n_cut_off) && (CLOSED(a)))
           )
          { /* suspend the arc */
            b = ( i -> first ) ++ ;

            EXCHANGE ( a, b );
          }
      }
  }

} /* end of price_out */


/**************************************************** update_epsilon *******/
/*----- decrease epsilon after epsilon-optimal flow is constructed */

static int update_epsilon()
{

if ( epsilon <= low_bound ) return ( 1 );

epsilon = ceil ( epsilon / f_scale );

cut_off        = cut_off_factor * epsilon;
cut_on         = cut_off * CUT_OFF_GAP;

return ( 0 );
}


/*************************************************** finishup ***********/
static void finishup (
double *obj_ad       /* objective */
)
{
arc   *a;            /* current arc */
long  na;            /* corresponding position in capacity array */
double  obj_internal;/* objective */
double cs;           /* actual arc cost */
long   flow;         /* flow through an arc */

obj_internal = 0;

for ( a = arcs, na = 0; a != sentinel_arc ; a ++, na ++ )
    {
      /*      cs = a -> cost / dn;  */
      cs = a -> cost;

      if ( cap[na]  > 0 && ( flow = cap[na] - (a -> r_cap) ) != 0 )
        obj_internal += cs * (double) flow;

      /*       a -> cost = cs;  */
    }

*obj_ad = obj_internal;

}


/*********************************************** init_solution ***********/
/*  static void init_solution ( ) */


/*  { */
/*  arc   *a; */   /* current arc  (i,j) */
/*  node  *i, */   /* tail of  a  */
/*        *j; */   /* head of  a  */
/*  long  df; */   /* ricidual capacity */

/*  for ( a = arcs; a != sentinel_arc ; a ++ ) */
/*      { */
/*        if ( a -> r_cap > 0 && a -> cost < 0 ) */
/*      { */
/*        df = a -> r_cap; */
/*        i  = ( a -> sister ) -> head; */
/*            j  = a -> head; */
/*        INCREASE_FLOW ( i, j, a, df ); */
/*      } */
/*      } */
/*  } */

  /* check complimentary slackness */
/*  int check_cs () */

/*  { */
/*    node *i; */
/*    arc *a, *a_stop; */

/*    FOR_ALL_NODES_i */
/*      FOR_ALL_ARCS_a_FROM_i */
/*        if (OPEN(a) && (REDUCED_COST(i, a->head, a) < 0)) */
/*      assert(0); */

/*    return(1); */
/*  } */

/************************************************* cs2 - head program ***/

static void  cs2 (
long    n_p,        /* number of nodes */
long     m_p,        /* number of arcs */
node    *nodes_p,   /* array of nodes */
arc     *arcs_p,    /* array of arcs */
long    f_sc,       /* scaling factor */
double  max_c,      /* maximal cost */
short   *cap_p,     /* capacities (changed to short by CWC) */
double  *obj_ad    /* objective */
)
{

int cc;             /* for storing return code */
cs_init ( n_p, m_p, nodes_p, arcs_p, f_sc, max_c, cap_p );

/*init_solution ( );*/
cc = 0;
update_epsilon ();

do{  /* scaling loop */

    refine ();

    if ( n_ref >= PRICE_OUT_START )
      {
        price_out ( );
      }

    if ( update_epsilon () ) break;

    while ( 1 )
      {
        if ( ! price_refine () ) break;

        if ( n_ref >= PRICE_OUT_START )
          {
            if ( price_in () )
              {
                break;
              }
          }
        if ((cc = update_epsilon ())) break;
      }
  } while ( cc == 0 );

finishup ( obj_ad );

}

/*-----------------------------------------------------------------------*/

/* SolveCS2-- formerly main() */

void SolveCS2(signed char **residue, short **mstcosts, long nrow, long ncol,
              long cs2scalefactor, short ***flowsptr)
{

  /*  double t; */
  arc *arp;
  node *ndp;
  long n, m, m2, nmin;
  node *i;
  long ni;
  arc *a;
  long nNrow, nNcol;
  long to, from, num, flow, ground;
  long f_sc;

  double cost,  c_max;
  short *cap;  /* cap changed to short by CWC */

  short **rowcost, **colcost;
  short **rowflow, **colflow;

  /* number of rows, cols, in residue network */
  nNrow=nrow-1;
  nNcol=ncol-1;
  ground=nNrow*nNcol+1;

  /* parse input, set up the problem */
  rowcost=mstcosts;
  colcost=&(mstcosts[nrow-1]);
  f_sc=cs2scalefactor;
  cs2mcfparse( residue,rowcost,colcost,nNrow,nNcol,
               &n,&m,&ndp,&arp,&nmin,&c_max,&cap );

  /* free memory that is no longer needed */
  Free2DArray((void **)residue,nrow-1);
  Free2DArray((void **)mstcosts,2*nrow-1);

  /* solve it! */
  fprintf(sp2,"Running cs2 MCF solver\n");
  m2 = 2 * m;
  cs2 ( n, m2, ndp, arp, f_sc, c_max, cap, &cost );


  /* parse flow solution and place into flow arrays */

  /* get memory for flow arrays */
  (*flowsptr)=(short **)Get2DRowColZeroMem(nrow,ncol,
                                           sizeof(short *),sizeof(short));
  rowflow=(*flowsptr);
  colflow=&((*flowsptr)[nrow-1]);

  /* loop over nodes */
  for ( i = ndp; i < ndp + n; i ++ ){
    ni = N_NODE ( i );

    /* loop over arcs */
    for ( a = i -> suspended; a != (i+1)->suspended; a ++ ){

      /* if finite (non-zero) flow */
      if ( cap[ N_ARC (a) ]  > 0 &&  (cap[ N_ARC (a) ] - ( a -> r_cap ) ) ){

        /* get to, from nodes and flow amount */
        from=ni;
        to=N_NODE( a -> head );
        flow=cap[ N_ARC (a) ] - ( a -> r_cap );

        if(flow>LARGESHORT || flow<-LARGESHORT){
          fprintf(sp0,"Flow will overflow short data type\nAbort\n");
          exit(ABNORMAL_EXIT);
        }

        /* node indices are indexed from 1, not 0 */
        /* node indices are in column major order, not row major */
        /* handle flow to/from ground first */
        if((from==ground) || (to==ground)){
          if(to==ground){
            num=to;
            to=from;
            from=num;
            flow=-flow;
          }
          if(!((to-1) % nNrow)){
            colflow[0][(int )((to-1)/nNrow)]+=flow;
          }else if(to<=nNrow){
            rowflow[to-1][0]+=flow;
          }else if(to>=(ground-nNrow-1)){
            rowflow[(to-1) % nNrow][nNcol]-=flow;
          }else if(!(to % nNrow)){
            colflow[nNrow][(int )((to/nNrow)-1)]-=flow;
          }else{
            fprintf(sp0,"Unassigned ground arc parsing cs2 solution\nAbort\n");
            exit(ABNORMAL_EXIT);
          }
        }else if(from==(to+1)){
          num=from+(int )((from-1)/nNrow);
          colflow[(num-1) % (nNrow+1)][(int )(num-1)/(nNrow+1)]-=flow;
        }else if(from==(to-1)){
          num=from+(int )((from-1)/nNrow)+1;
          colflow[(num-1) % (nNrow+1)][(int )(num-1)/(nNrow+1)]+=flow;
        }else if(from==(to-nNrow)){
          num=from+nNrow;
          rowflow[(num-1) % nNrow][(int )((num-1)/nNrow)]+=flow;
        }else if(from==(to+nNrow)){
          num=from;
          rowflow[(num-1) % nNrow][(int )((num-1)/nNrow)]-=flow;
        }else{
          fprintf(sp0,"Non-grid arc parsing cs2 solution\nAbort\n");
          exit(ABNORMAL_EXIT);
        }
      } /* end if flow on arc */

    } /* end for loop over arcs of node */
  } /* end for loop over nodes */

  /* free memory */
  free(ndp-nmin);
  free(arp);
  free(cap);
  free(buckets);

}

#endif  /* end #ifndef NO_CS2 */

/* end snaphu_cs2.c */
/* snaphu_io.c */
/*************************************************************************

  snaphu input/output source file
  Written by Curtis W. Chen
  Copyright 2002 Board of Trustees, Leland Stanford Jr. University
  Please see the supporting documentation for terms of use.
  No warranty.

*************************************************************************/



/* ouput stream pointers */
/* sp0=error messages, sp1=status output, sp2=verbose, sp3=verbose counter */
extern FILE *sp0, *sp1, *sp2, *sp3;
/* static (local) function prototypes */
static
int ParseConfigLine(char *buf, char *conffile, long nlines,
                    infileT *infiles, outfileT *outfiles,
                    long *linelenptr, paramT *params);
static
int LogStringParam(FILE *fp, char *key, char *value);
static
int LogBoolParam(FILE *fp, char *key, signed char boolvalue);
static
int LogFileFormat(FILE *fp, char *key, signed char fileformat);
static
int WriteAltLineFile(float **mag, float **phase, char *outfile,
                     long nrow, long ncol);
static
int WriteAltSampFile(float **arr1, float **arr2, char *outfile,
                     long nrow, long ncol);



/* function: SetDefaults()
 * -----------------------
 * Sets all parameters to their initial default values.
 */
int SetDefaults(infileT *infiles, outfileT *outfiles, paramT *params){


  /* initialize to all zero to start for extra robustness */
  memset(infiles,0,sizeof(infileT));
  memset(outfiles,0,sizeof(outfileT));
  memset(params,0,sizeof(paramT));

  /* input files */
  StrNCopy(infiles->weightfile,DEF_WEIGHTFILE,MAXSTRLEN);
  StrNCopy(infiles->corrfile,DEF_CORRFILE,MAXSTRLEN);
  StrNCopy(infiles->ampfile,DEF_AMPFILE,MAXSTRLEN);
  StrNCopy(infiles->ampfile2,DEF_AMPFILE2,MAXSTRLEN);
  StrNCopy(infiles->estfile,DEF_ESTFILE,MAXSTRLEN);
  StrNCopy(infiles->magfile,DEF_MAGFILE,MAXSTRLEN);
  StrNCopy(infiles->costinfile,DEF_COSTINFILE,MAXSTRLEN);
  StrNCopy(infiles->bytemaskfile,DEF_BYTEMASKFILE,MAXSTRLEN);
  StrNCopy(infiles->dotilemaskfile,DEF_DOTILEMASKFILE,MAXSTRLEN);

  /* output and dump files */
  StrNCopy(outfiles->initfile,DEF_INITFILE,MAXSTRLEN);
  StrNCopy(outfiles->flowfile,DEF_FLOWFILE,MAXSTRLEN);
  StrNCopy(outfiles->eifile,DEF_EIFILE,MAXSTRLEN);
  StrNCopy(outfiles->rowcostfile,DEF_ROWCOSTFILE,MAXSTRLEN);
  StrNCopy(outfiles->colcostfile,DEF_COLCOSTFILE,MAXSTRLEN);
  StrNCopy(outfiles->mstrowcostfile,DEF_MSTROWCOSTFILE,MAXSTRLEN);
  StrNCopy(outfiles->mstcolcostfile,DEF_MSTCOLCOSTFILE,MAXSTRLEN);
  StrNCopy(outfiles->mstcostsfile,DEF_MSTCOSTSFILE,MAXSTRLEN);
  StrNCopy(outfiles->corrdumpfile,DEF_CORRDUMPFILE,MAXSTRLEN);
  StrNCopy(outfiles->rawcorrdumpfile,DEF_RAWCORRDUMPFILE,MAXSTRLEN);
  StrNCopy(outfiles->costoutfile,DEF_COSTOUTFILE,MAXSTRLEN);
  StrNCopy(outfiles->conncompfile,DEF_CONNCOMPFILE,MAXSTRLEN);
  StrNCopy(outfiles->outfile,DEF_OUTFILE,MAXSTRLEN);
  StrNCopy(outfiles->logfile,DEF_LOGFILE,MAXSTRLEN);

  /* file formats */
  infiles->infileformat=DEF_INFILEFORMAT;
  infiles->unwrappedinfileformat=DEF_UNWRAPPEDINFILEFORMAT;
  infiles->magfileformat=DEF_MAGFILEFORMAT;
  infiles->corrfileformat=DEF_CORRFILEFORMAT;
  infiles->estfileformat=DEF_ESTFILEFORMAT;
  infiles->ampfileformat=DEF_AMPFILEFORMAT;
  outfiles->outfileformat=DEF_OUTFILEFORMAT;

  /* options and such */
  params->unwrapped=DEF_UNWRAPPED;
  params->regrowconncomps=DEF_REGROWCONNCOMPS;
  params->eval=DEF_EVAL;
  params->initonly=DEF_INITONLY;
  params->initmethod=DEF_INITMETHOD;
  params->costmode=DEF_COSTMODE;
  params->amplitude=DEF_AMPLITUDE;
  params->verbose=DEF_VERBOSE;

  /* SAR and geometry parameters */
  params->orbitradius=DEF_ORBITRADIUS;
  params->altitude=DEF_ALTITUDE;
  params->earthradius=DEF_EARTHRADIUS;
  params->bperp=DEF_BPERP;
  params->transmitmode=DEF_TRANSMITMODE;
  params->baseline=DEF_BASELINE;
  params->baselineangle=DEF_BASELINEANGLE;
  params->nlooksrange=DEF_NLOOKSRANGE;
  params->nlooksaz=DEF_NLOOKSAZ;
  params->nlooksother=DEF_NLOOKSOTHER;
  params->ncorrlooks=DEF_NCORRLOOKS;
  params->ncorrlooksrange=DEF_NCORRLOOKSRANGE;
  params->ncorrlooksaz=DEF_NCORRLOOKSAZ;
  params->nearrange=DEF_NEARRANGE;
  params->dr=DEF_DR;
  params->da=DEF_DA;
  params->rangeres=DEF_RANGERES;
  params->azres=DEF_AZRES;
  params->lambda=DEF_LAMBDA;

  /* scattering model parameters */
  params->kds=DEF_KDS;
  params->specularexp=DEF_SPECULAREXP;
  params->dzrcritfactor=DEF_DZRCRITFACTOR;
  params->shadow=DEF_SHADOW;
  params->dzeimin=DEF_DZEIMIN;
  params->laywidth=DEF_LAYWIDTH;
  params->layminei=DEF_LAYMINEI;
  params->sloperatiofactor=DEF_SLOPERATIOFACTOR;
  params->sigsqei=DEF_SIGSQEI;

  /* decorrelation model parameters */
  params->drho=DEF_DRHO;
  params->rhosconst1=DEF_RHOSCONST1;
  params->rhosconst2=DEF_RHOSCONST2;
  params->cstd1=DEF_CSTD1;
  params->cstd2=DEF_CSTD2;
  params->cstd3=DEF_CSTD3;
  params->defaultcorr=DEF_DEFAULTCORR;
  params->rhominfactor=DEF_RHOMINFACTOR;

  /* pdf model parameters */
  params->dzlaypeak=DEF_DZLAYPEAK;
  params->azdzfactor=DEF_AZDZFACTOR;
  params->dzeifactor=DEF_DZEIFACTOR;
  params->dzeiweight=DEF_DZEIWEIGHT;
  params->dzlayfactor=DEF_DZLAYFACTOR;
  params->layconst=DEF_LAYCONST;
  params->layfalloffconst=DEF_LAYFALLOFFCONST;
  params->sigsqshortmin=DEF_SIGSQSHORTMIN;
  params->sigsqlayfactor=DEF_SIGSQLAYFACTOR;

  /* deformation mode parameters */
  params->defoazdzfactor=DEF_DEFOAZDZFACTOR;
  params->defothreshfactor=DEF_DEFOTHRESHFACTOR;
  params->defomax=DEF_DEFOMAX;
  params->sigsqcorr=DEF_SIGSQCORR;
  params->defolayconst=DEF_DEFOLAYCONST;

  /* algorithm parameters */
  params->flipphasesign=DEF_FLIPPHASESIGN;
  params->onetilereopt=DEF_ONETILEREOPT;
  params->rmtileinit=DEF_RMTILEINIT;
  params->initmaxflow=DEF_INITMAXFLOW;
  params->arcmaxflowconst=DEF_ARCMAXFLOWCONST;
  params->maxflow=DEF_MAXFLOW;
  params->krowei=DEF_KROWEI;
  params->kcolei=DEF_KCOLEI;
  params->kperpdpsi=DEF_KPERPDPSI;
  params->kpardpsi=DEF_KPARDPSI;
  params->threshold=DEF_THRESHOLD;
  params->initdzr=DEF_INITDZR;
  params->initdzstep=DEF_INITDZSTEP;
  params->maxcost=DEF_MAXCOST;
  params->costscale=DEF_COSTSCALE;
  params->costscaleambight=DEF_COSTSCALEAMBIGHT;
  params->dnomincangle=DEF_DNOMINCANGLE;
  params->srcrow=DEF_SRCROW;
  params->srccol=DEF_SRCCOL;
  params->p=DEF_P;
  params->bidirlpn=DEF_BIDIRLPN;
  params->nshortcycle=DEF_NSHORTCYCLE;
  params->maxnewnodeconst=DEF_MAXNEWNODECONST;
  params->maxcyclefraction=DEF_MAXCYCLEFRACTION;
  params->nconnnodemin=DEF_NCONNNODEMIN;
  params->maxnflowcycles=DEF_MAXNFLOWCYCLES;
  params->dumpall=DEF_DUMPALL;
  params->cs2scalefactor=DEF_CS2SCALEFACTOR;
  params->nmajorprune=DEF_NMAJORPRUNE;
  params->prunecostthresh=DEF_PRUNECOSTTHRESH;
  params->edgemasktop=DEF_EDGEMASKTOP;
  params->edgemaskbot=DEF_EDGEMASKBOT;
  params->edgemaskleft=DEF_EDGEMASKLEFT;
  params->edgemaskright=DEF_EDGEMASKRIGHT;
  params->parentpid=(long )getpid();

  /* tile parameters */
  params->ntilerow=DEF_NTILEROW;
  params->ntilecol=DEF_NTILECOL;
  params->rowovrlp=DEF_ROWOVRLP;
  params->colovrlp=DEF_COLOVRLP;
  params->piecefirstrow=DEF_PIECEFIRSTROW;
  params->piecefirstcol=DEF_PIECEFIRSTCOL;
  params->piecenrow=DEF_PIECENROW;
  params->piecencol=DEF_PIECENCOL;
  params->tilecostthresh=DEF_TILECOSTTHRESH;
  params->minregionsize=DEF_MINREGIONSIZE;
  params->nthreads=DEF_NTHREADS;
  params->scndryarcflowmax=DEF_SCNDRYARCFLOWMAX;
  StrNCopy(params->tiledir,DEF_TILEDIR,MAXSTRLEN);
  params->assembleonly=DEF_ASSEMBLEONLY;
  params->rmtmptile=DEF_RMTMPTILE;
  params->tileedgeweight=DEF_TILEEDGEWEIGHT;

  /* connected component parameters */
  params->minconncompfrac=DEF_MINCONNCOMPFRAC;
  params->conncompthresh=DEF_CONNCOMPTHRESH;
  params->maxncomps=DEF_MAXNCOMPS;
  params->conncompouttype=DEF_CONNCOMPOUTTYPE;

  /* done */
  return(0);

}


/* function: ProcessArgs()
 * -----------------------
 * Parses command line inputs passed to main().
 */
int ProcessArgs(int argc, char *argv[], infileT *infiles, outfileT *outfiles,
                long *linelenptr, paramT *params){

  long i,j;
  signed char noarg_exit;

  /* required inputs */
  noarg_exit=FALSE;
  StrNCopy(infiles->infile,"",MAXSTRLEN);
  *linelenptr=0;

  /* loop over inputs */
  if(argc<2){                             /* catch zero arguments in */
    fprintf(sp1,OPTIONSHELPBRIEF);
    exit(ABNORMAL_EXIT);
  }
  for(i=1;i<argc;i++){
    /* if argument is an option */
    if(argv[i][0]=='-'){
      if(strlen(argv[i])==1){
        fflush(NULL);
        fprintf(sp0,"invalid command line argument -\n");
        exit(ABNORMAL_EXIT);
      }else if(argv[i][1]!='-'){
        for(j=1;j<strlen(argv[i]);j++){
          if(argv[i][j]=='h'){
            fprintf(sp1,OPTIONSHELPFULL);
            exit(ABNORMAL_EXIT);
          }else if(argv[i][j]=='u'){
            params->unwrapped=TRUE;
          }else if(argv[i][j]=='t'){
            params->costmode=TOPO;
          }else if(argv[i][j]=='d'){
            params->costmode=DEFO;
          }else if(argv[i][j]=='s'){
            params->costmode=SMOOTH;
            params->defomax=0.0;
          }else if(argv[i][j]=='q'){
            params->eval=TRUE;
            params->unwrapped=TRUE;
          }else if(argv[i][j]=='C'){
            if(++i<argc && j==strlen(argv[i-1])-1){

              /* parse argument as configuration line */
              ParseConfigLine(argv[i],"command line",0,
                              infiles,outfiles,linelenptr,params);
              break;
            }else{
              noarg_exit=TRUE;
            }
          }else if(argv[i][j]=='f'){
            if(++i<argc && j==strlen(argv[i-1])-1){

              /* read user-supplied configuration file */
              ReadConfigFile(argv[i],infiles,outfiles,linelenptr,params);
              break;
            }else{
              noarg_exit=TRUE;
            }
          }else if(argv[i][j]=='o'){
            if(++i<argc && j==strlen(argv[i-1])-1){
              StrNCopy(outfiles->outfile,argv[i],MAXSTRLEN);
              break;
            }else{
              noarg_exit=TRUE;
            }
          }else if(argv[i][j]=='c'){
            if(++i<argc && j==strlen(argv[i-1])-1){
              StrNCopy(infiles->corrfile,argv[i],MAXSTRLEN);
              break;
            }else{
              noarg_exit=TRUE;
            }
          }else if(argv[i][j]=='m'){
            if(++i<argc && j==strlen(argv[i-1])-1){
              StrNCopy(infiles->magfile,argv[i],MAXSTRLEN);
              break;
            }else{
              noarg_exit=TRUE;
            }
          }else if(argv[i][j]=='M'){
            if(++i<argc && j==strlen(argv[i-1])-1){
              StrNCopy(infiles->bytemaskfile,argv[i],MAXSTRLEN);
              break;
            }else{
              noarg_exit=TRUE;
            }
          }else if(argv[i][j]=='a'){
            if(++i<argc && j==strlen(argv[i-1])-1){
              StrNCopy(infiles->ampfile,argv[i],MAXSTRLEN);
              params->amplitude=TRUE;
              break;
            }else{
              noarg_exit=TRUE;
            }
          }else if(argv[i][j]=='A'){
            if(++i<argc && j==strlen(argv[i-1])-1){
              StrNCopy(infiles->ampfile,argv[i],MAXSTRLEN);
              params->amplitude=FALSE;
              break;
            }else{
              noarg_exit=TRUE;
            }
          }else if(argv[i][j]=='e'){
            if(++i<argc && j==strlen(argv[i-1])-1){
              StrNCopy(infiles->estfile,argv[i],MAXSTRLEN);
              break;
            }else{
              noarg_exit=TRUE;
            }
          }else if(argv[i][j]=='w'){
            if(++i<argc && j==strlen(argv[i-1])-1){
              StrNCopy(infiles->weightfile,argv[i],MAXSTRLEN);
              break;
            }else{
              noarg_exit=TRUE;
            }
          }else if(argv[i][j]=='g'){
            if(++i<argc && j==strlen(argv[i-1])-1){
              StrNCopy(outfiles->conncompfile,argv[i],MAXSTRLEN);
              break;
            }else{
              noarg_exit=TRUE;
            }
          }else if(argv[i][j]=='G'){
            params->regrowconncomps=TRUE;
            if(++i<argc && j==strlen(argv[i-1])-1){
              StrNCopy(outfiles->conncompfile,argv[i],MAXSTRLEN);
              break;
            }else{
              noarg_exit=TRUE;
            }
          }else if(argv[i][j]=='b'){
            if(++i<argc && j==strlen(argv[i-1])-1){
              if(StringToDouble(argv[i],&(params->bperp)) || !(params->bperp)){
                fflush(NULL);
                fprintf(sp0,"option -%c requires non-zero decimal argument\n",
                        argv[i-1][j]);
                exit(ABNORMAL_EXIT);
              }
              break;
            }else{
              noarg_exit=TRUE;
            }
          }else if(argv[i][j]=='p'){
            if(++i<argc && j==strlen(argv[i-1])-1){
              if(StringToDouble(argv[i],&(params->p))){
                fflush(NULL);
                fprintf(sp0,"option -%c requires decimal argument\n",
                        argv[i-1][j]);
                exit(ABNORMAL_EXIT);
              }
              break;
            }else{
              noarg_exit=TRUE;
            }
          }else if(argv[i][j]=='i'){
            params->initonly=TRUE;
          }else if(argv[i][j]=='S'){
            params->onetilereopt=TRUE;
          }else if(argv[i][j]=='k'){
            params->rmtmptile=FALSE;
            params->rmtileinit=FALSE;
          }else if(argv[i][j]=='n'){
            params->costmode=NOSTATCOSTS;
          }else if(argv[i][j]=='v'){
            params->verbose=TRUE;
          }else if(argv[i][j]=='l'){
            if(++i<argc && j==strlen(argv[i-1])-1){
              StrNCopy(outfiles->logfile,argv[i],MAXSTRLEN);
              break;
            }else{
              noarg_exit=TRUE;
            }
          }else{
            fflush(NULL);
            fprintf(sp0,"unrecognized option -%c\n",argv[i][j]);
            exit(ABNORMAL_EXIT);
          }
          if(noarg_exit){
            fflush(NULL);
            fprintf(sp0,"option -%c requires an argument\n",argv[i-1][j]);
            exit(ABNORMAL_EXIT);
          }
        }
      }else{
        /* argument is a "--" option */
        if(!strcmp(argv[i],"--costinfile")){
          if(++i<argc){
            StrNCopy(infiles->costinfile,argv[i],MAXSTRLEN);
          }else{
            noarg_exit=TRUE;
          }
        }else if(!strcmp(argv[i],"--costoutfile")){
          if(++i<argc){
            StrNCopy(outfiles->costoutfile,argv[i],MAXSTRLEN);
          }else{
            noarg_exit=TRUE;
          }
        }else if(!strcmp(argv[i],"--debug") || !strcmp(argv[i],"--dumpall")){
          params->dumpall=TRUE;
        }else if(!strcmp(argv[i],"--mst")){
          params->initmethod=MSTINIT;
        }else if(!strcmp(argv[i],"--mcf")){
          params->initmethod=MCFINIT;
        }else if(!strcmp(argv[i],"--aa")){
          if(i+2<argc){
            StrNCopy(infiles->ampfile,argv[++i],MAXSTRLEN);
            StrNCopy(infiles->ampfile2,argv[++i],MAXSTRLEN);
            infiles->ampfileformat=FLOAT_DATA;
            params->amplitude=TRUE;
          }else{
            noarg_exit=TRUE;
          }
        }else if(!strcmp(argv[i],"--AA")){
          if(++i+1<argc){
            StrNCopy(infiles->ampfile,argv[i++],MAXSTRLEN);
            StrNCopy(infiles->ampfile2,argv[i],MAXSTRLEN);
            infiles->ampfileformat=FLOAT_DATA;
            params->amplitude=FALSE;
          }else{
            noarg_exit=TRUE;
          }
        }else if(!strcmp(argv[i],"--tile")){
          if(++i+3<argc){
            if(StringToLong(argv[i++],&(params->ntilerow))
               || StringToLong(argv[i++],&(params->ntilecol))
               || StringToLong(argv[i++],&(params->rowovrlp))
               || StringToLong(argv[i],&(params->colovrlp))){
              fflush(NULL);
              fprintf(sp0,"option %s requires four integer arguments\n",
                      argv[i-4]);
              exit(ABNORMAL_EXIT);
            }
          }else{
            noarg_exit=TRUE;
          }
        }else if(!strcmp(argv[i],"--piece")){
          if(++i+3<argc){
            if(StringToLong(argv[i++],&(params->piecefirstrow))
               || StringToLong(argv[i++],&(params->piecefirstcol))
               || StringToLong(argv[i++],&(params->piecenrow))
               || StringToLong(argv[i],&(params->piecencol))){
              fflush(NULL);
              fprintf(sp0,"option %s requires four integer arguments\n",
                      argv[i-4]);
              exit(ABNORMAL_EXIT);
            }
          }else{
            noarg_exit=TRUE;
          }
        }else if(!strcmp(argv[i],"--nproc")){
          if(++i<argc){
            if(StringToLong(argv[i],&(params->nthreads))){
              fflush(NULL);
              fprintf(sp0,"option %s requires an integer arguemnt\n",
                      argv[i-1]);
              exit(ABNORMAL_EXIT);
            }
          }else{
            noarg_exit=TRUE;
          }
        }else if(!strcmp(argv[i],"--tiledir")){
          if(++i<argc){
            StrNCopy(params->tiledir,argv[i],MAXSTRLEN);
          }else{
            noarg_exit=TRUE;
          }
        }else if(!strcmp(argv[i],"--assemble")){
          params->assembleonly=TRUE;
        }else if(!strcmp(argv[i],"--copyright") || !strcmp(argv[i],"--info")){
          fprintf(sp1,COPYRIGHT);
          exit(ABNORMAL_EXIT);
        }else if(!strcmp(argv[i],"--help")){
          fflush(NULL);
          fprintf(sp1,OPTIONSHELPFULL);
          exit(ABNORMAL_EXIT);
        }else{
          fflush(NULL);
          fprintf(sp0,"unrecognized option %s\n",argv[i]);
          exit(ABNORMAL_EXIT);
        }
        if(noarg_exit){
          fflush(NULL);
          fprintf(sp0,"incorrect number of arguments for option %s\n",
                  argv[i-1]);
          exit(ABNORMAL_EXIT);
        }
      }
    }else{
      /* argument is not an option */
      if(!strlen(infiles->infile)){
        StrNCopy(infiles->infile,argv[i],MAXSTRLEN);
      }else if(*linelenptr==0){
        if(StringToLong(argv[i],linelenptr) || *linelenptr<=0){
          fflush(NULL);
          fprintf(sp0,"line length must be positive integer\n");
          exit(ABNORMAL_EXIT);
        }
      }else{
        fflush(NULL);
        fprintf(sp0,"multiple input files: %s and %s\n",
                infiles->infile,argv[i]);
        exit(ABNORMAL_EXIT);
      }
    }
  } /* end for loop over arguments */

  /* check to make sure we have required arguments */
  if(!strlen(infiles->infile) || !(*linelenptr)){
    fflush(NULL);
    fprintf(sp0,"not enough input arguments.  type %s -h for help\n",
            PROGRAMNAME);
    exit(ABNORMAL_EXIT);
  }

  /* done */
  return(0);

} /* end of ProcessArgs */


/* function: CheckParams()
 * -----------------------
 * Checks all parameters to make sure they are valid.  This is just a boring
 * function with lots of checks in it.
 */
int CheckParams(infileT *infiles, outfileT *outfiles,
                long linelen, long nlines, paramT *params){

  long ni, nj, n;
  FILE *fp;

  /* make sure output file is writable (try opening in append mode) */
  /* file will be opened in write mode later, clobbering existing file */
  if((fp=fopen(outfiles->outfile,"a"))==NULL){
    fflush(NULL);
    fprintf(sp0,"file %s is not writable\n",outfiles->outfile);
    exit(ABNORMAL_EXIT);
  }else{
    if(ftell(fp)){
      fclose(fp);
    }else{
      fclose(fp);
      remove(outfiles->outfile);
    }
    if(!strcmp(outfiles->outfile,infiles->infile)
       && !params->eval && !params->regrowconncomps){
      fflush(NULL);
      fprintf(sp0,"WARNING: output will overwrite input\n");
    }
  }

  /* make sure options aren't contradictory */
  if(params->initonly && params->unwrapped){
    fflush(NULL);
    fprintf(sp0,"cannot use initialize-only mode with unwrapped input\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->initonly && params->p>=0){
    fflush(NULL);
    fprintf(sp0,"cannot use initialize-only mode with Lp costs\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->costmode==NOSTATCOSTS && !(params->initonly || params->p>=0)){
    fflush(NULL);
    fprintf(sp0,"no-statistical-costs option can only be used in\n");
    fprintf(sp0,"  initialize-only or Lp-norm modes\n");
    exit(ABNORMAL_EXIT);
  }
  if(strlen(infiles->costinfile) && params->costmode==NOSTATCOSTS){
    fflush(NULL);
    fprintf(sp0,"no-statistical-costs option cannot be given\n");
    fprintf(sp0,"  if input cost file is specified\n");
    exit(ABNORMAL_EXIT);
  }
  if(strlen(outfiles->costoutfile) && params->costmode==NOSTATCOSTS){
    fflush(NULL);
    fprintf(sp0,"no-statistical-costs option cannot be given\n");
    fprintf(sp0,"  if output cost file is specified\n");
    exit(ABNORMAL_EXIT);
  }

  /* check geometry parameters */
  if(params->earthradius<=0){
    fflush(NULL);
    fprintf(sp0,"earth radius must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->altitude){
    if(params->altitude>0){
      params->orbitradius=params->earthradius+params->altitude;
    }else{
      fflush(NULL);
      fprintf(sp0,"platform altitude must be positive\n");
      exit(ABNORMAL_EXIT);
    }
  }else if(params->orbitradius < params->earthradius){
    fflush(NULL);
    fprintf(sp0,"platform orbit radius must be greater than earth radius\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->costmode==TOPO && params->baseline<0){
    fflush(NULL);
    fprintf(sp0,"baseline length must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->costmode==TOPO && params->baseline==0){
    fflush(NULL);
    fprintf(sp0,"WARNING: zero baseline may give unpredictable results\n");
  }
  if(params->ncorrlooks<=0){
    fflush(NULL);
    fprintf(sp0,"number of looks ncorrlooks must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->nearrange<=0){
    fflush(NULL);
    fprintf(sp0,"slant range parameter nearrange must be positive (meters)\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->dr<=0 || params->da<=0){
    fflush(NULL);
    fprintf(sp0,"pixel spacings dr and da must be positive (meters)\n");
    exit(ABNORMAL_EXIT);
  }
  /* dr and da after multilooking can be larger than rangeres, azres */
  /*
  if(params->rangeres<=(params->dr)
     || params->azres<=(params->da)){
    fflush(NULL);
    fprintf(sp0,"resolutions parameters must be larger than pixel spacings\n");
    exit(ABNORMAL_EXIT);
  }
  */
  if(params->lambda<=0){
    fflush(NULL);
    fprintf(sp0,"wavelength lambda  must be positive (meters)\n");
    exit(ABNORMAL_EXIT);
  }

  /* check scattering model defaults */
  if(params->kds<=0){
    fflush(NULL);
    fprintf(sp0,"scattering model parameter kds must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->specularexp<=0){
    fflush(NULL);
    fprintf(sp0,"scattering model parameter SPECULAREXP must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->dzrcritfactor<0){
    fflush(NULL);
    fprintf(sp0,"dzrcritfactor must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->laywidth<1){
    fflush(NULL);
    fprintf(sp0,"layover window width laywidth must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->layminei<0){
    fflush(NULL);
    fprintf(sp0,"layover minimum brightness must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->sloperatiofactor<0){
    fflush(NULL);
    fprintf(sp0,"slope ratio fudge factor must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->sigsqei<=0){
    fflush(NULL);
    fprintf(sp0,"intensity estimate variance must be positive\n");
    exit(ABNORMAL_EXIT);
  }

  /* check decorrelation model defaults */
  if(params->drho<=0){
    fflush(NULL);
    fprintf(sp0,"correlation step size drho must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->rhosconst1<=0 || params->rhosconst2<=0){
    fflush(NULL);
    fprintf(sp0,"parameters rhosconst1 and rhosconst2 must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if(!strlen(infiles->corrfile)
     && (params->defaultcorr<0 || params->defaultcorr>1)){
    fflush(NULL);
    fprintf(sp0,"default correlation must be between 0 and 1\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->rhominfactor<0){
    fflush(NULL);
    fprintf(sp0,"parameter rhominfactor must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->ncorrlooksaz<1 || params->ncorrlooksrange<1
     || params->nlooksaz<1 || params->nlooksrange<1
     || params->nlooksother<1){
    fflush(NULL);
    fprintf(sp0,"numbers of looks must be positive integer\n");
    exit(ABNORMAL_EXIT);
  }
  if(!strlen(infiles->corrfile)){
    if(params->ncorrlooksaz<params->nlooksaz){
      fflush(NULL);
      fprintf(sp0,"NCORRLOOKSAZ cannot be smaller than NLOOKSAZ\n");
      fprintf(sp0,"  setting NCORRLOOKSAZ to equal NLOOKSAZ\n");
      params->ncorrlooksaz=params->nlooksaz;
    }
    if(params->ncorrlooksrange<params->nlooksrange){
      fflush(NULL);
      fprintf(sp0,"NCORRLOOKSRANGE cannot be smaller than NLOOKSRANGE\n");
      fprintf(sp0,"  setting NCORRLOOKSRANGE to equal NLOOKSRANGE\n");
      params->ncorrlooksrange=params->nlooksrange;
    }
  }

  /* check pdf model parameters */
  if(params->azdzfactor<0){
    fflush(NULL);
    fprintf(sp0,"parameter azdzfactor must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->dzeifactor<0){
    fflush(NULL);
    fprintf(sp0,"parameter dzeifactor must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->dzeiweight<0 || params->dzeiweight>1.0){
    fflush(NULL);
    fprintf(sp0,"parameter dzeiweight must be between 0 and 1\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->dzlayfactor<0){
    fflush(NULL);
    fprintf(sp0,"parameter dzlayfactor must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->layconst<=0){
    fflush(NULL);
    fprintf(sp0,"parameter layconst must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->layfalloffconst<0){
    fflush(NULL);
    fprintf(sp0,"parameter layfalloffconst must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->sigsqshortmin<=0){
    fflush(NULL);
    fprintf(sp0,"parameter sigsqshortmin must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->sigsqlayfactor<0){
    fflush(NULL);
    fprintf(sp0,"parameter sigsqlayfactor must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }

  /* check deformation mode parameters */
  if(params->defoazdzfactor<0){
    fflush(NULL);
    fprintf(sp0,"parameter defoazdzfactor must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->defothreshfactor<0){
    fflush(NULL);
    fprintf(sp0,"parameter defothreshfactor must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->defomax<0){
    fflush(NULL);
    fprintf(sp0,"parameter defomax must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->sigsqcorr<0){
    fflush(NULL);
    fprintf(sp0,"parameter sigsqcorr must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->defolayconst<=0){
    fflush(NULL);
    fprintf(sp0,"parameter defolayconst must be positive\n");
    exit(ABNORMAL_EXIT);
  }

  /* check algorithm parameters */
  /* be sure to check for things that will cause type overflow */
  /* or floating point exception */
  if((params->initmaxflow)<1 && (params->initmaxflow)!=AUTOCALCSTATMAX){
    fflush(NULL);
    fprintf(sp0,"initialization maximum flow must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if((params->arcmaxflowconst)<1){
    fflush(NULL);
    fprintf(sp0,"arcmaxflowconst must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if((params->maxflow)<1){
    fflush(NULL);
    fprintf(sp0,"maxflow must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->krowei<=0 || params->kcolei<=0){
    fflush(NULL);
    fprintf(sp0,"averaging window sizes krowei and kcolei must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->kperpdpsi<=0 || params->kpardpsi<=0){
    fflush(NULL);
    fprintf(sp0,
          "averaging window sizes kperpdpsi and kpardpsi must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->threshold<=0){
    fflush(NULL);
    fprintf(sp0,"numerical solver threshold must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->initdzr<=0){
    fflush(NULL);
    fprintf(sp0,"initdzr must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->initdzstep<=0){
    fflush(NULL);
    fprintf(sp0,"initdzstep must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->maxcost>POSSHORTRANGE || params->maxcost<=0){
    fflush(NULL);
    fprintf(sp0,"maxcost must be positive and within range or short int\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->costscale<=0){
    fflush(NULL);
    fprintf(sp0,"cost scale factor costscale must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->p<0 && params->p!=PROBCOSTP){
    fflush(NULL);
    fprintf(sp0,"Lp-norm parameter p should be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->costmode==TOPO && (params->maxflow*params->nshortcycle)
     >POSSHORTRANGE){
    fflush(NULL);
    fprintf(sp0,"maxflow exceeds range of short int for given nshortcycle\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->costmode==DEFO && ceil(params->defomax*params->nshortcycle)
     >POSSHORTRANGE){
    fflush(NULL);
    fprintf(sp0,"defomax exceeds range of short int for given nshortcycle\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->nshortcycle < 1 || params->nshortcycle > MAXNSHORTCYCLE){
    fflush(NULL);
    fprintf(sp0,"illegal value for nshortcycle\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->maxnewnodeconst<=0 || params->maxnewnodeconst>1){
    fflush(NULL);
    fprintf(sp0,"maxnewnodeconst must be between 0 and 1\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->nconnnodemin<0){
    fflush(NULL);
    fprintf(sp0,"nconnnodemin must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(infiles->infileformat!=FLOAT_DATA || strlen(infiles->magfile)){
    params->havemagnitude=TRUE;
  }else{
    params->havemagnitude=FALSE;
  }
  if(params->maxnflowcycles==USEMAXCYCLEFRACTION){
    params->maxnflowcycles=LRound(params->maxcyclefraction
                                   *nlines/(double )params->ntilerow
                                   *linelen/(double )params->ntilecol);
  }
  if(params->initmaxflow==AUTOCALCSTATMAX
     && !(params->ntilerow==1 && params->ntilecol==1)){
    fflush(NULL);
    fprintf(sp0,"initial maximum flow cannot be calculated automatically in "
            "tile mode\n");
    exit(ABNORMAL_EXIT);
  }
#ifdef NO_CS2
  if(params->initmethod==MCFINIT && !params->unwrapped){
    fflush(NULL);
    fprintf(sp0,"program not compiled with cs2 MCF solver module\n");
    exit(ABNORMAL_EXIT);
  }
#endif

  /* masking parameters */
  if(strlen(infiles->bytemaskfile)
     || params->edgemasktop || params->edgemaskbot
     || params->edgemaskleft || params->edgemaskright){
    if(params->initonly){
      fflush(NULL);
      fprintf(sp0,"masking not applicable for initialize-only mode\n");
      exit(ABNORMAL_EXIT);
    }
  }
  if(params->edgemasktop<0 || params->edgemaskbot<0
     || params->edgemaskleft<0 || params->edgemaskright<0){
    fflush(NULL);
    fprintf(sp0,"edgemask parameters cannot be negative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->edgemasktop+params->edgemaskbot>=nlines
     || params->edgemaskleft+params->edgemaskright>=linelen){
    fflush(NULL);
    fprintf(sp0,"edge masks cannot exceed input array size\n");
    exit(ABNORMAL_EXIT);
  }

  /* tile parameters */
  if(params->ntilerow<1 || params->ntilecol<1){
    fflush(NULL);
    fprintf(sp0,"numbers of tile rows and columns must be positive\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->rowovrlp<0 || params->colovrlp<0){
    fflush(NULL);
    fprintf(sp0,"tile overlaps must be nonnegative\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->ntilerow>1 || params->ntilecol>1){
    ni=ceil((nlines+(params->ntilerow-1)*params->rowovrlp)
            /(double )params->ntilerow);
    nj=ceil((linelen+(params->ntilecol-1)*params->colovrlp)
            /(double )params->ntilecol);
    if(params->ntilerow+params->rowovrlp > nlines
       || params->ntilecol+params->colovrlp > linelen
       || params->ntilerow*params->ntilerow > nlines
       || params->ntilecol*params->ntilecol > linelen){
      fflush(NULL);
      fprintf(sp0,"tiles too small or overlap too large for given input\n");
      exit(ABNORMAL_EXIT);
    }
    if(params->minregionsize
       > ((nlines-(params->ntilerow-1)*(ni-params->rowovrlp))
          *(linelen-(params->ntilecol-1)*(nj-params->colovrlp)))){
      fflush(NULL);
      fprintf(sp0,"minimum region size too large for given tile parameters\n");
      exit(ABNORMAL_EXIT);
    }
    if(TMPTILEOUTFORMAT!=ALT_LINE_DATA && TMPTILEOUTFORMAT!=FLOAT_DATA){
      fflush(NULL);
      fprintf(sp0,"unsupported TMPTILEOUTFORMAT value in complied binary\n");
      exit(ABNORMAL_EXIT);
    }
    if(TMPTILEOUTFORMAT==FLOAT_DATA && outfiles->outfileformat!=FLOAT_DATA){
      fflush(NULL);
      fprintf(sp0,"precompiled tile format precludes given output format\n");
      exit(ABNORMAL_EXIT);
    }
    if(params->scndryarcflowmax<1){
      fflush(NULL);
      fprintf(sp0,"parameter scndryarcflowmax too small\n");
      exit(ABNORMAL_EXIT);
    }
    if(params->initonly){
      fflush(NULL);
      fprintf(sp0,
              "initialize-only mode and tile mode are mutually exclusive\n");
      exit(ABNORMAL_EXIT);
    }
    if(params->assembleonly){
      n=strlen(params->tiledir);
      while(--n>0 && params->tiledir[n]=='/'){
        params->tiledir[n]='\0';
      }
      if(!strlen(params->tiledir)){
        fflush(NULL);
        fprintf(sp0,"tile directory name must be specified\n");
        exit(ABNORMAL_EXIT);
      }
      if(!strcmp(params->tiledir,"/")){
        StrNCopy(params->tiledir,"",MAXSTRLEN);
      }
      params->rmtmptile=FALSE;     /* cowardly avoid removing tile dir input */
    }
    if(params->piecefirstrow!=DEF_PIECEFIRSTROW
       || params->piecefirstcol!=DEF_PIECEFIRSTCOL
       || params->piecenrow!=DEF_PIECENROW
       || params->piecencol!=DEF_PIECENCOL){
      fflush(NULL);
      fprintf(sp0,"piece-only mode cannot be used with multiple tiles\n");
      exit(ABNORMAL_EXIT);
    }
    if(params->costmode==NOSTATCOSTS){
      fflush(NULL);
      fprintf(sp0,"no-statistical-costs option cannot be used in tile mode\n");
      exit(ABNORMAL_EXIT);
    }
    if(params->rowovrlp<TILEOVRLPWARNTHRESH
       || params->colovrlp<TILEOVRLPWARNTHRESH){
      fflush(NULL);
      fprintf(sp0,"WARNING: Tile overlap is small (may give bad results)\n");
    }
  }else{
    if(params->assembleonly){
      fflush(NULL);
      fprintf(sp0,"assemble-only mode can only be used with multiple tiles\n");
      exit(ABNORMAL_EXIT);
    }
    if(params->nthreads>1){
      fflush(NULL);
      fprintf(sp0,"only one tile--disregarding multiprocessor option\n");
    }
    if(params->rowovrlp || params->colovrlp){
      fflush(NULL);
      fprintf(sp0,"only one tile--disregarding tile overlap values\n");
    }
    if(params->onetilereopt){
      fprintf(sp0,
              "cannot do single-tile reoptimization without tiling params\n");
      exit(ABNORMAL_EXIT);
    }
  }
  if(params->nthreads<1){
      fflush(NULL);
    fprintf(sp0,"number of processors must be at least one\n");
    exit(ABNORMAL_EXIT);
  }else if(params->nthreads>MAXTHREADS){
    fflush(NULL);
    fprintf(sp0,"number of processors exceeds precomplied limit of %d\n",
            MAXTHREADS);
    exit(ABNORMAL_EXIT);
  }

  /* piece params */
  params->piecefirstrow--;                   /* index from 0 instead of 1 */
  params->piecefirstcol--;                   /* index from 0 instead of 1 */
  if(!params->piecenrow){
    params->piecenrow=nlines;
  }
  if(!params->piecencol){
    params->piecencol=linelen;
  }
  if(params->piecefirstrow<0 || params->piecefirstcol<0
     || params->piecenrow<1 || params->piecencol<1
     || params->piecefirstrow+params->piecenrow>nlines
     || params->piecefirstcol+params->piecencol>linelen){
    fflush(NULL);
    fprintf(sp0,"illegal values for piece of interferogram to unwrap\n");
    exit(ABNORMAL_EXIT);
  }

  /* connected component parameters */
  if(params->regrowconncomps){
    if(!strlen(outfiles->conncompfile)){
      fflush(NULL);
      fprintf(sp0,"no connected component output file specified\n");
      exit(ABNORMAL_EXIT);
    }
    params->unwrapped=TRUE;
  }
  if(params->minconncompfrac<0 || params->minconncompfrac>1){
    fflush(NULL);
    fprintf(sp0,"illegal value for minimum connected component fraction\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->maxncomps<=0){
    fflush(NULL);
    fprintf(sp0,"illegal value for maximum number of connected components\n");
    exit(ABNORMAL_EXIT);
  }
  if(params->maxncomps>UCHAR_MAX
     && params->conncompouttype==CONNCOMPOUTTYPEUCHAR){
    fflush(NULL);
    fprintf(sp0,"WARNING: clipping max num conn comps to fit uchar out type\n");
    params->maxncomps=UCHAR_MAX;
  }
  if(params->maxncomps>UINT_MAX
     && params->conncompouttype==CONNCOMPOUTTYPEUINT){
    fflush(NULL);
    fprintf(sp0,"WARNING: clipping max num conn comps to fit uint out type\n");
    params->maxncomps=UINT_MAX;
  }
  if(strlen(outfiles->conncompfile)){
    if(params->initonly){
      fflush(NULL);
      fprintf(sp0,"WARNING: connected component mask cannot be generated "
              "in initialize-only mode\n         mask will not be output\n");
      StrNCopy(outfiles->conncompfile,"",MAXSTRLEN);
    }
    if(params->costmode==NOSTATCOSTS){
      fflush(NULL);
      fprintf(sp0,"WARNING: connected component mask cannot be generated "
              "without statistical costs\n         mask will not be output\n");
      StrNCopy(outfiles->conncompfile,"",MAXSTRLEN);
    }
  }

  /* done */
  return(0);

}


/* function: ReadConfigFile()
 * --------------------------
 * Read in parameter values from a file, overriding existing parameters.
 */
int ReadConfigFile(char *conffile, infileT *infiles, outfileT *outfiles,
                   long *linelenptr, paramT *params){

  int parsestatus;
  long nlines, nparams;
  char *ptr;
  char buf[MAXLINELEN];
  FILE *fp;


  /* open input config file */
  if(strlen(conffile)){
    if((fp=fopen(conffile,"r"))==NULL){

      /* abort if we were given a non-zero length name that is unreadable */
      fflush(NULL);
      fprintf(sp0,"unable to read configuration file %s\n",conffile);
      exit(ABNORMAL_EXIT);
    }
  }else{

    /* if we were given a zero-length name, just ignore it and go on */
    return(0);
  }

  /* read each line and convert the first two fields */
  nlines=0;
  nparams=0;
  while(TRUE){

    /* read a line from the file and store it in buffer buf */
    buf[0]='\0';
    ptr=fgets(buf,MAXLINELEN,fp);

    /* break when we read EOF without reading any text */
    if(ptr==NULL && !strlen(buf)){
      break;
    }
    nlines++;

    /* make sure we got the whole line */
    if(strlen(buf)>=MAXLINELEN-1){
      fflush(NULL);
      fprintf(sp0,"line %ld in file %s exceeds maximum line length\n",
              nlines,conffile);
      exit(ABNORMAL_EXIT);
    }

    /* parse config line */
    parsestatus=ParseConfigLine(buf,conffile,nlines,
                                infiles,outfiles,linelenptr,params);
    if(parsestatus>0){
      nparams++;
    }

  }

  /* finish up */
  fclose(fp);
  if(nparams>1){
    fprintf(sp1,"%ld parameters input from file %s (%ld lines total)\n",
            nparams,conffile,nlines);
  }else{
    if(nlines>1){
      fprintf(sp1,"%ld parameter input from file %s (%ld lines total)\n",
              nparams,conffile,nlines);
    }else{
      fprintf(sp1,"%ld parameter input from file %s (%ld line total)\n",
              nparams,conffile,nlines);
    }
  }

  /* done */
  return(0);

}


/* function: ParseConfigLine()
 * ---------------------------
 * Parse config line from passed buffer.
 */
static
int ParseConfigLine(char *buf, char *conffile, long nlines,
                    infileT *infiles, outfileT *outfiles,
                    long *linelenptr, paramT *params){

  int nparams;
  long nfields;
  char str1[MAXLINELEN], str2[MAXLINELEN];
  signed char badparam;


  /* set up */
  nparams=0;
  badparam=FALSE;

  /* read the first two fields */
  /* (str1, str2 same size as buf, so can't overflow them */
  nfields=sscanf(buf,"%s %s",str1,str2);

  /* if only one field is read, and it is not a comment, we have an error */
  if(nfields==1 && isalnum(str1[0])){
    fflush(NULL);
    fprintf(sp0,"unrecognized configuration parameter '%s' (%s:%ld)\n",
            str1,conffile,nlines);
    exit(ABNORMAL_EXIT);
  }

  /* if we have (at least) two non-comment fields */
  if(nfields==2 && isalnum(str1[0])){

    /* do the conversions */
    nparams++;
    if(!strcmp(str1,"INFILE")){
      StrNCopy(infiles->infile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"OUTFILE")){
      StrNCopy(outfiles->outfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"WEIGHTFILE")){
      StrNCopy(infiles->weightfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"AMPFILE") || !strcmp(str1,"AMPFILE1")){
      if(strlen(infiles->ampfile2) && !params->amplitude){
        fflush(NULL);
        fprintf(sp0,"cannot specify both amplitude and power\n");
        exit(ABNORMAL_EXIT);
      }
      StrNCopy(infiles->ampfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"AMPFILE2")){
      if(strlen(infiles->ampfile) && !params->amplitude){
        fflush(NULL);
        fprintf(sp0,"cannot specify both amplitude and power\n");
        exit(ABNORMAL_EXIT);
      }
      StrNCopy(infiles->ampfile2,str2,MAXSTRLEN);
      infiles->ampfileformat=FLOAT_DATA;
    }else if(!strcmp(str1,"PWRFILE") || !strcmp(str1,"PWRFILE1")){
      if(strlen(infiles->ampfile2) && params->amplitude){
        fflush(NULL);
        fprintf(sp0,"cannot specify both amplitude and power\n");
        exit(ABNORMAL_EXIT);
      }
      StrNCopy(infiles->ampfile,str2,MAXSTRLEN);
      params->amplitude=FALSE;
    }else if(!strcmp(str1,"PWRFILE2")){
      if(strlen(infiles->ampfile) && params->amplitude){
        fflush(NULL);
        fprintf(sp0,"cannot specify both amplitude and power\n");
        exit(ABNORMAL_EXIT);
      }
      StrNCopy(infiles->ampfile2,str2,MAXSTRLEN);
      params->amplitude=FALSE;
      infiles->ampfileformat=FLOAT_DATA;
    }else if(!strcmp(str1,"MAGFILE")){
      StrNCopy(infiles->magfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"CORRFILE")){
      StrNCopy(infiles->corrfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"ESTIMATEFILE")){
      StrNCopy(infiles->estfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"LINELENGTH") || !strcmp(str1,"LINELEN")){
      badparam=StringToLong(str2,linelenptr);
    }else if(!strcmp(str1,"STATCOSTMODE")){
      if(!strcmp(str2,"TOPO")){
        params->costmode=TOPO;
      }else if(!strcmp(str2,"DEFO")){
        params->costmode=DEFO;
      }else if(!strcmp(str2,"SMOOTH")){
        params->costmode=SMOOTH;
      }else if(!strcmp(str2,"NOSTATCOSTS")){
        params->costmode=NOSTATCOSTS;
      }else{
        badparam=TRUE;
      }
    }else if(!strcmp(str1,"INITONLY")){
      badparam=SetBooleanSignedChar(&(params->initonly),str2);
    }else if(!strcmp(str1,"UNWRAPPED_IN")){
      badparam=SetBooleanSignedChar(&(params->unwrapped),str2);
    }else if(!strcmp(str1,"DEBUG") || !strcmp(str1,"DUMPALL")){
      badparam=SetBooleanSignedChar(&(params->dumpall),str2);
    }else if(!strcmp(str1,"VERBOSE")){
      badparam=SetBooleanSignedChar(&(params->verbose),str2);
    }else if(!strcmp(str1,"INITMETHOD")){
      if(!strcmp(str2,"MST") || !strcmp(str2,"mst")){
        params->initmethod=MSTINIT;
      }else if(!strcmp(str2,"MCF") || !strcmp(str2,"mcf")
               || !strcmp(str2,"CS2") || !strcmp(str2,"cs2")){
        params->initmethod=MCFINIT;
      }else{
        badparam=TRUE;
      }
    }else if(!strcmp(str1,"ORBITRADIUS")){
      if(!(badparam=StringToDouble(str2,&(params->orbitradius)))){
        params->altitude=0;
      }
    }else if(!strcmp(str1,"ALTITUDE")){
      if(!(badparam=StringToDouble(str2,&(params->altitude)))){
        params->orbitradius=0;
      }
    }else if(!strcmp(str1,"EARTHRADIUS")){
      badparam=StringToDouble(str2,&(params->earthradius));
    }else if(!strcmp(str1,"BPERP")){
      badparam=StringToDouble(str2,&(params->bperp));
    }else if(!strcmp(str1,"TRANSMITMODE")){
      if(!strcmp(str2,"PINGPONG") || !strcmp(str2,"REPEATPASS")){
        params->transmitmode=PINGPONG;
      }else if(!strcmp(str2,"SINGLEANTENNATRANSMIT") || !strcmp(str2,"SAT")
               || !strcmp(str2,"SINGLEANTTRANSMIT")){
        params->transmitmode=SINGLEANTTRANSMIT;
      }else{
        badparam=TRUE;
      }
    }else if(!strcmp(str1,"BASELINE")){
      if(!(badparam=StringToDouble(str2,&(params->baseline)))){
        params->bperp=0;
      }
    }else if(!strcmp(str1,"BASELINEANGLE_RAD")){
      if(!(badparam=StringToDouble(str2,&(params->baselineangle)))){
        params->bperp=0;
      }
    }else if(!strcmp(str1,"BASELINEANGLE_DEG")){
      if(!(badparam=StringToDouble(str2,&(params->baselineangle)))){
        (params->baselineangle)*=(PI/180.0);
        params->bperp=0;
      }
    }else if(!strcmp(str1,"NLOOKSRANGE")){
      badparam=StringToLong(str2,&(params->nlooksrange));
    }else if(!strcmp(str1,"NLOOKSAZ")){
      badparam=StringToLong(str2,&(params->nlooksaz));
    }else if(!strcmp(str1,"NLOOKSOTHER")){
      badparam=StringToLong(str2,&(params->nlooksother));
    }else if(!strcmp(str1,"NCORRLOOKS")){
      badparam=StringToDouble(str2,&(params->ncorrlooks));
    }else if(!strcmp(str1,"NCORRLOOKSRANGE")){
      badparam=StringToLong(str2,&(params->ncorrlooksrange));
    }else if(!strcmp(str1,"NCORRLOOKSAZ")){
      badparam=StringToLong(str2,&(params->ncorrlooksaz));
    }else if(!strcmp(str1,"NEARRANGE") || !strcmp(str1,"NOMRANGE")){
      badparam=StringToDouble(str2,&(params->nearrange));
    }else if(!strcmp(str1,"DR")){
      badparam=StringToDouble(str2,&(params->dr));
    }else if(!strcmp(str1,"DA")){
      badparam=StringToDouble(str2,&(params->da));
    }else if(!strcmp(str1,"RANGERES")){
      badparam=StringToDouble(str2,&(params->rangeres));
    }else if(!strcmp(str1,"AZRES")){
      badparam=StringToDouble(str2,&(params->azres));
    }else if(!strcmp(str1,"LAMBDA")){
      badparam=StringToDouble(str2,&(params->lambda));
    }else if(!strcmp(str1,"KDS") || !strcmp(str1,"KSD")){
      if(!strcmp(str1,"KSD")){
        fflush(NULL);
        fprintf(sp0,"WARNING: parameter KSD interpreted as KDS (%s:%ld)\n",
                conffile,nlines);
      }
      badparam=StringToDouble(str2,&(params->kds));
    }else if(!strcmp(str1,"SPECULAREXP") || !strcmp(str1,"N")){
      badparam=StringToDouble(str2,&(params->specularexp));
    }else if(!strcmp(str1,"DZRCRITFACTOR")){
      badparam=StringToDouble(str2,&(params->dzrcritfactor));
    }else if(!strcmp(str1,"SHADOW")){
      badparam=SetBooleanSignedChar(&(params->shadow),str2);
    }else if(!strcmp(str1,"DZEIMIN")){
      badparam=StringToDouble(str2,&(params->dzeimin));
    }else if(!strcmp(str1,"LAYWIDTH")){
      badparam=StringToLong(str2,&(params->laywidth));
    }else if(!strcmp(str1,"LAYMINEI")){
      badparam=StringToDouble(str2,&(params->layminei));
    }else if(!strcmp(str1,"SLOPERATIOFACTOR")){
      badparam=StringToDouble(str2,&(params->sloperatiofactor));
    }else if(!strcmp(str1,"SIGSQEI")){
      badparam=StringToDouble(str2,&(params->sigsqei));
    }else if(!strcmp(str1,"DRHO")){
      badparam=StringToDouble(str2,&(params->drho));
    }else if(!strcmp(str1,"RHOSCONST1")){
      badparam=StringToDouble(str2,&(params->rhosconst1));
    }else if(!strcmp(str1,"RHOSCONST2")){
      badparam=StringToDouble(str2,&(params->rhosconst2));
    }else if(!strcmp(str1,"CSTD1")){
      badparam=StringToDouble(str2,&(params->cstd1));
    }else if(!strcmp(str1,"CSTD2")){
      badparam=StringToDouble(str2,&(params->cstd2));
    }else if(!strcmp(str1,"CSTD3")){
      badparam=StringToDouble(str2,&(params->cstd3));
    }else if(!strcmp(str1,"DEFAULTCORR")){
      badparam=StringToDouble(str2,&(params->defaultcorr));
    }else if(!strcmp(str1,"RHOMINFACTOR")){
      badparam=StringToDouble(str2,&(params->rhominfactor));
    }else if(!strcmp(str1,"DZLAYPEAK")){
      badparam=StringToDouble(str2,&(params->dzlaypeak));
    }else if(!strcmp(str1,"AZDZFACTOR")){
      badparam=StringToDouble(str2,&(params->azdzfactor));
    }else if(!strcmp(str1,"DZEIFACTOR")){
      badparam=StringToDouble(str2,&(params->dzeifactor));
    }else if(!strcmp(str1,"DZEIWEIGHT")){
      badparam=StringToDouble(str2,&(params->dzeiweight));
    }else if(!strcmp(str1,"DZLAYFACTOR")){
      badparam=StringToDouble(str2,&(params->dzlayfactor));
    }else if(!strcmp(str1,"LAYCONST")){
      badparam=StringToDouble(str2,&(params->layconst));
    }else if(!strcmp(str1,"LAYFALLOFFCONST")){
      badparam=StringToDouble(str2,&(params->layfalloffconst));
    }else if(!strcmp(str1,"SIGSQSHORTMIN")){
      badparam=StringToLong(str2,&(params->sigsqshortmin));
    }else if(!strcmp(str1,"SIGSQLAYFACTOR")){
      badparam=StringToDouble(str2,&(params->sigsqlayfactor));
    }else if(!strcmp(str1,"DEFOAZDZFACTOR")){
      badparam=StringToDouble(str2,&(params->defoazdzfactor));
    }else if(!strcmp(str1,"DEFOTHRESHFACTOR")){
      badparam=StringToDouble(str2,&(params->defothreshfactor));
    }else if(!strcmp(str1,"DEFOMAX_CYCLE")){
      badparam=StringToDouble(str2,&(params->defomax));
    }else if(!strcmp(str1,"DEFOMAX_RAD")){
      if(!(badparam=StringToDouble(str2,&(params->defomax)))){
        params->defomax/=TWOPI;
      }
    }else if(!strcmp(str1,"SIGSQCORR")){
      badparam=StringToDouble(str2,&(params->sigsqcorr));
    }else if(!strcmp(str1,"DEFOLAYCONST") || !strcmp(str1,"DEFOCONST")){
      badparam=StringToDouble(str2,&(params->defolayconst));
    }else if(!strcmp(str1,"INITMAXFLOW")){
      badparam=StringToLong(str2,&(params->initmaxflow));
    }else if(!strcmp(str1,"ARCMAXFLOWCONST")){
      badparam=StringToLong(str2,&(params->arcmaxflowconst));
    }else if(!strcmp(str1,"MAXFLOW")){
      badparam=StringToLong(str2,&(params->maxflow));
    }else if(!strcmp(str1,"KROWEI") || !strcmp(str1,"KROW")){
      badparam=StringToLong(str2,&(params->krowei));
    }else if(!strcmp(str1,"KCOLEI") || !strcmp(str1,"KCOL")){
      badparam=StringToLong(str2,&(params->kcolei));
    }else if(!strcmp(str1,"KPERPDPSI")){
      badparam=StringToLong(str2,&(params->kperpdpsi));
    }else if(!strcmp(str1,"KPARDPSI")){
      badparam=StringToLong(str2,&(params->kpardpsi));
    }else if(!strcmp(str1,"THRESHOLD")){
      badparam=StringToDouble(str2,&(params->threshold));
    }else if(!strcmp(str1,"INITDZR")){
      badparam=StringToDouble(str2,&(params->initdzr));
    }else if(!strcmp(str1,"INITDZSTEP")){
      badparam=StringToDouble(str2,&(params->initdzstep));
    }else if(!strcmp(str1,"MAXCOST")){
      badparam=StringToDouble(str2,&(params->maxcost));
    }else if(!strcmp(str1,"COSTSCALE")){
      badparam=StringToDouble(str2,&(params->costscale));
    }else if(!strcmp(str1,"COSTSCALEAMBIGHT")){
      badparam=StringToDouble(str2,&(params->costscaleambight));
    }else if(!strcmp(str1,"DNOMINCANGLE")){
      badparam=StringToDouble(str2,&(params->dnomincangle));
    }else if(!strcmp(str1,"CS2SCALEFACTOR")){
      badparam=StringToLong(str2,&(params->cs2scalefactor));
    }else if(!strcmp(str1,"NMAJORPRUNE")){
      badparam=StringToLong(str2,&(params->nmajorprune));
    }else if(!strcmp(str1,"PRUNECOSTTHRESH")){
      badparam=StringToLong(str2,&(params->prunecostthresh));
    }else if(!strcmp(str1,"PLPN")){
      badparam=StringToDouble(str2,&(params->p));
    }else if(!strcmp(str1,"BIDIRLPN")){
      badparam=SetBooleanSignedChar(&(params->bidirlpn),str2);
    }else if(!strcmp(str1,"EDGEMASKTOP")){
      badparam=StringToLong(str2,&(params->edgemasktop));
    }else if(!strcmp(str1,"EDGEMASKBOT")){
      badparam=StringToLong(str2,&(params->edgemaskbot));
    }else if(!strcmp(str1,"EDGEMASKLEFT")){
      badparam=StringToLong(str2,&(params->edgemaskleft));
    }else if(!strcmp(str1,"EDGEMASKRIGHT")){
      badparam=StringToLong(str2,&(params->edgemaskright));
    }else if(!strcmp(str1,"PIECEFIRSTROW")){
      badparam=StringToLong(str2,&(params->piecefirstrow));
    }else if(!strcmp(str1,"PIECEFIRSTCOL")){
      badparam=StringToLong(str2,&(params->piecefirstcol));
    }else if(!strcmp(str1,"PIECENROW")){
      badparam=StringToLong(str2,&(params->piecenrow));
    }else if(!strcmp(str1,"PIECENCOL")){
      badparam=StringToLong(str2,&(params->piecencol));
    }else if(!strcmp(str1,"NTILEROW")){
      badparam=StringToLong(str2,&(params->ntilerow));
    }else if(!strcmp(str1,"NTILECOL")){
      badparam=StringToLong(str2,&(params->ntilecol));
    }else if(!strcmp(str1,"ROWOVRLP")){
      badparam=StringToLong(str2,&(params->rowovrlp));
    }else if(!strcmp(str1,"COLOVRLP")){
      badparam=StringToLong(str2,&(params->colovrlp));
    }else if(!strcmp(str1,"TILECOSTTHRESH")){
      badparam=StringToLong(str2,&(params->tilecostthresh));
    }else if(!strcmp(str1,"MINREGIONSIZE")){
      badparam=StringToLong(str2,&(params->minregionsize));
    }else if(!strcmp(str1,"TILEEDGEWEIGHT")){
      badparam=StringToDouble(str2,&(params->tileedgeweight));
    }else if(!strcmp(str1,"SCNDRYARCFLOWMAX")){
      badparam=StringToLong(str2,&(params->scndryarcflowmax));
    }else if(!strcmp(str1,"TILEDIR")){
      StrNCopy(params->tiledir,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"ASSEMBLEONLY")){
      badparam=SetBooleanSignedChar(&(params->assembleonly),str2);
    }else if(!strcmp(str1,"SINGLETILEREOPTIMIZE")){
      badparam=SetBooleanSignedChar(&(params->onetilereopt),str2);
    }else if(!strcmp(str1,"RMTMPTILE")){
      badparam=SetBooleanSignedChar(&(params->rmtmptile),str2);
      params->rmtileinit=params->rmtmptile;
    }else if(!strcmp(str1,"MINCONNCOMPFRAC")){
      badparam=StringToDouble(str2,&(params->minconncompfrac));
    }else if(!strcmp(str1,"CONNCOMPTHRESH")){
      badparam=StringToLong(str2,&(params->conncompthresh));
    }else if(!strcmp(str1,"MAXNCOMPS")){
      badparam=StringToLong(str2,&(params->maxncomps));
    }else if(!strcmp(str1,"CONNCOMPOUTTYPE")){
      if(!strcmp(str2,"UCHAR")){
        params->conncompouttype=CONNCOMPOUTTYPEUCHAR;
      }else if(!strcmp(str2,"UINT")){
        params->conncompouttype=CONNCOMPOUTTYPEUINT;
      }else{
        badparam=TRUE;
      }
    }else if(!strcmp(str1,"NSHORTCYCLE")){
      badparam=StringToLong(str2,&(params->nshortcycle));
    }else if(!strcmp(str1,"MAXNEWNODECONST")){
      badparam=StringToDouble(str2,&(params->maxnewnodeconst));
    }else if(!strcmp(str1,"MAXNFLOWCYCLES")){
      badparam=StringToLong(str2,&(params->maxnflowcycles));
    }else if(!strcmp(str1,"MAXCYCLEFRACTION")){
      badparam=StringToDouble(str2,&(params->maxcyclefraction));
      params->maxnflowcycles=USEMAXCYCLEFRACTION;
    }else if(!strcmp(str1,"SOURCEMODE")){
      fflush(NULL);
      fprintf(sp0,
              "WARNING: SOURCEMODE keyword no longer supported--ignoring\n");
    }else if(!strcmp(str1,"NCONNNODEMIN")){
      badparam=StringToLong(str2,&(params->nconnnodemin));
    }else if(!strcmp(str1,"NPROC") || !strcmp(str1,"NTHREADS")){
      badparam=StringToLong(str2,&(params->nthreads));
    }else if(!strcmp(str1,"COSTINFILE")){
      StrNCopy(infiles->costinfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"BYTEMASKFILE")){
      StrNCopy(infiles->bytemaskfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"DOTILEMASKFILE")){
      StrNCopy(infiles->dotilemaskfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"COSTOUTFILE")){
      StrNCopy(outfiles->costoutfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"LOGFILE")){
      StrNCopy(outfiles->logfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"INFILEFORMAT")){
      if(!strcmp(str2,"COMPLEX_DATA")){
        infiles->infileformat=COMPLEX_DATA;
      }else if(!strcmp(str2,"FLOAT_DATA")){
        infiles->infileformat=FLOAT_DATA;
      }else if(!strcmp(str2,"ALT_LINE_DATA")){
        infiles->infileformat=ALT_LINE_DATA;
      }else if(!strcmp(str2,"ALT_SAMPLE_DATA")){
        infiles->infileformat=ALT_SAMPLE_DATA;
      }else{
        badparam=TRUE;
      }
    }else if(!strcmp(str1,"UNWRAPPEDINFILEFORMAT")){
      if(!strcmp(str2,"ALT_LINE_DATA")){
        infiles->unwrappedinfileformat=ALT_LINE_DATA;
      }else if(!strcmp(str2,"ALT_SAMPLE_DATA")){
        infiles->unwrappedinfileformat=ALT_SAMPLE_DATA;
      }else if(!strcmp(str2,"FLOAT_DATA")){
        infiles->unwrappedinfileformat=FLOAT_DATA;
      }else{
        badparam=TRUE;
      }
    }else if(!strcmp(str1,"MAGFILEFORMAT")){
      if(!strcmp(str2,"ALT_LINE_DATA")){
        infiles->magfileformat=ALT_LINE_DATA;
      }else if(!strcmp(str2,"ALT_SAMPLE_DATA")){
        infiles->magfileformat=ALT_SAMPLE_DATA;
      }else if(!strcmp(str2,"FLOAT_DATA")){
        infiles->magfileformat=FLOAT_DATA;
      }else if(!strcmp(str2,"COMPLEX_DATA")){
        infiles->magfileformat=COMPLEX_DATA;
      }else{
        badparam=TRUE;
      }
    }else if(!strcmp(str1,"OUTFILEFORMAT")){
      if(!strcmp(str2,"ALT_LINE_DATA")){
        outfiles->outfileformat=ALT_LINE_DATA;
      }else if(!strcmp(str2,"ALT_SAMPLE_DATA")){
        outfiles->outfileformat=ALT_SAMPLE_DATA;
      }else if(!strcmp(str2,"FLOAT_DATA")){
        outfiles->outfileformat=FLOAT_DATA;
      }else{
        badparam=TRUE;
      }
    }else if(!strcmp(str1,"CORRFILEFORMAT")){
      if(!strcmp(str2,"ALT_LINE_DATA")){
        infiles->corrfileformat=ALT_LINE_DATA;
      }else if(!strcmp(str2,"ALT_SAMPLE_DATA")){
        infiles->corrfileformat=ALT_SAMPLE_DATA;
      }else if(!strcmp(str2,"FLOAT_DATA")){
        infiles->corrfileformat=FLOAT_DATA;
      }else{
        badparam=TRUE;
      }
    }else if(!strcmp(str1,"AMPFILEFORMAT")){
      if(!strcmp(str2,"ALT_LINE_DATA")){
        infiles->ampfileformat=ALT_LINE_DATA;
      }else if(!strcmp(str2,"ALT_SAMPLE_DATA")){
        infiles->ampfileformat=ALT_SAMPLE_DATA;
      }else if(!strcmp(str2,"FLOAT_DATA")){
        infiles->ampfileformat=FLOAT_DATA;
      }else{
        badparam=TRUE;
      }
    }else if(!strcmp(str1,"ESTFILEFORMAT")){
      if(!strcmp(str2,"ALT_LINE_DATA")){
        infiles->estfileformat=ALT_LINE_DATA;
      }else if(!strcmp(str2,"ALT_SAMPLE_DATA")){
        infiles->estfileformat=ALT_SAMPLE_DATA;
      }else if(!strcmp(str2,"FLOAT_DATA")){
        infiles->estfileformat=FLOAT_DATA;
      }else{
        badparam=TRUE;
      }
    }else if(!strcmp(str1,"INITFILE")){
      StrNCopy(outfiles->initfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"FLOWFILE")){
      StrNCopy(outfiles->flowfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"EIFILE")){
      StrNCopy(outfiles->eifile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"ROWCOSTFILE")){
      StrNCopy(outfiles->rowcostfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"COLCOSTFILE")){
      StrNCopy(outfiles->colcostfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"MSTROWCOSTFILE")){
      StrNCopy(outfiles->mstrowcostfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"MSTCOLCOSTFILE")){
      StrNCopy(outfiles->mstcolcostfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"MSTCOSTSFILE")){
      StrNCopy(outfiles->mstcostsfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"CORRDUMPFILE")){
      StrNCopy(outfiles->corrdumpfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"RAWCORRDUMPFILE")){
      StrNCopy(outfiles->rawcorrdumpfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"CONNCOMPFILE")){
      StrNCopy(outfiles->conncompfile,str2,MAXSTRLEN);
    }else if(!strcmp(str1,"REGROWCONNCOMPS")){
      badparam=SetBooleanSignedChar(&(params->regrowconncomps),str2);
    }else{
      fflush(NULL);
      fprintf(sp0,"unrecognized configuration parameter '%s' (%s:%ld)\n",
              str1,conffile,nlines);
      exit(ABNORMAL_EXIT);
    }

    /* give an error if we had trouble interpreting the line */
    if(badparam){
      fflush(NULL);
      fprintf(sp0,"illegal argument %s for parameter %s (%s:%ld)\n",
              str2,str1,conffile,nlines);
      exit(ABNORMAL_EXIT);
    }

  }

  /* return number of parameters successfully parsed */
  return(nparams);

}


/* function: WriteConfigLogFile()
 * ------------------------------
 * Writes a text log file of configuration parameters and other
 * information.  The log file is in a format compatible to be used as
 * a configuration file.
 */
int WriteConfigLogFile(int argc, char *argv[], infileT *infiles,
                       outfileT *outfiles, long linelen, paramT *params){

  FILE *fp;
  time_t t[1];
  long k;
  char buf[MAXSTRLEN], *ptr;
  char hostnamestr[MAXSTRLEN];

  /* see if we need to write a log file */
  if(strlen(outfiles->logfile)){

    /* open the log file */
    if((fp=fopen(outfiles->logfile,"w"))==NULL){
      fflush(NULL);
      fprintf(sp0,"unable to write to log file %s\n",outfiles->logfile);
      exit(ABNORMAL_EXIT);
    }
    fprintf(sp1,"Logging run-time parameters to file %s\n",outfiles->logfile);

    /* print some run-time environment information */
    fprintf(fp,"# %s v%s\n",PROGRAMNAME,VERSION);
    time(t);
    fprintf(fp,"# Log file generated %s",ctime(t));
    if(gethostname(hostnamestr,MAXSTRLEN)){
      fprintf(fp,"# Could not determine host name\n");
    }else{
      fprintf(fp,"# Host name: %s\n",hostnamestr);
    }
    fprintf(fp,"# PID %ld\n",params->parentpid);
    ptr=getcwd(buf,MAXSTRLEN);
    if(ptr!=NULL){
      fprintf(fp,"# Current working directory: %s\n",buf);
    }else{
      fprintf(fp,"# Could not determine current working directory\n");
    }
    fprintf(fp,"# Command line call:");
    for(k=0;k<argc;k++){
      fprintf(fp," %s",argv[k]);
    }
    fprintf(fp,"\n\n");

    /* print some information about data type sizes */
    fprintf(fp,"# Data type size information for executable as compiled\n");
    fprintf(fp,"# sizeof(short):      %ld\n",sizeof(short));
    fprintf(fp,"# sizeof(int):        %ld\n",sizeof(int));
    fprintf(fp,"# sizeof(long):       %ld\n",sizeof(long));
    fprintf(fp,"# sizeof(float):      %ld\n",sizeof(float));
    fprintf(fp,"# sizeof(double):     %ld\n",sizeof(double));
    fprintf(fp,"# sizeof(void *):     %ld\n",sizeof(void *));
    fprintf(fp,"# sizeof(size_t):     %ld\n",sizeof(size_t));
    fprintf(fp,"\n");

    /* print an entry for each run-time parameter */
    /* input and output files and main runtime options */
    fprintf(fp,"# File input and output and runtime options\n");
    LogStringParam(fp,"INFILE",infiles->infile);
    fprintf(fp,"LINELENGTH  %ld\n",linelen);
    LogStringParam(fp,"OUTFILE",outfiles->outfile);
    LogStringParam(fp,"WEIGHTFILE",infiles->weightfile);
    if(params->amplitude){
      if(strlen(infiles->ampfile2)){
        LogStringParam(fp,"AMPFILE1",infiles->ampfile);
        LogStringParam(fp,"AMPFILE2",infiles->ampfile2);
      }else{
        LogStringParam(fp,"AMPFILE",infiles->ampfile);
      }
    }else{
      if(strlen(infiles->ampfile2)){
        LogStringParam(fp,"PWRFILE1",infiles->ampfile);
        LogStringParam(fp,"PWRFILE2",infiles->ampfile2);
      }else{
        LogStringParam(fp,"PWRFILE",infiles->ampfile);
      }
    }
    LogStringParam(fp,"MAGFILE",infiles->magfile);
    LogStringParam(fp,"CORRFILE",infiles->corrfile);
    LogStringParam(fp,"ESTIMATEFILE",infiles->estfile);
    LogStringParam(fp,"COSTINFILE",infiles->costinfile);
    LogStringParam(fp,"COSTOUTFILE",outfiles->costoutfile);
    LogStringParam(fp,"BYTEMASKFILE",infiles->bytemaskfile);
    LogStringParam(fp,"LOGFILE",outfiles->logfile);
    if(params->costmode==TOPO){
      fprintf(fp,"STATCOSTMODE  TOPO\n");
    }else if(params->costmode==DEFO){
      fprintf(fp,"STATCOSTMODE  DEFO\n");
    }else if(params->costmode==SMOOTH){
      fprintf(fp,"STATCOSTMODE  SMOOTH\n");
    }else if(params->costmode==NOSTATCOSTS){
      fprintf(fp,"STATCOSTMODE  NOSTATCOSTS\n");
    }
    LogBoolParam(fp,"INITONLY",params->initonly);
    LogBoolParam(fp,"UNWRAPPED_IN",params->unwrapped);
    LogBoolParam(fp,"DEBUG",params->dumpall);
    if(params->initmethod==MSTINIT){
      fprintf(fp,"INITMETHOD  MST\n");
    }else if(params->initmethod==MCFINIT){
      fprintf(fp,"INITMETHOD  MCF\n");
    }
    LogBoolParam(fp,"VERBOSE",params->verbose);

    /* file formats */
    fprintf(fp,"\n# File Formats\n");
    LogFileFormat(fp,"INFILEFORMAT",infiles->infileformat);
    LogFileFormat(fp,"OUTFILEFORMAT",outfiles->outfileformat);
    LogFileFormat(fp,"AMPFILEFORMAT",infiles->ampfileformat);
    LogFileFormat(fp,"MAGFILEFORMAT",infiles->magfileformat);
    LogFileFormat(fp,"CORRFILEFORMAT",infiles->corrfileformat);
    LogFileFormat(fp,"ESTFILEFORMAT",infiles->estfileformat);
    LogFileFormat(fp,"UNWRAPPEDINFILEFORMAT",infiles->unwrappedinfileformat);

    /* SAR and geometry parameters */
    fprintf(fp,"\n# SAR and Geometry Parameters\n");
    fprintf(fp,"ALTITUDE  %.8f\n",
            params->orbitradius-params->earthradius);
    fprintf(fp,"# ORBITRADIUS  %.8f\n",params->orbitradius);
    fprintf(fp,"EARTHRADIUS  %.8f\n",params->earthradius);
    if(params->bperp){
      fprintf(fp,"BPERP  %.8f\n",params->bperp);
    }else{
      fprintf(fp,"BASELINE %.8f\n",params->baseline);
      fprintf(fp,"BASELINEANGLE_DEG %.8f\n",
              params->baselineangle*(180.0/PI));
    }
    if(params->transmitmode==PINGPONG){
      fprintf(fp,"TRANSMITMODE  REPEATPASS\n");
    }else if(params->transmitmode==SINGLEANTTRANSMIT){
      fprintf(fp,"TRANSMITMODE  SINGLEANTENNATRANSMIT\n");
    }
    fprintf(fp,"NEARRANGE  %.8f\n",params->nearrange);
    fprintf(fp,"DR  %.8f\n",params->dr);
    fprintf(fp,"DA  %.8f\n",params->da);
    fprintf(fp,"RANGERES  %.8f\n",params->rangeres);
    fprintf(fp,"AZRES  %.8f\n",params->azres);
    fprintf(fp,"LAMBDA  %.8f\n",params->lambda);
    fprintf(fp,"NLOOKSRANGE  %ld\n",params->nlooksrange);
    fprintf(fp,"NLOOKSAZ  %ld\n",params->nlooksaz);
    fprintf(fp,"NLOOKSOTHER  %ld\n",params->nlooksother);
    fprintf(fp,"NCORRLOOKS  %.8f\n",params->ncorrlooks);
    fprintf(fp,"NCORRLOOKSRANGE  %ld\n",params->ncorrlooksrange);
    fprintf(fp,"NCORRLOOKSAZ  %ld\n",params->ncorrlooksaz);

    /* scattering model parameters */
    fprintf(fp,"\n# Scattering model parameters\n");
    fprintf(fp,"KDS  %.8f\n",params->kds);
    fprintf(fp,"SPECULAREXP  %.8f\n",params->specularexp);
    fprintf(fp,"DZRCRITFACTOR  %.8f\n",params->dzrcritfactor);
    LogBoolParam(fp,"SHADOW",params->shadow);
    fprintf(fp,"DZEIMIN  %.8f\n",params->dzeimin);
    fprintf(fp,"LAYWIDTH  %ld\n",params->laywidth);
    fprintf(fp,"LAYMINEI  %.8f\n",params->layminei);
    fprintf(fp,"SLOPERATIOFACTOR  %.8f\n",params->sloperatiofactor);
    fprintf(fp,"SIGSQEI  %.8f\n",params->sigsqei);

    /* decorrelation model paramters */
    fprintf(fp,"\n# Decorrelation model parameters\n");
    fprintf(fp,"DRHO  %.8f\n",params->drho);
    fprintf(fp,"RHOSCONST1  %.8f\n",params->rhosconst1);
    fprintf(fp,"RHOSCONST2  %.8f\n",params->rhosconst2);
    fprintf(fp,"CSTD1  %.8f\n",params->cstd1);
    fprintf(fp,"CSTD2  %.8f\n",params->cstd2);
    fprintf(fp,"CSTD3  %.8f\n",params->cstd3);
    fprintf(fp,"DEFAULTCORR  %.8f\n",params->defaultcorr);
    fprintf(fp,"RHOMINFACTOR  %.8f\n",params->rhominfactor);

    /* PDF model paramters */
    fprintf(fp,"\n# PDF model parameters\n");
    fprintf(fp,"DZLAYPEAK  %.8f\n",params->dzlaypeak);
    fprintf(fp,"AZDZFACTOR  %.8f\n",params->azdzfactor);
    fprintf(fp,"DZEIFACTOR  %.8f\n",params->dzeifactor);
    fprintf(fp,"DZEIWEIGHT  %.8f\n",params->dzeiweight);
    fprintf(fp,"DZLAYFACTOR  %.8f\n",params->dzlayfactor);
    fprintf(fp,"LAYCONST  %.8f\n",params->layconst);
    fprintf(fp,"LAYFALLOFFCONST  %.8f\n",params->layfalloffconst);
    fprintf(fp,"SIGSQSHORTMIN  %ld\n",params->sigsqshortmin);
    fprintf(fp,"SIGSQLAYFACTOR  %.8f\n",params->sigsqlayfactor);

    /* deformation mode paramters */
    fprintf(fp,"\n# Deformation mode parameters\n");
    fprintf(fp,"DEFOAZDZFACTOR  %.8f\n",params->defoazdzfactor);
    fprintf(fp,"DEFOTHRESHFACTOR  %.8f\n",params->defothreshfactor);
    fprintf(fp,"DEFOMAX_CYCLE  %.8f\n",params->defomax);
    fprintf(fp,"SIGSQCORR  %.8f\n",params->sigsqcorr);
    fprintf(fp,"DEFOCONST  %.8f\n",params->defolayconst);

    /* algorithm parameters */
    fprintf(fp,"\n# Algorithm parameters\n");
    fprintf(fp,"INITMAXFLOW  %ld\n",params->initmaxflow);
    fprintf(fp,"ARCMAXFLOWCONST  %ld\n",params->arcmaxflowconst);
    fprintf(fp,"MAXFLOW  %ld\n",params->maxflow);
    fprintf(fp,"KROWEI  %ld\n",params->krowei);
    fprintf(fp,"KCOLEI  %ld\n",params->kcolei);
    fprintf(fp,"KPARDPSI  %ld\n",params->kpardpsi);
    fprintf(fp,"KPERPDPSI  %ld\n",params->kperpdpsi);
    fprintf(fp,"THRESHOLD  %.8f\n",params->threshold);
    fprintf(fp,"INITDZR  %.8f\n",params->initdzr);
    fprintf(fp,"INITDZSTEP  %.8f\n",params->initdzstep);
    fprintf(fp,"MAXCOST  %.8f\n",params->maxcost);
    fprintf(fp,"COSTSCALE  %.8f\n",params->costscale);
    fprintf(fp,"COSTSCALEAMBIGHT  %.8f\n",params->costscaleambight);
    fprintf(fp,"DNOMINCANGLE  %.8f\n",params->dnomincangle);
    fprintf(fp,"NSHORTCYCLE  %ld\n",params->nshortcycle);
    fprintf(fp,"MAXNEWNODECONST  %.8f\n",params->maxnewnodeconst);
    if(params->maxnflowcycles==USEMAXCYCLEFRACTION){
      fprintf(fp,"MAXCYCLEFRACTION  %.8f\n",params->maxcyclefraction);
    }else{
      fprintf(fp,"MAXNFLOWCYCLES  %ld\n",params->maxnflowcycles);
    }
    fprintf(fp,"NCONNNODEMIN  %ld\n",params->nconnnodemin);
    fprintf(fp,"CS2SCALEFACTOR  %ld\n",params->cs2scalefactor);
    fprintf(fp,"NMAJORPRUNE  %ld\n",params->nmajorprune);
    fprintf(fp,"PRUNECOSTTHRESH  %ld\n",params->prunecostthresh);
    if(params->p!=PROBCOSTP){
      fprintf(fp,"PLPN  %.8g\n",params->p);
      LogBoolParam(fp,"BIDIRLPN",params->bidirlpn);
    }else{
      fprintf(fp,"# PLPN  %.8g  (not set)\n",params->p);
      LogBoolParam(fp,"# BIDIRLPN",params->bidirlpn);
    }

    /* file names for dumping intermediate arrays */
    fprintf(fp,"\n# File names for dumping intermediate arrays\n");
    LogStringParam(fp,"INITFILE",outfiles->initfile);
    LogStringParam(fp,"FLOWFILE",outfiles->flowfile);
    LogStringParam(fp,"EIFILE",outfiles->eifile);
    LogStringParam(fp,"ROWCOSTFILE",outfiles->rowcostfile);
    LogStringParam(fp,"COLCOSTFILE",outfiles->colcostfile);
    LogStringParam(fp,"MSTROWCOSTFILE",outfiles->mstrowcostfile);
    LogStringParam(fp,"MSTCOLCOSTFILE",outfiles->mstcolcostfile);
    LogStringParam(fp,"MSTCOSTSFILE",outfiles->mstcostsfile);
    LogStringParam(fp,"RAWCORRDUMPFILE",outfiles->rawcorrdumpfile);
    LogStringParam(fp,"CORRDUMPFILE",outfiles->corrdumpfile);

    /* edge masking parameters */
    fprintf(fp,"\n# Edge masking parameters\n");
    fprintf(fp,"EDGEMASKTOP    %ld\n",params->edgemasktop);
    fprintf(fp,"EDGEMASKBOT    %ld\n",params->edgemaskbot);
    fprintf(fp,"EDGEMASKLEFT   %ld\n",params->edgemaskleft);
    fprintf(fp,"EDGEMASKRIGHT  %ld\n",params->edgemaskright);

    /* piece extraction parameters */
    if(params->ntilerow==1 && params->ntilecol==1){
      fprintf(fp,"\n# Piece extraction parameters\n");
      fprintf(fp,"PIECEFIRSTROW  %ld\n",params->piecefirstrow+1);
      fprintf(fp,"PIECEFIRSTCOL  %ld\n",params->piecefirstcol+1);
      fprintf(fp,"PIECENROW  %ld\n",params->piecenrow);
      fprintf(fp,"PIECENCOL  %ld\n",params->piecencol);
    }else{
      fprintf(fp,"\n# Piece extraction parameters\n");
      fprintf(fp,"# Parameters ignored because of tile mode\n");
      fprintf(fp,"# PIECEFIRSTROW  %ld\n",params->piecefirstrow);
      fprintf(fp,"# PIECEFIRSTCOL  %ld\n",params->piecefirstcol);
      fprintf(fp,"# PIECENROW  %ld\n",params->piecenrow);
      fprintf(fp,"# PIECENCOL  %ld\n",params->piecencol);
    }

    /* tile control */
    fprintf(fp,"\n# Tile control\n");
    fprintf(fp,"NTILEROW  %ld\n",params->ntilerow);
    fprintf(fp,"NTILECOL  %ld\n",params->ntilecol);
    fprintf(fp,"ROWOVRLP  %ld\n",params->rowovrlp);
    fprintf(fp,"COLOVRLP  %ld\n",params->colovrlp);
    fprintf(fp,"NPROC  %ld\n",params->nthreads);
    fprintf(fp,"TILECOSTTHRESH  %ld\n",params->tilecostthresh);
    fprintf(fp,"MINREGIONSIZE  %ld\n",params->minregionsize);
    fprintf(fp,"TILEEDGEWEIGHT  %.8f\n",params->tileedgeweight);
    fprintf(fp,"SCNDRYARCFLOWMAX  %ld\n",params->scndryarcflowmax);
    LogBoolParam(fp,"RMTMPTILE",params->rmtmptile);
    LogStringParam(fp,"DOTILEMASKFILE",infiles->dotilemaskfile);
    LogStringParam(fp,"TILEDIR",params->tiledir);
    LogBoolParam(fp,"ASSEMBLEONLY",params->assembleonly);
    LogBoolParam(fp,"SINGLETILEREOPTIMIZE",params->onetilereopt);

    /* connected component control */
    fprintf(fp,"\n# Connected component control\n");
    LogStringParam(fp,"CONNCOMPFILE",outfiles->conncompfile);
    LogBoolParam(fp,"REGROWCONNCOMPS",params->regrowconncomps);
    fprintf(fp,"MINCONNCOMPFRAC  %.8f\n",params->minconncompfrac);
    fprintf(fp,"CONNCOMPTHRESH  %ld\n",params->conncompthresh);
    fprintf(fp,"MAXNCOMPS  %ld\n",params->maxncomps);
    if(params->conncompouttype==CONNCOMPOUTTYPEUCHAR){
      fprintf(fp,"CONNCOMPOUTTYPE  UCHAR\n");
    }else if(params->conncompouttype==CONNCOMPOUTTYPEUINT){
      fprintf(fp,"CONNCOMPOUTTYPE  UINT\n");
    }else{
      fflush(NULL);
      fprintf(sp0,"ERROR: bad value of params->conncompouttype\n");
      exit(ABNORMAL_EXIT);
    }

    /* close the log file */
    if(fclose(fp)){
      fflush(NULL);
      fprintf(sp0,"ERROR in closing log file %s (disk full?)\nAbort\n",
              outfiles->logfile);
      exit(ABNORMAL_EXIT);
    }
  }

  /* done */
  return(0);

}


/* function: LogStringParam()
 * --------------------------
 * Writes a line to the log file stream for the given keyword/value
 * pair.
 */
static
int LogStringParam(FILE *fp, char *key, char *value){

  /* see if we were passed a zero length value string */
  if(strlen(value)){
    fprintf(fp,"%s  %s\n",key,value);
    fflush(fp);
  }else{
    fprintf(fp,"# Empty value for parameter %s\n",key);
  }
  return(0);
}


/* LogBoolParam()
 * --------------
 * Writes a line to the log file stream for the given keyword/bool
 * pair.
 */
static
int LogBoolParam(FILE *fp, char *key, signed char boolvalue){

  if(boolvalue){
    fprintf(fp,"%s  TRUE\n",key);
  }else{
    fprintf(fp,"%s  FALSE\n",key);
  }
  return(0);
}

/* LogFileFormat()
 * ---------------
 * Writes a line to the log file stream for the given keyword/
 * file format pair.
 */
static
int LogFileFormat(FILE *fp, char *key, signed char fileformat){

  if(fileformat==COMPLEX_DATA){
    fprintf(fp,"%s  COMPLEX_DATA\n",key);
  }else if(fileformat==FLOAT_DATA){
    fprintf(fp,"%s  FLOAT_DATA\n",key);
  }else if(fileformat==ALT_LINE_DATA){
    fprintf(fp,"%s  ALT_LINE_DATA\n",key);
  }else if(fileformat==ALT_SAMPLE_DATA){
    fprintf(fp,"%s  ALT_SAMPLE_DATA\n",key);
  }
  return(0);
}


/* function: GetNLines()
 * ---------------------
 * Gets the number of lines of data in the input file based on the file
 * size.
 */
long GetNLines(infileT *infiles, long linelen, paramT *params){

  FILE *fp;
  long filesize, datasize;

  /* get size of input file in rows and columns */
  if((fp=fopen(infiles->infile,"r"))==NULL){
    fflush(NULL);
    fprintf(sp0,"can't open file %s\n",infiles->infile);
    exit(ABNORMAL_EXIT);
  }
  fseek(fp,0,SEEK_END);
  filesize=ftell(fp);
  fclose(fp);
  if((!params->unwrapped && infiles->infileformat==FLOAT_DATA)
     || (params->unwrapped && infiles->unwrappedinfileformat==FLOAT_DATA)){
    datasize=sizeof(float);
  }else{
    datasize=2*sizeof(float);
  }
  if(filesize % (datasize*linelen)){
    fflush(NULL);
    fprintf(sp0,"extra data in file %s (bad linelength?)\n",
            infiles->infile);
    exit(ABNORMAL_EXIT);
  }
  return(filesize/(datasize*linelen));               /* implicit floor */

}


/* function: WriteOutputFile()
 * ---------------------------
 * Writes the unwrapped phase to the output file specified, in the
 * format given in the parameter structure.
 */
int WriteOutputFile(float **mag, float **unwrappedphase, char *outfile,
                    outfileT *outfiles, long nrow, long ncol){

  if(outfiles->outfileformat==ALT_LINE_DATA){
    WriteAltLineFile(mag,unwrappedphase,outfile,nrow,ncol);
  }else if(outfiles->outfileformat==ALT_SAMPLE_DATA){
    WriteAltSampFile(mag,unwrappedphase,outfile,nrow,ncol);
  }else if(outfiles->outfileformat==FLOAT_DATA){
    Write2DArray((void **)unwrappedphase,outfile,
                 nrow,ncol,sizeof(float));
  }else{
    fflush(NULL);
    fprintf(sp0,"WARNING: Illegal format specified for output file\n");
    fprintf(sp0,"         using default floating-point format\n");
    Write2DArray((void **)unwrappedphase,outfile,
                 nrow,ncol,sizeof(float));
  }
  return(0);
}


/* function: OpenOutputFile()
 * --------------------------
 * Opens a file for writing.  If unable to open the file, tries to
 * open a file in a dump path.  The name of the opened output file
 * is written into the string realoutfile, for which at least
 * MAXSTRLEN bytes should already be allocated.
 */
FILE *OpenOutputFile(char *outfile, char *realoutfile){

  char path[MAXSTRLEN], basename[MAXSTRLEN], dumpfile[MAXSTRLEN];
  FILE *fp;

  if((fp=fopen(outfile,"w"))==NULL){

    /* if we can't write to the out file, get the file name from the path */
    /* and dump to the default path */
    ParseFilename(outfile,path,basename);
    StrNCopy(dumpfile,DUMP_PATH,MAXSTRLEN);
    strcat(dumpfile,basename);
    if((fp=fopen(dumpfile,"w"))!=NULL){
      fflush(NULL);
      fprintf(sp0,"WARNING: Can't write to file %s.  Dumping to file %s\n",
             outfile,dumpfile);
      StrNCopy(realoutfile,dumpfile,MAXSTRLEN);
    }else{
      fflush(NULL);
      fprintf(sp0,"Unable to write to file %s or dump to file %s\nAbort\n",
             outfile,dumpfile);
      exit(ABNORMAL_EXIT);
    }
  }else{
    StrNCopy(realoutfile,outfile,MAXSTRLEN);
  }
  return(fp);

}


/* function: WriteAltLineFile()
 * ----------------------------
 * Writes magnitude and phase data from separate arrays to file.
 * Data type is float.  For each line of data, a full line of magnitude data
 * is written, then a full line of phase data.  Dumps the file to a
 * default directory if the file name/path passed in cannot be used.
 */
static
int WriteAltLineFile(float **mag, float **phase, char *outfile,
                     long nrow, long ncol){

  int row;
  FILE *fp;
  char realoutfile[MAXSTRLEN];

  fp=OpenOutputFile(outfile,realoutfile);
  for(row=0; row<nrow; row++){
    if(fwrite(mag[row],sizeof(float),ncol,fp)!=ncol
       || fwrite(phase[row],sizeof(float),ncol,fp)!=ncol){
      fflush(NULL);
      fprintf(sp0,"Error while writing to file %s (device full?)\nAbort\n",
              realoutfile);
      exit(ABNORMAL_EXIT);
    }
  }
  if(fclose(fp)){
    fprintf(sp0,"WARNING: problem closing file %s (disk full?)\n",realoutfile);
  }
  return(0);
}


/* function: WriteAltSampFile()
 * ----------------------------
 * Writes data from separate arrays to file, alternating samples.
 * Data type is float.  nrow and ncol are the sizes of each input
 * array.  Dumps the file to a default directory if the file name/path
 * passed in cannot be used.
 */
static
int WriteAltSampFile(float **arr1, float **arr2, char *outfile,
                     long nrow, long ncol){

  long row, col;
  FILE *fp;
  float *outline;
  char realoutfile[MAXSTRLEN];

  outline=MAlloc(2*ncol*sizeof(float));
  fp=OpenOutputFile(outfile,realoutfile);
  for(row=0; row<nrow; row++){
    for(col=0;col<ncol;col++){
      outline[2*col]=arr1[row][col];
      outline[2*col+1]=arr2[row][col];
    }
    if(fwrite(outline,sizeof(float),2*ncol,fp)!=2*ncol){
      fflush(NULL);
      fprintf(sp0,"Error while writing to file %s (device full?)\nAbort\n",
              realoutfile);
      exit(ABNORMAL_EXIT);
    }
  }
  if(fclose(fp)){
    fflush(NULL);
    fprintf(sp0,"WARNING: problem closing file %s (disk full?)\n",realoutfile);
  }
  return(0);
}


/* function: Write2DArray()
 * ------------------------
 * Write data in a two dimensional array to a file.  Data elements are
 * have the number of bytes specified by size (use sizeof() when
 * calling this function.
 */
int Write2DArray(void **array, char *filename, long nrow, long ncol,
                 size_t size){

  int row;
  FILE *fp;
  char realoutfile[MAXSTRLEN];

  fp=OpenOutputFile(filename,realoutfile);
  for(row=0; row<nrow; row++){
    if(fwrite(array[row],size,ncol,fp)!=ncol){
      fflush(NULL);
      fprintf(sp0,"Error while writing to file %s (device full?)\nAbort\n",
              realoutfile);
      exit(ABNORMAL_EXIT);
    }
  }
  if(fclose(fp)){
    fflush(NULL);
    fprintf(sp0,"WARNING: problem closing file %s (disk full?)\n",realoutfile);
  }
  return(0);
}


/* function: Write2DRowColArray()
 * ------------------------------
 * Write data in a 2-D row-and-column array to a file.  Data elements
 * have the number of bytes specified by size (use sizeof() when
 * calling this function.  The format of the array is nrow-1 rows
 * of ncol elements, followed by nrow rows of ncol-1 elements each.
 */
int Write2DRowColArray(void **array, char *filename, long nrow,
                        long ncol, size_t size){

  int row;
  FILE *fp;
  char realoutfile[MAXSTRLEN];

  fp=OpenOutputFile(filename,realoutfile);
  for(row=0; row<nrow-1; row++){
    if(fwrite(array[row],size,ncol,fp)!=ncol){
      fflush(NULL);
      fprintf(sp0,"Error while writing to file %s (device full?)\nAbort\n",
              realoutfile);
      exit(ABNORMAL_EXIT);
    }
  }
  for(row=nrow-1; row<2*nrow-1; row++){
    if(fwrite(array[row],size,ncol-1,fp)!=ncol-1){
      fflush(NULL);
      fprintf(sp0,"Error while writing to file %s (device full?)\nAbort\n",
              realoutfile);
      exit(ABNORMAL_EXIT);
    }
  }
  if(fclose(fp)){
    fflush(NULL);
    fprintf(sp0,"WARNING: problem closing file %s (disk full?)\n",realoutfile);
  }
  return(0);
}


/* function: ReadInputFile()
 * -------------------------
 * Reads the input file specified on the command line.
 */
int ReadInputFile(infileT *infiles, float ***magptr, float ***wrappedphaseptr,
                  short ***flowsptr, long linelen, long nlines,
                  paramT *params, tileparamT *tileparams){

  long row, col, nrow, ncol;
  float **mag, **wrappedphase, **unwrappedphase;
  short **flows;

  /* initialize */
  mag=NULL;
  wrappedphase=NULL;
  unwrappedphase=NULL;
  flows=NULL;
  nrow=tileparams->nrow;
  ncol=tileparams->ncol;

  /* check data size */
  if(tileparams->ncol>LARGESHORT || tileparams->nrow>LARGESHORT){
    fflush(NULL);
    fprintf(sp0,"one or more interferogram dimensions too large\n");
    exit(ABNORMAL_EXIT);
  }
  if(tileparams->ncol<2 || tileparams->nrow<2){
    fflush(NULL);
    fprintf(sp0,"input interferogram must be at least 2x2\n");
    exit(ABNORMAL_EXIT);
  }

  /* is the input file already unwrapped? */
  if(!params->unwrapped){

    /* read wrapped phase and possibly interferogram magnitude data */
    fprintf(sp1,"Reading wrapped phase from file %s\n",infiles->infile);
    if(infiles->infileformat==COMPLEX_DATA){
      ReadComplexFile(&mag,&wrappedphase,infiles->infile,
                      linelen,nlines,tileparams);
    }else if(infiles->infileformat==ALT_LINE_DATA){
      ReadAltLineFile(&mag,&wrappedphase,infiles->infile,
                      linelen,nlines,tileparams);
    }else if(infiles->infileformat==ALT_SAMPLE_DATA){
      ReadAltSampFile(&mag,&wrappedphase,infiles->infile,
                      linelen,nlines,tileparams);
    }else if(infiles->infileformat==FLOAT_DATA){
      Read2DArray((void ***)&wrappedphase,infiles->infile,linelen,nlines,
                  tileparams,sizeof(float *),sizeof(float));
    }else{
      fflush(NULL);
      fprintf(sp0,"illegal input file format specification\n");
      exit(ABNORMAL_EXIT);
    }

    /* check to make sure the input data doesn't contain NaNs or infs */
    if(!ValidDataArray(wrappedphase,nrow,ncol)
       || (mag!=NULL && !ValidDataArray(mag,nrow,ncol))){
      fflush(NULL);
      fprintf(sp0,"NaN or infinity found in input float data\nAbort\n");
      exit(ABNORMAL_EXIT);
    }
    if(mag!=NULL && !NonNegDataArray(mag,nrow,ncol)){
      fflush(NULL);
      fprintf(sp0,"Negative magnitude found in input magnitude data\nAbort\n");
      exit(ABNORMAL_EXIT);
    }

    /* flip the sign of the wrapped phase if flip flag is set */
    FlipPhaseArraySign(wrappedphase,params,nrow,ncol);

    /* make sure the wrapped phase is properly wrapped */
    WrapPhase(wrappedphase,nrow,ncol);

  }else{

    /* read unwrapped phase input */
    fprintf(sp1,"Reading unwrapped phase from file %s\n",infiles->infile);
    if(infiles->unwrappedinfileformat==ALT_LINE_DATA){
      ReadAltLineFile(&mag,&unwrappedphase,infiles->infile,
                      linelen,nlines,tileparams);
    }else if(infiles->unwrappedinfileformat==ALT_SAMPLE_DATA){
      ReadAltSampFile(&mag,&unwrappedphase,infiles->infile,
                           linelen,nlines,tileparams);
    }else if(infiles->unwrappedinfileformat==FLOAT_DATA){
      Read2DArray((void ***)&unwrappedphase,infiles->infile,linelen,nlines,
                  tileparams,sizeof(float *),sizeof(float));
    }else{
      fflush(NULL);
      fprintf(sp0,"Illegal input file format specification\nAbort\n");
      exit(ABNORMAL_EXIT);
    }

    /* check to make sure the input data doesn't contain NaNs or infs */
    if(!ValidDataArray(unwrappedphase,nrow,ncol)
       || (mag!=NULL && !ValidDataArray(mag,nrow,ncol))){
      fflush(NULL);
      fprintf(sp0,"NaN or infinity found in input float data\nAbort\n");
      exit(ABNORMAL_EXIT);
    }
    if(mag!=NULL && !NonNegDataArray(mag,nrow,ncol)){
      fflush(NULL);
      fprintf(sp0,"Negative magnitude found in input magnitude data\nAbort\n");
      exit(ABNORMAL_EXIT);
    }


    /* flip the sign of the input unwrapped phase if flip flag is set */
    FlipPhaseArraySign(unwrappedphase,params,nrow,ncol);

    /* parse flows of unwrapped phase */
    wrappedphase=ExtractFlow(unwrappedphase,&flows,nrow,ncol);

    /* free unwrapped phase array to save memory */
    Free2DArray((void **)unwrappedphase,nrow);

  }

  /* show which pixels read if tiling */
  if(tileparams->nrow!=nlines || tileparams->ncol!=linelen){
    fprintf(sp2,
            "Read %ldx%ld array of pixels starting at row,col %ld,%ld\n",
            tileparams->nrow,tileparams->ncol,
            tileparams->firstrow,tileparams->firstcol);
  }

  /* get memory for mag (power) image and set to unity if not passed */
  if(mag==NULL){
    mag=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
    for(row=0;row<nrow;row++){
      for(col=0;col<ncol;col++){
        mag[row][col]=1.0;
      }
    }
  }

  /* set passed pointers and return the number of rows in data */
  *wrappedphaseptr=wrappedphase;
  *magptr=mag;
  *flowsptr=flows;

  /* done */
  return(0);

}


/* function: ReadMagnitude()
 * -------------------------
 * Reads the interferogram magnitude in the specfied file if it exists.
 * Memory for the magnitude array should already have been allocated by
 * ReadInputFile().
 */
int ReadMagnitude(float **mag, infileT *infiles, long linelen, long nlines,
                  tileparamT *tileparams){

  float **dummy;

  dummy=NULL;
  if(strlen(infiles->magfile)){
    fprintf(sp1,"Reading interferogram magnitude from file %s\n",
            infiles->magfile);
    if(infiles->magfileformat==FLOAT_DATA){
      Read2DArray((void ***)&mag,infiles->magfile,linelen,nlines,tileparams,
                  sizeof(float *),sizeof(float));
    }else if(infiles->magfileformat==COMPLEX_DATA){
      ReadComplexFile(&mag,&dummy,infiles->magfile,linelen,nlines,
                      tileparams);
    }else if(infiles->magfileformat==ALT_LINE_DATA){
      ReadAltLineFile(&mag,&dummy,infiles->magfile,linelen,nlines,
                      tileparams);
    }else if(infiles->magfileformat==ALT_SAMPLE_DATA){
      ReadAltSampFile(&mag,&dummy,infiles->magfile,linelen,nlines,
                      tileparams);
    }
  }
  if(dummy!=NULL){
    Free2DArray((void **)dummy,tileparams->nrow);
  }
  return(0);
}

/* function: ReadByteMask()
 * ------------------------
 * Read signed byte mask value; set magnitude to zero where byte mask
 * is zero or where pixel is close enough to edge as defined by
 * edgemask parameters; leave magnitude unchanged otherwise.
 */
int ReadByteMask(float **mag, infileT *infiles, long linelen, long nlines,
                 tileparamT *tileparams, paramT *params){

  long row, col, nrow, ncol, fullrow, fullcol;
  signed char **bytemask;

  /* set up */
  nrow=tileparams->nrow;
  ncol=tileparams->ncol;

  /* read byte mask (memory allocated by read function) */
  bytemask=NULL;
  if(strlen(infiles->bytemaskfile)){
    fprintf(sp1,"Reading byte mask from file %s\n",infiles->bytemaskfile);
    Read2DArray((void ***)&bytemask,infiles->bytemaskfile,linelen,nlines,
                tileparams,sizeof(signed char *),sizeof(signed char));
  }

  /* loop over rows and columns and zero out magnitude where mask is zero */
  /* also mask edges according to edgemask parameters */
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      fullrow=tileparams->firstrow+row;
      fullcol=tileparams->firstcol+col;
      if((bytemask!=NULL && bytemask[row][col]==0)
         || fullrow<params->edgemasktop
         || fullcol<params->edgemaskleft
         || fullrow>=nlines-params->edgemaskbot
         || fullcol>=linelen-params->edgemaskright){
        mag[row][col]=0;
      }
    }
  }

  /* free bytemask memory */
  if(bytemask!=NULL){
    Free2DArray((void **)bytemask,nrow);
  }

  /* done */
  return(0);

}


/* function: ReadUnwrappedEstimateFile()
 * -------------------------------------
 * Reads the unwrapped-phase estimate from a file (assumes file name exists).
 */
int ReadUnwrappedEstimateFile(float ***unwrappedestptr, infileT *infiles,
                              long linelen, long nlines,
                              paramT *params, tileparamT *tileparams){

  float **dummy;
  long nrow, ncol;


  /* initialize */
  dummy=NULL;
  nrow=tileparams->nrow;
  ncol=tileparams->ncol;

  /* read data */
  fprintf(sp1,"Reading coarse unwrapped estimate from file %s\n",
          infiles->estfile);
  if(infiles->estfileformat==ALT_LINE_DATA){
    ReadAltLineFilePhase(unwrappedestptr,infiles->estfile,
                         linelen,nlines,tileparams);
  }else if(infiles->estfileformat==FLOAT_DATA){
    Read2DArray((void ***)unwrappedestptr,infiles->estfile,linelen,nlines,
                tileparams,sizeof(float *),sizeof(float));
  }else if(infiles->estfileformat==ALT_SAMPLE_DATA){
    ReadAltSampFile(&dummy,unwrappedestptr,infiles->estfile,
                    linelen,nlines,tileparams);
  }else{
    fflush(NULL);
    fprintf(sp0,"Illegal file format specification for file %s\nAbort\n",
            infiles->estfile);
  }
  if(dummy!=NULL){
    Free2DArray((void **)dummy,nrow);
  }

  /* make sure data is valid */
  if(!ValidDataArray(*unwrappedestptr,nrow,ncol)){
    fflush(NULL);
    fprintf(sp0,"Infinity or NaN found in file %s\nAbort\n",infiles->estfile);
    exit(ABNORMAL_EXIT);
  }

  /* flip the sign of the unwrapped estimate if the flip flag is set */
  FlipPhaseArraySign(*unwrappedestptr,params,nrow,ncol);

  /* done */
  return(0);

}


/* function: ReadWeightsFile()
 * ---------------------------
 * Read in weights form rowcol format file of short ints.
 */
int ReadWeightsFile(short ***weightsptr,char *weightfile,
                    long linelen, long nlines, tileparamT *tileparams){

  long row, col, nrow, ncol;
  short **rowweight, **colweight;
  signed char printwarning;


  /* set up and read data */
  nrow=tileparams->nrow;
  ncol=tileparams->ncol;
  if(strlen(weightfile)){
    fprintf(sp1,"Reading weights from file %s\n",weightfile);
    Read2DRowColFile((void ***)weightsptr,weightfile,linelen,nlines,
                     tileparams,sizeof(short));
    rowweight=*weightsptr;
    colweight=&(*weightsptr)[nrow-1];
    printwarning=FALSE;
    for(row=0;row<nrow-1;row++){
      for(col=0;col<ncol;col++){
        if(rowweight[row][col]<0){
          rowweight[row][col]=0;
          printwarning=TRUE;
        }
      }
    }
    for(row=0;row<nrow;row++){
      for(col=0;col<ncol-1;col++){
        if(colweight[row][col]<0){
          colweight[row][col]=0;
          printwarning=TRUE;
        }
      }
    }
    if(printwarning){
      fflush(NULL);
      fprintf(sp0,"WARNING: Weights must be nonnegative.  Clipping to 0\n");
    }
  }else{
    fprintf(sp1,"No weight file specified.  Assuming uniform weights\n");
    *weightsptr=(short **)Get2DRowColMem(nrow,ncol,
                                         sizeof(short *),sizeof(short));
    rowweight=*weightsptr;
    colweight=&(*weightsptr)[nrow-1];
    Set2DShortArray(rowweight,nrow-1,ncol,DEF_WEIGHT);
    Set2DShortArray(colweight,nrow,ncol-1,DEF_WEIGHT);
  }

  /* done */
  return(0);

}


/* function: ReadIntensity()
 * -------------------------
 * Reads the intensity information from specified file(s).  If possilbe,
 * sets arrays for average power and individual powers of single-pass
 * SAR images.
 */
int ReadIntensity(float ***pwrptr, float ***pwr1ptr, float ***pwr2ptr,
                  infileT *infiles, long linelen, long nlines,
                  paramT *params, tileparamT *tileparams){

  float **pwr, **pwr1, **pwr2;
  long row, col, nrow, ncol;


  /* initialize */
  nrow=tileparams->nrow;
  ncol=tileparams->ncol;
  pwr=NULL;
  pwr1=NULL;
  pwr2=NULL;

  /* read the data */
  if(strlen(infiles->ampfile2)){

    /* data is given in two separate files */
    fprintf(sp1,"Reading brightness data from files %s and %s\n",
            infiles->ampfile,infiles->ampfile2);
    if(infiles->ampfileformat==FLOAT_DATA){
      Read2DArray((void ***)&pwr1,infiles->ampfile,linelen,nlines,tileparams,
                  sizeof(float *),sizeof(float));
      Read2DArray((void ***)&pwr2,infiles->ampfile2,linelen,nlines,tileparams,
                  sizeof(float *),sizeof(float));
    }else{
      fflush(NULL);
      fprintf(sp0,"Illegal file formats specified for files %s, %s\nAbort\n",
              infiles->ampfile,infiles->ampfile2);
      exit(ABNORMAL_EXIT);
    }

  }else{

    /* data is in single file */
    fprintf(sp1,"Reading brightness data from file %s\n",infiles->ampfile);
    if(infiles->ampfileformat==ALT_SAMPLE_DATA){
      ReadAltSampFile(&pwr1,&pwr2,infiles->ampfile,linelen,nlines,
                      tileparams);
    }else if(infiles->ampfileformat==ALT_LINE_DATA){
      ReadAltLineFile(&pwr1,&pwr2,infiles->ampfile,linelen,nlines,
                      tileparams);
    }else if(infiles->ampfileformat==FLOAT_DATA){
      Read2DArray((void ***)&pwr,infiles->ampfile,linelen,nlines,tileparams,
                  sizeof(float *),sizeof(float));
      pwr1=NULL;
      pwr2=NULL;
    }else{
      fflush(NULL);
      fprintf(sp0,"Illegal file format specified for file %s\nAbort\n",
              infiles->ampfile);
      exit(ABNORMAL_EXIT);
    }
  }

  /* check data validity */
  if((pwr1!=NULL && !ValidDataArray(pwr1,nrow,ncol))
     || (pwr2!=NULL && !ValidDataArray(pwr2,nrow,ncol))
     || (pwr!=NULL && !ValidDataArray(pwr,nrow,ncol))){
    fflush(NULL);
    fprintf(sp0,"Infinity or NaN found in amplitude or power data\nAbort\n");
    exit(ABNORMAL_EXIT);
  }
  if((pwr1!=NULL && !NonNegDataArray(pwr1,nrow,ncol))
     || (pwr2!=NULL && !NonNegDataArray(pwr2,nrow,ncol))
     || (pwr!=NULL && !NonNegDataArray(pwr,nrow,ncol))){
    fflush(NULL);
    fprintf(sp0,"Negative value found in amplitude or power data\nAbort\n");
    exit(ABNORMAL_EXIT);
  }

  /* if data is amplitude, square to get power */
  if(params->amplitude){
    for(row=0;row<nrow;row++){
      for(col=0;col<ncol;col++){
        if(pwr1!=NULL && pwr2!=NULL){
          pwr1[row][col]*=pwr1[row][col];
          pwr2[row][col]*=pwr2[row][col];
        }else{
          pwr[row][col]*=pwr[row][col];
        }
      }
    }
  }

  /* get the average power */
  if(pwr1!=NULL && pwr2!=NULL){
    if(pwr==NULL){
      pwr=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
    }
    for(row=0;row<nrow;row++){
      for(col=0;col<ncol;col++){
        pwr[row][col]=(pwr1[row][col]+pwr2[row][col])/2.0;
      }
    }
  }

  /* set output pointers */
  *pwrptr=pwr;
  *pwr1ptr=pwr1;
  *pwr2ptr=pwr2;

  /* done */
  return(0);

}


/* function: ReadCorrelation()
 * ---------------------------
 * Reads the correlation information from specified file.
 */
int ReadCorrelation(float ***corrptr, infileT *infiles,
                    long linelen, long nlines, tileparamT *tileparams){

  float **corr, **dummy;
  long nrow;


  /* initialize */
  nrow=tileparams->nrow;
  dummy=NULL;
  corr=NULL;

  /* read the data */
  fprintf(sp1,"Reading correlation data from file %s\n",infiles->corrfile);
  if(infiles->corrfileformat==ALT_SAMPLE_DATA){
    ReadAltSampFile(&dummy,&corr,infiles->corrfile,linelen,nlines,tileparams);
  }else if(infiles->corrfileformat==ALT_LINE_DATA){
    ReadAltLineFilePhase(&corr,infiles->corrfile,linelen,nlines,tileparams);
  }else if(infiles->corrfileformat==FLOAT_DATA){
    Read2DArray((void ***)&corr,infiles->corrfile,linelen,nlines,tileparams,
                sizeof(float *),sizeof(float));
  }else{
    fflush(NULL);
    fprintf(sp0,"Illegal file format specified for file %s\nAbort\n",
            infiles->corrfile);
    exit(ABNORMAL_EXIT);
  }

  /* set output pointer and free memory */
  if(dummy!=NULL){
    Free2DArray((void **)dummy,nrow);
  }
  *corrptr=corr;

  /* done */
  return(0);

}


/* function: ReadAltLineFile()
 * ---------------------------
 * Read in the data from a file containing magnitude and phase
 * data.  File should have one line of magnitude data, one line
 * of phase data, another line of magnitude data, etc.
 * ncol refers to the number of complex elements in one line of
 * data.
 */
int ReadAltLineFile(float ***mag, float ***phase, char *alfile,
                    long linelen, long nlines, tileparamT *tileparams){

  FILE *fp;
  long filesize,row,nrow,ncol,padlen;

  /* open the file */
  if((fp=fopen(alfile,"r"))==NULL){
    fflush(NULL);
    fprintf(sp0,"Can't open file %s\nAbort\n",alfile);
    exit(ABNORMAL_EXIT);
  }

  /* get number of lines based on file size and line length */
  fseek(fp,0,SEEK_END);
  filesize=ftell(fp);
  if(filesize!=(2*nlines*linelen*sizeof(float))){
    fflush(NULL);
    fprintf(sp0,"File %s wrong size (%ldx%ld array expected)\nAbort\n",
            alfile,nlines,linelen);
    exit(ABNORMAL_EXIT);
  }
  fseek(fp,0,SEEK_SET);

  /* get memory */
  nrow=tileparams->nrow;
  ncol=tileparams->ncol;
  if(*mag==NULL){
    (*mag)=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
  }
  if(*phase==NULL){
    (*phase)=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
  }

  /* read the data */
  fseek(fp,(tileparams->firstrow*2*linelen+tileparams->firstcol)
        *sizeof(float),SEEK_CUR);
  padlen=(linelen-ncol)*sizeof(float);
  for(row=0; row<nrow; row++){
    if(fread((*mag)[row],sizeof(float),ncol,fp)!=ncol){
      fflush(NULL);
      fprintf(sp0,"Error while reading from file %s\nAbort\n",alfile);
      exit(ABNORMAL_EXIT);
    }
    fseek(fp,padlen,SEEK_CUR);
    if(fread((*phase)[row],sizeof(float),ncol,fp)!=ncol){
      fflush(NULL);
      fprintf(sp0,"Error while reading from file %s\nAbort\n",alfile);
      exit(ABNORMAL_EXIT);
    }
    fseek(fp,padlen,SEEK_CUR);
  }
  fclose(fp);


  /* done */
  return(0);

}


/* function: ReadAltLineFilePhase()
 * --------------------------------
 * Read only the phase data from a file containing magnitude and phase
 * data.  File should have one line of magnitude data, one line
 * of phase data, another line of magnitude data, etc.
 * ncol refers to the number of complex elements in one line of
 * data.
 */
int ReadAltLineFilePhase(float ***phase, char *alfile,
                          long linelen, long nlines, tileparamT *tileparams){

  FILE *fp;
  long filesize,row,nrow,ncol,padlen;

  /* open the file */
  if((fp=fopen(alfile,"r"))==NULL){
    fflush(NULL);
    fprintf(sp0,"Can't open file %s\nAbort\n",alfile);
    exit(ABNORMAL_EXIT);
  }

  /* get number of lines based on file size and line length */
  fseek(fp,0,SEEK_END);
  filesize=ftell(fp);
  if(filesize!=(2*nlines*linelen*sizeof(float))){
    fflush(NULL);
    fprintf(sp0,"File %s wrong size (%ldx%ld array expected)\nAbort\n",
            alfile,nlines,linelen);
    exit(ABNORMAL_EXIT);
  }
  fseek(fp,0,SEEK_SET);

  /* get memory */
  nrow=tileparams->nrow;
  ncol=tileparams->ncol;
  if(*phase==NULL){
    (*phase)=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
  }

  /* read the phase data */
  fseek(fp,(tileparams->firstrow*2*linelen+linelen
            +tileparams->firstcol)*sizeof(float),SEEK_CUR);
  padlen=(2*linelen-ncol)*sizeof(float);
  for(row=0; row<nrow; row++){
    if(fread((*phase)[row],sizeof(float),ncol,fp)!=ncol){
      fflush(NULL);
      fprintf(sp0,"Error while reading from file %s\nAbort\n",alfile);
      exit(ABNORMAL_EXIT);
    }
    fseek(fp,padlen,SEEK_CUR);
  }
  fclose(fp);

  /* done */
  return(0);
}


/* function: ReadComplexFile()
 * ---------------------------
 * Reads file of complex floats of the form real,imag,real,imag...
 * ncol is the number of complex samples (half the number of real
 * floats per line).  Ensures that phase values are in the range
 * [0,2pi).
 */
int ReadComplexFile(float ***mag, float ***phase, char *rifile,
                    long linelen, long nlines, tileparamT *tileparams){

  FILE *fp;
  long filesize,ncol,nrow,row,col,padlen;
  float *inpline;

  /* open the file */
  if((fp=fopen(rifile,"r"))==NULL){
    fflush(NULL);
    fprintf(sp0,"Can't open file %s\nAbort\n",rifile);
    exit(ABNORMAL_EXIT);
  }

  /* get number of lines based on file size and line length */
  fseek(fp,0,SEEK_END);
  filesize=ftell(fp);
  if(filesize!=(2*nlines*linelen*sizeof(float))){
    fflush(NULL);
    fprintf(sp0,"File %s wrong size (%ldx%ld array expected)\nAbort\n",
            rifile,nlines,linelen);
    exit(ABNORMAL_EXIT);
  }
  fseek(fp,0,SEEK_SET);

  /* get memory */
  nrow=tileparams->nrow;
  ncol=tileparams->ncol;
  if(*mag==NULL){
    (*mag)=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
  }
  if(*phase==NULL){
    (*phase)=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
  }
  inpline=(float *)MAlloc(2*ncol*sizeof(float));

  /* read the data and convert to magnitude and phase */
  fseek(fp,(tileparams->firstrow*linelen+tileparams->firstcol)
        *2*sizeof(float),SEEK_CUR);
  padlen=(linelen-ncol)*2*sizeof(float);
  for(row=0; row<nrow; row++){
    if(fread(inpline,sizeof(float),2*ncol,fp)!=2*ncol){
      fflush(NULL);
      fprintf(sp0,"Error while reading from file %s\nAbort\n",rifile);
      exit(ABNORMAL_EXIT);
    }
    for(col=0; col<ncol; col++){
      (*mag)[row][col]=sqrt(inpline[2*col]*inpline[2*col]
                            +inpline[2*col+1]*inpline[2*col+1]);
      if(inpline[2*col+1]==0 && inpline[2*col]==0){
        (*phase)[row][col]=0;
      }else if(!IsFinite((*phase)[row][col]=atan2(inpline[2*col+1],
                                                  inpline[2*col]))){
        (*phase)[row][col]=0;
      }else if((*phase)[row][col]<0){
        (*phase)[row][col]+=TWOPI;
      }else if((*phase)[row][col]>=TWOPI){
        (*phase)[row][col]-=TWOPI;
      }
    }
    fseek(fp,padlen,SEEK_CUR);
  }
  free(inpline);
  fclose(fp);

  /* done */
  return(0);

}


/* function: Read2DArray()
 * -------------------------
 * Reads file of real data of size elsize.  Assumes the native byte order
 * of the platform.
 */
int Read2DArray(void ***arr, char *infile, long linelen, long nlines,
                tileparamT *tileparams, size_t elptrsize, size_t elsize){

  FILE *fp;
  long filesize,row,nrow,ncol,padlen;

  /* open the file */
  if((fp=fopen(infile,"r"))==NULL){
    fflush(NULL);
    fprintf(sp0,"Can't open file %s\nAbort\n",infile);
    exit(ABNORMAL_EXIT);
  }

  /* get number of lines based on file size and line length */
  fseek(fp,0,SEEK_END);
  filesize=ftell(fp);
  if(filesize!=(nlines*linelen*elsize)){
    fflush(NULL);
    fprintf(sp0,"File %s wrong size (%ldx%ld array expected)\nAbort\n",
            infile,nlines,linelen);
    exit(ABNORMAL_EXIT);
  }
  fseek(fp,0,SEEK_SET);

  /* get memory */
  nrow=tileparams->nrow;
  ncol=tileparams->ncol;
  if(*arr==NULL){
    (*arr)=(void **)Get2DMem(nrow,ncol,elptrsize,elsize);
  }

  /* read the data */
  fseek(fp,(linelen*tileparams->firstrow+tileparams->firstcol)
        *elsize,SEEK_CUR);
  padlen=(linelen-ncol)*elsize;
  for(row=0; row<nrow; row++){
    if(fread((*arr)[row],elsize,ncol,fp)!=ncol){
      fflush(NULL);
      fprintf(sp0,"Error while reading from file %s\nAbort\n",infile);
      exit(ABNORMAL_EXIT);
    }
    fseek(fp,padlen,SEEK_CUR);
  }
  fclose(fp);

  /* done */
  return(0);

}


/* function: ReadAltSampFile()
 * ---------------------------
 * Reads file of real alternating floats from separate images.  Format is
 * real0A, real0B, real1A, real1B, real2A, real2B,...
 * ncol is the number of samples in each image (note the number of
 * floats per line in the specified file).
 */
int ReadAltSampFile(float ***arr1, float ***arr2, char *infile,
                    long linelen, long nlines, tileparamT *tileparams){

  FILE *fp;
  long filesize,row,col,nrow,ncol,padlen;
  float *inpline;

  /* open the file */
  if((fp=fopen(infile,"r"))==NULL){
    fflush(NULL);
    fprintf(sp0,"Can't open file %s\nAbort\n",infile);
    exit(ABNORMAL_EXIT);
  }

  /* get number of lines based on file size and line length */
  fseek(fp,0,SEEK_END);
  filesize=ftell(fp);
  if(filesize!=(2*nlines*linelen*sizeof(float))){
    fflush(NULL);
    fprintf(sp0,"File %s wrong size (%ldx%ld array expected)\nAbort\n",
            infile,nlines,linelen);
    exit(ABNORMAL_EXIT);
  }
  fseek(fp,0,SEEK_SET);

  /* get memory */
  nrow=tileparams->nrow;
  ncol=tileparams->ncol;
  if(*arr1==NULL){
    (*arr1)=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
  }
  if(*arr2==NULL){
    (*arr2)=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
  }
  inpline=(float *)MAlloc(2*ncol*sizeof(float));

  /* read the data */
  fseek(fp,(tileparams->firstrow*linelen+tileparams->firstcol)
        *2*sizeof(float),SEEK_CUR);
  padlen=(linelen-ncol)*2*sizeof(float);
  for(row=0; row<nrow; row++){
    if(fread(inpline,sizeof(float),2*ncol,fp)!=2*ncol){
      fflush(NULL);
      fprintf(sp0,"Error while reading from file %s\nAbort\n",infile);
      exit(ABNORMAL_EXIT);
    }
    for(col=0; col<ncol; col++){
      (*arr1)[row][col]=inpline[2*col];
      (*arr2)[row][col]=inpline[2*col+1];
    }
    fseek(fp,padlen,SEEK_CUR);
  }
  free(inpline);
  fclose(fp);

  /* done */
  return(0);

}


/* function: Read2DRowColFile()
 * ----------------------------
 * Gets memory and reads single array from a file.  Array should be in the
 * file line by line starting with the row array (size nrow-1 x ncol) and
 * followed by the column array (size nrow x ncol-1).  Both arrays
 * are placed into the passed array as they were in the file.
 */
int Read2DRowColFile(void ***arr, char *filename, long linelen, long nlines,
                     tileparamT *tileparams, size_t size){

  FILE *fp;
  long row, nel, nrow, ncol, padlen, filelen;

  /* open the file */
  if((fp=fopen(filename,"r"))==NULL){
    fflush(NULL);
    fprintf(sp0,"Can't open file %s\nAbort\n",filename);
    exit(ABNORMAL_EXIT);
  }

  /* get number of data elements in file */
  fseek(fp,0,SEEK_END);
  filelen=ftell(fp);
  fseek(fp,0,SEEK_SET);
  nel=(long )(filelen/size);

  /* check file size */
  if(2*linelen*nlines-nlines-linelen != nel || (filelen % size)){
    fflush(NULL);
    fprintf(sp0,"File %s wrong size (%ld elements expected)\nAbort\n",
            filename,2*linelen*nlines-nlines-linelen);
    exit(ABNORMAL_EXIT);
  }

  /* get memory if passed pointer is NULL */
  nrow=tileparams->nrow;
  ncol=tileparams->ncol;
  if(*arr==NULL){
    (*arr)=Get2DRowColMem(nrow,ncol,sizeof(void *),size);
  }

  /* read arrays */
  fseek(fp,(linelen*tileparams->firstrow+tileparams->firstcol)
        *size,SEEK_SET);
  padlen=(linelen-ncol)*size;
  for(row=0; row<nrow-1; row++){
    if(fread((*arr)[row],size,ncol,fp)!=ncol){
      fflush(NULL);
      fprintf(sp0,"Error while reading from file %s\nAbort\n",filename);
      exit(ABNORMAL_EXIT);
    }
    fseek(fp,padlen,SEEK_CUR);
  }
  fseek(fp,(linelen*(nlines-1)+(linelen-1)*tileparams->firstrow
            +tileparams->firstcol)*size,SEEK_SET);
  for(row=nrow-1; row<2*nrow-1; row++){
    if(fread((*arr)[row],size,ncol-1,fp)!=ncol-1){
      fflush(NULL);
      fprintf(sp0,"Error while reading from file %s\nAbort\n",filename);
      exit(ABNORMAL_EXIT);
    }
    fseek(fp,padlen,SEEK_CUR);
  }
  fclose(fp);

  /* done */
  return(0);

}


/* function: Read2DRowColFileRows()
 * --------------------------------
 * Similar to Read2DRowColFile(), except reads only row (horizontal) data
 * at specified locations.  tileparams->nrow is treated as the number of
 * rows of data to be read from the RowCol file, not the number of
 * equivalent rows in the orginal pixel file (whose arcs are represented
 * in the RowCol file).
 */
int Read2DRowColFileRows(void ***arr, char *filename, long linelen,
                         long nlines, tileparamT *tileparams, size_t size){

  FILE *fp;
  long row, nel, nrow, ncol, padlen, filelen;

  /* open the file */
  if((fp=fopen(filename,"r"))==NULL){
    fflush(NULL);
    fprintf(sp0,"Can't open file %s\nAbort\n",filename);
    exit(ABNORMAL_EXIT);
  }

  /* get number of data elements in file */
  fseek(fp,0,SEEK_END);
  filelen=ftell(fp);
  fseek(fp,0,SEEK_SET);
  nel=(long )(filelen/size);

  /* check file size */
  if(2*linelen*nlines-nlines-linelen != nel || (filelen % size)){
    fflush(NULL);
    fprintf(sp0,"File %s wrong size (%ld elements expected)\nAbort\n",
            filename,2*linelen*nlines-nlines-linelen);
    exit(ABNORMAL_EXIT);
  }

  /* get memory if passed pointer is NULL */
  nrow=tileparams->nrow;
  ncol=tileparams->ncol;
  if(*arr==NULL){
    (*arr)=Get2DMem(nrow,ncol,sizeof(void *),size);
  }

  /* read arrays */
  fseek(fp,(linelen*tileparams->firstrow+tileparams->firstcol)
        *size,SEEK_SET);
  padlen=(linelen-ncol)*size;
  for(row=0; row<nrow; row++){
    if(fread((*arr)[row],size,ncol,fp)!=ncol){
      fflush(NULL);
      fprintf(sp0,"Error while reading from file %s\nAbort\n",filename);
      exit(ABNORMAL_EXIT);
    }
    fseek(fp,padlen,SEEK_CUR);
  }
  fclose(fp);

  /* done */
  return(0);

}


/* function: SetDumpAll()
 * ----------------------
 * Sets names of output files so that the program will dump intermediate
 * arrays.  Only sets names if they are not set already.
 */
int SetDumpAll(outfileT *outfiles, paramT *params){

  if(params->dumpall){
    if(!strlen(outfiles->initfile)){
      StrNCopy(outfiles->initfile,DUMP_INITFILE,MAXSTRLEN);
    }
    if(!strlen(outfiles->flowfile)){
      StrNCopy(outfiles->flowfile,DUMP_FLOWFILE,MAXSTRLEN);
    }
    if(!strlen(outfiles->eifile)){
      StrNCopy(outfiles->eifile,DUMP_EIFILE,MAXSTRLEN);
    }
    if(!strlen(outfiles->rowcostfile)){
      StrNCopy(outfiles->rowcostfile,DUMP_ROWCOSTFILE,MAXSTRLEN);
    }
    if(!strlen(outfiles->colcostfile)){
      StrNCopy(outfiles->colcostfile,DUMP_COLCOSTFILE,MAXSTRLEN);
    }
    if(!strlen(outfiles->mstrowcostfile)){
      StrNCopy(outfiles->mstrowcostfile,DUMP_MSTROWCOSTFILE,MAXSTRLEN);
    }
    if(!strlen(outfiles->mstcolcostfile)){
      StrNCopy(outfiles->mstcolcostfile,DUMP_MSTCOLCOSTFILE,MAXSTRLEN);
    }
    if(!strlen(outfiles->mstcostsfile)){
      StrNCopy(outfiles->mstcostsfile,DUMP_MSTCOSTSFILE,MAXSTRLEN);
    }
    if(!strlen(outfiles->corrdumpfile)){
      StrNCopy(outfiles->corrdumpfile,DUMP_CORRDUMPFILE,MAXSTRLEN);
    }
    if(!strlen(outfiles->rawcorrdumpfile)){
      StrNCopy(outfiles->rawcorrdumpfile,DUMP_RAWCORRDUMPFILE,MAXSTRLEN);
    }
  }
  return(0);
}


/* function: SetStreamPointers()
 * -----------------------------
 * Sets the default stream pointers (global variables).
 */
int SetStreamPointers(void){

  fflush(NULL);
  if((sp0=DEF_ERRORSTREAM)==NULL){
    if((sp0=fopen(NULLFILE,"w"))==NULL){
      fflush(NULL);
      fprintf(sp0,"unable to open null file %s\n",NULLFILE);
      exit(ABNORMAL_EXIT);
    }
  }
  if((sp1=DEF_OUTPUTSTREAM)==NULL){
    if((sp1=fopen(NULLFILE,"w"))==NULL){
      fflush(NULL);
      fprintf(sp0,"unable to open null file %s\n",NULLFILE);
      exit(ABNORMAL_EXIT);
    }
  }
  if((sp2=DEF_VERBOSESTREAM)==NULL){
    if((sp2=fopen(NULLFILE,"w"))==NULL){
      fflush(NULL);
      fprintf(sp0,"unable to open null file %s\n",NULLFILE);
      exit(ABNORMAL_EXIT);
    }
  }
  if((sp3=DEF_COUNTERSTREAM)==NULL){
    if((sp3=fopen(NULLFILE,"w"))==NULL){
      fflush(NULL);
      fprintf(sp0,"unable to open null file %s\n",NULLFILE);
      exit(ABNORMAL_EXIT);
    }
  }
  return(0);
}


/* function: SetVerboseOut()
 * -------------------------
 * Set the global stream pointer sp2 to be stdout if the verbose flag
 * is set in the parameter data type.
 */
int SetVerboseOut(paramT *params){

  fflush(NULL);
  if(params->verbose){
    if(sp2!=stdout && sp2!=stderr && sp2!=stdin && sp2!=NULL){
      fclose(sp2);
    }
    sp2=stdout;
    if(sp3!=stdout && sp3!=stderr && sp3!=stdin && sp3!=NULL){
      fclose(sp3);
    }
    sp3=stdout;
  }
  return(0);
}

/* function: DumpIncrCostFiles()
 * -----------------------------
 * Dumps incremental cost arrays, creating file names for them.
 */
int DumpIncrCostFiles(incrcostT **incrcosts, long iincrcostfile,
                      long nflow, long nrow, long ncol){

  long row, col, maxcol;
  char incrcostfile[MAXSTRLEN];
  char tempstr[MAXSTRLEN];
  short **tempcosts;

  /* get memory for tempcosts */
  tempcosts=(short **)Get2DRowColMem(nrow,ncol,sizeof(short *),sizeof(short));

  /* create the file names and dump the files */
  /* snprintf() is more elegant, but its unavailable on some machines */
  for(row=0;row<2*nrow-1;row++){
    if(row<nrow-1){
      maxcol=ncol;
    }else{
      maxcol=ncol-1;
    }
    for(col=0;col<maxcol;col++){
      tempcosts[row][col]=incrcosts[row][col].poscost;
    }
  }
  strncpy(incrcostfile,INCRCOSTFILEPOS,MAXSTRLEN-1);
  sprintf(tempstr,".%ld_%ld",iincrcostfile,nflow);
  strncat(incrcostfile,tempstr,MAXSTRLEN-strlen(incrcostfile)-1);
  Write2DRowColArray((void **)tempcosts,incrcostfile,
                     nrow,ncol,sizeof(short));
  for(row=0;row<2*nrow-1;row++){
    if(row<nrow-1){
      maxcol=ncol;
    }else{
      maxcol=ncol-1;
    }
    for(col=0;col<maxcol;col++){
      tempcosts[row][col]=incrcosts[row][col].negcost;
    }
  }
  strncpy(incrcostfile,INCRCOSTFILENEG,MAXSTRLEN-1);
  sprintf(tempstr,".%ld_%ld",iincrcostfile,nflow);
  strncat(incrcostfile,tempstr,MAXSTRLEN-strlen(incrcostfile)-1);
  Write2DRowColArray((void **)tempcosts,incrcostfile,
                     nrow,ncol,sizeof(short));

  /* free memory */
  Free2DArray((void **)tempcosts,2*nrow-1);

  /* done */
  return(0);

}


/* function: MakeTileDir()
 * -----------------------
 * Create a temporary directory for tile files in directory of output file.
 * Save directory name in buffer in paramT structure.
 */
int MakeTileDir(paramT *params, outfileT *outfiles){

  char path[MAXSTRLEN], basename[MAXSTRLEN];
  struct stat statbuf[1];


  /* initialize, including statubf for good measure even though not used */
  memset(path,0,MAXSTRLEN);
  memset(basename,0,MAXSTRLEN);
  memset(statbuf,0,sizeof(struct stat));

  /* create name for tile directory if necessary (use pid to make unique) */
  if(!strlen(params->tiledir)){
    ParseFilename(outfiles->outfile,path,basename);
    sprintf(params->tiledir,"%s%s%ld",path,TMPTILEDIRROOT,params->parentpid);
  }

  /* return if directory exists */
  /* this is a hack; tiledir could be file or could give other stat() error */
  /*   but if there is a problem, the error will be caught later */
  if(!stat(params->tiledir,statbuf)){
    return(0);
  }

  /* create tile directory */
  fprintf(sp1,"Creating temporary directory %s\n",params->tiledir);
  if(mkdir(params->tiledir,TILEDIRMODE)){
    fflush(NULL);
    fprintf(sp0,"Error creating directory %s\nAbort\n",params->tiledir);
    exit(ABNORMAL_EXIT);
  }

  /* done */
  return(0);

}


/* function: SetTileInitOutfile()
 * ------------------------------
 * Set name of temporary tile-mode output assuming nominal output file
 * name is in string passed.  Write new name in string memory pointed
 * to by input.
 */
int SetTileInitOutfile(char *outfile, long pid){

  char path[MAXSTRLEN], basename[MAXSTRLEN];
  struct stat statbuf[1];


  /* initialize, including statubf for good measure even though not used */
  memset(path,0,MAXSTRLEN);
  memset(basename,0,MAXSTRLEN);
  memset(statbuf,0,sizeof(struct stat));

  /* create name for output file (use pid to make unique) */
  ParseFilename(outfile,path,basename);
  sprintf(outfile,"%s%s%ld_%s",path,TILEINITFILEROOT,pid,basename);

  /* see if file already exists and exit if so */
  if(!stat(outfile,statbuf)){
    fprintf(sp0,
            "ERROR: refusing to write tile init to existing file %s\n",outfile);
    exit(ABNORMAL_EXIT);
  }

  /* done */
  return(0);

}


/* function: ParseFilename()
 * -------------------------
 * Given a filename, separates it into path and base filename.  Output
 * buffers should be at least MAXSTRLEN characters, and filename buffer
 * should be no more than MAXSTRLEN characters.  The output path
 * has a trailing "/" character.
 */
int ParseFilename(char *filename, char *path, char *basename){

  char tempstring[MAXSTRLEN];
  char *tempouttok;

  /* make sure we have a nonzero filename */
  if(!strlen(filename)){
    fflush(NULL);
    fprintf(sp0,"Zero-length filename passed to ParseFilename()\nAbort\n");
    exit(ABNORMAL_EXIT);
  }

  /* initialize path */
  if(filename[0]=='/'){
    StrNCopy(path,"/",MAXSTRLEN);
  }else{
    StrNCopy(path,"",MAXSTRLEN);
  }

  /* parse the filename */
  StrNCopy(tempstring,filename,MAXSTRLEN);
  tempouttok=strtok(tempstring,"/");
  while(TRUE){
    StrNCopy(basename,tempouttok,MAXSTRLEN);
    if((tempouttok=strtok(NULL,"/"))==NULL){
      break;
    }
    strcat(path,basename);
    strcat(path,"/");
  }

  /* make sure we have a nonzero base filename */
  if(!strlen(basename)){
    fflush(NULL);
    fprintf(sp0,"Zero-length base filename found in ParseFilename()\nAbort\n");
    exit(ABNORMAL_EXIT);
  }

  /* done */
  return(0);

}

/* end snaphu_io.c */
/* snaphu_solver.c */
/*************************************************************************

  snaphu network-flow solver source file
  Written by Curtis W. Chen
  Copyright 2002 Board of Trustees, Leland Stanford Jr. University
  Please see the supporting documentation for terms of use.
  No warranty.

*************************************************************************/

/* static variables local this file */

/* pointers to functions for tailoring network solver to specific topologies */
static nodeT *(*NeighborNode)(nodeT *, long, long *, nodeT **, nodeT *, long *,
                              long *, long *, long, long, boundaryT *,
                              nodesuppT **);
static void (*GetArc)(nodeT *, nodeT *, long *, long *, long *, long, long,
                      nodeT **, nodesuppT **);

/* static (local) function prototypes */
static
void AddNewNode(nodeT *from, nodeT *to, long arcdir, bucketT *bkts,
                long nflow, incrcostT **incrcosts, long arcrow, long arccol,
                paramT *params);
static
void CheckArcReducedCost(nodeT *from, nodeT *to, nodeT *apex,
                         long arcrow, long arccol, long arcdir,
                         candidateT **candidatebagptr,
                         long *candidatebagnextptr,
                         long *candidatebagsizeptr, incrcostT **incrcosts,
                         signed char **iscandidate, paramT *params);
static
nodeT *InitBoundary(nodeT *source, nodeT **nodes,
                    boundaryT *boundary, nodesuppT **nodesupp, float **mag,
                    nodeT *ground, long ngroundarcs, long nrow, long ncol,
                    paramT *params, long *nconnectedptr);
static
long CheckBoundary(nodeT **nodes, float **mag, nodeT *ground, long ngroundarcs,
                   boundaryT *boundary, long nrow, long ncol,
                   paramT *params, nodeT *start);
static
int IsRegionEdgeArc(float **mag, long arcrow, long arccol,
                    long nrow, long ncol);
static
int IsRegionInteriorArc(float **mag, long arcrow, long arccol,
                        long nrow, long ncol);
static
int IsRegionArc(float **mag, long arcrow, long arccol,
                long nrow, long ncol);
static
int IsRegionEdgeNode(float **mag, long row, long col, long nrow, long ncol);
static
int CleanUpBoundaryNodes(nodeT *source, boundaryT *boundary, float **mag,
                         nodeT **nodes, nodeT *ground,
                         long nrow, long ncol, long ngroundarcs,
                         nodesuppT **nodesupp);
static
int DischargeBoundary(nodeT **nodes, nodeT *ground,
                      boundaryT *boundary, nodesuppT **nodesupp, short **flows,
                      signed char **iscandidate, float **mag,
                      float **wrappedphase, long ngroundarcs,
                      long nrow, long ncol);
static
int InitTree(nodeT *source, nodeT **nodes,
             boundaryT *boundary, nodesuppT **nodesupp,
             nodeT *ground, long ngroundarcs, bucketT *bkts, long nflow,
             incrcostT **incrcosts, long nrow, long ncol, paramT *params);
static
nodeT *FindApex(nodeT *from, nodeT *to);
static
int CandidateCompare(const void *c1, const void *c2);
static
nodeT *NeighborNodeGrid(nodeT *node1, long arcnum, long *upperarcnumptr,
                        nodeT **nodes, nodeT *ground, long *arcrowptr,
                        long *arccolptr, long *arcdirptr, long nrow,
                        long ncol, boundaryT *boundary, nodesuppT **nodesupp);
static inline
long GetArcNumLims(long fromrow, long *upperarcnumptr,
                   long ngroundarcs, boundaryT *boundary);
static
nodeT *NeighborNodeNonGrid(nodeT *node1, long arcnum, long *upperarcnumptr,
                           nodeT **nodes, nodeT *ground, long *arcrowptr,
                           long *arccolptr, long *arcdirptr, long nrow,
                           long ncol, boundaryT *boundary,
                           nodesuppT **nodesupp);
static
void GetArcGrid(nodeT *from, nodeT *to, long *arcrow, long *arccol,
                long *arcdir, long nrow, long ncol,
                nodeT **nodes, nodesuppT **nodesupp);
static
void GetArcNonGrid(nodeT *from, nodeT *to, long *arcrow, long *arccol,
                   long *arcdir, long nrow, long ncol,
                   nodeT **nodes, nodesuppT **nodesupp);
static
void NonDegenUpdateChildren(nodeT *startnode, nodeT *lastnode,
                            nodeT *nextonpath, long dgroup,
                            long ngroundarcs, long nflow, nodeT **nodes,
                            nodesuppT **nodesupp, nodeT *ground,
                            boundaryT *boundary,
                            nodeT ***apexes, incrcostT **incrcosts,
                            long nrow, long ncol, paramT *params);
static
long PruneTree(nodeT *source, nodeT **nodes, nodeT *ground, boundaryT *boundary,
               nodesuppT **nodesupp, incrcostT **incrcosts,
               short **flows, long ngroundarcs, long prunecostthresh,
               long nrow, long ncol);
static
int CheckLeaf(nodeT *node1, nodeT **nodes, nodeT *ground, boundaryT *boundary,
              nodesuppT **nodesupp, incrcostT **incrcosts,
              short **flows, long ngroundarcs, long nrow, long ncol,
              long prunecostthresh);
static
int GridNodeMaskStatus(long row, long col, float **mag);
static
int GroundMaskStatus(long nrow, long ncol, float **mag);
static
int InitBuckets(bucketT *bkts, nodeT *source, long nbuckets);
static
nodeT *MinOutCostNode(bucketT *bkts);
static
nodeT *SelectConnNodeSource(nodeT **nodes, float **mag,
                            nodeT *ground, long ngroundarcs,
                            long nrow, long ncol,
                            paramT *params, nodeT *start, long *nconnectedptr);
static
long ScanRegion(nodeT *start, nodeT **nodes, float **mag,
                nodeT *ground, long ngroundarcs,
                long nrow, long ncol, int groupsetting);
static
short GetCost(incrcostT **incrcosts, long arcrow, long arccol,
              long arcdir);
static
void SolveMST(nodeT **nodes, nodeT *source, nodeT *ground,
              bucketT *bkts, short **mstcosts, signed char **residue,
              signed char **arcstatus, long nrow, long ncol);
static
long DischargeTree(nodeT *source, short **mstcosts, short **flows,
                   signed char **residue, signed char **arcstatus,
                   nodeT **nodes, nodeT *ground, long nrow, long ncol);
static
signed char ClipFlow(signed char **residue, short **flows,
                     short **mstcosts, long nrow, long ncol,
                     long maxflow);



/* function: SetGridNetworkFunctionPointers()
 * ------------------------------------------
 */
int SetGridNetworkFunctionPointers(void){

  /* set static pointers to functions */
  NeighborNode=NeighborNodeGrid;
  GetArc=GetArcGrid;
  return(0);

}


/* function: SetNonGridNetworkFunctionPointers()
 * ---------------------------------------------
 */
int SetNonGridNetworkFunctionPointers(void){

  /* set static pointers to functions */
  NeighborNode=NeighborNodeNonGrid;
  GetArc=GetArcNonGrid;
  return(0);

}


/* function: TreeSolve()
 * ---------------------
 * Solves the nonlinear network optimization problem.
 */
long TreeSolve(nodeT **nodes, nodesuppT **nodesupp, nodeT *ground,
               nodeT *source, candidateT **candidatelistptr,
               candidateT **candidatebagptr, long *candidatelistsizeptr,
               long *candidatebagsizeptr, bucketT *bkts, short **flows,
               void **costs, incrcostT **incrcosts, nodeT ***apexes,
               signed char **iscandidate, long ngroundarcs, long nflow,
               float **mag, float **wrappedphase, char *outfile,
               long nnoderow, int *nnodesperrow, long narcrow,
               int *narcsperrow, long nrow, long ncol,
               outfileT *outfiles, long nconnected, paramT *params){

  long i, row, col, arcrow, arccol, arcdir, arcnum, upperarcnum;
  long arcrow1, arccol1, arcdir1, arcrow2, arccol2, arcdir2;
  long treesize, candidatelistsize, candidatebagsize;
  long violation, groupcounter, fromgroup, group1, apexlistbase, apexlistlen;
  long cyclecost, outcostto, startlevel, dlevel, doutcost, dincost;
  long candidatelistlen, candidatebagnext;
  long inondegen, ipivots, nnewnodes, maxnewnodes, templong;
  long nmajor, nmajorprune, npruned, prunecostthresh;
  signed char fromside;
  candidateT *candidatelist, *candidatebag, *tempcandidateptr;
  nodeT *from, *to, *cycleapex, *node1, *node2, *leavingparent, *leavingchild;
  nodeT *root, *mntpt, *oldmntpt, *skipthread, *tempnode1, *tempnode2;
  nodeT *firstfromnode, *firsttonode;
  nodeT **apexlist;
  boundaryT boundary[1];
  float **unwrappedphase;


  /* initilize structures on stack to zero for good measure */
  memset(boundary,0,sizeof(boundaryT));

  /* initialize some variables to zero to stop compiler warnings */
  from=NULL;
  to=NULL;
  cycleapex=NULL;
  node1=NULL;
  node2=NULL;
  leavingparent=NULL;
  leavingchild=NULL;
  root=NULL;
  mntpt=NULL;
  oldmntpt=NULL;
  skipthread=NULL;
  tempnode1=NULL;
  tempnode2=NULL;
  firstfromnode=NULL;
  firsttonode=NULL;

  /* dereference some pointers and store as local variables */
  candidatelist=(*candidatelistptr);
  candidatebag=(*candidatebagptr);
  candidatelistsize=(*candidatelistsizeptr);
  candidatebagsize=(*candidatebagsizeptr);
  candidatelistlen=0;
  candidatebagnext=0;

  /* initialize boundary, which affects network structure */
  /* recompute number of connected nodes since setting boundary may make */
  /*   some nodes inaccessible */
  source=InitBoundary(source,nodes,boundary,nodesupp,mag,
                      ground,ngroundarcs,nrow,ncol,params,&nconnected);

  /* set up */
  bkts->curr=bkts->maxind;
  InitTree(source,nodes,boundary,nodesupp,ground,ngroundarcs,bkts,nflow,
           incrcosts,nrow,ncol,params);
  apexlistlen=INITARRSIZE;
  apexlist=MAlloc(apexlistlen*sizeof(nodeT *));
  groupcounter=2;
  ipivots=0;
  inondegen=0;
  maxnewnodes=ceil(nconnected*params->maxnewnodeconst);
  nnewnodes=0;
  treesize=1;
  npruned=0;
  nmajor=0;
  nmajorprune=params->nmajorprune;
  prunecostthresh=params->prunecostthresh;;
  fprintf(sp3,"Treesize: %-10ld Pivots: %-11ld Improvements: %-11ld",
          treesize,ipivots,inondegen);

  /* loop over each entering node (note, source already on tree) */
  while(treesize<nconnected){

    nnewnodes=0;
    while(nnewnodes<maxnewnodes && treesize<nconnected){

      /* get node with lowest outcost */
      to=MinOutCostNode(bkts);
      from=to->pred;

      /* add new node to the tree */
      GetArc(from,to,&arcrow,&arccol,&arcdir,nrow,ncol,nodes,nodesupp);
      to->group=1;
      to->level=from->level+1;
      to->incost=from->incost+GetCost(incrcosts,arcrow,arccol,-arcdir);
      to->next=from->next;
      to->prev=from;
      to->next->prev=to;
      from->next=to;

      /* scan new node's neighbors */
      from=to;
      arcnum=GetArcNumLims(from->row,&upperarcnum,ngroundarcs,boundary);
      while(arcnum<upperarcnum){

        /* get row, col indices and distance of next node */
        to=NeighborNode(from,++arcnum,&upperarcnum,nodes,ground,
                        &arcrow,&arccol,&arcdir,nrow,ncol,boundary,nodesupp);

        /* if to node is on tree */
        if(to->group>0){
          if(to!=from->pred){
            cycleapex=FindApex(from,to);
            apexes[arcrow][arccol]=cycleapex;
            CheckArcReducedCost(from,to,cycleapex,arcrow,arccol,arcdir,
                                &candidatebag,&candidatebagnext,
                                &candidatebagsize,incrcosts,iscandidate,
                                params);
          }else{
            apexes[arcrow][arccol]=NULL;
          }

        }else if(to->group!=PRUNED && to->group!=MASKED){

          /* if to is not on tree, update outcost and add to bucket */
          AddNewNode(from,to,arcdir,bkts,nflow,incrcosts,arcrow,arccol,params);

        }
      }
      nnewnodes++;
      treesize++;
    }

    /* keep looping until no more arcs have negative reduced costs */
    while(candidatebagnext){

      /* if we received SIGINT or SIGHUP signal, dump results */
      /* keep this stuff out of the signal handler so we don't risk */
      /* writing a non-feasible solution (ie, if signal during augment) */
      /* signal handler disabled for all but primary (grid) networks */
      if(dumpresults_global){
        fflush(NULL);
        fprintf(sp0,"\n\nDumping current solution to file %s\n",
                outfile);
        if(requestedstop_global){
          Free2DArray((void **)costs,2*nrow-1);
        }
        unwrappedphase=(float **)Get2DMem(nrow,ncol,sizeof(float *),
                                          sizeof(float));
        IntegratePhase(wrappedphase,unwrappedphase,flows,nrow,ncol);
        FlipPhaseArraySign(unwrappedphase,params,nrow,ncol);
        WriteOutputFile(mag,unwrappedphase,outfiles->outfile,outfiles,
                        nrow,ncol);
        if(requestedstop_global){
          fflush(NULL);
          fprintf(sp0,"Program exiting\n");
          exit(ABNORMAL_EXIT);
        }
        Free2DArray((void **)unwrappedphase,nrow);
        dumpresults_global=FALSE;
        fflush(NULL);
        fprintf(sp0,"\n\nProgram continuing\n");
      }

      /* swap candidate bag and candidate list pointers and sizes */
      tempcandidateptr=candidatebag;
      candidatebag=candidatelist;
      candidatelist=tempcandidateptr;
      templong=candidatebagsize;
      candidatebagsize=candidatelistsize;
      candidatelistsize=templong;
      candidatelistlen=candidatebagnext;
      candidatebagnext=0;

      /* sort candidate list by violation, with augmenting arcs always first */
      qsort((void *)candidatelist,candidatelistlen,sizeof(candidateT),
            CandidateCompare);

      /* set all arc directions to be plus/minus 1 */
      for(i=0;i<candidatelistlen;i++){
        if(candidatelist[i].arcdir>1){
          candidatelist[i].arcdir=1;
        }else if(candidatelist[i].arcdir<-1){
          candidatelist[i].arcdir=-1;
        }
      }

      /* this doesn't seem to make it any faster, so just do all of them */
      /* set the number of candidates to process */
      /* (must change candidatelistlen to ncandidates in for loop below) */
      /*
      maxcandidates=MAXCANDIDATES;
      if(maxcandidates>candidatelistlen){
        ncandidates=candidatelistlen;
      }else{
        ncandidates=maxcandidates;
      }
      */

      /* now pivot for each arc in the candidate list */
      for(i=0;i<candidatelistlen;i++){

        /* get arc info */
        from=candidatelist[i].from;
        to=candidatelist[i].to;
        arcdir=candidatelist[i].arcdir;
        arcrow=candidatelist[i].arcrow;
        arccol=candidatelist[i].arccol;

        /* unset iscandidate */
        iscandidate[arcrow][arccol]=FALSE;

        /* make sure the next arc still has a negative violation */
        outcostto=from->outcost+
          GetCost(incrcosts,arcrow,arccol,arcdir);
        cyclecost=outcostto + to->incost
          -apexes[arcrow][arccol]->outcost
          -apexes[arcrow][arccol]->incost;

        /* if violation no longer negative, check reverse arc */
        if(!((outcostto < to->outcost) || (cyclecost < 0))){
          from=to;
          to=candidatelist[i].from;
          arcdir=-arcdir;
          outcostto=from->outcost+
            GetCost(incrcosts,arcrow,arccol,arcdir);
          cyclecost=outcostto + to->incost
            -apexes[arcrow][arccol]->outcost
            -apexes[arcrow][arccol]->incost;
        }

        /* see if the cycle is negative (see if there is a violation) */
        if((outcostto < to->outcost) || (cyclecost < 0)){

          /* make sure the group counter hasn't gotten too big */
          if(++groupcounter>MAXGROUPBASE){
            for(row=0;row<nnoderow;row++){
              for(col=0;col<nnodesperrow[row];col++){
                if(nodes[row][col].group>0){
                  nodes[row][col].group=1;
                }
              }
            }
            if(ground!=NULL && ground->group>0){
              ground->group=1;
            }
            if(boundary->node->group>0){
              boundary->node->group=1;
            }
            groupcounter=2;
          }

          /* if augmenting cycle (nondegenerate pivot) */
          if(cyclecost<0){

            /* augment flow along cycle and select leaving arc */
            /* if we are augmenting non-zero flow, any arc with zero flow */
            /* after the augmentation is a blocking arc */
            while(TRUE){
              fromside=TRUE;
              node1=from;
              node2=to;
              leavingchild=NULL;
              flows[arcrow][arccol]+=arcdir*nflow;
              ReCalcCost(costs,incrcosts,flows[arcrow][arccol],arcrow,arccol,
                         nflow,nrow,params);
              violation=GetCost(incrcosts,arcrow,arccol,arcdir);
              if(node1->level > node2->level){
                while(node1->level != node2->level){
                  GetArc(node1->pred,node1,&arcrow1,&arccol1,&arcdir1,
                         nrow,ncol,nodes,nodesupp);
                  flows[arcrow1][arccol1]+=(arcdir1*nflow);
                  ReCalcCost(costs,incrcosts,flows[arcrow1][arccol1],
                             arcrow1,arccol1,nflow,nrow,params);
                  if(leavingchild==NULL
                     && !flows[arcrow1][arccol1]){
                    leavingchild=node1;
                  }
                  violation+=GetCost(incrcosts,arcrow1,arccol1,arcdir1);
                  node1->group=groupcounter+1;
                  node1=node1->pred;
                }
              }else{
                while(node1->level != node2->level){
                  GetArc(node2->pred,node2,&arcrow2,&arccol2,&arcdir2,
                         nrow,ncol,nodes,nodesupp);
                  flows[arcrow2][arccol2]-=(arcdir2*nflow);
                  ReCalcCost(costs,incrcosts,flows[arcrow2][arccol2],
                             arcrow2,arccol2,nflow,nrow,params);
                  if(!flows[arcrow2][arccol2]){
                    leavingchild=node2;
                    fromside=FALSE;
                  }
                  violation+=GetCost(incrcosts,arcrow2,arccol2,-arcdir2);
                  node2->group=groupcounter;
                  node2=node2->pred;
                }
              }
              while(node1!=node2){
                GetArc(node1->pred,node1,&arcrow1,&arccol1,&arcdir1,nrow,ncol,
                       nodes,nodesupp);
                GetArc(node2->pred,node2,&arcrow2,&arccol2,&arcdir2,nrow,ncol,
                       nodes,nodesupp);
                flows[arcrow1][arccol1]+=(arcdir1*nflow);
                flows[arcrow2][arccol2]-=(arcdir2*nflow);
                ReCalcCost(costs,incrcosts,flows[arcrow1][arccol1],
                           arcrow1,arccol1,nflow,nrow,params);
                ReCalcCost(costs,incrcosts,flows[arcrow2][arccol2],
                           arcrow2,arccol2,nflow,nrow,params);
                violation+=(GetCost(incrcosts,arcrow1,arccol1,arcdir1)
                            +GetCost(incrcosts,arcrow2,arccol2,-arcdir2));
                if(!flows[arcrow2][arccol2]){
                  leavingchild=node2;
                  fromside=FALSE;
                }else if(leavingchild==NULL
                         && !flows[arcrow1][arccol1]){
                  leavingchild=node1;
                }
                node1->group=groupcounter+1;
                node2->group=groupcounter;
                node1=node1->pred;
                node2=node2->pred;
              }
              if(violation>=0){
                break;
              }
            }
            inondegen++;

          }else{

            /* We are not augmenting flow, but just updating potentials. */
            /* Arcs with zero flow are implicitly directed upwards to */
            /* maintain a strongly feasible spanning tree, so arcs with zero */
            /* flow on the path between to node and apex are blocking arcs. */
            /* Leaving arc is last one whose child's new outcost is less */
            /* than its old outcost.  Such an arc must exist, or else */
            /* we'd be augmenting flow on a negative cycle. */

            /* trace the cycle and select leaving arc */
            fromside=FALSE;
            node1=from;
            node2=to;
            leavingchild=NULL;
            if(node1->level > node2->level){
              while(node1->level != node2->level){
                node1->group=groupcounter+1;
                node1=node1->pred;
              }
            }else{
              while(node1->level != node2->level){
                if(outcostto < node2->outcost){
                  leavingchild=node2;
                  GetArc(node2->pred,node2,&arcrow2,&arccol2,&arcdir2,
                         nrow,ncol,nodes,nodesupp);
                  outcostto+=GetCost(incrcosts,arcrow2,arccol2,-arcdir2);
                }else{
                  outcostto=VERYFAR;
                }
                node2->group=groupcounter;
                node2=node2->pred;
              }
            }
            while(node1!=node2){
              if(outcostto < node2->outcost){
                leavingchild=node2;
                GetArc(node2->pred,node2,&arcrow2,&arccol2,&arcdir2,nrow,ncol,
                       nodes,nodesupp);
                outcostto+=GetCost(incrcosts,arcrow2,arccol2,-arcdir2);
              }else{
                outcostto=VERYFAR;
              }
              node1->group=groupcounter+1;
              node2->group=groupcounter;
              node1=node1->pred;
              node2=node2->pred;
            }
          }
          cycleapex=node1;

          /* set leaving parent */
          if(leavingchild==NULL){
            fromside=TRUE;
            leavingparent=from;
          }else{
            leavingparent=leavingchild->pred;
          }

          /* swap from and to if leaving arc is on the from side */
          if(fromside){
            groupcounter++;
            fromgroup=groupcounter-1;
            tempnode1=from;
            from=to;
            to=tempnode1;
          }else{
            fromgroup=groupcounter+1;
          }

          /* if augmenting pivot */
          if(cyclecost<0){

            /* find first child of apex on either cycle path */
            firstfromnode=NULL;
            firsttonode=NULL;
            arcnum=GetArcNumLims(cycleapex->row,&upperarcnum,
                                 ngroundarcs,boundary);
            while(arcnum<upperarcnum){
              tempnode1=NeighborNode(cycleapex,++arcnum,&upperarcnum,nodes,
                                     ground,&arcrow,&arccol,&arcdir,nrow,ncol,
                                     boundary,nodesupp);
              if(tempnode1->group==groupcounter
                 && apexes[arcrow][arccol]==NULL){
                firsttonode=tempnode1;
                if(firstfromnode!=NULL){
                  break;
                }
              }else if(tempnode1->group==fromgroup
                       && apexes[arcrow][arccol]==NULL){
                firstfromnode=tempnode1;
                if(firsttonode!=NULL){
                  break;
                }
              }
            }

            /* update potentials, mark stationary parts of tree */
            cycleapex->group=groupcounter+2;
            if(firsttonode!=NULL){
              NonDegenUpdateChildren(cycleapex,leavingparent,firsttonode,0,
                                     ngroundarcs,nflow,nodes,nodesupp,ground,
                                     boundary,apexes,incrcosts,nrow,ncol,
                                     params);
            }
            if(firstfromnode!=NULL){
              NonDegenUpdateChildren(cycleapex,from,firstfromnode,1,
                                     ngroundarcs,nflow,nodes,nodesupp,ground,
                                     boundary,apexes,incrcosts,nrow,ncol,
                                     params);
            }
            groupcounter=from->group;
            apexlistbase=cycleapex->group;

            /* children of cycleapex are not marked, so we set fromgroup */
            /*   equal to cycleapex group for use with apex updates below */
            /* all other children of cycle will be in apexlist if we had an */
            /*   augmenting pivot, so fromgroup only important for cycleapex */
            fromgroup=cycleapex->group;

          }else{

            /* set this stuff for use with apex updates below */
            cycleapex->group=fromgroup;
            groupcounter+=2;
            apexlistbase=groupcounter+1;
          }

          /* remount subtree at new mount point */
          if(leavingchild==NULL){

            skipthread=to;

          }else{

            root=from;
            oldmntpt=to;

            /* for each node on the path from to node to leaving child */
            while(oldmntpt!=leavingparent){

              /* remount the subtree at the new mount point */
              mntpt=root;
              root=oldmntpt;
              oldmntpt=root->pred;
              root->pred=mntpt;
              GetArc(mntpt,root,&arcrow,&arccol,&arcdir,nrow,ncol,
                     nodes,nodesupp);

              /* calculate differences for updating potentials and levels */
              dlevel=mntpt->level-root->level+1;
              doutcost=mntpt->outcost - root->outcost
                + GetCost(incrcosts,arcrow,arccol,arcdir);
              dincost=mntpt->incost - root->incost
                + GetCost(incrcosts,arcrow,arccol,-arcdir);

              /* update all children */
              /* group of each remounted tree used to reset apexes below */
              node1=root;
              startlevel=root->level;
              groupcounter++;
              while(TRUE){

                /* update the level, potentials, and group of the node */
                node1->level+=dlevel;
                node1->outcost+=doutcost;
                node1->incost+=dincost;
                node1->group=groupcounter;

                /* break when node1 is no longer descendent of the root */
                if(node1->next->level <= startlevel){
                  break;
                }
                node1=node1->next;
              }

              /* update threads */
              root->prev->next=node1->next;
              node1->next->prev=root->prev;
              node1->next=mntpt->next;
              mntpt->next->prev=node1;
              mntpt->next=root;
              root->prev=mntpt;

            }
            skipthread=node1->next;

            /* reset apex pointers for entering and leaving arcs */
            GetArc(from,to,&arcrow,&arccol,&arcdir,nrow,ncol,nodes,nodesupp);
            apexes[arcrow][arccol]=NULL;
            GetArc(leavingparent,leavingchild,&arcrow,&arccol,
                   &arcdir,nrow,ncol,nodes,nodesupp);
            apexes[arcrow][arccol]=cycleapex;

            /* make sure we have enough memory for the apex list */
            if(groupcounter-apexlistbase+1>apexlistlen){
              apexlistlen=1.5*(groupcounter-apexlistbase+1);
              apexlist=ReAlloc(apexlist,apexlistlen*sizeof(nodeT *));
            }

            /* set the apex list */
            /* the apex list is a look up table of apex node pointers indexed */
            /*   by the group number relative to a base group value */
            node2=leavingchild;
            for(group1=groupcounter;group1>=apexlistbase;group1--){
              apexlist[group1-apexlistbase]=node2;
              node2=node2->pred;
            }

            /* reset apex pointers on remounted tree */
            /* only nodes which are in different groups need new apexes */
            node1=to;
            startlevel=to->level;
            while(TRUE){

              /* loop over outgoing arcs */
              arcnum=GetArcNumLims(node1->row,&upperarcnum,
                                   ngroundarcs,boundary);
              while(arcnum<upperarcnum){
                node2=NeighborNode(node1,++arcnum,&upperarcnum,nodes,ground,
                                   &arcrow,&arccol,&arcdir,nrow,ncol,
                                   boundary,nodesupp);

                /* if node2 on tree */
                if(node2->group>0){


                  /* if node2 is either not part of remounted tree or */
                  /*   it is higher on remounted tree than node1, */
                  /*   and arc isn't already on tree */
                  if(node2->group < node1->group
                     && apexes[arcrow][arccol]!=NULL){

                    /* if new apex in apexlist */
                    /* node2 on remounted tree, if nonaugmenting pivot */
                    if(node2->group >= apexlistbase){

                      apexes[arcrow][arccol]=apexlist[node2->group
                                                     -apexlistbase];

                    }else{

                      /* if old apex below level of cycleapex, */
                      /*   node2 is on "to" node's side of tree */
                      /* implicitly, if old apex above cycleapex, */
                      /*   we do nothing since apex won't change */
                      if(apexes[arcrow][arccol]->level > cycleapex->level){

                        /* since new apex not in apexlist (tested above), */
                        /* node2 above leaving arc so new apex is cycleapex */
                        apexes[arcrow][arccol]=cycleapex;

                      }else{

                        /* node2 not on "to" side of tree */
                        /* if old apex is cycleapex, node2 is on "from" side */
                        if(apexes[arcrow][arccol]==cycleapex){

                          /* new apex will be on cycle, so trace node2->pred */
                          /*   until we hit a node with group==fromgroup */
                          tempnode2=node2;
                          while(tempnode2->group != fromgroup){
                            tempnode2=tempnode2->pred;
                          }
                          apexes[arcrow][arccol]=tempnode2;

                        }
                      }
                    }

                    /* check outgoing arcs for negative reduced costs */
                    CheckArcReducedCost(node1,node2,apexes[arcrow][arccol],
                                        arcrow,arccol,arcdir,&candidatebag,
                                        &candidatebagnext,&candidatebagsize,
                                        incrcosts,iscandidate,params);

                  } /* end if node2 below node1 and arc not on tree */

                }else if(node2->group!=PRUNED && node2->group!=MASKED){

                  /* node2 is not on tree, so put it in correct bucket */
                  AddNewNode(node1,node2,arcdir,bkts,nflow,incrcosts,
                             arcrow,arccol,params);

                } /* end if node2 on tree */
              } /* end loop over node1 outgoing arcs */


              /* move to next node in thread, break if we left the subtree */
              node1=node1->next;
              if(node1->level <= startlevel){
                break;
              }
            }
          } /* end if leavingchild!=NULL */

          /* if we had an augmenting cycle */
          /* we need to check outarcs from descendents of any cycle node */
          /* (except apex, since apex potentials don't change) */
          if(cyclecost<0){

            /* check descendents of cycle children of apex */
            while(TRUE){

              /* firstfromnode, firsttonode may have changed */
              if(firstfromnode!=NULL && firstfromnode->pred==cycleapex){
                node1=firstfromnode;
                firstfromnode=NULL;
              }else if(firsttonode!=NULL && firsttonode->pred==cycleapex){
                node1=firsttonode;
                firsttonode=NULL;
              }else{
                break;
              }
              startlevel=node1->level;

              /* loop over all descendents */
              while(TRUE){

                /* loop over outgoing arcs */
                arcnum=GetArcNumLims(node1->row,&upperarcnum,
                                     ngroundarcs,boundary);
                while(arcnum<upperarcnum){
                  node2=NeighborNode(node1,++arcnum,&upperarcnum,nodes,ground,
                                     &arcrow,&arccol,&arcdir,nrow,ncol,
                                     boundary,nodesupp);

                  /* check for outcost updates or negative reduced costs */
                  if(node2->group>0){
                    if(apexes[arcrow][arccol]!=NULL
                       && (node2->group!=node1->group
                           || node1->group==apexlistbase)){
                      CheckArcReducedCost(node1,node2,apexes[arcrow][arccol],
                                          arcrow,arccol,arcdir,&candidatebag,
                                          &candidatebagnext,&candidatebagsize,
                                          incrcosts,iscandidate,params);
                    }
                  }else if(node2->group!=PRUNED && node2->group!=MASKED){
                    AddNewNode(node1,node2,arcdir,bkts,nflow,incrcosts,
                               arcrow,arccol,params);
                  }
                }

                /* move to next node in thread, break if left the subtree */
                /*   but skip the remounted tree, since we checked it above */
                node1=node1->next;
                if(node1==to){
                  node1=skipthread;
                }
                if(node1->level <= startlevel){
                  break;
                }
              }
            }
          }
          ipivots++;
        } /* end if cyclecost<0 || outcostto<to->outcost */
      } /* end of for loop over candidates in list */

      /* this is needed only if we don't process all candidates above */
      /* copy remaining candidates into candidatebag */
      /*
      while(candidatebagnext+(candidatelistlen-ncandidates)>candidatebagsize){
        candidatebagsize+=CANDIDATEBAGSTEP;
        candidatebag=ReAlloc(candidatebag,candidatebagsize*sizeof(candidateT));
      }
      for(i=ncandidates;i<candidatelistlen;i++){
        candidatebag[candidatebagnext++]=candidatelist[i];
      }
      */

      /* display status */
      fprintf(sp3,"\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b"
              "\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b"
              "\b\b\b\b\b\b"
              "Treesize: %-10ld Pivots: %-11ld Improvements: %-11ld",
              treesize,ipivots,inondegen);
      fflush(sp3);

    } /* end of while loop on candidatebagnext */

    /* prune tree by removing unneeded leaf nodes */
    if(!((++nmajor) % nmajorprune)){
      npruned+=PruneTree(source,nodes,ground,boundary,nodesupp,incrcosts,flows,
                         ngroundarcs,prunecostthresh,nrow,ncol);
    }

  } /* end while treesize<number of total nodes */

  /* sanity check tree structure */
  node1=source->next;
  while(node1!=source){
    if(node1->pred->level!=node1->level-1){
      printf("Error detected: row %d, col%d, level %d "
             "has pred row %d, col%d, level %d\n",
             node1->row,node1->col,node1->level,node1->pred->row,node1->pred->col,
             node1->pred->level);
    }
    node1=node1->next;
  }

  /* discharge boundary */
  /* flow to edge of region goes along normal grid arcs, but flow along edge */
  /*   of region that should go along zero-cost arcs along edge is not */
  /*   captured in solver code above since nodes of edge are collapsed into */
  /*   single boundary node.  This accumulates surplus/demand at edge nodes. */
  /*   Here, find surplus/demand by balancing flow in/out of edge nodes and */
  /*   discharge sending flow along zero-cost edge arcs */
  DischargeBoundary(nodes,ground,boundary,nodesupp,flows,iscandidate,
                    mag,wrappedphase,ngroundarcs,nrow,ncol);

  /* sanity check that buckets are actually all empty after optimizer is done */
  for(i=0;i<bkts->size;i++){
    if(bkts->bucketbase[i]!=NULL){
      printf("ERROR: bucket %ld not empty after TreeSolve (row=%d, col=%d)\n",
             i,bkts->bucketbase[i]->row,bkts->bucketbase[i]->col);
      break;
    }
  }

  /* reset group numbers of nodes along boundary */
  /* nodes in neighboring regions may have been set to be MASKED in */
  /*   in InitBoundary() to avoid reaching them across single line of */
  /*   masked pixels, so return mask status of nodes along boundary to normal */
  CleanUpBoundaryNodes(source,boundary,mag,nodes,ground,nrow,ncol,ngroundarcs,
                       nodesupp);
  if(boundary->neighborlist!=NULL){
    free(boundary->neighborlist);
  }
  if(boundary->boundarylist!=NULL){
    free(boundary->boundarylist);
  }

  /* clean up: set pointers for outputs */
  fprintf(sp3,"\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b"
          "\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b"
          "\b\b\b\b\b\b"
          "Treesize: %-10ld Pivots: %-11ld Improvements: %-11ld\n",
          treesize,ipivots,inondegen);
  fflush(sp3);
  *candidatelistptr=candidatelist;
  *candidatebagptr=candidatebag;
  *candidatelistsizeptr=candidatelistsize;
  *candidatebagsizeptr=candidatebagsize;
  free(apexlist);

  /* return the number of nondegenerate pivots (number of improvements) */
  return(inondegen);

}


/* function: AddNewNode()
 * ----------------------
 * Adds a node to a bucket if it is not already in a bucket.  Updates
 * outcosts of to node if the new distance is less or if to's pred is
 * from (then we have to do the update).
 */
static
void AddNewNode(nodeT *from, nodeT *to, long arcdir, bucketT *bkts,
                long nflow, incrcostT **incrcosts, long arcrow, long arccol,
                paramT *params){

  long newoutcost;


  newoutcost=from->outcost
    +GetCost(incrcosts,arcrow,arccol,arcdir);
  if(newoutcost<to->outcost || to->pred==from){
    if(to->group==INBUCKET){      /* if to is already in a bucket */
      if(to->outcost<bkts->maxind){
        if(to->outcost>bkts->minind){
          BucketRemove(to,to->outcost,bkts);
        }else{
          BucketRemove(to,bkts->minind,bkts);
        }
      }else{
        BucketRemove(to,bkts->maxind,bkts);
      }
    }
    to->outcost=newoutcost;
    to->pred=from;
    if(newoutcost<bkts->maxind){
      if(newoutcost>bkts->minind){
        BucketInsert(to,newoutcost,bkts);
        if(newoutcost<bkts->curr){
          bkts->curr=newoutcost;
        }
      }else{
        BucketInsert(to,bkts->minind,bkts);
        bkts->curr=bkts->minind;
      }
    }else{
      BucketInsert(to,bkts->maxind,bkts);
    }
    to->group=INBUCKET;
  }
  return;
}


/* function: CheckArcReducedCost()
 * -------------------------------
 * Given a from and to node, checks for negative reduced cost, and adds
 * the arc to the entering arc candidate bag if one is found.
 */
static
void CheckArcReducedCost(nodeT *from, nodeT *to, nodeT *apex,
                         long arcrow, long arccol, long arcdir,
                         candidateT **candidatebagptr,
                         long *candidatebagnextptr,
                         long *candidatebagsizeptr, incrcostT **incrcosts,
                         signed char **iscandidate, paramT *params){

  long apexcost, fwdarcdist, revarcdist, violation;
  nodeT *temp;


  /* do nothing if already candidate */
  /* illegal corner arcs have iscandidate=TRUE set ahead of time */
  if(iscandidate[arcrow][arccol]){
    return;
  }

  /* set the apex cost */
  apexcost=apex->outcost+apex->incost;

  /* check forward arc */
  fwdarcdist=GetCost(incrcosts,arcrow,arccol,arcdir);
  violation=fwdarcdist+from->outcost+to->incost-apexcost;
  if(violation<0){
    arcdir*=2;  /* magnitude 2 for sorting */
  }else{
    revarcdist=GetCost(incrcosts,arcrow,arccol,-arcdir);
    violation=revarcdist+to->outcost+from->incost-apexcost;
    if(violation<0){
      arcdir*=-2;  /* magnitude 2 for sorting */
      temp=from;
      from=to;
      to=temp;
    }else{
      violation=fwdarcdist+from->outcost-to->outcost;
      if(violation>=0){
        violation=revarcdist+to->outcost-from->outcost;
        if(violation<0){
          arcdir=-arcdir;
          temp=from;
          from=to;
          to=temp;
        }
      }
    }
  }

  /* see if we have a violation, and if so, add arc to candidate bag */
  if(violation<0){
    if((*candidatebagnextptr)>=(*candidatebagsizeptr)){
      (*candidatebagsizeptr)+=CANDIDATEBAGSTEP;
      (*candidatebagptr)=ReAlloc(*candidatebagptr,
                                 (*candidatebagsizeptr)*sizeof(candidateT));
    }
    (*candidatebagptr)[*candidatebagnextptr].violation=violation;
    (*candidatebagptr)[*candidatebagnextptr].from=from;
    (*candidatebagptr)[*candidatebagnextptr].to=to;
    (*candidatebagptr)[*candidatebagnextptr].arcrow=arcrow;
    (*candidatebagptr)[*candidatebagnextptr].arccol=arccol;
    (*candidatebagptr)[*candidatebagnextptr].arcdir=arcdir;
    (*candidatebagnextptr)++;
    iscandidate[arcrow][arccol]=TRUE;
  }

  /* done */
  return;
}


/* function: InitBoundary()
 * ------------------------
 * Initialize boundary structure for region to be unwrapped assuming
 * source is on boundary.
 *
 * This function makes several passes over the boundary nodes.  The
 * first pass finds nodes linked by zero-weight arcs that are candidates
 * for being on the boundary.  The second pass decides which of the
 * candidate nodes should actually point to the boundary node while
 * ensuring that no node has multiple valid arcs to the boundary node.
 * The third pass builds the neighbor list given the boundary pointer
 * nodes from the previous pass.  The fourth pass sets the group
 * members of boundary pointer nodes, which cannot be done earlier
 * since it would mess up how NeighborNodeGrid() works.
 *
 * Return pointer to source since source may become pointer to boundary.
 */
static
nodeT *InitBoundary(nodeT *source, nodeT **nodes,
                    boundaryT *boundary, nodesuppT **nodesupp, float **mag,
                    nodeT *ground, long ngroundarcs, long nrow, long ncol,
                    paramT *params, long *nconnectedptr){

  int iseligible, isinteriornode;
  long k, nlist, ninteriorneighbor;
  long nlistmem, nboundarymem, nneighbormem;
  long arcnum, upperarcnum;
  long neighborarcnum, neighborupperarcnum;
  long arcrow, arccol, arcdir;
  long nconnected;
  nodeT **nodelist, **boundarylist, *from, *to, *end;
  neighborT *neighborlist;


  /* initialize to null first */
  boundary->node->row=BOUNDARYROW;
  boundary->node->col=BOUNDARYCOL;
  boundary->node->next=NULL;
  boundary->node->prev=NULL;
  boundary->node->pred=NULL;
  boundary->node->level=0;
  boundary->node->group=0;
  boundary->node->incost=VERYFAR;
  boundary->node->outcost=VERYFAR;
  boundary->neighborlist=NULL;
  boundary->boundarylist=NULL;
  boundary->nneighbor=0;
  boundary->nboundary=0;

  /* if this is non-grid network, do nothing */
  if(nodesupp!=NULL){
    return(source);
  }

  /* make sure magnitude exists */
  if(mag==NULL){
    return(source);
  }

  /* scan region and mask any nodes that are not already masked but are not */
  /*   reachable through non-region arcs */
  /* such nodes can exist if there is single line of masked pixels separating */
  /*   regions */
  /* boundary is not yet set up, so scanning will search node neighbors as */
  /*   normal grid neighbors */
  nconnected=ScanRegion(source,nodes,mag,ground,ngroundarcs,nrow,ncol,MASKED);

  /* if source is ground, do nothing, since do not want boundary with ground */
  if(source==ground){
    return(source);
  }

  /* make sure source is on edge */
  /* we already know source is not ground from check above */
  if(!IsRegionEdgeNode(mag,source->row,source->col,nrow,ncol)){
    fprintf(sp0,"WARNING: Non edge node as source in InitBoundary()\n");
  }

  /* get memory for node list */
  nlistmem=NLISTMEMINCR;
  nodelist=(nodeT **)MAlloc(nlistmem*sizeof(nodeT *));
  nodelist[0]=source;
  nlist=1;

  /* first pass: build list of nodes on boundary */
  /* this should handle double-corner cases where all four arcs out of grid */
  /*   node will be region edge arcs (eg, mag[i][j] and mag[i+1][j+1] are */
  /*   both zero and mag[i+1][j] and mag[i][j+1] are both nonzero */
  source->next=NULL;
  source->group=BOUNDARYCANDIDATE;
  from=source;
  end=source;
  while(TRUE){

    /* see if neighbors can be reached through region-edge arcs */
    arcnum=GetArcNumLims(from->row,&upperarcnum,ngroundarcs,NULL);
    while(arcnum<upperarcnum){

      /* get neighboring node */
      to=NeighborNode(from,++arcnum,&upperarcnum,nodes,ground,
                      &arcrow,&arccol,&arcdir,nrow,ncol,NULL,nodesupp);

      /* see if node is reached through edge arc and node is not yet in list */
      if(IsRegionEdgeArc(mag,arcrow,arccol,nrow,ncol)
         && to->group!=BOUNDARYCANDIDATE){

        /* keep node in list */
        if(nlist==nlistmem){
          nlistmem+=NLISTMEMINCR;
          nodelist=(nodeT **)ReAlloc(nodelist,nlistmem*sizeof(nodeT *));
        }
        nodelist[nlist++]=to;
        to->group=BOUNDARYCANDIDATE;

        /* add node to list of nodes to be searched */
        end->next=to;
        to->next=NULL;
        end=to;

      }
    }

    /* move to next node to search */
    if(from->next==NULL){
      break;
    }
    from=from->next;

  }

  /* get memory for boundary list */
  nboundarymem=NLISTMEMINCR;
  boundarylist=(nodeT **)MAlloc(nboundarymem*sizeof(nodeT *));

  /* second pass to avoid multiple arcs to same node */
  /* go through nodes in list and check criteria for including on boundary */
  for(k=0;k<nlist;k++){

    /* only consider if node is not edge ground */
    /* we do not want ground node to be a boundary node since ground already */
    /*   acts similar to boundary node */
    if(nodelist[k]->row!=GROUNDROW){

      /* loop over neighbors */
      iseligible=TRUE;
      ninteriorneighbor=0;
      arcnum=GetArcNumLims(nodelist[k]->row,&upperarcnum,ngroundarcs,NULL);
      while(arcnum<upperarcnum){

        /* get neighbor */
        from=NeighborNode(nodelist[k],++arcnum,&upperarcnum,nodes,ground,
                          &arcrow,&arccol,&arcdir,nrow,ncol,NULL,nodesupp);

        /* see if node can be reached through interior arc */
        /*   and node is not masked or on boundary */
        /* node may be on edge of island of masked pixels, so it does not */
        /*   need to be interior node in sense of not touching masked pixels  */
        isinteriornode=(IsRegionInteriorArc(mag,arcrow,arccol,nrow,ncol)
                        && from->group!=MASKED
                        && from->level!=BOUNDARYLEVEL);
        if(isinteriornode){
          ninteriorneighbor++;
        }

        /* scan neighbor's neighbors if neighbor is interior node */
        /*    or if it is edge node that is not yet on boundary  */
        /* edge node may have been reached through a non-region arc, but */
        /*    that is okay since that non-region arc will have zero cost */
        /*    given that non-region nodes will be masked; need to let this */
        /*    happen because solver will use such an arc too */
        if(isinteriornode || (from->group==BOUNDARYCANDIDATE
                              && from->level!=BOUNDARYLEVEL)){

          /* loop over neighbors of neighbor */
          neighborarcnum=GetArcNumLims(from->row,&neighborupperarcnum,
                                       ngroundarcs,NULL);
          while(neighborarcnum<neighborupperarcnum){

            /* get neighbor of neighbor */
            to=NeighborNode(from,++neighborarcnum,&neighborupperarcnum,
                            nodes,ground,&arcrow,&arccol,&arcdir,
                            nrow,ncol,NULL,nodesupp);

            /* see if neighbor of neighbor is on boundary already */
            if(to->level==BOUNDARYLEVEL){
              iseligible=FALSE;
              break;
            }
          }
        }

        /* break if already ineligible */
        if(!iseligible){
          break;
        }
      }

      /* see if we should include this node in boundary */
      if(iseligible && ninteriorneighbor>0){
        nodelist[k]->level=BOUNDARYLEVEL;
        if(++boundary->nboundary > nboundarymem){
          nboundarymem+=NLISTMEMINCR;
          boundarylist=(nodeT **)ReAlloc(boundarylist,
                                         nboundarymem*sizeof(nodeT *));
        }
        boundarylist[boundary->nboundary-1]=nodelist[k];
      }
    }
  }

  /* set groups of all edge nodes back to zero */
  for(k=0;k<nlist;k++){
    nodelist[k]->group=0;
    nodelist[k]->next=NULL;
  }
  free(nodelist);

  /* punt if there were too few boundary nodes */
  /* region should be unwrapped with nodes behaving like normal grid nodes */
  /*   but with neighbors that are not part of region masked */
  if(boundary->nboundary<MINBOUNDARYSIZE){
    for(k=0;k<boundary->nboundary;k++){
      boundarylist[k]->level=0;
      boundarylist[k]->group=0;
    }
    free(boundarylist);
    boundary->node->row=BOUNDARYROW;
    boundary->node->col=BOUNDARYCOL;
    boundary->node->next=NULL;
    boundary->node->prev=NULL;
    boundary->node->pred=NULL;
    boundary->node->level=0;
    boundary->node->group=0;
    boundary->node->incost=VERYFAR;
    boundary->node->outcost=VERYFAR;
    boundary->neighborlist=NULL;
    boundary->boundarylist=NULL;
    boundary->nneighbor=0;
    boundary->nboundary=0;
    return(source);
  }

  /* third pass */
  /* set up for creating neighbor list */
  nneighbormem=NLISTMEMINCR;
  neighborlist=(neighborT *)MAlloc(nneighbormem*sizeof(neighborT));

  /* now go through boundary pointer nodes and build neighbor list */
  for(k=0;k<boundary->nboundary;k++){

    /* loop over neighbors to keep in neighbor list */
    /* checks above should ensure that unmasked neighbors of this boundary */
    /*    pointer node are not reachable by any other boundary pointer node */
    arcnum=GetArcNumLims(boundarylist[k]->row,&upperarcnum,ngroundarcs,NULL);
    while(arcnum<upperarcnum){

      /* get neighbor */
      to=NeighborNode(boundarylist[k],++arcnum,&upperarcnum,nodes,ground,
                      &arcrow,&arccol,&arcdir,nrow,ncol,NULL,nodesupp);

      /* see if node is not masked and not a boundary node */
      /* node may or may not be reachable through region arc, but if */
      /*   non region arc is used, it is okay since it will have zero cost; */
      /*   solver would use these arcs if there were no boundary, so let */
      /*   those nodes stay in neighbor list of boundary */
      if(to->group!=MASKED && to->level!=BOUNDARYLEVEL){

        /* add neighbor as neighbor of boundary */
        boundary->nneighbor++;
        if(boundary->nneighbor>nneighbormem){
          nneighbormem+=NLISTMEMINCR;
          neighborlist=(neighborT *)ReAlloc(neighborlist,
                                            nneighbormem*sizeof(neighborT));
        }
        neighborlist[boundary->nneighbor-1].neighbor=to;
        neighborlist[boundary->nneighbor-1].arcrow=arcrow;
        neighborlist[boundary->nneighbor-1].arccol=arccol;
        neighborlist[boundary->nneighbor-1].arcdir=arcdir;

      }

    }
  }

  /* fourth pass */
  /* now that boundary is properly set up, make one last pass to set groups */
  for(k=0;k<boundary->nboundary;k++){
    boundarylist[k]->group=BOUNDARYPTR;
    boundarylist[k]->level=0;
  }

  /* keep only needed memory and store pointers in boundary structure */
  boundary->neighborlist=(neighborT *)ReAlloc(neighborlist,
                                              (boundary->nneighbor
                                               *sizeof(neighborT)));
  boundary->boundarylist=(nodeT **)ReAlloc(boundarylist,(boundary->nboundary
                                                         *sizeof(nodeT *)));

  /* check boundary for consistency */
  /* count number of connected nodes, which should have changed by number */
  /*   outer edge nodes that got collapsed into single boundary node (minus 1) */
  nconnected=CheckBoundary(nodes,mag,ground,ngroundarcs,boundary,nrow,ncol,
                           params,boundary->node);
  if(nconnectedptr!=NULL){
    if(nconnected+boundary->nboundary-1!=(*nconnectedptr)){
      fprintf(sp1,
              "WARNING: Changed number of connected nodes in InitBoundary()\n");
    }
    (*nconnectedptr)=nconnected;
  }

  /* done */
  return(boundary->node);

}


/* function: CheckBoundary()
 * -------------------------
 * Similar to SelectConnNodeSource, but reset group to zero and check boundary.
 */
static
long CheckBoundary(nodeT **nodes, float **mag, nodeT *ground, long ngroundarcs,
                   boundaryT *boundary, long nrow, long ncol,
                   paramT *params, nodeT *start){

  long arcrow, arccol, arcdir, arcnum, upperarcnum;
  long nontree, nboundaryarc, nboundarynode, nconnected;
  nodeT *node1, *node2, *end;
  nodesuppT **nodesupp;


  /* if start node is not eligible, give error */
  if(start->group==MASKED){
    fflush(NULL);
    fprintf(sp0,"ERROR: ineligible starting node in CheckBoundary()\n");
    exit(ABNORMAL_EXIT);
  }

  /* initialize local variables */
  nconnected=0;
  end=start;
  nodesupp=NULL;
  node1=start;
  node1->group=INBUCKET;

  /* loop to search for connected, unmasked nodes */
  while(node1!=NULL){

    /* loop over neighbors of current node */
    arcnum=GetArcNumLims(node1->row,&upperarcnum,ngroundarcs,boundary);
    while(arcnum<upperarcnum){
      node2=NeighborNode(node1,++arcnum,&upperarcnum,nodes,ground,
                         &arcrow,&arccol,&arcdir,nrow,ncol,boundary,nodesupp);

      /* if neighbor is not masked or visited, add to list of nodes to search */
      if(node2->group!=MASKED && node2->group!=ONTREE
         && node2->group!=INBUCKET){

        node2->group=INBUCKET;
        end->next=node2;
        node2->next=NULL;
        end=node2;
      }
    }

    /* mark this node visited */
    node1->group=ONTREE;
    nconnected++;

    /* move to next node in list */
    node1=node1->next;

  }

  /* loop over connected nodes to check connectivity and reset group numbers */
  node1=start;
  nontree=0;
  nboundaryarc=0;
  nboundarynode=0;
  while(node1!=NULL){

    /* loop over neighbors of current node */
    arcnum=GetArcNumLims(node1->row,&upperarcnum,ngroundarcs,boundary);
    while(arcnum<upperarcnum){
      node2=NeighborNode(node1,++arcnum,&upperarcnum,nodes,ground,
                         &arcrow,&arccol,&arcdir,nrow,ncol,boundary,nodesupp);

      /* see if we have an arc to boundary */
      /* this may or may not use region arc, but if non-region arc is used */
      /*   it should have zero cost */
      if(node2->row==BOUNDARYROW){
        nboundaryarc++;
      }
    }

    /* check number of boundary nodes */
    if(node1->row==BOUNDARYROW){
      nboundarynode++;
    }

    /* count total number of nodes */
    nontree++;

    /* reset group number */
    if(node1->group==ONTREE){
      node1->group=0;
    }

    /* move to next node in list */
    node1=node1->next;

  }

  /* check for consistency */
  if(nontree!=nconnected){
    fflush(NULL);
    fprintf(sp0,
            "ERROR: inconsistent num connected nodes in CheckBoundary()\n");
    exit(ABNORMAL_EXIT);
  }
  if(nboundaryarc!=boundary->nneighbor){
    fflush(NULL);
    fprintf(sp0,
            "ERROR: inconsistent num neighbor nodes in CheckBoundary()\n");
    exit(ABNORMAL_EXIT);
  }
  if(nboundarynode!=1){
    fflush(NULL);
    fprintf(sp0,
            "ERROR: number of boundary nodes is not 1 in CheckBoundary()\n");
    exit(ABNORMAL_EXIT);
  }

  /* return number of connected nodes */
  return(nconnected);

}


/* function: IsRegionEdgeArc()
 * ---------------------------
 * Return TRUE if arc is on edge of region, FALSE otherwise.
 */
static
int IsRegionEdgeArc(float **mag, long arcrow, long arccol,
                    long nrow, long ncol){

  long row1, col1, row2, col2, nzeromag;

  /* if no magnitude, then everything is in single region */
  if(mag==NULL){
    return(FALSE);
  }

  /* determine indices of pixels on either side of this arc */
  if(arcrow<nrow-1){
    row1=arcrow;
    row2=row1+1;
    col1=arccol;
    col2=col1;
  }else{
    row1=arcrow-(nrow-1);
    row2=row1;
    col1=arccol;
    col2=col1+1;
  }

  /* see whether exactly one pixel has zero magnitude */
  nzeromag=0;
  if(mag[row1][col1]==0){
    nzeromag++;
  }
  if(mag[row2][col2]==0){
    nzeromag++;
  }
  if(nzeromag==1){
    return(TRUE);
  }
  return(FALSE);

}


/* function: IsRegionInteriorArc()
 * -------------------------------
 * Return TRUE if arc goes between two nodes in same region such that
 * both pixel magnitudes on either side of arc are nonzero.
 */
static
int IsRegionInteriorArc(float **mag, long arcrow, long arccol,
                        long nrow, long ncol){

  long row1, col1, row2, col2;


  /* if no magnitude, everything is in single region */
  if(mag==NULL){
    return(TRUE);
  }

  /* determine indices of pixels on either side of this arc */
  if(arcrow<nrow-1){
    row1=arcrow;
    row2=row1+1;
    col1=arccol;
    col2=col1;
  }else{
    row1=arcrow-(nrow-1);
    row2=row1;
    col1=arccol;
    col2=col1+1;
  }

  /* see whether both pixels have nonzero magnitude */
  if(mag[row1][col1]>0 && mag[row2][col2]>0){
    return(TRUE);
  }
  return(FALSE);

}


/* function: IsRegionArc()
 * -----------------------
 * Return TRUE if arc goes between two nodes in same region such that
 * at least one pixel magnitude on either side of arc is nonzero.
 */
static
int IsRegionArc(float **mag, long arcrow, long arccol,
                long nrow, long ncol){

  long row1, col1, row2, col2;


  /* if no magnitude, everything is in single region */
  if(mag==NULL){
    return(TRUE);
  }

  /* determine indices of pixels on either side of this arc */
  if(arcrow<nrow-1){
    row1=arcrow;
    row2=row1+1;
    col1=arccol;
    col2=col1;
  }else{
    row1=arcrow-(nrow-1);
    row2=row1;
    col1=arccol;
    col2=col1+1;
  }

  /* see whether at least one pixel has nonzero magnitude */
  if(mag[row1][col1]>0 || mag[row2][col2]>0){
    return(TRUE);
  }
  return(FALSE);

}


/* function: IsInteriorNode()
 * --------------------------
 * Return TRUE if node does not touch any zero magnitude pixels, FALSE
 * otherwise.
 */
/*
 * This function is no longer used
 */
#if 0
static
int IsInteriorNode(float **mag, long row, long col, long nrow, long ncol);
static
int IsInteriorNode(float **mag, long row, long col, long nrow, long ncol){

  /* if there is no magnitude info, then all nodes are interior nodes */
  if(mag==NULL){
    return(TRUE);
  }

  /* check for ground */
  if(row==GROUNDROW){
    return(FALSE);
  }

  /* check mag */
  if(mag[row][col]==0 || mag[row+1][col]==0
     || mag[row][col+1]==0 || mag[row+1][col+1]==0){
    return(FALSE);
  }
  return(TRUE);

}
#endif


/* function: IsRegionEdgeNode()
 * ----------------------------
 * Return TRUE if node touches at least one zero magnitude pixel and
 * at least one non-zero magnitude pixel, FALSE otherwise.
 */
static
int IsRegionEdgeNode(float **mag, long row, long col, long nrow, long ncol){

  int onezeromag, onenonzeromag;


  /* if there is no magnitude info, then no nodes are on edge */
  if(mag==NULL){
    return(FALSE);
  }

  /* check for ground */
  if(row==GROUNDROW){
    return(FALSE);
  }

  /* check mag for one zero mag and one nonzero mag pixel around node */
  onezeromag=FALSE;
  onenonzeromag=FALSE;
  if(mag[row][col]==0 || mag[row+1][col]==0
     || mag[row][col+1]==0 || mag[row+1][col+1]==0){
    onezeromag=TRUE;
  }
  if(mag[row][col]!=0 || mag[row+1][col]!=0
     || mag[row][col+1]!=0 || mag[row+1][col+1]!=0){
    onenonzeromag=TRUE;
  }
  if(onezeromag && onenonzeromag){
    return(TRUE);
  }
  return(FALSE);

}


/* function: CleanUpBoundaryNodes()
 * --------------------------------
 * Unset group values indicating boundary nodes on network.  This is
 * necessary because InitBoundary() temporarily sets node groups to
 * MASKED if the node is in a different region than the current but
 * can be reached from the current region.  This can occur if two
 * regions are separated by a single row or column of masked pixels.
 */
static
int CleanUpBoundaryNodes(nodeT *source, boundaryT *boundary, float **mag,
                         nodeT **nodes, nodeT *ground,
                         long nrow, long ncol, long ngroundarcs,
                         nodesuppT **nodesupp){

  long nconnected;
  nodeT *start;


  /* do nothing if this is not a grid network */
  if(nodesupp!=NULL){
    return(0);
  }

  /* starting node should not be boundary */
  if(source->row==BOUNDARYROW){
    start=boundary->neighborlist[0].neighbor;
  }else{
    start=source;
  }

  /* scan region and unmask any nodes that touch good pixels since they */
  /*   may have been masked by the ScanRegion() call in InitBoundaries() */
  nconnected=ScanRegion(start,nodes,mag,ground,ngroundarcs,nrow,ncol,0);

  /* done */
  return(nconnected);

}


/* function: DischargeBoundary()
 * -----------------------------
 * Find nodes and arcs along edge of region (defined by zero magnitude) and
 * compute surplus/demand by balancing flow in/out of nodes.  Then discharge
 * surplus/demand by sending flow along zero-cost arcs along region edge.
 */
static
int DischargeBoundary(nodeT **nodes, nodeT *ground,
                      boundaryT *boundary, nodesuppT **nodesupp, short **flows,
                      signed char **iscandidate, float **mag,
                      float **wrappedphase, long ngroundarcs,
                      long nrow, long ncol){

  long nedgenode;
  long row, col, fromrow, fromcol, todir;
  long arcnum, upperarcnum, arcrow, arccol, arcdir, narccol;
  long surplus, residue, excess;
  nodeT *from, *to, *nextnode;


  /* do nothing if we have no boundary */
  if(nodesupp!=NULL || boundary==NULL
     || boundary->nboundary==0 || boundary->nneighbor==0){
    return(0);
  }

  /* find initial region edge node */
  nextnode=boundary->boundarylist[0];
  row=nextnode->row;
  col=nextnode->col;
  if(!IsRegionEdgeNode(mag,row,col,nrow,ncol)){
    fprintf(sp0,"ERROR: DischargeBoundary() start node %ld, %ld not on edge\n",
            row,col);
    exit(ABNORMAL_EXIT);
  }

  /* silence compiler warnings */
  row=0;
  col=0;
  todir=0;

  /* make sure iscandidate is zero */
  /* temporarily set illegal corner arcs to 0 to simplify logic (reset later) */
  for(row=0;row<2*nrow-1;row++){
    if(row<nrow-1){
      narccol=ncol;
    }else{
      narccol=ncol-1;
    }
    for(col=0;col<narccol;col++){
      iscandidate[row][col]=0;
    }
  }

  /* initialize counter of edge nodes to 1 */
  /* (tree root doesn't get push or counter increment) */
  nedgenode=1;

  /* use similar algorithm as DischargeTree() to descend region edge arcs */
  /* this will form tree (do not need to close cycle if we discharge */
  /*   all nodes--last arc that would close to form cycle will have no flow) */
  /* tree could branch (not just single chain) because of double corner cases */
  /*   where mag[i][j] and mag[i+1][j+1] are zero and mag[i+1][j] and */
  /*   mag[i][j+1] are nonzero */
  while(TRUE){

    /* get next node */
    /* mark with -1 cost to denote that they have been visited */
    from=nextnode;
    from->outcost=-1;
    nextnode=NULL;

    /* loop over outgoing arcs */
    /* pass NULL to NeighborNode() for boundary so as not to follow */
    /*   node pointers to boundary node */
    arcnum=GetArcNumLims(from->row,&upperarcnum,ngroundarcs,NULL);
    while(arcnum<upperarcnum){

      /* get neighbor node */
      to=NeighborNode(from,++arcnum,&upperarcnum,nodes,ground,
                      &arcrow,&arccol,&arcdir,nrow,ncol,NULL,nodesupp);

      /* see if arc is on region edge and not done */
      if(IsRegionEdgeArc(mag,arcrow,arccol,nrow,ncol)
         && (iscandidate[arcrow][arccol]==-1 ||
             (iscandidate[arcrow][arccol]==0 && to->outcost!=-1))){

        /* save arc */
        nextnode=to;
        row=arcrow;
        col=arccol;
        todir=arcdir;

        /* stop and follow arc if arc not yet followed */
        if(!iscandidate[arcrow][arccol]){
          break;
        }

      }
    }

    /* break if no unfollowed arcs (ie, we are done examining tree) */
    if(nextnode==NULL){
      break;
    }

    /* if we found leaf and we're moving back up the tree, do a push */
    /* otherwise, just mark the path by decrementing iscandidate */
    if((--iscandidate[row][col])==-2){

      /* integrate flow into current node */
      fromrow=from->row;
      fromcol=from->col;
      surplus=(flows[fromrow][fromcol]
               -flows[fromrow][fromcol+1]
               +flows[fromrow+nrow-1][fromcol]
               -flows[fromrow+1+nrow-1][fromcol]);

      /* compute residue from wrapped phase */
      residue=NodeResidue(wrappedphase,fromrow,fromcol);

      /* compute excess as surplus plus residue */
      excess=surplus+residue;

      /* augment flow */
      flows[row][col]+=todir*excess;

      /* increment counter of edge nodes */
      nedgenode++;

    }

  }

  /* reset all iscandidate and outcost values */
  /* set illegal corner arc iscandidate values back to TRUE */
  /* outcost of region edge nodes should be zero if source was on edge */
  /*   and arc costs along boundary are all zero */
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      if(row<nrow-1){
        if(iscandidate[row][col]){
          if(col>0){
            nodes[row][col-1].outcost=0;
          }
          if(col<ncol-1){
            nodes[row][col].outcost=0;
          }
        }
        iscandidate[row][col]=FALSE;
      }
      if(col<ncol-1){
        if(iscandidate[row+nrow-1][col]){
          if(row>0){
            nodes[row-1][col].outcost=0;
          }
          if(row<nrow-1){
            nodes[row][col].outcost=0;
          }
        }
        iscandidate[row+nrow-1][col]=FALSE;
      }
    }
  }
  iscandidate[nrow-1][0]=TRUE;
  iscandidate[2*nrow-2][0]=TRUE;
  iscandidate[nrow-1][ncol-2]=TRUE;
  iscandidate[2*nrow-2][ncol-2]=TRUE;

  /* done */
  return(nedgenode);

}


/* function: InitTree()
 * --------------------
 * Initialize tree for tree solver.
 */
static
int InitTree(nodeT *source, nodeT **nodes,
             boundaryT *boundary, nodesuppT **nodesupp,
             nodeT *ground, long ngroundarcs, bucketT *bkts, long nflow,
             incrcostT **incrcosts, long nrow, long ncol, paramT *params){

  long arcnum, upperarcnum, arcrow, arccol, arcdir;
  nodeT *to;


  /* put source on tree */
  source->group=1;
  source->outcost=0;
  source->incost=0;
  source->pred=NULL;
  source->prev=source;
  source->next=source;
  source->level=0;

  /* loop over outgoing arcs and add to buckets */
  arcnum=GetArcNumLims(source->row,&upperarcnum,ngroundarcs,boundary);
  while(arcnum<upperarcnum){

    /* get node reached by outgoing arc */
    to=NeighborNode(source,++arcnum,&upperarcnum,nodes,ground,
                    &arcrow,&arccol,&arcdir,nrow,ncol,boundary,nodesupp);

    /* add to node to bucket */
    if(to->group!=PRUNED && to->group!=MASKED){
      AddNewNode(source,to,arcdir,bkts,nflow,incrcosts,arcrow,arccol,params);
    }

  }

  /* done */
  return(0);

}


/* function: FindApex()
 * --------------------
 * Given pointers to two nodes on a spanning tree, the function finds
 * and returns a pointer to their deepest common ancestor, the apex of
 * a cycle formed by joining the two nodes with an arc.
 */
static
nodeT *FindApex(nodeT *from, nodeT *to){

  if(from->level > to->level){
    while(from->level != to->level){
      from=from->pred;
    }
  }else{
    while(from->level != to->level){
      to=to->pred;
    }
  }
  while(from != to){
    from=from->pred;
    to=to->pred;
  }
  return(from);
}


/* function: CandidateCompare()
 * ----------------------------
 * Compares the violations of candidate arcs for sorting.  First checks
 * if either candidate has an arcdir magnitude greater than 1, denoting
 * an augmenting cycle.  Augmenting candidates are always placed before
 * non-augmenting candidates.  Otherwise, returns positive if the first
 * candidate has a greater (less negative) violation than the second, 0
 * if they are the same, and negative otherwise.
 */
static
int CandidateCompare(const void *c1, const void *c2){

  if(labs(((candidateT *)c1)->arcdir) > 1){
    if(labs(((candidateT *)c2)->arcdir) < 2){
      return(-1);
    }
  }else if(labs(((candidateT *)c2)->arcdir) > 1){
    return(1);
  }

  return(((candidateT *)c1)->violation - ((candidateT *)c2)->violation);

  /*
  if(((candidateT *)c1)->violation > ((candidateT *)c2)->violation){
    return(1);
  }else if(((candidateT *)c1)->violation < ((candidateT *)c2)->violation){
    return(-1);
  }else{
    return(0);
  }
  */
}


/* function: GetArcNumLims()
 * -------------------------
 * Get the initial and ending values for arcnum to find neighbors of
 * the passed node.
 */
static inline
long GetArcNumLims(long fromrow, long *upperarcnumptr,
                   long ngroundarcs, boundaryT *boundary){

  long arcnum;

  /* set arcnum limits based on node type */
  if(fromrow<0){
    arcnum=-1;
    if(fromrow==GROUNDROW){
      *upperarcnumptr=ngroundarcs-1;
    }else{
      *upperarcnumptr=boundary->nneighbor-1;
    }
  }else{
    arcnum=-5;
    *upperarcnumptr=-1;
  }
  return(arcnum);

}


/* function: NeighborNodeGrid()
 * ----------------------------
 * Return the neighboring node of the given node corresponding to the
 * given arc number for a grid network with a ground node.
 */
static
nodeT *NeighborNodeGrid(nodeT *node1, long arcnum, long *upperarcnumptr,
                        nodeT **nodes, nodeT *ground, long *arcrowptr,
                        long *arccolptr, long *arcdirptr, long nrow,
                        long ncol, boundaryT *boundary, nodesuppT **nodesupp){
  long row, col;
  nodeT *neighbor;

  /* get starting node row and col for convenience */
  row=node1->row;
  col=node1->col;

  /* see if starting node is boundary node */
  if(row==BOUNDARYROW){

    /* get neighbor info from boundary structure */
    neighbor=boundary->neighborlist[arcnum].neighbor;
    *arcrowptr=boundary->neighborlist[arcnum].arcrow;
    *arccolptr=boundary->neighborlist[arcnum].arccol;
    *arcdirptr=boundary->neighborlist[arcnum].arcdir;

  }else{

    /* starting node is normal node */
    switch(arcnum){
    case -4:
      *arcrowptr=row;
      *arccolptr=col+1;
      *arcdirptr=1;
      if(col==ncol-2){
        neighbor=ground;
      }else{
        neighbor=&nodes[row][col+1];
      }
      break;
    case -3:
      *arcrowptr=nrow+row;
      *arccolptr=col;
      *arcdirptr=1;
      if(row==nrow-2){
        neighbor=ground;
      }else{
        neighbor=&nodes[row+1][col];
      }
      break;
    case -2:
      *arcrowptr=row;
      *arccolptr=col;
      *arcdirptr=-1;
      if(col==0){
        neighbor=ground;
      }else{
        neighbor=&nodes[row][col-1];
      }
      break;

    case -1:
      *arcrowptr=nrow-1+row;
      *arccolptr=col;
      *arcdirptr=-1;
      if(row==0){
        neighbor=ground;
      }else{
        neighbor=&nodes[row-1][col];
      }
      break;
    default:
      if(arcnum<nrow-1){
        *arcrowptr=arcnum;
        *arccolptr=0;
        *arcdirptr=1;
        neighbor=&nodes[*arcrowptr][0];
      }else if(arcnum<2*(nrow-1)){
        *arcrowptr=arcnum-(nrow-1);
        *arccolptr=ncol-1;
        *arcdirptr=-1;
        neighbor=&nodes[*arcrowptr][ncol-2];
      }else if(arcnum<2*(nrow-1)+ncol-3){
        *arcrowptr=nrow-1;
        *arccolptr=arcnum-2*(nrow-1)+1;
        *arcdirptr=1;
        neighbor=&nodes[0][*arccolptr];
      }else{
        *arcrowptr=2*nrow-2;
        *arccolptr=arcnum-(2*(nrow-1)+ncol-3)+1;
        *arcdirptr=-1;
        neighbor=&nodes[nrow-2][*arccolptr];
      }
      break;
    }

    /* get boundary node if neighbor is pointer one */
    if(neighbor->group==BOUNDARYPTR && boundary!=NULL){
      neighbor=boundary->node;
    }

  }

  /* return neighbor */
  return(neighbor);

}


/* function: NeighborNodeNonGrid()
 * -------------------------------
 * Return the neighboring node of the given node corresponding to the
 * given arc number for a nongrid network (ie, arbitrary topology).
 */
static
nodeT *NeighborNodeNonGrid(nodeT *node1, long arcnum, long *upperarcnumptr,
                           nodeT **nodes, nodeT *ground, long *arcrowptr,
                           long *arccolptr, long *arcdirptr, long nrow,
                           long ncol, boundaryT *boundary,
                           nodesuppT **nodesupp){

  long tilenum, nodenum;
  scndryarcT *outarc;

  /* set up */
  tilenum=node1->row;
  nodenum=node1->col;
  *upperarcnumptr=nodesupp[tilenum][nodenum].noutarcs-5;

  /* set the arc row (tilenumber) and column (arcnumber) */
  outarc=nodesupp[tilenum][nodenum].outarcs[arcnum+4];
  *arcrowptr=outarc->arcrow;
  *arccolptr=outarc->arccol;
  if(node1==outarc->from){
    *arcdirptr=1;
  }else{
    *arcdirptr=-1;
  }

  /* return the neighbor node */
  return(nodesupp[tilenum][nodenum].neighbornodes[arcnum+4]);

}


/* function: GetArcGrid()
 * ----------------------
 * Given a from node and a to node, sets pointers for indices into
 * arc arrays, assuming primary (grid) network.
 */
static
void GetArcGrid(nodeT *from, nodeT *to, long *arcrow, long *arccol,
                long *arcdir, long nrow, long ncol,
                nodeT **nodes, nodesuppT **nodesupp){

  long fromrow, fromcol, torow, tocol;


  fromrow=from->row;
  fromcol=from->col;
  torow=to->row;
  tocol=to->col;

  if(fromcol==tocol-1){           /* normal arcs (neither endpoint ground) */
    *arcrow=fromrow;
    *arccol=fromcol+1;
    *arcdir=1;
  }else if(fromcol==tocol+1){
    *arcrow=fromrow;
    *arccol=fromcol;
    *arcdir=-1;
  }else if(fromrow==torow-1){
    *arcrow=fromrow+1+nrow-1;
    *arccol=fromcol;
    *arcdir=1;
  }else if(fromrow==torow+1){
    *arcrow=fromrow+nrow-1;
    *arccol=fromcol;
    *arcdir=-1;
  }else if(fromrow==BOUNDARYROW){ /* arc from boundary pointer */
    if(tocol<ncol-2 && nodes[torow][tocol+1].group==BOUNDARYPTR){
      *arcrow=torow;
      *arccol=tocol+1;
      *arcdir=-1;
    }else if(tocol>0 && nodes[torow][tocol-1].group==BOUNDARYPTR){
      *arcrow=torow;
      *arccol=tocol;
      *arcdir=1;
    }else if(torow<nrow-2 && nodes[torow+1][tocol].group==BOUNDARYPTR){
      *arcrow=torow+1+nrow-1;
      *arccol=tocol;
      *arcdir=-1;
    }else{
      *arcrow=torow+nrow-1;
      *arccol=tocol;
      *arcdir=1;
    }
  }else if(torow==BOUNDARYROW){   /* arc to boundary pointer */
    if(fromcol<ncol-2 && nodes[fromrow][fromcol+1].group==BOUNDARYPTR){
      *arcrow=fromrow;
      *arccol=fromcol+1;
      *arcdir=1;
    }else if(fromcol>0 && nodes[fromrow][fromcol-1].group==BOUNDARYPTR){
      *arcrow=fromrow;
      *arccol=fromcol;
      *arcdir=-1;
    }else if(fromrow<nrow-2 && nodes[fromrow+1][fromcol].group==BOUNDARYPTR){
      *arcrow=fromrow+1+nrow-1;
      *arccol=fromcol;
      *arcdir=1;
    }else{
      *arcrow=fromrow+nrow-1;
      *arccol=fromcol;
      *arcdir=-1;
    }
  }else if(fromcol==0){           /* arcs to ground */
    *arcrow=fromrow;
    *arccol=0;
    *arcdir=-1;
  }else if(fromcol==ncol-2){
    *arcrow=fromrow;
    *arccol=ncol-1;
    *arcdir=1;
  }else if(fromrow==0){
    *arcrow=nrow-1;
    *arccol=fromcol;
    *arcdir=-1;
  }else if(fromrow==nrow-2){
    *arcrow=2*(nrow-1);
    *arccol=fromcol;
    *arcdir=1;
  }else if(tocol==0){             /* arcs from ground */
    *arcrow=torow;
    *arccol=0;
    *arcdir=1;
  }else if(tocol==ncol-2){
    *arcrow=torow;
    *arccol=ncol-1;
    *arcdir=-1;
  }else if(torow==0){
    *arcrow=nrow-1;
    *arccol=tocol;
    *arcdir=1;
  }else{
    *arcrow=2*(nrow-1);
    *arccol=tocol;
    *arcdir=-1;
  }
  return;
}


/* function: GetArcNonGrid()
 * -------------------------
 * Given a from node and a to node, sets pointers for indices into
 * arc arrays, assuming secondary (arbitrary topology) network.
 */
static
void GetArcNonGrid(nodeT *from, nodeT *to, long *arcrow, long *arccol,
                   long *arcdir, long nrow, long ncol,
                   nodeT **nodes, nodesuppT **nodesupp){

  long tilenum, nodenum, arcnum;
  scndryarcT *outarc;

  /* get tile and node numbers for from node */
  tilenum=from->row;
  nodenum=from->col;

  /* loop over all outgoing arcs of from node */
  arcnum=0;
  while(TRUE){
    outarc=nodesupp[tilenum][nodenum].outarcs[arcnum++];
    if(outarc->from==to){
      *arcrow=outarc->arcrow;
      *arccol=outarc->arccol;
      *arcdir=-1;
      return;
    }else if(outarc->to==to){
      *arcrow=outarc->arcrow;
      *arccol=outarc->arccol;
      *arcdir=1;
      return;
    }
  }
  return;
}


/* Function: NonDegenUpdateChildren()
 * ----------------------------------
 * Updates potentials and groups of all childredn along an augmenting path,
 * until a stop node is hit.
 */
static
void NonDegenUpdateChildren(nodeT *startnode, nodeT *lastnode,
                            nodeT *nextonpath, long dgroup,
                            long ngroundarcs, long nflow, nodeT **nodes,
                            nodesuppT **nodesupp, nodeT *ground,
                            boundaryT *boundary,
                            nodeT ***apexes, incrcostT **incrcosts,
                            long nrow, long ncol, paramT *params){

  nodeT *node1, *node2;
  long dincost, doutcost, arcnum, upperarcnum, startlevel;
  long group1, pathgroup, arcrow, arccol, arcdir;

  /* loop along flow path  */
  node1=startnode;
  pathgroup=lastnode->group;
  while(node1!=lastnode){

    /* update potentials along the flow path by calculating arc distances */
    node2=nextonpath;
    GetArc(node2->pred,node2,&arcrow,&arccol,&arcdir,nrow,ncol,
           nodes,nodesupp);
    doutcost=node1->outcost - node2->outcost
      + GetCost(incrcosts,arcrow,arccol,arcdir);
    node2->outcost+=doutcost;
    dincost=node1->incost - node2->incost
      + GetCost(incrcosts,arcrow,arccol,-arcdir);
    node2->incost+=dincost;
    node2->group=node1->group+dgroup;

    /* update potentials of children of this node in the flow path */
    node1=node2;
    arcnum=GetArcNumLims(node1->row,&upperarcnum,ngroundarcs,boundary);
    while(arcnum<upperarcnum){
      node2=NeighborNode(node1,++arcnum,&upperarcnum,nodes,ground,
                         &arcrow,&arccol,&arcdir,nrow,ncol,boundary,nodesupp);
      if(node2->pred==node1 && node2->group>0){
        if(node2->group==pathgroup){
          nextonpath=node2;
        }else{
          startlevel=node2->level;
          group1=node1->group;
          while(TRUE){
            node2->group=group1;
            node2->incost+=dincost;
            node2->outcost+=doutcost;
            node2=node2->next;
            if(node2->level <= startlevel){
              break;
            }
          }
        }
      }
    }
  }
  return;
}


/* function: PruneTree()
 * ---------------------
 * Descends the tree from the source and finds leaves that can be
 * removed.
 */
static
long PruneTree(nodeT *source, nodeT **nodes, nodeT *ground, boundaryT *boundary,
               nodesuppT **nodesupp, incrcostT **incrcosts,
               short **flows, long ngroundarcs, long prunecostthresh,
               long nrow, long ncol){

  long npruned;
  nodeT *node1;

  /* set up */
  npruned=0;

  /* descend tree and look for leaves to prune */
  node1=source->next;
  while(node1!=source){

    /* see if current node is a leaf that should be pruned */
    if(CheckLeaf(node1,nodes,ground,boundary,nodesupp,incrcosts,flows,
                 ngroundarcs,nrow,ncol,prunecostthresh)){

      /* remove the current node from the tree */
      node1->prev->next=node1->next;
      node1->next->prev=node1->prev;
      node1->group=PRUNED;
      npruned++;

      /* see if last node checked was current node's parent */
      /*   if so, it may need pruning since its child has been pruned */
      if(node1->prev->level < node1->level){
        node1=node1->prev;
      }else{
        node1=node1->next;
      }

    }else{

      /* move on to next node */
      node1=node1->next;

    }
  }

  /* show status */
  fprintf(sp3,"\n  Pruned %ld nodes\n",npruned);

  /* return number of pruned nodes */
  return(npruned);

}


/* function: CheckLeaf()
 * ---------------------
 * Checks to see if the passed node should be pruned from the tree.
 * The node should be pruned if it is a leaf and if all of its outgoing
 * arcs have very high costs and only lead to other nodes that are already
 *  on the tree or are already pruned.
 */
static
int CheckLeaf(nodeT *node1, nodeT **nodes, nodeT *ground, boundaryT *boundary,
              nodesuppT **nodesupp, incrcostT **incrcosts,
              short **flows, long ngroundarcs, long nrow, long ncol,
              long prunecostthresh){

  long arcnum, upperarcnum, arcrow, arccol, arcdir;
  nodeT *node2;


  /* first, check to see if node1 is a leaf */
  if(node1->next->level > node1->level){
    return(FALSE);
  }

  /* loop over outgoing arcs */
  arcnum=GetArcNumLims(node1->row,&upperarcnum,ngroundarcs,boundary);
  while(arcnum<upperarcnum){
    node2=NeighborNode(node1,++arcnum,&upperarcnum,nodes,ground,
                       &arcrow,&arccol,&arcdir,nrow,ncol,boundary,nodesupp);

    /* current node should not be pruned if adjacent node */
    /*   has not been added to tree yet */
    /*   or if incremental costs of arc are too small */
    /*   or there is nonzero flow on the arc */
    if(node2->group==0 || node2->group==INBUCKET
       || incrcosts[arcrow][arccol].poscost<prunecostthresh
       || incrcosts[arcrow][arccol].negcost<prunecostthresh
       || flows[arcrow][arccol]!=0){
      return(FALSE);
    }
  }

  /* all conditions for pruning are satisfied */
  return(TRUE);

}


/* function: InitNetowrk()
 * -----------------------
 */
int InitNetwork(short **flows, long *ngroundarcsptr, long *ncycleptr,
                long *nflowdoneptr, long *mostflowptr, long *nflowptr,
                long *candidatebagsizeptr, candidateT **candidatebagptr,
                long *candidatelistsizeptr, candidateT **candidatelistptr,
                signed char ***iscandidateptr, nodeT ****apexesptr,
                bucketT **bktsptr, long *iincrcostfileptr,
                incrcostT ***incrcostsptr, nodeT ***nodesptr, nodeT *ground,
                long *nnoderowptr, int **nnodesperrowptr, long *narcrowptr,
                int **narcsperrowptr, long nrow, long ncol,
                signed char *notfirstloopptr, totalcostT *totalcostptr,
                paramT *params){

  long i;


  /* get and initialize memory for nodes */
  if(ground!=NULL && *nodesptr==NULL){
    *nodesptr=(nodeT **)Get2DMem(nrow-1,ncol-1,sizeof(nodeT *),sizeof(nodeT));
    InitNodeNums(nrow-1,ncol-1,*nodesptr,ground);
  }

  /* take care of ambiguous flows to ground at corners */
  if(ground!=NULL){
    flows[0][0]+=flows[nrow-1][0];
    flows[nrow-1][0]=0;
    flows[0][ncol-1]-=flows[nrow-1][ncol-2];
    flows[nrow-1][ncol-2]=0;
    flows[nrow-2][0]-=flows[2*nrow-2][0];
    flows[2*nrow-2][0]=0;
    flows[nrow-2][ncol-1]+=flows[2*nrow-2][ncol-2];
    flows[2*nrow-2][ncol-2]=0;
  }

  /* initialize network solver variables */
  *ncycleptr=0;
  *nflowptr=1;
  *candidatebagsizeptr=INITARRSIZE;
  *candidatebagptr=MAlloc(*candidatebagsizeptr*sizeof(candidateT));
  *candidatelistsizeptr=INITARRSIZE;
  *candidatelistptr=MAlloc(*candidatelistsizeptr*sizeof(candidateT));
  if(ground!=NULL){
    *nflowdoneptr=0;
    *mostflowptr=Short2DRowColAbsMax(flows,nrow,ncol);
    if(*mostflowptr*params->nshortcycle>LARGESHORT){
      fprintf(sp1,"Maximum flow on network: %ld\n",*mostflowptr);
      fflush(NULL);
      fprintf(sp0,"((Maximum flow) * NSHORTCYCLE) too large\nAbort\n");
      exit(ABNORMAL_EXIT);
    }
    if(ncol>2){
      *ngroundarcsptr=2*(nrow+ncol-2)-4; /* don't include corner column arcs */
    }else{
      *ngroundarcsptr=2*(nrow+ncol-2)-2;
    }
    *iscandidateptr=(signed char **)Get2DRowColMem(nrow,ncol,
                                                   sizeof(signed char *),
                                                   sizeof(signed char));
    *apexesptr=(nodeT ***)Get2DRowColMem(nrow,ncol,sizeof(nodeT **),
                                         sizeof(nodeT *));
  }

  /* set up buckets for TreeSolve (MSTInitFlows() has local set of buckets) */
  *bktsptr=MAlloc(sizeof(bucketT));
  if(ground!=NULL){
    (*bktsptr)->minind=-LRound((params->maxcost+1)*(nrow+ncol)
                               *NEGBUCKETFRACTION);
    (*bktsptr)->maxind=LRound((params->maxcost+1)*(nrow+ncol)
                              *POSBUCKETFRACTION);
  }else{
    (*bktsptr)->minind=-LRound((params->maxcost+1)*(nrow)
                               *NEGBUCKETFRACTION);
    (*bktsptr)->maxind=LRound((params->maxcost+1)*(nrow)
                              *POSBUCKETFRACTION);
  }
  (*bktsptr)->size=(*bktsptr)->maxind-(*bktsptr)->minind+1;
  (*bktsptr)->bucketbase=(nodeT **)MAlloc((*bktsptr)->size*sizeof(nodeT *));
  (*bktsptr)->bucket=&((*bktsptr)->bucketbase[-(*bktsptr)->minind]);
  for(i=0;i<(*bktsptr)->size;i++){
    (*bktsptr)->bucketbase[i]=NULL;
  }

  /* get memory for incremental cost arrays */
  *iincrcostfileptr=0;
  if(ground!=NULL){
    (*incrcostsptr)=(incrcostT **)Get2DRowColMem(nrow,ncol,sizeof(incrcostT *),
                                                 sizeof(incrcostT));
  }

  /* set number of nodes and arcs per row */
  if(ground!=NULL){
    (*nnoderowptr)=nrow-1;
    (*nnodesperrowptr)=(int *)MAlloc((nrow-1)*sizeof(int));
    for(i=0;i<nrow-1;i++){
      (*nnodesperrowptr)[i]=ncol-1;
    }
    (*narcrowptr)=2*nrow-1;
    (*narcsperrowptr)=(int *)MAlloc((2*nrow-1)*sizeof(int));
    for(i=0;i<nrow-1;i++){
      (*narcsperrowptr)[i]=ncol;
    }
    for(i=nrow-1;i<2*nrow-1;i++){
      (*narcsperrowptr)[i]=ncol-1;
    }
  }

  /* initialize variables for main optimizer loop */
  (*notfirstloopptr)=FALSE;
  (*totalcostptr)=INITTOTALCOST;

  /* done */
  return(0);

}


/* function: SetupTreeSolveNetwork()
 * ---------------------------------
 */
long SetupTreeSolveNetwork(nodeT **nodes, nodeT *ground, nodeT ***apexes,
                           signed char **iscandidate, long nnoderow,
                           int *nnodesperrow, long narcrow, int *narcsperrow,
                           long nrow, long ncol){

  long row, col, nnodes;


  /* loop over each node and initialize values */
  nnodes=0;
  for(row=0;row<nnoderow;row++){
    for(col=0;col<nnodesperrow[row];col++){
      if(nodes[row][col].group!=MASKED){
        nodes[row][col].group=0;
        nnodes++;
      }
      nodes[row][col].incost=VERYFAR;
      nodes[row][col].outcost=VERYFAR;
      nodes[row][col].pred=NULL;
    }
  }

  /* initialize the ground node */
  if(ground!=NULL){
    if(ground->group!=MASKED){
      ground->group=0;
      nnodes++;
    }
    ground->incost=VERYFAR;
    ground->outcost=VERYFAR;
    ground->pred=NULL;
  }

  /* initialize arcs */
  for(row=0;row<narcrow;row++){
    for(col=0;col<narcsperrow[row];col++){
      apexes[row][col]=NONTREEARC;
      iscandidate[row][col]=FALSE;
    }
  }

  /* if in grid mode, ground will exist */
  if(ground!=NULL){

    /* set iscandidate=TRUE for illegal corner arcs so they're never used */
    iscandidate[nrow-1][0]=TRUE;
    iscandidate[2*nrow-2][0]=TRUE;
    iscandidate[nrow-1][ncol-2]=TRUE;
    iscandidate[2*nrow-2][ncol-2]=TRUE;

  }

  /* return the number of nodes in the network */
  return(nnodes);

}


/* function: CheckMagMasking()
 * ---------------------------
 * Check magnitude masking to make sure we have at least one pixel
 * that is not masked; return 0 on success, or 1 if all pixels masked.
 */
signed char CheckMagMasking(float **mag, long nrow, long ncol){

  long row, col;

  /* loop over pixels and see if any are unmasked */
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      if(mag[row][col]>0){
        return(0);
      }
    }
  }
  return(1);

}


/* function: MaskNodes()
 * ---------------------
 * Set group numbers of nodes to MASKED if they are surrounded by
 * zero-magnitude pixels, 0 otherwise.
 */
int MaskNodes(long nrow, long ncol, nodeT **nodes, nodeT *ground,
              float **mag){

  long row, col;

  /* loop over grid nodes and see if masking is necessary */
  for(row=0;row<nrow-1;row++){
    for(col=0;col<ncol-1;col++){
      nodes[row][col].group=GridNodeMaskStatus(row,col,mag);
    }
  }

  /* check whether ground node should be masked */
  ground->group=GroundMaskStatus(nrow,ncol,mag);

  /* done */
  return(0);

}


/* function: GridNodeMaskStatus()
 * ---------------------------------
 * Given row and column of grid node, return MASKED if all pixels around node
 * have zero magnitude, and 0 otherwise.
 */
static
int GridNodeMaskStatus(long row, long col, float **mag){

  /* return 0 if any pixel is not masked */
  if(mag[row][col] || mag[row][col+1]
     || mag[row+1][col] || mag[row+1][col+1]){
    return(0);
  }
  return(MASKED);

}


/* function: GroundMaskStatus()
 * ----------------------------
 * Return MASKED if all pixels around grid edge have zero magnitude, 0
 * otherwise.
 */
static
int GroundMaskStatus(long nrow, long ncol, float **mag){

  long row, col;


  /* check all pixels along edge */
  for(row=0;row<nrow;row++){
    if(mag[row][0] || mag[row][ncol-1]){
      return(0);
    }
  }
  for(col=0;col<ncol;col++){
    if(mag[0][col] || mag[nrow-1][col]){
      return(0);
    }
  }
  return(MASKED);

}


/* function: MaxNonMaskFlowMag()
 * -----------------------------
 * Return maximum flow magnitude that does not traverse an arc adjacent
 * to a masked interferogram pixel.
 */
long MaxNonMaskFlow(short **flows, float **mag, long nrow, long ncol){

  long row, col;
  long mostflow, flowvalue;

  /* find max flow by checking row arcs then col arcs */
  mostflow=0;
  for(row=0;row<nrow-1;row++){
    for(col=0;col<ncol;col++){
      flowvalue=labs((long )flows[row][col]);
      if(flowvalue>mostflow && mag[row][col]>0 && mag[row+1][col]>0){
        mostflow=flowvalue;
      }
    }
  }
  for(row=nrow-1;row<2*nrow-1;row++){
    for(col=0;col<ncol-1;col++){
      flowvalue=labs((long )flows[row][col]);
      if(flowvalue>mostflow
         && mag[row-nrow+1][col]>0 && mag[row-nrow+1][col+1]>0){
          mostflow=flowvalue;
      }
    }
  }
  return(mostflow);
}


/* function: InitNodeNums()
 * ------------------------
 */
int InitNodeNums(long nrow, long ncol, nodeT **nodes, nodeT *ground){

  long row, col;

  /* loop over each element and initialize values */
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      nodes[row][col].row=row;
      nodes[row][col].col=col;
    }
  }

  /* initialize the ground node */
  if(ground!=NULL){
    ground->row=GROUNDROW;
    ground->col=GROUNDCOL;
  }

  /* done */
  return(0);

}


/* function: InitBuckets()
 * -----------------------
 */
static
int InitBuckets(bucketT *bkts, nodeT *source, long nbuckets){

  long i;

  /* set up bucket array parameters */
  bkts->curr=0;
  bkts->wrapped=FALSE;

  /* initialize the buckets */
  for(i=0;i<nbuckets;i++){
    bkts->bucketbase[i]=NULL;
  }

  /* put the source in the zeroth distance index bucket */
  bkts->bucket[0]=source;
  source->next=NULL;
  source->prev=NULL;
  source->group=INBUCKET;
  source->outcost=0;

  /* done */
  return(0);

}


/* function: InitNodes()
 * ---------------------
 */
int InitNodes(long nnrow, long nncol, nodeT **nodes, nodeT *ground){

  long row, col;

  /* loop over each element and initialize values */
  for(row=0;row<nnrow;row++){
    for(col=0;col<nncol;col++){
      nodes[row][col].group=NOTINBUCKET;
      nodes[row][col].incost=VERYFAR;
      nodes[row][col].outcost=VERYFAR;
      nodes[row][col].pred=NULL;
    }
  }

  /* initialize the ground node */
  if(ground!=NULL){
    ground->group=NOTINBUCKET;
    ground->incost=VERYFAR;
    ground->outcost=VERYFAR;
    ground->pred=NULL;
  }

  /* done */
  return(0);

}


/* function: BucketInsert()
 * ------------------------
 */
void BucketInsert(nodeT *node, long ind, bucketT *bkts){

  /* put node at beginning of bucket list */
  node->next=bkts->bucket[ind];
  if((bkts->bucket[ind])!=NULL){
    bkts->bucket[ind]->prev=node;
  }
  bkts->bucket[ind]=node;
  node->prev=NULL;

  /* mark node in bucket array */
  node->group=INBUCKET;

  /* done */
  return;

}


/* function: BucketRemove()
 * ------------------------
 */
void BucketRemove(nodeT *node, long ind, bucketT *bkts){

  /* remove node from doubly linked list */
  if((node->next)!=NULL){
    node->next->prev=node->prev;
  }
  if(node->prev!=NULL){
    node->prev->next=node->next;
  }else if(node->next==NULL){
    bkts->bucket[ind]=NULL;
  }else{
    bkts->bucket[ind]=node->next;
  }

  /* done */
  return;

}


/* function: ClosestNode()
 * -----------------------
 */
nodeT *ClosestNode(bucketT *bkts){

  nodeT *node;

  /* find the first bucket with nodes in it */
  while(TRUE){

    /* see if we got to the last bucket */
    if((bkts->curr)>(bkts->maxind)){
        return(NULL);
    }

    /* see if we found a nonempty bucket; if so, return it */
    if((bkts->bucket[bkts->curr])!=NULL){
      node=bkts->bucket[bkts->curr];
      node->group=ONTREE;
      bkts->bucket[bkts->curr]=node->next;
      if((node->next)!=NULL){
        node->next->prev=NULL;
      }
      return(node);
    }

    /* move to next bucket */
    bkts->curr++;

  }
}


/* function: ClosestNodeCircular()
 * -------------------------------
 * Similar to ClosestNode(), but assumes circular buckets.  This
 * function should NOT be used if negative arc weights exist on the
 * network; initial value of bkts->minind should always be zero.
 */
/*
 * This function is no longer used
 */
#if 0
static
nodeT *ClosestNodeCircular(bucketT *bkts);
static
nodeT *ClosestNodeCircular(bucketT *bkts){

  nodeT *node;

  /* find the first bucket with nodes in it */
  while(TRUE){

    /* see if we got to the last bucket */
    if((bkts->curr+bkts->minind)>(bkts->maxind)){
      if(bkts->wrapped){
        bkts->wrapped=FALSE;
        bkts->curr=0;
        bkts->minind+=bkts->size;
        bkts->maxind+=bkts->size;
      }else{
        return(NULL);
      }
    }

    /* see if we found a nonempty bucket; if so, return it */
    if((bkts->bucket[bkts->curr])!=NULL){
      node=bkts->bucket[bkts->curr];
      node->group=ONTREE;
      bkts->bucket[bkts->curr]=node->next;
      if((node->next)!=NULL){
        node->next->prev=NULL;
      }
      return(node);
    }

    /* move to next bucket */
    bkts->curr++;

  }
}
#endif


/* function: MinOutCostNode()
 * --------------------------
 * Similar to ClosestNode(), but always returns closest node even if its
 * outcost is less than the minimum bucket index.  Does not handle circular
 * buckets.  Does not handle no nodes left condition (this should be handled
 * by calling function).
 */
static
nodeT *MinOutCostNode(bucketT *bkts){

  long minoutcost;
  nodeT *node1, *node2;

  /* move to next non-empty bucket */
  while(bkts->curr<bkts->maxind && bkts->bucket[bkts->curr]==NULL){
    bkts->curr++;
  }

  /* scan the whole bucket if it is the overflow or underflow bag */
  if(bkts->curr==bkts->minind || bkts->curr==bkts->maxind){

    node2=bkts->bucket[bkts->curr];
    node1=node2;
    minoutcost=node1->outcost;
    while(node2!=NULL){
      if(node2->outcost<minoutcost){
        minoutcost=node2->outcost;
        node1=node2;
      }
      node2=node2->next;
    }
    BucketRemove(node1,bkts->curr,bkts);

  }else{

    node1=bkts->bucket[bkts->curr];
    bkts->bucket[bkts->curr]=node1->next;
    if(node1->next!=NULL){
      node1->next->prev=NULL;
    }

  }

  return(node1);

}


/* function: SelectSources()
 * -------------------------
 * Create a list of node pointers to be sources for each set of
 * connected pixels (not disconnected by masking).  Return the number
 * of sources (ie, the number of connected sets of pixels).
 */
long SelectSources(nodeT **nodes, float **mag, nodeT *ground, long nflow,
                   short **flows, long ngroundarcs,
                   long nrow, long ncol, paramT *params,
                   nodeT ***sourcelistptr, long **nconnectedarrptr){

  long row, col, nsource, nsourcelistmem, nconnected;
  long *nconnectedarr;
  nodeT *source;
  nodeT **sourcelist;


  /* initialize local variables */
  nsource=0;
  nsourcelistmem=0;
  sourcelist=NULL;
  nconnectedarr=NULL;

  /* loop over nodes to initialize */
  if(ground->group!=MASKED && ground->group!=BOUNDARYPTR){
    ground->group=0;
  }
  ground->next=NULL;
  for(row=0;row<nrow-1;row++){
    for(col=0;col<ncol-1;col++){
      if(nodes[row][col].group!=MASKED && nodes[row][col].group!=BOUNDARYPTR){
        nodes[row][col].group=0;
      }
      nodes[row][col].next=NULL;
    }
  }

  /* check ground node (NULL for boundary since not yet defined) */
  source=SelectConnNodeSource(nodes,mag,
                              ground,ngroundarcs,nrow,ncol,params,
                              ground,&nconnected);
  if(source!=NULL){

    /* get more memory for source list if needed */
    if(++nsource>nsourcelistmem){
      nsourcelistmem+=NSOURCELISTMEMINCR;
      sourcelist=ReAlloc(sourcelist,nsourcelistmem*sizeof(nodeT *));
      nconnectedarr=ReAlloc(nconnectedarr,nsourcelistmem*sizeof(long));
    }

    /* store source in list */
    sourcelist[nsource-1]=source;
    nconnectedarr[nsource-1]=nconnected;

  }

  /* loop over nodes to find next set of connected nodes */
  for(row=0;row<nrow-1;row++){
    for(col=0;col<ncol-1;col++){

      /* check pixel */
      /*   boundary not yet defined, so connectivity via grid arcs only */
      source=SelectConnNodeSource(nodes,mag,
                                  ground,ngroundarcs,nrow,ncol,params,
                                  &nodes[row][col],&nconnected);
      if(source!=NULL){

        /* get more memory for source list if needed */
        if(++nsource>nsourcelistmem){
          nsourcelistmem+=NSOURCELISTMEMINCR;
          sourcelist=ReAlloc(sourcelist,nsourcelistmem*sizeof(nodeT *));
          nconnectedarr=ReAlloc(nconnectedarr,nsourcelistmem*sizeof(long));
        }

        /* store source in list */
        sourcelist[nsource-1]=source;
        nconnectedarr[nsource-1]=nconnected;

      }
    }
  }

  /* show message about number of connected regions */
  fprintf(sp1,"Found %ld valid set(s) of connected nodes\n",nsource);

  /* reset group values for all nodes */
  if(ground->group!=MASKED && ground->group!=BOUNDARYPTR){
    ground->group=0;
  }
  ground->next=NULL;
  for(row=0;row<nrow-1;row++){
    for(col=0;col<ncol-1;col++){
      if(nodes[row][col].group==INBUCKET
         || nodes[row][col].group==NOTINBUCKET
         || nodes[row][col].group==BOUNDARYCANDIDATE
         || nodes[row][col].group==PRUNED){
        fflush(NULL);
        fprintf(stderr,
                "WARNING: weird nodes[%ld][%ld].group=%d in SelectSources()\n",
                row,col,nodes[row][col].group);
      }

      if(nodes[row][col].group!=MASKED && nodes[row][col].group!=BOUNDARYPTR){
        nodes[row][col].group=0;
      }
      nodes[row][col].next=NULL;
    }
  }


  /* done */
  if(sourcelistptr!=NULL){
    (*sourcelistptr)=sourcelist;
  }else{
    fflush(NULL);
    fprintf(sp0,"ERROR: NULL sourcelistptr in SelectSources()\nAbort\n");
    exit(ABNORMAL_EXIT);
  }
  if(nconnectedarrptr!=NULL){
    (*nconnectedarrptr)=nconnectedarr;
  }else{
    fflush(NULL);
    fprintf(sp0,"ERROR: NULL nconnectedarrptr SelectSources()\nAbort\n");
    exit(ABNORMAL_EXIT);
  }
  return(nsource);

}


/* function: SelectConnNodeSource()
 * --------------------------------
 * Select source from among set of connected nodes specified by
 * starting node.  Return NULL if the start node is masked or already
 * part of another connected set, or if the connected set is too
 * small.
 */
static
nodeT *SelectConnNodeSource(nodeT **nodes, float **mag,
                            nodeT *ground, long ngroundarcs,
                            long nrow, long ncol,
                            paramT *params, nodeT *start, long *nconnectedptr){

  long nconnected;
  nodeT *source;


  /* if start node is not eligible, just return NULL */
  if(start->group==MASKED || start->group==ONTREE){
    return(NULL);
  }

  /* find all nodes for this set of connected pixels and mark them on tree */
  nconnected=ScanRegion(start,nodes,mag,ground,ngroundarcs,nrow,ncol,ONTREE);

  /* see if number of nodes in this connected set is big enough */
  if(nconnected>params->nconnnodemin){

    /* set source to first node in chain */
    /* this ensures that the soruce is the ground node or on the edge */
    /*   of the connected region, which tends to be faster */
    source = start;

  }else{
    source=NULL;
  }

  /* set number of connected nodes and return source */
  if(nconnectedptr!=NULL){
    (*nconnectedptr)=nconnected;
  }
  return(source);

}


/* function: ScanRegion()
 * ----------------------
 * Find all connected grid nodes of region, defined by reachability without
 * crossing non-region arcs, and set group according to desired
 * behavior defined by groupsetting, which should be either ONTREE for
 * call from SelectConnNodeSourcre(), MASKED for a call from
 * InitBoundary(), or 0 for a call from CleanUpBoundaryNodes().  Return
 * number of connected nodes.
 */
static
long ScanRegion(nodeT *start, nodeT **nodes, float **mag,
                nodeT *ground, long ngroundarcs,
                long nrow, long ncol, int groupsetting){

  nodeT *node1, *node2, *end;
  long arcrow, arccol, arcdir, arcnum, upperarcnum, nconnected;
  nodesuppT **nodesupp;
  boundaryT *boundary;


  /* set up */
  nconnected=0;
  end=start;
  nodesupp=NULL;
  boundary=NULL;
  node1=start;
  node1->group=INBUCKET;

  /* loop to search for connected nodes */
  while(node1!=NULL){

    /* loop over neighbors of current node */
    arcnum=GetArcNumLims(node1->row,&upperarcnum,ngroundarcs,boundary);
    while(arcnum<upperarcnum){
      node2=NeighborNode(node1,++arcnum,&upperarcnum,nodes,ground,
                         &arcrow,&arccol,&arcdir,nrow,ncol,boundary,nodesupp);

      /* if neighbor is pointer to boundary, unset so we scan grid nodes */
      if(node2->group==BOUNDARYPTR){
        node2->group=0;
      }

      /* see if neighbor is in region */
      if(IsRegionArc(mag,arcrow,arccol,nrow,ncol)){

        /* if neighbor is in region and not yet in list to be scanned, add it */
        if(node2->group!=ONTREE && node2->group!=INBUCKET){
          node2->group=INBUCKET;
          end->next=node2;
          node2->next=NULL;
          end=node2;
        }
      }
    }

    /* mark this node visited */
    node1->group=ONTREE;

    /* make sure level is initialized */
    if(groupsetting==ONTREE){
      node1->level=0;
    }

    /* count this node */
    nconnected++;

    /* move to next node in list */
    node1=node1->next;

  }

  /* for each node in region, scan neighbors to mask or unmask unreachable */
  /*   nodes that may be in other regions and therefore not yet masked */
  if(groupsetting!=ONTREE){

    /* loop over nodes in region */
    node1=start;
    while(node1!=NULL){

      /* loop over neighbors of current node */
      arcnum=GetArcNumLims(node1->row,&upperarcnum,ngroundarcs,boundary);
      while(arcnum<upperarcnum){
        node2=NeighborNode(node1,++arcnum,&upperarcnum,nodes,ground,
                           &arcrow,&arccol,&arcdir,nrow,ncol,boundary,nodesupp);

        /* see if neighbor is not in region */
        if(node2->group!=ONTREE){

          /* set mask status according to desired behavior */
          if(groupsetting==MASKED){
            node2->group=MASKED;
          }else if(groupsetting==0){
            if(node2->row==GROUNDROW){
              node2->group=GroundMaskStatus(nrow,ncol,mag);
            }else{
              node2->group=GridNodeMaskStatus(node2->row,node2->col,mag);
            }
          }
        }
      }

      /* move to next node */
      node1=node1->next;

    }

    /* reset groups of all nodes within region */
    node1=start;
    while(node1!=NULL){
      node1->group=0;
      node1=node1->next;
    }
  }

  /* return number of connected nodes */
  return(nconnected);

}


/* function: GetCost()
 * -------------------
 * Returns incremental flow cost for current flow increment dflow from
 * lookup array.
 */
static
short GetCost(incrcostT **incrcosts, long arcrow, long arccol,
              long arcdir){

  /* look up cost and return it for the appropriate arc direction */
  /* we may want add a check here for clipped incremental costs */
  if(arcdir>0){
    return(incrcosts[arcrow][arccol].poscost);
  }else{
    return(incrcosts[arcrow][arccol].negcost);
  }
}


/* function: ReCalcCost()
 * ----------------------
 * Updates the incremental cost for an arc.
 */
long ReCalcCost(void **costs, incrcostT **incrcosts, long flow,
                long arcrow, long arccol, long nflow, long nrow,
                paramT *params){

  long poscost, negcost, iclipped;

  /* calculate new positive and negative nflow costs, as long ints */
  CalcCost(costs,flow,arcrow,arccol,nflow,nrow,params,
           &poscost,&negcost);

  /* clip costs to short int */
  iclipped=0;
  if(poscost>LARGESHORT){
    incrcosts[arcrow][arccol].poscost=LARGESHORT;
    iclipped++;
  }else{
    if(poscost<-LARGESHORT){
      incrcosts[arcrow][arccol].poscost=-LARGESHORT;
      iclipped++;
    }else{
      incrcosts[arcrow][arccol].poscost=poscost;
    }
  }
  if(negcost>LARGESHORT){
    incrcosts[arcrow][arccol].negcost=LARGESHORT;
    iclipped++;
  }else{
    if(negcost<-LARGESHORT){
      incrcosts[arcrow][arccol].negcost=-LARGESHORT;
      iclipped++;
    }else{
      incrcosts[arcrow][arccol].negcost=negcost;
    }
  }

  /* return the number of clipped incremental costs (0, 1, or 2) */
  return(iclipped);
}


/* function: SetupIncrFlowCosts()
 * ------------------------------
 * Calculates the costs for positive and negative dflow flow increment
 * if there is zero flow on the arc.
 */
int SetupIncrFlowCosts(void **costs, incrcostT **incrcosts, short **flows,
                       long nflow, long nrow, long narcrow,
                       int *narcsperrow, paramT *params){

  long arcrow, arccol, iclipped, narcs;
  char pl[2];


  /* loop over all rows and columns */
  narcs=0;
  iclipped=0;
  for(arcrow=0;arcrow<narcrow;arcrow++){
    narcs+=narcsperrow[arcrow];
    for(arccol=0;arccol<narcsperrow[arcrow];arccol++){

      /* calculate new positive and negative nflow costs, as long ints */
      iclipped+=ReCalcCost(costs,incrcosts,flows[arcrow][arccol],
                           arcrow,arccol,nflow,nrow,params);
    }
  }

  /* print overflow warning if applicable */
  if(iclipped){
    if(iclipped>1){
      strcpy(pl,"s");
    }else{
      strcpy(pl,"");
    }
    fflush(NULL);
    fprintf(sp0,"%ld incremental cost%s clipped to avoid overflow (%.3f%%)\n",
            iclipped,pl,((double )iclipped)/(2*narcs));
  }

  /* done */
  return(0);

}


/* function: EvaluateTotalCost()
 * -----------------------------
 * Computes the total cost of the flow array and prints it out.  Pass nrow
 * and ncol if in grid mode (primary network), or pass nrow=ntiles and
 * ncol=0 for nongrid mode (secondary network).
 */
totalcostT EvaluateTotalCost(void **costs, short **flows, long nrow, long ncol,
                             int *narcsperrow,paramT *params){

  totalcostT rowcost, totalcost;
  long row, col, maxrow, maxcol;

  /* sum cost for each row and column arc */
  totalcost=0;
  if(ncol){
    maxrow=2*nrow-1;
  }else{
    maxrow=nrow;
  }
  for(row=0;row<maxrow;row++){
    rowcost=0;
    if(ncol){
      if(row<nrow-1){
        maxcol=ncol;
      }else{
        maxcol=ncol-1;
      }
    }else{
      maxcol=narcsperrow[row];
    }
    for(col=0;col<maxcol;col++){
      rowcost+=EvalCost(costs,flows,row,col,nrow,params);
    }
    totalcost+=rowcost;
  }

  return(totalcost);
}


/* function: MSTInitFlows()
 * ------------------------
 * Initializes the flow on a the network using minimum spanning tree
 * algorithm.
 */
int MSTInitFlows(float **wrappedphase, short ***flowsptr,
                 short **mstcosts, long nrow, long ncol,
                 nodeT ***nodesptr, nodeT *ground, long maxflow){

  long row, col, i, maxcost;
  signed char **residue, **arcstatus;
  short **flows;
  nodeT *source;
  bucketT bkts[1];


  /* initialize stack structures to zero for good measure */
  memset(bkts,0,sizeof(bucketT));

  /* get and initialize memory for ground, nodes, buckets, and child array */
  *nodesptr=(nodeT **)Get2DMem(nrow-1,ncol-1,sizeof(nodeT *),sizeof(nodeT));
  InitNodeNums(nrow-1,ncol-1,*nodesptr,ground);

  /* find maximum cost */
  maxcost=0;
  for(row=0;row<2*nrow-1;row++){
    if(row<nrow-1){
      i=ncol;
    }else{
      i=ncol-1;
    }
    for(col=0;col<i;col++){
      if(mstcosts[row][col]>maxcost
         && !((row==nrow-1 || 2*nrow-2) && (col==0 || col==ncol-2))){
        maxcost=mstcosts[row][col];
      }
    }
  }

  /* get memory for buckets and arc status */
  bkts->size=LRound((maxcost+1)*(nrow+ncol+1));
  bkts->bucketbase=(nodeT **)MAlloc(bkts->size*sizeof(nodeT *));
  bkts->minind=0;
  bkts->maxind=bkts->size-1;
  bkts->bucket=bkts->bucketbase;
  arcstatus=(signed char **)Get2DRowColMem(nrow,ncol,sizeof(signed char *),
                                           sizeof(signed char));

  /* calculate phase residues (integer numbers of cycles) */
  fprintf(sp1,"Initializing flows with MST algorithm\n");
  residue=(signed char **)Get2DMem(nrow-1,ncol-1,sizeof(signed char *),
                                   sizeof(signed char));
  CycleResidue(wrappedphase,residue,nrow,ncol);

  /* get memory for flow arrays */
  (*flowsptr)=(short **)Get2DRowColZeroMem(nrow,ncol,
                                           sizeof(short *),sizeof(short));
  flows=*flowsptr;

  /* loop until no flows exceed the maximum flow */
  fprintf(sp2,"Running approximate minimum spanning tree solver\n");
  while(TRUE){

    /* set up the source to be the first non-zero residue that we find */
    source=NULL;
    for(row=0;row<nrow-1 && source==NULL;row++){
      for(col=0;col<ncol-1 && source==NULL;col++){
        if(residue[row][col]){
          source=&(*nodesptr)[row][col];
        }
      }
    }
    if(source==NULL){
      fprintf(sp1,"No residues found\n");
      break;
    }

    /* initialize data structures */
    InitNodes(nrow-1,ncol-1,*nodesptr,ground);
    InitBuckets(bkts,source,bkts->size);

    /* solve the mst problem */
    SolveMST(*nodesptr,source,ground,bkts,mstcosts,residue,arcstatus,
             nrow,ncol);

    /* find flows on minimum tree (only one feasible flow exists) */
    DischargeTree(source,mstcosts,flows,residue,arcstatus,
                  *nodesptr,ground,nrow,ncol);

    /* do pushes to clip the flows and make saturated arcs ineligible */
    /* break out of loop if there is no flow greater than the limit */
    if(ClipFlow(residue,flows,mstcosts,nrow,ncol,maxflow)){
      break;
    }
  }

  /* free memory and return */
  Free2DArray((void **)residue,nrow-1);
  Free2DArray((void **)arcstatus,2*nrow-1);
  Free2DArray((void **)mstcosts,2*nrow-1);
  free(bkts->bucketbase);
  return(0);

}


/* function: SolveMST()
 * --------------------
 * Finds tree which spans all residue nodes of approximately minimal length.
 * Note that this function may produce a Steiner tree (tree may split at
 * non-residue node), though finding the exactly minimum Steiner tree is
 * NP-hard.  This function uses Prim's algorithm, nesting Dijkstra's
 * shortest path algorithm in each iteration to find next closest residue
 * node to tree.  See Ahuja, Orlin, and Magnanti 1993 for details.
 *
 * Dijkstra implementation and some associated functions adapted from SPLIB
 * shortest path codes written by Cherkassky, Goldberg, and Radzik.
 */
static
void SolveMST(nodeT **nodes, nodeT *source, nodeT *ground,
              bucketT *bkts, short **mstcosts, signed char **residue,
              signed char **arcstatus, long nrow, long ncol){

  nodeT *from, *to, *pathfrom, *pathto;
  nodesuppT **nodesupp;
  long fromdist, newdist, arcdist, ngroundarcs, groundcharge;
  long fromrow, fromcol, row, col, arcnum, upperarcnum, maxcol;
  long pathfromrow, pathfromcol;
  long arcrow, arccol, arcdir;

  /* initialize some variables */
  nodesupp=NULL;

  /* calculate the number of ground arcs */
  ngroundarcs=2*(nrow+ncol-2)-4;

  /* calculate charge on ground */
  groundcharge=0;
  for(row=0;row<nrow-1;row++){
    for(col=0;col<ncol-1;col++){
      groundcharge-=residue[row][col];
    }
  }

  /* initialize arc status array */
  for(arcrow=0;arcrow<2*nrow-1;arcrow++){
    if(arcrow<nrow-1){
      maxcol=ncol;
    }else{
      maxcol=ncol-1;
    }
    for(arccol=0;arccol<maxcol;arccol++){
      arcstatus[arcrow][arccol]=0;
    }
  }

  /* loop until there are no more nodes in any bucket */
  while((from=ClosestNode(bkts))!=NULL){

    /* info for current node */
    fromrow=from->row;
    fromcol=from->col;

    /* if we found a residue */
    if(((fromrow!=GROUNDROW && residue[fromrow][fromcol]) ||
       (fromrow==GROUNDROW && groundcharge)) && from!=source){

      /* set node and its predecessor */
      pathto=from;
      pathfrom=from->pred;

      /* go back and make arcstatus -1 along path */
      while(TRUE){

        /* give to node zero distance label */
        pathto->outcost=0;

        /* get arc indices for arc between pathfrom and pathto */
        GetArc(pathfrom,pathto,&arcrow,&arccol,&arcdir,nrow,ncol,
               nodes,nodesupp);

        /* set arc status to -1 to mark arc on tree */
        arcstatus[arcrow][arccol]=-1;

        /* stop when we get to a residue */
        pathfromrow=pathfrom->row;
        pathfromcol=pathfrom->col;
        if((pathfromrow!=GROUNDROW && residue[pathfromrow][pathfromcol])
           || (pathfromrow==GROUNDROW && groundcharge)){
          break;
        }

        /* move up to previous node pair in path */
        pathto=pathfrom;
        pathfrom=pathfrom->pred;

      } /* end while loop marking costs on path */

    } /* end if we found a residue */

    /* set a variable for from node's distance */
    fromdist=from->outcost;

    /* scan from's neighbors */
    arcnum=GetArcNumLims(fromrow,&upperarcnum,ngroundarcs,NULL);
    while(arcnum<upperarcnum){

      /* get row, col indices and distance of next node */
      to=NeighborNode(from,++arcnum,&upperarcnum,nodes,ground,
                      &arcrow,&arccol,&arcdir,nrow,ncol,NULL,nodesupp);
      row=to->row;
      col=to->col;

      /* get cost of arc to new node (if arc on tree, cost is 0) */
      if(arcstatus[arcrow][arccol]<0){
        arcdist=0;
      }else if((arcdist=mstcosts[arcrow][arccol])==LARGESHORT){
        arcdist=VERYFAR;
      }

      /* compare distance of new nodes to temp labels */
      if((newdist=fromdist+arcdist)<(to->outcost)){

        /* if to node is already in a bucket, remove it */
        if(to->group==INBUCKET){
          if(to->outcost<bkts->maxind){
            BucketRemove(to,to->outcost,bkts);
          }else{
            BucketRemove(to,bkts->maxind,bkts);
          }
        }

        /* update to node */
        to->outcost=newdist;
        to->pred=from;

        /* insert to node into appropriate bucket */
        if(newdist<bkts->maxind){
          BucketInsert(to,newdist,bkts);
          if(newdist<bkts->curr){
            bkts->curr=newdist;
          }
        }else{
          BucketInsert(to,bkts->maxind,bkts);
        }

      } /* end if newdist < old dist */

    } /* end loop over outgoing arcs */
  } /* end while ClosestNode()!=NULL */

}


/* function: DischargeTree()
 * -------------------------
 * Does depth-first search on result tree from SolveMST.  Integrates
 * charges from tree leaves back up to set arc flows.  This implementation
 * is non-recursive; a recursive implementation might be faster, but
 * would also use much more stack memory.  This method is equivalent to
 * walking the tree, so it should be nore more than a factor of 2 slower.
 */
static
long DischargeTree(nodeT *source, short **mstcosts, short **flows,
                   signed char **residue, signed char **arcstatus,
                   nodeT **nodes, nodeT *ground, long nrow, long ncol){

  long row, col, todir, arcrow, arccol, arcdir;
  long arcnum, upperarcnum, ngroundarcs;
  nodeT *from, *to, *nextnode;
  nodesuppT **nodesupp;


  /* set up */
  /* use outcost member of node structure to temporarily store charge */
  nextnode=source;
  ground->outcost=0;
  for(row=0;row<nrow-1;row++){
    for(col=0;col<ncol-1;col++){
      nodes[row][col].outcost=residue[row][col];
      ground->outcost-=residue[row][col];
    }
  }
  ngroundarcs=2*(nrow+ncol-2)-4;
  nodesupp=NULL;
  todir=0;

  /* keep looping unitl we've walked the entire tree */
  while(TRUE){

    from=nextnode;
    nextnode=NULL;

    /* loop over outgoing arcs from this node */
    arcnum=GetArcNumLims(from->row,&upperarcnum,ngroundarcs,NULL);
    while(arcnum<upperarcnum){

      /* get row, col indices and distance of next node */
      to=NeighborNode(from,++arcnum,&upperarcnum,nodes,ground,
                      &arcrow,&arccol,&arcdir,nrow,ncol,NULL,nodesupp);

      /* see if the arc is on the tree and if it has been followed yet */
      if(arcstatus[arcrow][arccol]==-1){

        /* arc has not yet been followed: move down the tree */
        nextnode=to;
        row=arcrow;
        col=arccol;
        break;

      }else if(arcstatus[arcrow][arccol]==-2){

        /* arc has already been followed and leads back up the tree: */
        /* save it, but keep looking for downwards arc */
        nextnode=to;
        row=arcrow;
        col=arccol;
        todir=arcdir;

      }
    }

    /* break if no unfollowed arcs (ie, we are done examining the tree) */
    if(nextnode==NULL){
      break;
    }

    /* if we found leaf and we're moving back up the tree, do a push */
    /* otherwise, just mark the path by decrementing arcstatus */
    if((--arcstatus[row][col])==-3){
      flows[row][col]+=todir*from->outcost;
      nextnode->outcost+=from->outcost;
      from->outcost=0;
    }
  }

  /* finish up */
  return(from->outcost);

} /* end of DischargeTree() */


/* function: ClipFlow()
 * ---------------------
 * Given a flow, clips flow magnitudes to a computed limit, resets
 * residues so sum of solution of network problem with new residues
 * and solution of clipped problem give total solution.  Upper flow limit
 * is 2/3 the maximum flow on the network or the passed value maxflow,
 * whichever is greater.  Clipped flow arcs get costs of passed variable
 * maxcost.  Residues should have been set to zero by DischargeTree().
 */
static
signed char ClipFlow(signed char **residue, short **flows,
                     short **mstcosts, long nrow, long ncol,
                     long maxflow){

  long row, col, cliplimit, maxcol, excess, tempcharge, sign;
  long mostflow, maxcost;


  /* find maximum flow */
  mostflow=Short2DRowColAbsMax(flows,nrow,ncol);

  /* if there is no flow greater than the maximum, return TRUE */
  if(mostflow<=maxflow){
    return(TRUE);
  }
  fprintf(sp2,"Maximum flow on network: %ld\n",mostflow);

  /* set upper flow limit */
  cliplimit=(long )ceil(mostflow*CLIPFACTOR)+1;
  if(maxflow>cliplimit){
    cliplimit=maxflow;
  }

  /* find maximum cost (excluding ineligible corner arcs) */
  maxcost=0;
  for(row=0;row<2*nrow-1;row++){
    if(row<nrow-1){
      maxcol=ncol;
    }else{
      maxcol=ncol-1;
    }
    for(col=0;col<maxcol;col++){
      if(mstcosts[row][col]>maxcost && mstcosts[row][col]<LARGESHORT){
        maxcost=mstcosts[row][col];
      }
    }
  }

  /* set the new maximum cost and make sure it doesn't overflow short int */
  maxcost+=INITMAXCOSTINCR;
  if(maxcost>=LARGESHORT){
    fflush(NULL);
    fprintf(sp0,"WARNING: escaping ClipFlow loop to prevent cost overflow\n");
    return(TRUE);
  }

  /* clip flows and do pushes */
  for(row=0;row<2*nrow-1;row++){
    if(row<nrow-1){
      maxcol=ncol;
    }else{
      maxcol=ncol-1;
    }
    for(col=0;col<maxcol;col++){
      if(labs(flows[row][col])>cliplimit){
        if(flows[row][col]>0){
          sign=1;
          excess=flows[row][col]-cliplimit;
        }else{
          sign=-1;
          excess=flows[row][col]+cliplimit;
        }
        if(row<nrow-1){
          if(col!=0){
            tempcharge=residue[row][col-1]+excess;
            if(tempcharge>MAXRES || tempcharge<MINRES){
              fflush(NULL);
              fprintf(sp0,"Overflow of residue data type\nAbort\n");
              exit(ABNORMAL_EXIT);
            }
            residue[row][col-1]=tempcharge;
          }
          if(col!=ncol-1){
            tempcharge=residue[row][col]-excess;
            if(tempcharge<MINRES || tempcharge>MAXRES){
              fflush(NULL);
              fprintf(sp0,"Overflow of residue data type\nAbort\n");
              exit(ABNORMAL_EXIT);
            }
            residue[row][col]=tempcharge;
          }
        }else{
          if(row!=nrow-1){
            tempcharge=residue[row-nrow][col]+excess;
            if(tempcharge>MAXRES || tempcharge<MINRES){
              fflush(NULL);
              fprintf(sp0,"Overflow of residue data type\nAbort\n");
              exit(ABNORMAL_EXIT);
            }
            residue[row-nrow][col]=tempcharge;
          }
          if(row!=2*nrow-2){
            tempcharge=residue[row-nrow+1][col]-excess;
            if(tempcharge<MINRES || tempcharge>MAXRES){
              fflush(NULL);
              fprintf(sp0,"Overflow of residue data type\nAbort\n");
              exit(ABNORMAL_EXIT);
            }
            residue[row-nrow+1][col]=tempcharge;
          }
        }
        flows[row][col]=sign*cliplimit;
        mstcosts[row][col]=maxcost;
      }
    }
  }

  /* return value indicates that flows have been clipped */
  fprintf(sp2,"Flows clipped to %ld.  Rerunning MST solver.\n",cliplimit);
  return(FALSE);

}


/* function: MCFInitFlows()
 * ------------------------
 * Initializes the flow on a the network using minimum cost flow
 * algorithm.
 */
int MCFInitFlows(float **wrappedphase, short ***flowsptr, short **mstcosts,
                 long nrow, long ncol, long cs2scalefactor){

#ifndef NO_CS2

  signed char **residue;


  /* calculate phase residues (integer numbers of cycles) */
  fprintf(sp1,"Initializing flows with MCF algorithm\n");
  residue=(signed char **)Get2DMem(nrow-1,ncol-1,sizeof(signed char *),
                                   sizeof(signed char));
  CycleResidue(wrappedphase,residue,nrow,ncol);

  /* run the solver (memory freed within solver) */
  SolveCS2(residue,mstcosts,nrow,ncol,cs2scalefactor,flowsptr);

#endif

  /* done */
  return(0);
}

/* end snaphu_solver.c */
/* snaphu_tile.c */
/*************************************************************************

  snaphu tile-mode source file
  Written by Curtis W. Chen
  Copyright 2002 Board of Trustees, Leland Stanford Jr. University
  Please see the supporting documentation for terms of use.
  No warranty.

*************************************************************************/


FILE *OpenOutputFile(char *outfile, char *realoutfile);

/* static (local) function prototypes */
static
long ThickenCosts(incrcostT **incrcosts, long nrow, long ncol);
static
nodeT *RegionsNeighborNode(nodeT *node1, long *arcnumptr, nodeT **nodes,
                           long *arcrowptr, long *arccolptr,
                           long nrow, long ncol);
static
int ClearBuckets(bucketT *bkts);
static
int MergeRegions(nodeT **nodes, nodeT *source, long *regionsizes,
                  long closestregion, long nrow, long ncol);
static
int RenumberRegion(nodeT **nodes, nodeT *source, long newnum,
                   long nrow, long ncol);
static
int ReadNextRegion(long tilerow, long tilecol, long nlines, long linelen,
                   outfileT *outfiles, paramT *params,
                   short ***nextregionsptr, float ***nextunwphaseptr,
                   void ***nextcostsptr,
                   long *nextnrowptr, long *nextncolptr);
static
int SetTileReadParams(tileparamT *tileparams, long nexttilenlines,
                      long nexttilelinelen, long tilerow, long tilecol,
                      long nlines, long linelen, paramT *params);
static
int ReadEdgesAboveAndBelow(long tilerow, long tilecol, long nlines,
                           long linelen, paramT *params, outfileT *outfiles,
                           short *regionsabove, short *regionsbelow,
                           float *unwphaseabove, float *unwphasebelow,
                           void *costsabove, void *costsbelow);
static
int TraceRegions(short **regions, short **nextregions, short **lastregions,
                 short *regionsabove, short *regionsbelow, float **unwphase,
                 float **nextunwphase, float **lastunwphase,
                 float *unwphaseabove, float *unwphasebelow, void **costs,
                 void **nextcosts, void **lastcosts, void *costsabove,
                 void *costsbelow, long prevnrow, long prevncol, long tilerow,
                 long tilecol, long nrow, long ncol, nodeT **scndrynodes,
                 nodesuppT **nodesupp, scndryarcT **scndryarcs,
                 long ***scndrycosts, int *nscndrynodes,
                 int *nscndryarcs, long *totarclens, short **bulkoffsets,
                 paramT *params);
static
long FindNumPathsOut(nodeT *from, paramT *params, long tilerow, long tilecol,
                     long nnrow, long nncol, short **regions,
                     short **nextregions, short **lastregions,
                     short *regionsabove, short *regionsbelow, long prevncol);
static
int RegionTraceCheckNeighbors(nodeT *from, nodeT **nextnodeptr,
                              nodeT **primarynodes, short **regions,
                              short **nextregions, short **lastregions,
                              short *regionsabove, short *regionsbelow,
                              long tilerow, long tilecol, long nnrow,
                              long nncol, nodeT **scndrynodes,
                              nodesuppT **nodesupp, scndryarcT **scndryarcs,
                              long *nnewnodesptr, long *nnewarcsptr,
                              long flowmax, long nrow, long ncol,
                              long prevnrow, long prevncol, paramT *params,
                              void **costs, void **rightedgecosts,
                              void **loweredgecosts, void **leftedgecosts,
                              void **upperedgecosts, short **flows,
                              short **rightedgeflows, short **loweredgeflows,
                              short **leftedgeflows, short **upperedgeflows,
                              long ***scndrycosts,
                              nodeT ***updatednontilenodesptr,
                              long *nupdatednontilenodesptr,
                              long *updatednontilenodesizeptr,
                              short **inontilenodeoutarcptr,
                              long *totarclenptr);
static
int SetUpperEdge(long ncol, long tilerow, long tilecol, void **voidcosts,
                 void *voidcostsabove, float **unwphase,
                 float *unwphaseabove, void **voidupperedgecosts,
                 short **upperedgeflows, paramT *params, short **bulkoffsets);
static
int SetLowerEdge(long nrow, long ncol, long tilerow, long tilecol,
                 void **voidcosts, void *voidcostsbelow,
                 float **unwphase, float *unwphasebelow,
                 void **voidloweredgecosts, short **loweredgeflows,
                 paramT *params, short **bulkoffsets);
static
int SetLeftEdge(long nrow, long prevncol, long tilerow, long tilecol,
                void **voidcosts, void **voidlastcosts, float **unwphase,
                float **lastunwphase, void **voidleftedgecosts,
                short **leftedgeflows, paramT *params, short **bulkoffsets);
static
int SetRightEdge(long nrow, long ncol, long tilerow, long tilecol,
                 void **voidcosts, void **voidnextcosts,
                 float **unwphase, float **nextunwphase,
                 void **voidrightedgecosts, short **rightedgeflows,
                 paramT *params, short **bulkoffsets);
static
short AvgSigSq(short sigsq1, short sigsq2);
static
int TraceSecondaryArc(nodeT *primaryhead, nodeT **scndrynodes,
                      nodesuppT **nodesupp, scndryarcT **scndryarcs,
                      long ***scndrycosts, long *nnewnodesptr,
                      long *nnewarcsptr, long tilerow, long tilecol,
                      long flowmax, long nrow, long ncol,
                      long prevnrow, long prevncol, paramT *params,
                      void **tilecosts, void **rightedgecosts,
                      void **loweredgecosts, void **leftedgecosts,
                      void **upperedgecosts, short **tileflows,
                      short **rightedgeflows, short **loweredgeflows,
                      short **leftedgeflows, short **upperedgeflows,
                      nodeT ***updatednontilenodesptr,
                      long *nupdatednontilenodesptr,
                      long *updatednontilenodesizeptr,
                      short **inontilenodeoutarcptr, long *totarclenptr);
static
nodeT *FindScndryNode(nodeT **scndrynodes, nodesuppT **nodesupp,
                      long tilenum, long primaryrow, long primarycol);
static
int IntegrateSecondaryFlows(long linelen, long nlines, nodeT **scndrynodes,
                            nodesuppT **nodesupp, scndryarcT **scndryarcs,
                            int *nscndryarcs, short **scndryflows,
                            short **bulkoffsets, outfileT *outfiles,
                            paramT *params);
static
int ParseSecondaryFlows(long tilenum, int *nscndryarcs, short **tileflows,
                        short **regions, short **scndryflows,
                        nodesuppT **nodesupp, scndryarcT **scndryarcs,
                        long nrow, long ncol, long ntilerow, long ntilecol,
                        paramT *params);
static
int AssembleTileConnComps(long linelen, long nlines,
                          outfileT *outfiles, paramT *params);
static
int ConnCompSizeNPixCompare(const void *ptr1, const void *ptr2);



/* function: SetupTile()
 * ---------------------
 * Sets up tile parameters and output file names for the current tile.
 */
int SetupTile(long nlines, long linelen, paramT *params,
              tileparamT *tileparams, outfileT *outfiles,
              outfileT *tileoutfiles, long tilerow, long tilecol){

  long ni, nj;
  char tempstring[MAXTMPSTRLEN], path[MAXSTRLEN], basename[MAXSTRLEN];
  char *tiledir;


  /* set parameters for current tile */
  ni=ceil((nlines+(params->ntilerow-1)*params->rowovrlp)
          /(double )params->ntilerow);
  nj=ceil((linelen+(params->ntilecol-1)*params->colovrlp)
          /(double )params->ntilecol);
  tileparams->firstrow=tilerow*(ni-params->rowovrlp);
  tileparams->firstcol=tilecol*(nj-params->colovrlp);
  if(tilerow==params->ntilerow-1){
    tileparams->nrow=nlines-(params->ntilerow-1)*(ni-params->rowovrlp);
  }else{
    tileparams->nrow=ni;
  }
  if(tilecol==params->ntilecol-1){
    tileparams->ncol=linelen-(params->ntilecol-1)*(nj-params->colovrlp);
  }else{
    tileparams->ncol=nj;
  }

  /* error checking on tile size */
  if(params->minregionsize > (tileparams->nrow)*(tileparams->ncol)){
    fflush(NULL);
    fprintf(sp0,"Minimum region size cannot exceed tile size\nAbort\n");
    exit(ABNORMAL_EXIT);
  }

  /* set output files */
  tiledir=params->tiledir;
  ParseFilename(outfiles->outfile,path,basename);
  sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld",
          tiledir,TMPTILEROOT,basename,tilerow,tilecol,tileparams->ncol);
  StrNCopy(tileoutfiles->outfile,tempstring,MAXSTRLEN);
  if(strlen(outfiles->initfile)){
    ParseFilename(outfiles->initfile,path,basename);
    sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld",
            tiledir,TMPTILEROOT,basename,tilerow,tilecol,tileparams->ncol);
    StrNCopy(tileoutfiles->initfile,tempstring,MAXSTRLEN);
  }else{
    StrNCopy(tileoutfiles->initfile,"",MAXSTRLEN);
  }
  if(strlen(outfiles->flowfile)){
    ParseFilename(outfiles->flowfile,path,basename);
    sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld",
            tiledir,TMPTILEROOT,basename,tilerow,tilecol,tileparams->ncol);
    StrNCopy(tileoutfiles->flowfile,tempstring,MAXSTRLEN);
  }else{
    StrNCopy(tileoutfiles->flowfile,"",MAXSTRLEN);
  }
  if(strlen(outfiles->eifile)){
    ParseFilename(outfiles->eifile,path,basename);
    sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld",
            tiledir,TMPTILEROOT,basename,tilerow,tilecol,tileparams->ncol);
    StrNCopy(tileoutfiles->eifile,tempstring,MAXSTRLEN);
  }else{
    StrNCopy(tileoutfiles->eifile,"",MAXSTRLEN);
  }
  if(strlen(outfiles->rowcostfile)){
    ParseFilename(outfiles->rowcostfile,path,basename);
    sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld",
            tiledir,TMPTILEROOT,basename,tilerow,tilecol,tileparams->ncol);
    StrNCopy(tileoutfiles->rowcostfile,tempstring,MAXSTRLEN);
  }else{
    StrNCopy(tileoutfiles->rowcostfile,"",MAXSTRLEN);
  }
  if(strlen(outfiles->colcostfile)){
    ParseFilename(outfiles->colcostfile,path,basename);
    sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld",
            tiledir,TMPTILEROOT,basename,tilerow,tilecol,tileparams->ncol);
    StrNCopy(tileoutfiles->colcostfile,tempstring,MAXSTRLEN);
  }else{
    StrNCopy(tileoutfiles->colcostfile,"",MAXSTRLEN);
  }
  if(strlen(outfiles->mstrowcostfile)){
    ParseFilename(outfiles->mstrowcostfile,path,basename);
    sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld",
            tiledir,TMPTILEROOT,basename,tilerow,tilecol,tileparams->ncol);
    StrNCopy(tileoutfiles->mstrowcostfile,tempstring,MAXSTRLEN);
  }else{
    StrNCopy(tileoutfiles->mstrowcostfile,"",MAXSTRLEN);
  }
  if(strlen(outfiles->mstcolcostfile)){
    ParseFilename(outfiles->mstcolcostfile,path,basename);
    sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld",
            tiledir,TMPTILEROOT,basename,tilerow,tilecol,tileparams->ncol);
    StrNCopy(tileoutfiles->mstcolcostfile,tempstring,MAXSTRLEN);
  }else{
    StrNCopy(tileoutfiles->mstcolcostfile,"",MAXSTRLEN);
  }
  if(strlen(outfiles->mstcostsfile)){
    ParseFilename(outfiles->mstcostsfile,path,basename);
    sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld",
            tiledir,TMPTILEROOT,basename,tilerow,tilecol,tileparams->ncol);
    StrNCopy(tileoutfiles->mstcostsfile,tempstring,MAXSTRLEN);
  }else{
    StrNCopy(tileoutfiles->mstcostsfile,"",MAXSTRLEN);
  }
  if(strlen(outfiles->corrdumpfile)){
    ParseFilename(outfiles->corrdumpfile,path,basename);
    sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld",
            tiledir,TMPTILEROOT,basename,tilerow,tilecol,tileparams->ncol);
    StrNCopy(tileoutfiles->corrdumpfile,tempstring,MAXSTRLEN);
  }else{
    StrNCopy(tileoutfiles->corrdumpfile,"",MAXSTRLEN);
  }
  if(strlen(outfiles->rawcorrdumpfile)){
    ParseFilename(outfiles->rawcorrdumpfile,path,basename);
    sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld",
            tiledir,TMPTILEROOT,basename,tilerow,tilecol,tileparams->ncol);
    StrNCopy(tileoutfiles->rawcorrdumpfile,tempstring,MAXSTRLEN);
  }else{
    StrNCopy(tileoutfiles->rawcorrdumpfile,"",MAXSTRLEN);
  }
  if(strlen(outfiles->conncompfile)){
    ParseFilename(outfiles->conncompfile,path,basename);
    sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld",
            tiledir,TMPTILEROOT,basename,tilerow,tilecol,tileparams->ncol);
    StrNCopy(tileoutfiles->conncompfile,tempstring,MAXSTRLEN);
  }else{
    StrNCopy(tileoutfiles->conncompfile,"",MAXSTRLEN);
  }
  if(strlen(outfiles->costoutfile)){
    ParseFilename(outfiles->costoutfile,path,basename);
    sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld",
            tiledir,TMPTILEROOT,basename,tilerow,tilecol,tileparams->ncol);
    StrNCopy(tileoutfiles->costoutfile,tempstring,MAXSTRLEN);
  }else{
    sprintf(tempstring,"%s/%s%s%ld_%ld.%ld",
            tiledir,TMPTILEROOT,TMPTILECOSTSUFFIX,tilerow,tilecol,
            tileparams->ncol);
    StrNCopy(tileoutfiles->costoutfile,tempstring,MAXSTRLEN);
  }
  if(strlen(outfiles->logfile)){
    ParseFilename(outfiles->logfile,path,basename);
    sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld",
            tiledir,TMPTILEROOT,basename,tilerow,tilecol,tileparams->ncol);
    StrNCopy(tileoutfiles->logfile,tempstring,MAXSTRLEN);
  }else{
    StrNCopy(tileoutfiles->logfile,"",MAXSTRLEN);
  }
  tileoutfiles->outfileformat=TMPTILEOUTFORMAT;

  /* done */
  return(0);

}


/* function: SetUpDoTileMask()
 * ---------------------------
 * Read the tile mask if a file name is specified in the infiles structure,
 * otherwise return an array of all ones.
 */
signed char **SetUpDoTileMask(infileT *infiles, long ntilerow, long ntilecol){

  long row, col;
  signed char **dotilemask;
  tileparamT readparams[1];


  /* initialize stack structures to zero for good measure */
  memset(readparams,0,sizeof(tileparamT));

  /* get memory */
  dotilemask=(signed char **)Get2DMem(ntilerow,ntilecol,sizeof(signed char *),
                                      sizeof(signed char));

  /* see if a file name was passed */
  if(strlen(infiles->dotilemaskfile)){

    /* read the input file */
    readparams->nrow=ntilerow;
    readparams->ncol=ntilecol;
    readparams->firstrow=0;
    readparams->firstcol=0;
    Read2DArray((void ***)&dotilemask,infiles->dotilemaskfile,ntilecol,
                ntilerow,readparams,sizeof(signed char *),sizeof(signed char));

  }else{

    /* set array to be all ones */
    for(row=0;row<ntilerow;row++){
      for(col=0;col<ntilecol;col++){
        dotilemask[row][col]=1;
      }
    }
  }

  /* return the array pointer */
  return(dotilemask);

}


/* function: GrowRegions()
 * -----------------------
 * Grows contiguous regions demarcated by arcs whose residual costs are
 * less than some threshold.  Numbers the regions sequentially from 0.
 */
int GrowRegions(void **costs, short **flows, long nrow, long ncol,
                incrcostT **incrcosts, outfileT *outfiles,
                tileparamT *tileparams, paramT *params){

  long i, row, col, maxcol;
  long arcrow, arccol, arcnum, fromdist, arcdist;
  long regioncounter, *regionsizes, regionsizeslen, *thisregionsize;
  long closestregiondist, closestregion, lastfromdist;
  long costthresh, minsize, maxcost;
  long costtypesize;
  short **regions;
  nodeT **nodes;
  nodeT *source, *from, *to, *ground;
  char regionfile[MAXSTRLEN];
  bucketT bkts[1];
  void **growregionscosts;
  tileparamT temptileparams[1];
  void (*tempcalccostfnptr)();
  long (*tempevalcostfnptr)();


  /* initialize stack structures to zero for good measure */
  memset(regionfile,0,MAXSTRLEN);
  memset(bkts,0,sizeof(bucketT));
  memset(temptileparams,0,sizeof(tileparamT));

  /* set up */
  fprintf(sp1,"Growing reliable regions\n");
  minsize=params->minregionsize;
  costthresh=params->tilecostthresh;
  closestregion=0;

  /* store current values of CalcCost and EvalCost function pointers */
  tempcalccostfnptr=CalcCost;
  tempevalcostfnptr=EvalCost;

  /* reread statistical costs from stored file if costs array is for Lp mode */
  if(params->p >= 0){
    if(params->costmode==TOPO){
      costtypesize=sizeof(costT);
      CalcCost=CalcCostTopo;
      EvalCost=EvalCostTopo;
    }else if(params->costmode==DEFO){
      costtypesize=sizeof(costT);
      CalcCost=CalcCostDefo;
      EvalCost=EvalCostDefo;
    }else if(params->costmode==SMOOTH){
      costtypesize=sizeof(smoothcostT);
      CalcCost=CalcCostSmooth;
      EvalCost=EvalCostSmooth;
    }else{
      fflush(NULL);
      fprintf(sp0,"Illegal cost mode in GrowRegions().  This is a bug.\n");
      exit(ABNORMAL_EXIT);
    }
    temptileparams->firstrow=0;
    temptileparams->firstcol=0;
    temptileparams->nrow=nrow;
    temptileparams->ncol=ncol;
    growregionscosts=NULL;
    Read2DRowColFile((void ***)&growregionscosts,outfiles->costoutfile,
                     ncol,nrow,temptileparams,costtypesize);
  }else{
    growregionscosts=costs;
  }

  /* loop over all arcs */
  for(arcrow=0;arcrow<2*nrow-1;arcrow++){
    if(arcrow<nrow-1){
      maxcol=ncol;
    }else{
      maxcol=ncol-1;
    }
    for(arccol=0;arccol<maxcol;arccol++){

      /* compute incremental costs of unit flows in either direction */
      ReCalcCost(growregionscosts,incrcosts,flows[arcrow][arccol],
                 arcrow,arccol,1,nrow,params);

      /* store lesser of incremental costs in first field */
      if(incrcosts[arcrow][arccol].negcost<incrcosts[arcrow][arccol].poscost){
        incrcosts[arcrow][arccol].poscost=incrcosts[arcrow][arccol].negcost;
      }

      /* subtract costthresh and take negative of costs, then clip to zero */
      incrcosts[arcrow][arccol].poscost
        =-(incrcosts[arcrow][arccol].poscost-costthresh);
      if(incrcosts[arcrow][arccol].poscost<0){
        incrcosts[arcrow][arccol].poscost=0;
      }
    }
  }

  /* thicken the costs arrays; results stored in negcost field */
  maxcost=ThickenCosts(incrcosts,nrow,ncol);

  /* initialize nodes and buckets for region growing */
  ground=NULL;
  nodes=(nodeT **)Get2DMem(nrow,ncol,sizeof(nodeT *),sizeof(nodeT));
  InitNodeNums(nrow,ncol,nodes,ground);
  InitNodes(nrow,ncol,nodes,ground);
  bkts->size=maxcost+2;
  bkts->minind=0;
  bkts->maxind=bkts->size-1;
  bkts->curr=0;
  bkts->wrapped=FALSE;
  bkts->bucketbase=(nodeT **)MAlloc(bkts->size*sizeof(nodeT *));
  bkts->bucket=bkts->bucketbase;
  for(i=0;i<bkts->size;i++){
    bkts->bucket[i]=NULL;
  }

  /* initialize region variables */
  regioncounter=-1;
  regionsizeslen=INITARRSIZE;
  regionsizes=(long *)MAlloc(regionsizeslen*sizeof(long));
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      nodes[row][col].incost=-1;
    }
  }

  /* loop over all nodes (pixels) to make sure each is in a group */
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){

      /* see if node is not in a group */
      if(nodes[row][col].incost<0){

        /* clear the buckets */
        ClearBuckets(bkts);

        /* make node source and put it in the first bucket */
        source=&nodes[row][col];
        source->next=NULL;
        source->prev=NULL;
        source->group=INBUCKET;
        source->outcost=0;
        bkts->bucket[0]=source;
        bkts->curr=0;
        lastfromdist=0;

        /* increment the region counter */
        if(++regioncounter>=regionsizeslen){
          regionsizeslen+=INITARRSIZE;
          regionsizes=(long *)ReAlloc(regionsizes,
                                       regionsizeslen*sizeof(long));
        }
        thisregionsize=&regionsizes[regioncounter];

        /* set up */
        (*thisregionsize)=0;
        closestregiondist=VERYFAR;

        /* loop to grow region */
        while(TRUE){

          /* set from node to closest node in circular bucket structure */
          from=ClosestNode(bkts);

          /* break if we can't grow any more and the region is big enough */
          if(from==NULL){
            if(*thisregionsize>=minsize){

              /* no more nonregion nodes, and current region is big enough */
              break;

            }else{

              /* no more nonregion nodes, but current region still too small */
              /* merge with another region */
              MergeRegions(nodes,source,regionsizes,closestregion,nrow,ncol);
              regioncounter--;
              break;

            }
          }else{
            fromdist=from->outcost;
            if(fromdist>lastfromdist){
              if(regionsizes[regioncounter]>=minsize){

                /* region grown to all nodes within mincost, is big enough */
                break;

              }
              if(fromdist>closestregiondist){

                /* another region closer than new node, so merge regions */
                MergeRegions(nodes,source,regionsizes,closestregion,nrow,ncol);
                regioncounter--;
                break;
              }
            }
          }

          /* make from node a part of the current region */
          from->incost=regioncounter;
          (*thisregionsize)++;
          lastfromdist=fromdist;

          /* scan from's neighbors */
          arcnum=0;
          while((to=RegionsNeighborNode(from,&arcnum,nodes,
                                        &arcrow,&arccol,nrow,ncol))!=NULL){

            /* get cost of arc to the to node */
            arcdist=incrcosts[arcrow][arccol].negcost;

            /* see if to node is already in another region */
            if(to->incost>=0){

              /* keep track of which neighboring region is closest */
              if(to->incost!=regioncounter && arcdist<closestregiondist){
                closestregiondist=arcdist;
                closestregion=to->incost;
              }

            }else{

              /* to node is not in another region */
              /* compare distance of new nodes to temp labels */
              if(arcdist<(to->outcost)){

                /* if to node is already in a (circular) bucket, remove it */
                if(to->group==INBUCKET){
                  BucketRemove(to,to->outcost,bkts);
                }

                /* update to node */
                to->outcost=arcdist;
                to->pred=from;

                /* insert to node into appropriate (circular) bucket */
                BucketInsert(to,arcdist,bkts);
                if(arcdist<bkts->curr){
                  bkts->curr=arcdist;
                }
              }
            }
          }
        }
      }
    }
  }
  fprintf(sp2,"Tile partitioned into %ld regions\n",regioncounter+1);

  /* write regions array */
  /* write as shorts if multiple tiles */
  if(params->ntilerow > 1 || params->ntilecol>1){
    regions=(short **)Get2DMem(nrow,ncol,sizeof(short *),sizeof(short));
    for(row=0;row<nrow;row++){
      for(col=0;col<ncol;col++){
        if(nodes[row][col].incost>LARGESHORT){
          fflush(NULL);
          fprintf(sp0,
                  "Number of regions in tile exceeds max allowed\nAbort\n");
          exit(ABNORMAL_EXIT);
        }
        regions[row][col]=nodes[row][col].incost;
      }
    }
    sprintf(regionfile,"%s%s",outfiles->outfile,REGIONSUFFIX);
    fprintf(sp2,"Writing region data to file %s\n",regionfile);
    Write2DArray((void **)regions,regionfile,nrow,ncol,sizeof(short));
  }else{
    regions=NULL;
  }

  /* return CalcCost and EvalCost function pointers to initial values */
  CalcCost=tempcalccostfnptr;
  EvalCost=tempevalcostfnptr;

  /* free memory */
  if(params->p >= 0){
    Free2DArray((void **)growregionscosts,2*nrow-1);
  }
  Free2DArray((void **)nodes,nrow);
  if(regions!=NULL){
    Free2DArray((void **)regions,nrow);
  }
  free(bkts->bucketbase);

  /* done */
  return(0);

}


/* function: GrowConnCompsMask()
 * -----------------------------
 * Grows contiguous regions demarcated by arcs whose residual costs are
 * less than some threshold.  Numbers the regions sequentially from 1.
 * Writes out byte file of connected component mask, with 0 for any pixels
 * not assigned to a component.
 */
int GrowConnCompsMask(void **costs, short **flows, long nrow, long ncol,
                      incrcostT **incrcosts, outfileT *outfiles,
                      paramT *params){

  long i, row, col, maxcol;
  long arcrow, arccol, arcnum;
  long regioncounter, *regionsizes, regionsizeslen, *thisregionsize;
  long *sortedregionsizes;
  long costthresh, minsize, maxncomps, ntied, newnum;
  unsigned long outtypemax, outtypesize;
  nodeT **nodes;
  nodeT *source, *from, *to, *ground;
  unsigned char *ucharbuf;
  unsigned int *uintbuf;
  void *outbufptr;
  bucketT bkts[1];
  char realoutfile[MAXSTRLEN];
  FILE *conncompfp;


  /* initialize stack structures to zero for good measure */
  memset(bkts,0,sizeof(bucketT));

  /* error checking */
  fprintf(sp1,"Growing connected component mask\n");
  minsize=params->minconncompfrac*nrow*ncol;
  maxncomps=params->maxncomps;
  costthresh=params->conncompthresh;
  if(minsize>nrow*ncol){
    fflush(NULL);
    fprintf(sp0,"Minimum region size cannot exceed tile size\nAbort\n");
    exit(ABNORMAL_EXIT);
  }

  /* loop over all arcs */
  for(arcrow=0;arcrow<2*nrow-1;arcrow++){
    if(arcrow<nrow-1){
      maxcol=ncol;
    }else{
      maxcol=ncol-1;
    }
    for(arccol=0;arccol<maxcol;arccol++){

      /* compute incremental costs of unit flows in either direction */
      ReCalcCost(costs,incrcosts,flows[arcrow][arccol],
                 arcrow,arccol,1,nrow,params);

      /* store lesser of incremental costs in first field */
      if(incrcosts[arcrow][arccol].negcost<incrcosts[arcrow][arccol].poscost){
        incrcosts[arcrow][arccol].poscost=incrcosts[arcrow][arccol].negcost;
      }

      /* subtract costthresh and take negative of costs, then clip to zero */
      incrcosts[arcrow][arccol].poscost
        =-(incrcosts[arcrow][arccol].poscost-costthresh);
      if(incrcosts[arcrow][arccol].poscost<0){
        incrcosts[arcrow][arccol].poscost=0;
      }
    }
  }

  /* thicken the costs arrays; results stored in negcost field */
  ThickenCosts(incrcosts,nrow,ncol);

  /* initialize nodes and buckets for region growing */
  ground=NULL;
  nodes=(nodeT **)Get2DMem(nrow,ncol,sizeof(nodeT *),sizeof(nodeT));
  InitNodeNums(nrow,ncol,nodes,ground);
  InitNodes(nrow,ncol,nodes,ground);
  bkts->size=1;
  bkts->minind=0;
  bkts->maxind=0;
  bkts->wrapped=FALSE;
  bkts->bucketbase=(nodeT **)MAlloc(sizeof(nodeT *));
  bkts->bucket=bkts->bucketbase;
  bkts->bucket[0]=NULL;

  /* initialize region variables */
  regioncounter=0;
  regionsizeslen=INITARRSIZE;
  regionsizes=(long *)MAlloc(regionsizeslen*sizeof(long));
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      nodes[row][col].incost=-1;
    }
  }

  /* loop over all nodes (pixels) to make sure each is in a group */
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){

      /* see if node is not in a group */
      if(nodes[row][col].incost<0){

        /* clear the buckets */
        ClearBuckets(bkts);

        /* make node source and put it in the first bucket */
        source=&nodes[row][col];
        source->next=NULL;
        source->prev=NULL;
        source->group=INBUCKET;
        source->outcost=0;
        bkts->bucket[0]=source;
        bkts->curr=0;

        /* increment the region counter */
        if(++regioncounter>=regionsizeslen){
          regionsizeslen+=INITARRSIZE;
          regionsizes=(long *)ReAlloc(regionsizes,
                                       regionsizeslen*sizeof(long));
        }
        thisregionsize=&regionsizes[regioncounter];

        /* set up */
        (*thisregionsize)=0;

        /* loop to grow region */
        while(TRUE){

          /* set from node to closest node in circular bucket structure */
          from=ClosestNode(bkts);

          /* break if we can't grow any more and the region is big enough */
          if(from==NULL){
            if(regionsizes[regioncounter]>=minsize){

              /* no more nonregion nodes, and current region is big enough */
              break;

            }else{

              /* no more nonregion nodes, but current region still too small */
              /* zero out the region */
              RenumberRegion(nodes,source,0,nrow,ncol);
              regioncounter--;
              break;

            }
          }

          /* make from node a part of the current region */
          from->incost=regioncounter;
          (*thisregionsize)++;

          /* scan from's neighbors */
          arcnum=0;
          while((to=RegionsNeighborNode(from,&arcnum,nodes,
                                        &arcrow,&arccol,nrow,ncol))!=NULL){

            /* see if to can be reached */
            if(to->incost<0 && incrcosts[arcrow][arccol].negcost==0
               && to->group!=INBUCKET){

              /* update to node */
              to->pred=from;
              BucketInsert(to,0,bkts);

            }
          }
        }
      }
    }
  }
  fprintf(sp2,"%ld connected components formed\n",regioncounter);

  /* make sure we don't have too many components */
  if(regioncounter>maxncomps){

    /* copy regionsizes array and sort to find new minimum region size */
    fprintf(sp2,"Keeping only %ld connected components\n",maxncomps);
    sortedregionsizes=(long *)MAlloc(regioncounter*sizeof(long));
    for(i=0;i<regioncounter;i++){
      sortedregionsizes[i]=regionsizes[i+1];
    }
    qsort((void *)sortedregionsizes,regioncounter,sizeof(long),LongCompare);
    minsize=sortedregionsizes[regioncounter-maxncomps];

    /* see how many regions of size minsize still need zeroing */
    ntied=0;
    i=regioncounter-maxncomps-1;
    while(i>=0 && sortedregionsizes[i]==minsize){
      ntied++;
      i--;
    }

    /* zero out regions that are too small */
    newnum=-1;
    for(row=0;row<nrow;row++){
      for(col=0;col<ncol;col++){
        i=nodes[row][col].incost;
        if(i>0){
          if(regionsizes[i]<minsize
             || (regionsizes[i]==minsize && (ntied--)>0)){

            /* region too small, so zero it out */
            RenumberRegion(nodes,&(nodes[row][col]),0,nrow,ncol);

          }else{

            /* keep region, assign it new region number */
            /* temporarily assign negative of new number to avoid collisions */
            RenumberRegion(nodes,&(nodes[row][col]),newnum--,nrow,ncol);

          }
        }
      }
    }

    /* flip temporary negative region numbers so they are positive */
    for(row=0;row<nrow;row++){
      for(col=0;col<ncol;col++){
        nodes[row][col].incost=-nodes[row][col].incost;
      }
    }
  }

  /* write connected components as appropriate data type */
  ucharbuf=(unsigned char *)MAlloc(ncol*sizeof(unsigned char));
  uintbuf=(unsigned int *)MAlloc(ncol*sizeof(unsigned int));
  if(params->conncompouttype==CONNCOMPOUTTYPEUCHAR){
    outtypemax=UCHAR_MAX;
    outtypesize=(int )sizeof(unsigned char);
    outbufptr=(void *)ucharbuf;
  }else if(params->conncompouttype==CONNCOMPOUTTYPEUINT){
    outtypemax=UINT_MAX;
    outtypesize=(int )sizeof(unsigned int);
    outbufptr=(void *)uintbuf;
  }else{
    fflush(NULL);
    fprintf(sp0,"Bad conncompouttype in GrowConnCompMask()\n");
    exit(ABNORMAL_EXIT);
  }
  fprintf(sp1,"Writing connected components to file %s"
          " as %d-byte unsigned ints\n",
          outfiles->conncompfile,((int )outtypesize));
  conncompfp=OpenOutputFile(outfiles->conncompfile,realoutfile);
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      if(nodes[row][col].incost>outtypemax){
        fflush(NULL);
        fprintf(sp0,"Number of connected components too large for output type\n"
                "Abort\n");
        exit(ABNORMAL_EXIT);
      }
      uintbuf[col]=(unsigned int)(nodes[row][col].incost);
    }
    if(params->conncompouttype==CONNCOMPOUTTYPEUCHAR){
      for(col=0;col<ncol;col++){
        ucharbuf[col]=(unsigned char )uintbuf[col];
      }
    }
    if(fwrite(outbufptr,outtypesize,ncol,conncompfp)!=ncol){
      fflush(NULL);
      fprintf(sp0,"Error while writing to file %s (device full?)\nAbort\n",
              realoutfile);
      exit(ABNORMAL_EXIT);
    }
  }
  if(fclose(conncompfp)){
    fflush(NULL);
    fprintf(sp0,"WARNING: problem closing file %s (disk full?)\n",
            outfiles->conncompfile);
  }

  /* free memory */
  Free2DArray((void **)nodes,nrow);
  free(bkts->bucketbase);
  free(uintbuf);
  free(ucharbuf);
  return(0);

}


/* function: ThickenCosts()
 * ------------------------
 */
static
long ThickenCosts(incrcostT **incrcosts, long nrow, long ncol){

  long row, col, templong, maxcost;
  double n;


  /* initialize variable storing maximum cost */
  maxcost=-LARGEINT;

  /* loop over row arcs and convolve */
  for(row=0;row<nrow-1;row++){
    for(col=0;col<ncol;col++){
      templong=2*incrcosts[row][col].poscost;
      n=2.0;
      if(col!=0){
        templong+=incrcosts[row][col-1].poscost;
        n+=1.0;
      }
      if(col!=ncol-1){
        templong+=incrcosts[row][col+1].poscost;
        n+=1.0;
      }
      templong=LRound(templong/n);
      if(templong>LARGESHORT){
        fflush(NULL);
        fprintf(sp0,"WARNING: COSTS CLIPPED IN ThickenCosts()\n");
        incrcosts[row][col].negcost=LARGESHORT;
      }else{
        incrcosts[row][col].negcost=templong;
      }
      if(incrcosts[row][col].negcost>maxcost){
        maxcost=incrcosts[row][col].negcost;
      }
    }
  }

  /* loop over column arcs and convolve */
  for(row=nrow-1;row<2*nrow-1;row++){
    for(col=0;col<ncol-1;col++){
      templong=2*incrcosts[row][col].poscost;
      n=2.0;
      if(row!=nrow-1){
        templong+=incrcosts[row-1][col].poscost;
        n+=1.0;
      }
      if(row!=2*nrow-2){
        templong+=incrcosts[row+1][col].poscost;
        n+=1.0;
      }
      templong=LRound(templong/n);
      if(templong>LARGESHORT){
        fflush(NULL);
        fprintf(sp0,"WARNING: COSTS CLIPPED IN ThickenCosts()\n");
        incrcosts[row][col].negcost=LARGESHORT;
      }else{
        incrcosts[row][col].negcost=templong;
      }
      if(incrcosts[row][col].negcost>maxcost){
        maxcost=incrcosts[row][col].negcost;
      }
    }
  }

  /* return maximum cost */
  return(maxcost);

}


/* function: RegionsNeighborNode()
 * -------------------------------
 * Return the neighboring node of the given node corresponding to the
 * given arc number.
 */
static
nodeT *RegionsNeighborNode(nodeT *node1, long *arcnumptr, nodeT **nodes,
                           long *arcrowptr, long *arccolptr,
                           long nrow, long ncol){

  long row, col;

  row=node1->row;
  col=node1->col;

  while(TRUE){
    switch((*arcnumptr)++){
    case 0:
      if(col!=ncol-1){
        *arcrowptr=nrow-1+row;
        *arccolptr=col;
        return(&nodes[row][col+1]);
      }
      break;
    case 1:
      if(row!=nrow-1){
      *arcrowptr=row;
      *arccolptr=col;
        return(&nodes[row+1][col]);
      }
      break;
    case 2:
      if(col!=0){
        *arcrowptr=nrow-1+row;
        *arccolptr=col-1;
        return(&nodes[row][col-1]);
      }
      break;
    case 3:
      if(row!=0){
        *arcrowptr=row-1;
        *arccolptr=col;
        return(&nodes[row-1][col]);
      }
      break;
    default:
      return(NULL);
    }
  }
}


/* function: ClearBuckets()
 * ------------------------
 * Removes any nodes in the bucket data structure passed, and resets
 * their distances to VERYFAR.  Assumes bukets indexed from 0.
 */
static
int ClearBuckets(bucketT *bkts){

  nodeT *currentnode, *nextnode;
  long i;

  /* loop over all buckets */
  for(i=0;i<bkts->size;i++){

    /* clear the bucket */
    nextnode=bkts->bucketbase[i];
    while(nextnode!=NULL){
      currentnode=nextnode;
      nextnode=currentnode->next;
      currentnode->group=NOTINBUCKET;
      currentnode->outcost=VERYFAR;
      currentnode->pred=NULL;
    }
    bkts->bucketbase[i]=NULL;
  }

  /* reset bucket parameters */
  bkts->minind=0;
  bkts->maxind=bkts->size-1;
  bkts->wrapped=FALSE;

  /* done */
  return(0);
}


/* function: MergeRegions()
 * ------------------------
 *
 */
static
int MergeRegions(nodeT **nodes, nodeT *source, long *regionsizes,
                 long closestregion, long nrow, long ncol){

  long nextnodelistlen, nextnodelistnext, arcnum, arcrow, arccol, regionnum;
  nodeT *from, *to, **nextnodelist;


  /* initialize */
  nextnodelistlen=INITARRSIZE;
  nextnodelist=(nodeT **)MAlloc(nextnodelistlen*sizeof(nodeT **));
  nextnodelist[0]=source;
  nextnodelistnext=1;
  regionnum=source->incost;


  /* find all nodes in current region and switch their regions */
  while(nextnodelistnext){
    from=nextnodelist[--nextnodelistnext];
    from->incost=closestregion;
    arcnum=0;
    while((to=RegionsNeighborNode(from,&arcnum,nodes,
                                  &arcrow,&arccol,nrow,ncol))!=NULL){
      if(to->incost==regionnum){
        if(nextnodelistnext>=nextnodelistlen){
          nextnodelistlen+=INITARRSIZE;
          nextnodelist=(nodeT **)ReAlloc(nextnodelist,
                                         nextnodelistlen*sizeof(nodeT *));
        }
        nextnodelist[nextnodelistnext++]=to;
      }
    }
  }

  /* update size of region to which we are merging */
  regionsizes[closestregion]+=regionsizes[regionnum];

  /* free memory */
  free(nextnodelist);
  return(0);

}


/* function: RenumberRegion()
 * --------------------------
 *
 */
static
int RenumberRegion(nodeT **nodes, nodeT *source, long newnum,
                   long nrow, long ncol){

  long nextnodelistlen, nextnodelistnext, arcnum, arcrow, arccol, regionnum;
  nodeT *from, *to, **nextnodelist;


  /* initialize */
  nextnodelistlen=INITARRSIZE;
  nextnodelist=(nodeT **)MAlloc(nextnodelistlen*sizeof(nodeT **));
  nextnodelist[0]=source;
  nextnodelistnext=1;
  regionnum=source->incost;


  /* find all nodes in current region and switch their regions */
  while(nextnodelistnext){
    from=nextnodelist[--nextnodelistnext];
    from->incost=newnum;
    arcnum=0;
    while((to=RegionsNeighborNode(from,&arcnum,nodes,
                                  &arcrow,&arccol,nrow,ncol))!=NULL){
      if(to->incost==regionnum){
        if(nextnodelistnext>=nextnodelistlen){
          nextnodelistlen+=INITARRSIZE;
          nextnodelist=(nodeT **)ReAlloc(nextnodelist,
                                         nextnodelistlen*sizeof(nodeT *));
        }
        nextnodelist[nextnodelistnext++]=to;
      }
    }
  }

  /* free memory */
  free(nextnodelist);
  return(0);

}


/* function: AssembleTiles()
 * -------------------------
 */
int AssembleTiles(outfileT *outfiles, paramT *params,
                  long nlines, long linelen){

  long tilerow, tilecol, ntilerow, ntilecol, ntiles, rowovrlp, colovrlp;
  long i, j, k, ni, nj, dummylong, costtypesize;
  long nrow, ncol, prevnrow, prevncol, nextnrow, nextncol;
  long n, ncycle, nflowdone, nflow, candidatelistsize, candidatebagsize;
  long nnodes, maxnflowcycles, arclen, narcs, sourcetilenum, flowmax;
  long nnondecreasedcostiter;
  long *totarclens;
  long ***scndrycosts;
  double avgarclen;
  float **unwphase, **nextunwphase, **lastunwphase, **tempunwphase;
  float *unwphaseabove, *unwphasebelow;
  void **costs, **nextcosts, **lastcosts, **tempcosts;
  void *costsabove, *costsbelow;
  short **scndryflows, **bulkoffsets, **regions, **nextregions, **lastregions;
  short **tempregions, *regionsbelow, *regionsabove;
  int *nscndrynodes, *nscndryarcs;
  incrcostT **incrcosts;
  totalcostT totalcost, oldtotalcost, mintotalcost;
  nodeT *source;
  nodeT **scndrynodes, ***scndryapexes;
  signed char **iscandidate;
  signed char notfirstloop;
  candidateT *candidatebag, *candidatelist;
  nodesuppT **nodesupp;
  scndryarcT **scndryarcs;
  bucketT *bkts;
  char filename[MAXSTRLEN];


  /* set up */
  fprintf(sp1,"Assembling tiles\n");
  ntilerow=params->ntilerow;
  ntilecol=params->ntilecol;
  ntiles=ntilerow*ntilecol;
  rowovrlp=params->rowovrlp;
  colovrlp=params->colovrlp;
  ni=ceil((nlines+(ntilerow-1)*rowovrlp)/(double )ntilerow);
  nj=ceil((linelen+(ntilecol-1)*colovrlp)/(double )ntilecol);
  nrow=0;
  ncol=0;
  flowmax=params->scndryarcflowmax;
  prevnrow=0;

  /* we are reading statistical costs from file even if (params->p >= 0) */
  /* need to set size of cost type and CalcCost function pointer to */
  /*   be consistent with data stored in file written by BuildCostArrays() */
  if(params->costmode==TOPO){
    costtypesize=sizeof(costT);
    CalcCost=CalcCostTopo;
    EvalCost=EvalCostTopo;
  }else if(params->costmode==DEFO){
    costtypesize=sizeof(costT);
    CalcCost=CalcCostDefo;
    EvalCost=EvalCostDefo;
  }else if(params->costmode==SMOOTH){
    costtypesize=sizeof(smoothcostT);
    CalcCost=CalcCostSmooth;
    EvalCost=EvalCostSmooth;
  }else{
    fflush(NULL);
    fprintf(sp0,"Illegal cost mode in AssembleTiles().  Abort\n");
    exit(ABNORMAL_EXIT);
  }
  /*
  if(CalcCost==CalcCostTopo || CalcCost==CalcCostDefo){
    costtypesize=sizeof(costT);
  }else if(CalcCost==CalcCostSmooth){
    costtypesize=sizeof(smoothcostT);
  }else if(CalcCost==CalcCostL0 || CalcCost==CalcCostL1
           || CalcCost==CalcCostL2 || CalcCost==CalcCostLP){
    costtypesize=sizeof(short);
  }else if(CalcCost==CalcCostL0BiDir || CalcCost==CalcCostL1BiDir
           || CalcCost==CalcCostL2BiDir || CalcCost==CalcCostLPBiDir){
    costtypesize=sizeof(bidircostT);
  }
  */

  /* get memory */
  regions=(short **)Get2DMem(ni,nj,sizeof(short *),sizeof(short));
  nextregions=(short **)Get2DMem(ni,nj,sizeof(short *),sizeof(short));
  lastregions=(short **)Get2DMem(ni,nj,sizeof(short *),sizeof(short));
  regionsbelow=(short *)MAlloc(nj*sizeof(short));
  regionsabove=(short *)MAlloc(nj*sizeof(short));
  unwphase=(float **)Get2DMem(ni,nj,sizeof(float *),sizeof(float));
  nextunwphase=(float **)Get2DMem(ni,nj,sizeof(float *),sizeof(float));
  lastunwphase=(float **)Get2DMem(ni,nj,sizeof(float *),sizeof(float));
  unwphaseabove=(float *)MAlloc(nj*sizeof(float));
  unwphasebelow=(float *)MAlloc(nj*sizeof(float));
  scndrynodes=(nodeT **)MAlloc(ntiles*sizeof(nodeT *));
  nodesupp=(nodesuppT **)MAlloc(ntiles*sizeof(nodesuppT *));
  scndryarcs=(scndryarcT **)MAlloc(ntiles*sizeof(scndryarcT *));
  scndrycosts=(long ***)MAlloc(ntiles*sizeof(long **));
  nscndrynodes=(int *)MAlloc(ntiles*sizeof(int));
  nscndryarcs=(int *)MAlloc(ntiles*sizeof(int));
  totarclens=(long *)MAlloc(ntiles*sizeof(long));
  bulkoffsets=(short **)Get2DMem(ntilerow,ntilecol,sizeof(short *),
                                 sizeof(short));
  costs=(void **)Get2DRowColMem(ni+2,nj+2,sizeof(void *),costtypesize);
  nextcosts=(void **)Get2DRowColMem(ni+2,nj+2,sizeof(void *),costtypesize);
  lastcosts=(void **)Get2DRowColMem(ni+2,nj+2,sizeof(void *),costtypesize);
  costsabove=(void *)MAlloc(nj*costtypesize);
  costsbelow=(void *)MAlloc(nj*costtypesize);


  /* trace regions and parse secondary nodes and arcs for each tile */
  bulkoffsets[0][0]=0;
  for(tilerow=0;tilerow<ntilerow;tilerow++){
    for(tilecol=0;tilecol<ntilecol;tilecol++){

      /* read region, unwrapped phase, and flow data */
      if(tilecol==0){
        ReadNextRegion(tilerow,0,nlines,linelen,outfiles,params,
                       &nextregions,&nextunwphase,&nextcosts,
                       &nextnrow,&nextncol);
        prevnrow=nrow;
        nrow=nextnrow;
      }
      prevncol=ncol;
      ncol=nextncol;
      tempregions=lastregions;
      lastregions=regions;
      regions=nextregions;
      nextregions=tempregions;
      tempunwphase=lastunwphase;
      lastunwphase=unwphase;
      unwphase=nextunwphase;
      nextunwphase=tempunwphase;
      tempcosts=lastcosts;
      lastcosts=costs;
      costs=nextcosts;
      nextcosts=tempcosts;
      if(tilecol!=ntilecol-1){
        ReadNextRegion(tilerow,tilecol+1,nlines,linelen,outfiles,params,
                       &nextregions,&nextunwphase,&nextcosts,
                       &nextnrow,&nextncol);
      }
      ReadEdgesAboveAndBelow(tilerow,tilecol,nlines,linelen,params,
                             outfiles,regionsabove,regionsbelow,
                             unwphaseabove,unwphasebelow,
                             costsabove,costsbelow);

      /* trace region edges to form nodes and arcs */
      TraceRegions(regions,nextregions,lastregions,regionsabove,regionsbelow,
                   unwphase,nextunwphase,lastunwphase,unwphaseabove,
                   unwphasebelow,costs,nextcosts,lastcosts,costsabove,
                   costsbelow,prevnrow,prevncol,tilerow,tilecol,
                   nrow,ncol,scndrynodes,nodesupp,scndryarcs,
                   scndrycosts,nscndrynodes,nscndryarcs,totarclens,
                   bulkoffsets,params);

    }
  }

  /* scale costs based on average number of primary arcs per secondary arc */
  arclen=0;
  narcs=0;
  for(i=0;i<ntiles;i++){
    arclen+=totarclens[i];
    narcs+=nscndryarcs[i];
  }
  avgarclen=arclen/narcs;

  /* may need to adjust scaling so fewer costs clipped */
  for(i=0;i<ntiles;i++){
    for(j=0;j<nscndryarcs[i];j++){
      if(scndrycosts[i][j][2*flowmax+1]!=ZEROCOSTARC){
        for(k=1;k<=2*flowmax;k++){
          scndrycosts[i][j][k]=(long )ceil(scndrycosts[i][j][k]/avgarclen);
        }
        scndrycosts[i][j][2*flowmax+1]=LRound(scndrycosts[i][j][2*flowmax+1]
                                              /avgarclen);
        if(scndrycosts[i][j][2*flowmax+1]<0){
          scndrycosts[i][j][2*flowmax+1]=0;
        }
      }
    }
  }

  /* free some intermediate memory */
  Free2DArray((void **)regions,ni);
  Free2DArray((void **)nextregions,ni);
  Free2DArray((void **)lastregions,ni);
  Free2DArray((void **)unwphase,ni);
  Free2DArray((void **)nextunwphase,ni);
  Free2DArray((void **)lastunwphase,ni);
  Free2DArray((void **)costs,ni);
  Free2DArray((void **)nextcosts,ni);
  Free2DArray((void **)lastcosts,ni);
  free(costsabove);
  free(costsbelow);
  free(unwphaseabove);
  free(unwphasebelow);
  free(regionsabove);
  free(regionsbelow);


  /* get memory for nongrid arrays of secondary network problem */
  scndryflows=(short **)MAlloc(ntiles*sizeof(short *));
  iscandidate=(signed char **)MAlloc(ntiles*sizeof(signed char*));
  scndryapexes=(nodeT ***)MAlloc(ntiles*sizeof(nodeT **));
  incrcosts=(incrcostT **)MAlloc(ntiles*sizeof(incrcostT *));
  nnodes=0;
  for(i=0;i<ntiles;i++){
    scndryflows[i]=(short *)CAlloc(nscndryarcs[i],sizeof(short));
    iscandidate[i]=(signed char *)MAlloc(nscndryarcs[i]*sizeof(signed char));
    scndryapexes[i]=(nodeT **)MAlloc(nscndryarcs[i]*sizeof(nodeT *));
    incrcosts[i]=(incrcostT *)MAlloc(nscndryarcs[i]*sizeof(incrcostT));
    nnodes+=nscndrynodes[i];
  }

  /* set up network for secondary solver */
  InitNetwork(scndryflows,&dummylong,&ncycle,&nflowdone,&dummylong,&nflow,
              &candidatebagsize,&candidatebag,&candidatelistsize,
              &candidatelist,NULL,NULL,&bkts,&dummylong,NULL,NULL,NULL,
              NULL,NULL,NULL,NULL,ntiles,0,&notfirstloop,&totalcost,params);
  oldtotalcost=totalcost;
  mintotalcost=totalcost;
  nnondecreasedcostiter=0;

  /* set pointers to functions for nongrid secondary network */
  CalcCost=CalcCostNonGrid;
  EvalCost=EvalCostNonGrid;
  SetNonGridNetworkFunctionPointers();

  /* solve the secondary network problem */
  /* main loop: loop over flow increments and sources */
  fprintf(sp1,"Running optimizer for secondary network\n");
  fprintf(sp1,"Number of nodes in secondary network: %ld\n",nnodes);
  maxnflowcycles=LRound(nnodes*params->maxcyclefraction);
  while(TRUE){

    fprintf(sp1,"Flow increment: %ld  (Total improvements: %ld)\n",
            nflow,ncycle);

    /* set up the incremental (residual) cost arrays */
    SetupIncrFlowCosts((void **)scndrycosts,incrcosts,scndryflows,nflow,ntiles,
                       ntiles,nscndryarcs,params);

    /* set the tree root (equivalent to source of shortest path problem) */
    sourcetilenum=(long )ntilecol*floor(ntilerow/2.0)+floor(ntilecol/2.0);
    source=&scndrynodes[sourcetilenum][0];

    /* set up network variables for tree solver */
    SetupTreeSolveNetwork(scndrynodes,NULL,scndryapexes,iscandidate,
                          ntiles,nscndrynodes,ntiles,nscndryarcs,
                          ntiles,0);

    /* run the solver, and increment nflowdone if no cycles are found */
    n=TreeSolve(scndrynodes,nodesupp,NULL,source,&candidatelist,&candidatebag,
                &candidatelistsize,&candidatebagsize,bkts,scndryflows,
                (void **)scndrycosts,incrcosts,scndryapexes,iscandidate,0,
                nflow,NULL,NULL,NULL,ntiles,nscndrynodes,ntiles,nscndryarcs,
                ntiles,0,NULL,nnodes,params);

    /* evaluate and save the total cost (skip if first loop through nflow) */
    if(notfirstloop){
      oldtotalcost=totalcost;
      totalcost=EvaluateTotalCost((void **)scndrycosts,scndryflows,ntiles,0,
                                  nscndryarcs,params);
      if(totalcost<mintotalcost){
        mintotalcost=totalcost;
      }
      if(totalcost>oldtotalcost || (n>0 && totalcost==oldtotalcost)){
        fflush(NULL);
        fprintf(sp1,"Caution: Unexpected increase in total cost\n");
      }
      if(totalcost>=mintotalcost){
        nnondecreasedcostiter++;
      }else{
        nnondecreasedcostiter=0;
      }
    }

    /* consider this flow increment done if not too many neg cycles found */
    ncycle+=n;
    if(n<=maxnflowcycles){
      nflowdone++;
    }else{
      nflowdone=1;
    }

    /* break if lack of total cost reduction is sustained */
    if(nnondecreasedcostiter>=2*params->maxflow){
      fflush(NULL);
      fprintf(sp0,"WARNING: No overall cost reduction for too many iterations."
              "  Breaking loop\n");
      break;
    }


    /* break if we're done with all flow increments or problem is convex */
    if(nflowdone>=params->maxflow){
      break;
    }

    /* update flow increment */
    nflow++;
    if(nflow>params->maxflow){
      nflow=1;
      notfirstloop=TRUE;
    }

  } /* end loop until no more neg cycles */

  /* free some memory */
  for(i=0;i<ntiles;i++){
    for(j=0;j<nscndryarcs[i];j++){
      free(scndrycosts[i][j]);
    }
  }
  Free2DArray((void **)scndrycosts,ntiles);
  Free2DArray((void **)scndryapexes,ntiles);
  Free2DArray((void **)iscandidate,ntiles);
  Free2DArray((void **)incrcosts,ntiles);
  free(candidatebag);
  free(candidatelist);
  free(bkts->bucketbase);

  /* assemble connected component files if needed */
  if(strlen(outfiles->conncompfile)){
    AssembleTileConnComps(linelen,nlines,outfiles,params);
  }

  /* integrate phase from secondary network problem */
  IntegrateSecondaryFlows(linelen,nlines,scndrynodes,nodesupp,scndryarcs,
                          nscndryarcs,scndryflows,bulkoffsets,outfiles,params);

  /* free remaining memory */
  for(i=0;i<ntiles;i++){
    for(j=0;j<nscndrynodes[i];j++){
      free(nodesupp[i][j].neighbornodes);
      free(nodesupp[i][j].outarcs);
    }
  }
  Free2DArray((void **)nodesupp,ntiles);
  Free2DArray((void **)scndrynodes,ntiles);
  Free2DArray((void **)scndryarcs,ntiles);
  Free2DArray((void **)scndryflows,ntiles);
  free(nscndrynodes);
  free(nscndryarcs);
  Free2DArray((void **)bulkoffsets,ntilerow);

  /* remove temporary tile log files and tile directory */
  if(params->rmtmptile){
    fflush(NULL);
    fprintf(sp1,"Removing temporary directory %s\n",params->tiledir);
    for(tilerow=0;tilerow<ntilerow;tilerow++){
      for(tilecol=0;tilecol<ntilecol;tilecol++){
        sprintf(filename,"%s/%s%ld_%ld",
                params->tiledir,LOGFILEROOT,tilerow,tilecol);
        unlink(filename);
      }
    }
    rmdir(params->tiledir);
  }

  /* Give notice about increasing overlap if there are edge artifacts */
  if(params->rowovrlp<ni || params->colovrlp<nj){
    fflush(NULL);
    fprintf(sp1,
            "SUGGESTION: Try increasing tile overlap and/or size"
            " if solution has edge artifacts\n");
  }

  /* done */
  return(0);
}


/* function: ReadNextRegion()
 * --------------------------
 */
static
int ReadNextRegion(long tilerow, long tilecol, long nlines, long linelen,
                   outfileT *outfiles, paramT *params,
                   short ***nextregionsptr, float ***nextunwphaseptr,
                   void ***nextcostsptr,
                   long *nextnrowptr, long *nextncolptr){

  long nexttilelinelen, nexttilenlines, costtypesize;
  tileparamT nexttileparams[1];
  outfileT nexttileoutfiles[1];
  char nextfile[MAXSTRLEN], tempstring[MAXTMPSTRLEN];
  char path[MAXSTRLEN], basename[MAXSTRLEN];


  /* initialize stack structures and strings to zero for good measure */
  memset(nexttileparams,0,sizeof(tileparamT));
  memset(nexttileoutfiles,0,sizeof(outfileT));
  memset(nextfile,0,MAXSTRLEN);
  memset(tempstring,0,MAXSTRLEN);
  memset(path,0,MAXSTRLEN);
  memset(basename,0,MAXSTRLEN);

  /* size of the data type for holding cost data depends on cost mode */
  if(CalcCost==CalcCostTopo || CalcCost==CalcCostDefo){
    costtypesize=sizeof(costT);
  }else if(CalcCost==CalcCostSmooth){
    costtypesize=sizeof(smoothcostT);
  }else if(CalcCost==CalcCostL0 || CalcCost==CalcCostL1
           || CalcCost==CalcCostL2 || CalcCost==CalcCostLP){
    costtypesize=sizeof(short);
  }else if(CalcCost==CalcCostL0BiDir || CalcCost==CalcCostL1BiDir
           || CalcCost==CalcCostL2BiDir || CalcCost==CalcCostLPBiDir){
    costtypesize=sizeof(bidircostT);
  }else{
    fflush(NULL);
    fprintf(sp0,"ERROR: Bad CalcCost func ptr in ReadNextRegion()\n");
    exit(ABNORMAL_EXIT);
  }

  /* use SetupTile() to set filenames only; tile params overwritten below */
  SetupTile(nlines,linelen,params,nexttileparams,outfiles,nexttileoutfiles,
            tilerow,tilecol);
  nexttilenlines=nexttileparams->nrow;
  nexttilelinelen=nexttileparams->ncol;

  /* set tile parameters, overwriting values set by SetupTile() above */
  SetTileReadParams(nexttileparams,nexttilenlines,nexttilelinelen,
                    tilerow,tilecol,nlines,linelen,params);

  /* read region data */
  ParseFilename(outfiles->outfile,path,basename);
  sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld%s",
          params->tiledir,TMPTILEROOT,basename,tilerow,tilecol,
          nexttilelinelen,REGIONSUFFIX);
  StrNCopy(nextfile,tempstring,MAXSTRLEN);
  Read2DArray((void ***)nextregionsptr,nextfile,
              nexttilelinelen,nexttilenlines,
              nexttileparams,sizeof(short *),sizeof(short));

  /* read unwrapped phase data */
  if(TMPTILEOUTFORMAT==ALT_LINE_DATA){
    ReadAltLineFilePhase(nextunwphaseptr,nexttileoutfiles->outfile,
                         nexttilelinelen,nexttilenlines,nexttileparams);
  }else if(TMPTILEOUTFORMAT==FLOAT_DATA){
    Read2DArray((void ***)nextunwphaseptr,nexttileoutfiles->outfile,
                nexttilelinelen,nexttilenlines,nexttileparams,
                sizeof(float *),sizeof(float));
  }else{
    fflush(NULL);
    fprintf(sp0,"Cannot read format of unwrapped phase tile data\nAbort\n");
    exit(ABNORMAL_EXIT);
  }

  /* read cost data */
  Read2DRowColFile((void ***)nextcostsptr,nexttileoutfiles->costoutfile,
                   nexttilelinelen,nexttilenlines,nexttileparams,
                   costtypesize);

  /* flip sign of wrapped phase if flip flag is set */
  FlipPhaseArraySign(*nextunwphaseptr,params,
                     nexttileparams->nrow,nexttileparams->ncol);

  /* set outputs */
  (*nextnrowptr)=nexttileparams->nrow;
  (*nextncolptr)=nexttileparams->ncol;
  return(0);

}

/* function: SetTileReadParams()
 * -----------------------------
 * Set parameters for reading the nonoverlapping piece of each tile.
 * ni and nj are the numbers of rows and columns in this particular tile.
 * The meanings of these variables are different for the last row
 * and column.
 */
static
int SetTileReadParams(tileparamT *tileparams, long nexttilenlines,
                      long nexttilelinelen, long tilerow, long tilecol,
                      long nlines, long linelen, paramT *params){

  long rowovrlp, colovrlp;

  /* set temporary variables */
  rowovrlp=params->rowovrlp;
  colovrlp=params->colovrlp;

  /* row parameters */
  if(tilerow==0){
    tileparams->firstrow=0;
  }else{
    tileparams->firstrow=ceil(rowovrlp/2.0);
  }
  if(tilerow!=params->ntilerow-1){
    tileparams->nrow=nexttilenlines-floor(rowovrlp/2.0)-tileparams->firstrow;
  }else{
    tileparams->nrow=nexttilenlines-tileparams->firstrow;
  }

  /* column parameters */
  if(tilecol==0){
    tileparams->firstcol=0;
  }else{
    tileparams->firstcol=ceil(colovrlp/2.0);
  }
  if(tilecol!=params->ntilecol-1){
    tileparams->ncol=nexttilelinelen-floor(colovrlp/2.0)-tileparams->firstcol;
  }else{
    tileparams->ncol=nexttilelinelen-tileparams->firstcol;
  }
  return(0);
}


/* function: ReadEdgesAboveAndBelow()
 * ----------------------------------
 */
static
int ReadEdgesAboveAndBelow(long tilerow, long tilecol, long nlines,
                           long linelen, paramT *params, outfileT *outfiles,
                           short *regionsabove, short *regionsbelow,
                           float *unwphaseabove, float *unwphasebelow,
                           void *costsabove, void *costsbelow){

  long ni, nj, readtilelinelen, readtilenlines, costtypesize;
  long ntilerow, ntilecol, rowovrlp, colovrlp;
  tileparamT tileparams[1];
  outfileT outfilesabove[1], outfilesbelow[1];
  float **unwphaseaboveptr, **unwphasebelowptr;
  void **costsaboveptr, **costsbelowptr;
  short **regionsaboveptr, **regionsbelowptr;
  char tempstring[MAXTMPSTRLEN], readregionfile[MAXSTRLEN];
  char path[MAXSTRLEN], basename[MAXSTRLEN];


  /* initialize stack structures to zero for good measure */
  memset(tileparams,0,sizeof(tileparamT));
  memset(outfilesabove,0,sizeof(outfileT));
  memset(outfilesbelow,0,sizeof(outfileT));
  memset(tempstring,0,MAXSTRLEN);
  memset(readregionfile,0,MAXSTRLEN);
  memset(path,0,MAXSTRLEN);
  memset(basename,0,MAXSTRLEN);

  /* set temporary variables */
  ntilerow=params->ntilerow;
  ntilecol=params->ntilecol;
  rowovrlp=params->rowovrlp;
  colovrlp=params->colovrlp;
  ni=ceil((nlines+(ntilerow-1)*rowovrlp)/(double )ntilerow);
  nj=ceil((linelen+(ntilecol-1)*colovrlp)/(double )ntilecol);

  /* size of the data type for holding cost data depends on cost mode */
  if(CalcCost==CalcCostTopo || CalcCost==CalcCostDefo){
    costtypesize=sizeof(costT);
  }else if(CalcCost==CalcCostSmooth){
    costtypesize=sizeof(smoothcostT);
  }else if(CalcCost==CalcCostL0 || CalcCost==CalcCostL1
           || CalcCost==CalcCostL2 || CalcCost==CalcCostLP){
    costtypesize=sizeof(short);
  }else if(CalcCost==CalcCostL0BiDir || CalcCost==CalcCostL1BiDir
           || CalcCost==CalcCostL2BiDir || CalcCost==CalcCostLPBiDir){
    costtypesize=sizeof(bidircostT);
  }else{
    fflush(NULL);
    fprintf(sp0,"ERROR: Bad CalcCost func ptr in ReadEdgesAboveAndBelow()\n");
    exit(ABNORMAL_EXIT);
  }

  /* set names of files with SetupTile() */
  /* tile parameters set by SetupTile() will be overwritten below */
  if(tilerow!=0){
    SetupTile(nlines,linelen,params,tileparams,outfiles,outfilesabove,
              tilerow-1,tilecol);
  }
  if(tilerow!=ntilerow-1){
    SetupTile(nlines,linelen,params,tileparams,outfiles,outfilesbelow,
              tilerow+1,tilecol);
  }

  /* temporary pointers, so we can use Read2DArray() with 1D output array */
  unwphaseaboveptr=&unwphaseabove;
  unwphasebelowptr=&unwphasebelow;
  costsaboveptr=&costsabove;
  costsbelowptr=&costsbelow;
  regionsaboveptr=&regionsabove;
  regionsbelowptr=&regionsbelow;

  /* set some reading parameters */
  if(tilecol==0){
    tileparams->firstcol=0;
  }else{
    tileparams->firstcol=ceil(colovrlp/2.0);
  }
  if(tilecol!=params->ntilecol-1){
    readtilelinelen=nj;
    tileparams->ncol=readtilelinelen-floor(colovrlp/2.0)-tileparams->firstcol;
  }else{
    readtilelinelen=linelen-(ntilecol-1)*(nj-colovrlp);
    tileparams->ncol=readtilelinelen-tileparams->firstcol;
  }
  tileparams->nrow=1;

  /* read last line of tile above */
  readtilenlines=ni;
  if(tilerow!=0){
    tileparams->firstrow=readtilenlines-floor(rowovrlp/2.0)-1;

    /* read region data */
    ParseFilename(outfiles->outfile,path,basename);
    sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld%s",
            params->tiledir,TMPTILEROOT,basename,tilerow-1,tilecol,
            readtilelinelen,REGIONSUFFIX);
    StrNCopy(readregionfile,tempstring,MAXSTRLEN);
    Read2DArray((void ***)&regionsaboveptr,readregionfile,
                readtilelinelen,readtilenlines,
                tileparams,sizeof(short *),sizeof(short));

    /* read unwrapped phase data */
    if(TMPTILEOUTFORMAT==ALT_LINE_DATA){
      ReadAltLineFilePhase(&unwphaseaboveptr,outfilesabove->outfile,
                           readtilelinelen,readtilenlines,tileparams);
    }else if(TMPTILEOUTFORMAT==FLOAT_DATA){
      Read2DArray((void ***)&unwphaseaboveptr,outfilesabove->outfile,
                  readtilelinelen,readtilenlines,tileparams,
                  sizeof(float *),sizeof(float));
    }

    /* flip sign of wrapped phase if flip flag is set */
    FlipPhaseArraySign(unwphaseaboveptr,params,
                       tileparams->nrow,tileparams->ncol);

    /* read costs data */
    tileparams->firstrow--;
    Read2DRowColFileRows((void ***)&costsaboveptr,outfilesabove->costoutfile,
                         readtilelinelen,readtilenlines,tileparams,
                         costtypesize);

    /* remove temporary tile cost file unless told to save it */
    if(params->rmtmptile && !strlen(outfiles->costoutfile)){
      unlink(outfilesabove->costoutfile);
    }
  }

  /* read first line of tile below */
  if(tilerow!=ntilerow-1){
    if(tilerow==params->ntilerow-2){
      readtilenlines=nlines-(ntilerow-1)*(ni-rowovrlp);
    }
    tileparams->firstrow=ceil(rowovrlp/2.0);

    /* read region data */
    ParseFilename(outfiles->outfile,path,basename);
    sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld%s",
            params->tiledir,TMPTILEROOT,basename,tilerow+1,tilecol,
            readtilelinelen,REGIONSUFFIX);
    StrNCopy(readregionfile,tempstring,MAXSTRLEN);
    Read2DArray((void ***)&regionsbelowptr,readregionfile,
                readtilelinelen,readtilenlines,
                tileparams,sizeof(short *),sizeof(short));

    /* read unwrapped phase data */
    if(TMPTILEOUTFORMAT==ALT_LINE_DATA){
      ReadAltLineFilePhase(&unwphasebelowptr,outfilesbelow->outfile,
                           readtilelinelen,readtilenlines,tileparams);
    }else if(TMPTILEOUTFORMAT==FLOAT_DATA){
      Read2DArray((void ***)&unwphasebelowptr,outfilesbelow->outfile,
                  readtilelinelen,readtilenlines,tileparams,
                  sizeof(float *),sizeof(float));
    }

    /* flip the sign of the wrapped phase if flip flag is set */
    FlipPhaseArraySign(unwphasebelowptr,params,
                       tileparams->nrow,tileparams->ncol);

    /* read costs data */
    Read2DRowColFileRows((void ***)&costsbelowptr,outfilesbelow->costoutfile,
                         readtilelinelen,readtilenlines,tileparams,
                         costtypesize);

  }else{

    /* remove temporoary tile cost file for last row unless told to save it */
    if(params->rmtmptile && !strlen(outfiles->costoutfile)){
      SetupTile(nlines,linelen,params,tileparams,outfiles,outfilesbelow,
                tilerow,tilecol);
      unlink(outfilesbelow->costoutfile);
    }
  }

  /* done */
  return(0);

}


/* function: TraceRegions()
 * ------------------------
 * Trace edges of region data to form nodes and arcs of secondary
 * (ie, region-level) network problem.  Primary nodes and arcs are
 * those of the original, pixel-level network problem.  Flows along
 * edges are computed knowing the unwrapped phase values of edges
 * of adjacent tiles.  Costs along edges are approximated in that they
 * are calculated from combining adjacent cost parameters, not from
 * using the exact method in BuildCostArrays().
 */
static
int TraceRegions(short **regions, short **nextregions, short **lastregions,
                 short *regionsabove, short *regionsbelow, float **unwphase,
                 float **nextunwphase, float **lastunwphase,
                 float *unwphaseabove, float *unwphasebelow, void **costs,
                 void **nextcosts, void **lastcosts, void *costsabove,
                 void *costsbelow, long prevnrow, long prevncol, long tilerow,
                 long tilecol, long nrow, long ncol, nodeT **scndrynodes,
                 nodesuppT **nodesupp, scndryarcT **scndryarcs,
                 long ***scndrycosts, int *nscndrynodes,
                 int *nscndryarcs, long *totarclens, short **bulkoffsets,
                 paramT *params){

  long i, j, row, col, nnrow, nncol, tilenum, costtypesize;
  long nnewnodes, nnewarcs, npathsout, flowmax, totarclen;
  long nupdatednontilenodes, updatednontilenodesize, ntilecol;
  short **flows;
  short **rightedgeflows, **loweredgeflows, **leftedgeflows, **upperedgeflows;
  short *inontilenodeoutarc;
  void **rightedgecosts, **loweredgecosts, **leftedgecosts, **upperedgecosts;
  nodeT **primarynodes, **updatednontilenodes;
  nodeT *from, *to, *nextnode, *tempnode;
  nodesuppT *fromsupp, *tosupp;


  /* initialize */
  ntilecol=params->ntilecol;
  nnrow=nrow+1;
  nncol=ncol+1;
  primarynodes=(nodeT **)Get2DMem(nnrow,nncol,sizeof(nodeT *),sizeof(nodeT));
  for(row=0;row<nnrow;row++){
    for(col=0;col<nncol;col++){
      primarynodes[row][col].row=row;
      primarynodes[row][col].col=col;
      primarynodes[row][col].group=NOTINBUCKET;
      primarynodes[row][col].pred=NULL;
      primarynodes[row][col].next=NULL;
    }
  }
  nextnode=&primarynodes[0][0];
  tilenum=tilerow*ntilecol+tilecol;
  scndrynodes[tilenum]=NULL;
  nodesupp[tilenum]=NULL;
  scndryarcs[tilenum]=NULL;
  scndrycosts[tilenum]=NULL;
  nnewnodes=0;
  nnewarcs=0;
  totarclen=0;
  flowmax=params->scndryarcflowmax;
  updatednontilenodesize=INITARRSIZE;
  nupdatednontilenodes=0;

  /* size of the data type for holding cost data depends on cost mode */
  if(CalcCost==CalcCostTopo || CalcCost==CalcCostDefo){
    costtypesize=sizeof(costT);
  }else if(CalcCost==CalcCostSmooth){
    costtypesize=sizeof(smoothcostT);
  }else if(CalcCost==CalcCostL0 || CalcCost==CalcCostL1
           || CalcCost==CalcCostL2 || CalcCost==CalcCostLP){
    costtypesize=sizeof(short);
  }else if(CalcCost==CalcCostL0BiDir || CalcCost==CalcCostL1BiDir
           || CalcCost==CalcCostL2BiDir || CalcCost==CalcCostLPBiDir){
    costtypesize=sizeof(bidircostT);
  }else{
    fflush(NULL);
    fprintf(sp0,"ERROR: Bad CalcCost func ptr in TraceRegions()\n");
    exit(ABNORMAL_EXIT);
  }

  /* get memory */
  updatednontilenodes=(nodeT **)MAlloc(updatednontilenodesize*sizeof(nodeT *));
  inontilenodeoutarc=(short *)MAlloc(updatednontilenodesize*sizeof(short));
  flows=(short **)Get2DRowColMem(nrow+1,ncol+1,sizeof(short *),sizeof(short));
  rightedgeflows=(short **)Get2DMem(nrow,1,sizeof(short *),sizeof(short));
  leftedgeflows=(short **)Get2DMem(nrow,1,sizeof(short *),sizeof(short));
  upperedgeflows=(short **)Get2DMem(1,ncol,sizeof(short *),sizeof(short));
  loweredgeflows=(short **)Get2DMem(1,ncol,sizeof(short *),sizeof(short));
  rightedgecosts=(void **)Get2DMem(nrow,1,sizeof(void *),costtypesize);
  leftedgecosts=(void **)Get2DMem(nrow,1,sizeof(void *),costtypesize);
  upperedgecosts=(void **)Get2DMem(1,ncol,sizeof(void *),costtypesize);
  loweredgecosts=(void **)Get2DMem(1,ncol,sizeof(void *),costtypesize);

  /* parse flows for this tile */
  CalcFlow(unwphase,&flows,nrow,ncol);

  /* set up cost and flow arrays for boundaries */
  SetUpperEdge(ncol,tilerow,tilecol,costs,costsabove,unwphase,unwphaseabove,
               upperedgecosts,upperedgeflows,params, bulkoffsets);
  SetLowerEdge(nrow,ncol,tilerow,tilecol,costs,costsbelow,unwphase,
               unwphasebelow,loweredgecosts,loweredgeflows,
               params,bulkoffsets);
  SetLeftEdge(nrow,prevncol,tilerow,tilecol,costs,lastcosts,unwphase,
              lastunwphase,leftedgecosts,leftedgeflows,params, bulkoffsets);
  SetRightEdge(nrow,ncol,tilerow,tilecol,costs,nextcosts,unwphase,
               nextunwphase,rightedgecosts,rightedgeflows,
               params,bulkoffsets);

  /* trace edges between regions */
  while(nextnode!=NULL){

    /* get next primary node from stack */
    from=nextnode;
    nextnode=nextnode->next;
    from->group=NOTINBUCKET;

    /* find number of paths out of from node */
    npathsout=FindNumPathsOut(from,params,tilerow,tilecol,nnrow,nncol,regions,
                              nextregions,lastregions,regionsabove,
                              regionsbelow,prevncol);

    /* secondary node exists if region edges fork */
    if(npathsout>2){

      /* mark primary node to indicate that secondary node exists for it */
      from->group=ONTREE;

      /* create secondary node if not already created in another tile */
      if((from->row!=0 || tilerow==0) && (from->col!=0 || tilecol==0)){

        /* create the secondary node */
        nnewnodes++;
        if(nnewnodes > SHRT_MAX){
          fflush(NULL);
          fprintf(sp0,"Exceeded maximum number of secondary nodes\n"
                  "Decrease TILECOSTTHRESH and/or increase MINREGIONSIZE\n");
          exit(ABNORMAL_EXIT);
        }
        scndrynodes[tilenum]=(nodeT *)ReAlloc(scndrynodes[tilenum],
                                              nnewnodes*sizeof(nodeT));
        nodesupp[tilenum]=(nodesuppT *)ReAlloc(nodesupp[tilenum],
                                               nnewnodes*sizeof(nodesuppT));
        scndrynodes[tilenum][nnewnodes-1].row=tilenum;
        scndrynodes[tilenum][nnewnodes-1].col=nnewnodes-1;
        nodesupp[tilenum][nnewnodes-1].row=from->row;
        nodesupp[tilenum][nnewnodes-1].col=from->col;
        nodesupp[tilenum][nnewnodes-1].noutarcs=0;
        nodesupp[tilenum][nnewnodes-1].neighbornodes=NULL;
        nodesupp[tilenum][nnewnodes-1].outarcs=NULL;
      }

      /* create the secondary arc to this node if it doesn't already exist */
      if(from->pred!=NULL
         && ((from->row==from->pred->row && (from->row!=0 || tilerow==0))
             || (from->col==from->pred->col && (from->col!=0 || tilecol==0)))){

        TraceSecondaryArc(from,scndrynodes,nodesupp,scndryarcs,scndrycosts,
                          &nnewnodes,&nnewarcs,tilerow,tilecol,flowmax,
                          nrow,ncol,prevnrow,prevncol,params,costs,
                          rightedgecosts,loweredgecosts,leftedgecosts,
                          upperedgecosts,flows,rightedgeflows,loweredgeflows,
                          leftedgeflows,upperedgeflows,&updatednontilenodes,
                          &nupdatednontilenodes,&updatednontilenodesize,
                          &inontilenodeoutarc,&totarclen);
      }
    }

    /* scan neighboring primary nodes and place path candidates into stack */
    RegionTraceCheckNeighbors(from,&nextnode,primarynodes,regions,
                              nextregions,lastregions,regionsabove,
                              regionsbelow,tilerow,tilecol,nnrow,nncol,
                              scndrynodes,nodesupp,scndryarcs,&nnewnodes,
                              &nnewarcs,flowmax,nrow,ncol,prevnrow,prevncol,
                              params,costs,rightedgecosts,loweredgecosts,
                              leftedgecosts,upperedgecosts,flows,
                              rightedgeflows,loweredgeflows,leftedgeflows,
                              upperedgeflows,scndrycosts,&updatednontilenodes,
                              &nupdatednontilenodes,&updatednontilenodesize,
                              &inontilenodeoutarc,&totarclen);
  }


  /* reset temporary secondary node and arc pointers in data structures */
  /* secondary node row, col stored level, incost of primary node pointed to */

  /* update nodes in this tile */
  for(i=0;i<nnewnodes;i++){
    for(j=0;j<nodesupp[tilenum][i].noutarcs;j++){
      tempnode=nodesupp[tilenum][i].neighbornodes[j];
      nodesupp[tilenum][i].neighbornodes[j]
        =&scndrynodes[tempnode->level][tempnode->incost];
    }
  }

  /* update nodes not in this tile that were affected (that have new arcs) */
  for(i=0;i<nupdatednontilenodes;i++){
    row=updatednontilenodes[i]->row;
    col=updatednontilenodes[i]->col;
    j=inontilenodeoutarc[i];
    tempnode=nodesupp[row][col].neighbornodes[j];
    nodesupp[row][col].neighbornodes[j]
      =&scndrynodes[tempnode->level][tempnode->incost];
  }

  /* update secondary arcs */
  for(i=0;i<nnewarcs;i++){

    /* update node pointers in secondary arc structure */
    tempnode=scndryarcs[tilenum][i].from;
    scndryarcs[tilenum][i].from
      =&scndrynodes[tempnode->level][tempnode->incost];
    from=scndryarcs[tilenum][i].from;
    tempnode=scndryarcs[tilenum][i].to;
    scndryarcs[tilenum][i].to
      =&scndrynodes[tempnode->level][tempnode->incost];
    to=scndryarcs[tilenum][i].to;

    /* update secondary arc pointers in nodesupp strcutres */
    fromsupp=&nodesupp[from->row][from->col];
    j=0;
    while(fromsupp->neighbornodes[j]!=to){
      j++;
    }
    fromsupp->outarcs[j]=&scndryarcs[tilenum][i];
    tosupp=&nodesupp[to->row][to->col];
    j=0;
    while(tosupp->neighbornodes[j]!=from){
      j++;
    }
    tosupp->outarcs[j]=&scndryarcs[tilenum][i];
  }

  /* set outputs */
  nscndrynodes[tilenum]=nnewnodes;
  nscndryarcs[tilenum]=nnewarcs;
  totarclens[tilenum]=totarclen;

  /* free memory */
  Free2DArray((void **)primarynodes,nnrow);
  Free2DArray((void **)flows,2*nrow-1);
  Free2DArray((void **)rightedgeflows,nrow);
  Free2DArray((void **)leftedgeflows,nrow);
  Free2DArray((void **)upperedgeflows,1);
  Free2DArray((void **)loweredgeflows,1);
  Free2DArray((void **)rightedgecosts,nrow);
  Free2DArray((void **)leftedgecosts,nrow);
  Free2DArray((void **)upperedgecosts,1);
  Free2DArray((void **)loweredgecosts,1);
  return(0);
}


/* function: FindNumPathsOut()
 * ---------------------------
 * Check all outgoing arcs to see how many paths out there are.
 */
static
long FindNumPathsOut(nodeT *from, paramT *params, long tilerow, long tilecol,
                     long nnrow, long nncol, short **regions,
                     short **nextregions, short **lastregions,
                     short *regionsabove, short *regionsbelow, long prevncol){

  long npathsout, ntilerow, ntilecol, fromrow, fromcol;

  /* initialize */
  ntilerow=params->ntilerow;
  ntilecol=params->ntilecol;
  fromrow=from->row;
  fromcol=from->col;
  npathsout=0;

  /* rightward arc */
  if(fromcol!=nncol-1){
    if(fromrow==0 || fromrow==nnrow-1
       || regions[fromrow-1][fromcol]!=regions[fromrow][fromcol]){
      npathsout++;
    }
  }else{
    if(fromrow==0 || fromrow==nnrow-1 ||
       (tilecol!=ntilecol-1
        && nextregions[fromrow-1][0]!=nextregions[fromrow][0])){
      npathsout++;
    }
  }

  /* downward arc */
  if(fromrow!=nnrow-1){
    if(fromcol==0 || fromcol==nncol-1
       || regions[fromrow][fromcol]!=regions[fromrow][fromcol-1]){
      npathsout++;
    }
  }else{
    if(fromcol==0 || fromcol==nncol-1 ||
       (tilerow!=ntilerow-1
        && regionsbelow[fromcol]!=regionsbelow[fromcol-1])){
      npathsout++;
    }
  }

  /* leftward arc */
  if(fromcol!=0){
    if(fromrow==0 || fromrow==nnrow-1
       || regions[fromrow][fromcol-1]!=regions[fromrow-1][fromcol-1]){
      npathsout++;
    }
  }else{
    if(fromrow==0 || fromrow==nnrow-1 ||
       (tilecol!=0
        && (lastregions[fromrow][prevncol-1]
            !=lastregions[fromrow-1][prevncol-1]))){
      npathsout++;
    }
  }

  /* upward arc */
  if(fromrow!=0){
    if(fromcol==0 || fromcol==nncol-1
       || regions[fromrow-1][fromcol-1]!=regions[fromrow-1][fromcol]){
      npathsout++;
    }
  }else{
    if(fromcol==0 || fromcol==nncol-1 ||
       (tilerow!=0
        && regionsabove[fromcol-1]!=regionsabove[fromcol])){
      npathsout++;
    }
  }

  /* return number of paths out of node */
  return(npathsout);

}


/* function: RegionTraceCheckNeighbors()
 * -------------------------------------
 */
static
int RegionTraceCheckNeighbors(nodeT *from, nodeT **nextnodeptr,
                              nodeT **primarynodes, short **regions,
                              short **nextregions, short **lastregions,
                              short *regionsabove, short *regionsbelow,
                              long tilerow, long tilecol, long nnrow,
                              long nncol, nodeT **scndrynodes,
                              nodesuppT **nodesupp, scndryarcT **scndryarcs,
                              long *nnewnodesptr, long *nnewarcsptr,
                              long flowmax, long nrow, long ncol,
                              long prevnrow, long prevncol, paramT *params,
                              void **costs, void **rightedgecosts,
                              void **loweredgecosts, void **leftedgecosts,
                              void **upperedgecosts, short **flows,
                              short **rightedgeflows, short **loweredgeflows,
                              short **leftedgeflows, short **upperedgeflows,
                              long ***scndrycosts,
                              nodeT ***updatednontilenodesptr,
                              long *nupdatednontilenodesptr,
                              long *updatednontilenodesizeptr,
                              short **inontilenodeoutarcptr,
                              long *totarclenptr){

  long fromrow, fromcol;
  nodeT *to, *nextnode;


  /* initialize */
  fromrow=from->row;
  fromcol=from->col;
  nextnode=(*nextnodeptr);


  /* check rightward arc */
  if(fromcol!=nncol-1){
    to=&primarynodes[fromrow][fromcol+1];
    if(fromrow==0 || fromrow==nnrow-1
       || regions[fromrow-1][fromcol]!=regions[fromrow][fromcol]){
      if(to!=from->pred){
        to->pred=from;
        if(to->group==NOTINBUCKET){
          to->group=INBUCKET;
          to->next=nextnode;
          nextnode=to;
        }else if(to->group==ONTREE && (fromrow!=0 || tilerow==0)){
          TraceSecondaryArc(to,scndrynodes,nodesupp,scndryarcs,scndrycosts,
                            nnewnodesptr,nnewarcsptr,tilerow,tilecol,flowmax,
                            nrow,ncol,prevnrow,prevncol,params,costs,
                            rightedgecosts,loweredgecosts,leftedgecosts,
                            upperedgecosts,flows,rightedgeflows,
                            loweredgeflows,leftedgeflows,upperedgeflows,
                            updatednontilenodesptr,nupdatednontilenodesptr,
                            updatednontilenodesizeptr,inontilenodeoutarcptr,
                            totarclenptr);
        }
      }
    }
  }


  /* check downward arc */
  if(fromrow!=nnrow-1){
    to=&primarynodes[fromrow+1][fromcol];
    if(fromcol==0 || fromcol==nncol-1
       || regions[fromrow][fromcol]!=regions[fromrow][fromcol-1]){
      if(to!=from->pred){
        to->pred=from;
        if(to->group==NOTINBUCKET){
          to->group=INBUCKET;
          to->next=nextnode;
          nextnode=to;
        }else if(to->group==ONTREE && (fromcol!=0 || tilecol==0)){
          TraceSecondaryArc(to,scndrynodes,nodesupp,scndryarcs,scndrycosts,
                            nnewnodesptr,nnewarcsptr,tilerow,tilecol,flowmax,
                            nrow,ncol,prevnrow,prevncol,params,costs,
                            rightedgecosts,loweredgecosts,leftedgecosts,
                            upperedgecosts,flows,rightedgeflows,
                            loweredgeflows,leftedgeflows,upperedgeflows,
                            updatednontilenodesptr,nupdatednontilenodesptr,
                            updatednontilenodesizeptr,inontilenodeoutarcptr,
                            totarclenptr);
        }
      }
    }
  }


  /* check leftward arc */
  if(fromcol!=0){
    to=&primarynodes[fromrow][fromcol-1];
    if(fromrow==0 || fromrow==nnrow-1
       || regions[fromrow][fromcol-1]!=regions[fromrow-1][fromcol-1]){
      if(to!=from->pred){
        to->pred=from;
        if(to->group==NOTINBUCKET){
          to->group=INBUCKET;
          to->next=nextnode;
          nextnode=to;
        }else if(to->group==ONTREE && (fromrow!=0 || tilerow==0)){
          TraceSecondaryArc(to,scndrynodes,nodesupp,scndryarcs,scndrycosts,
                            nnewnodesptr,nnewarcsptr,tilerow,tilecol,flowmax,
                            nrow,ncol,prevnrow,prevncol,params,costs,
                            rightedgecosts,loweredgecosts,leftedgecosts,
                            upperedgecosts,flows,rightedgeflows,
                            loweredgeflows,leftedgeflows,upperedgeflows,
                            updatednontilenodesptr,nupdatednontilenodesptr,
                            updatednontilenodesizeptr,inontilenodeoutarcptr,
                            totarclenptr);
        }
      }
    }
  }


  /* check upward arc */
  if(fromrow!=0){
    to=&primarynodes[fromrow-1][fromcol];
    if(fromcol==0 || fromcol==nncol-1
       || regions[fromrow-1][fromcol-1]!=regions[fromrow-1][fromcol]){
      if(to!=from->pred){
        to->pred=from;
        if(to->group==NOTINBUCKET){
          to->group=INBUCKET;
          to->next=nextnode;
          nextnode=to;
        }else if(to->group==ONTREE && (fromcol!=0 || tilecol==0)){
          TraceSecondaryArc(to,scndrynodes,nodesupp,scndryarcs,scndrycosts,
                            nnewnodesptr,nnewarcsptr,tilerow,tilecol,flowmax,
                            nrow,ncol,prevnrow,prevncol,params,costs,
                            rightedgecosts,loweredgecosts,leftedgecosts,
                            upperedgecosts,flows,rightedgeflows,
                            loweredgeflows,leftedgeflows,upperedgeflows,
                            updatednontilenodesptr,nupdatednontilenodesptr,
                            updatednontilenodesizeptr,inontilenodeoutarcptr,
                            totarclenptr);
        }
      }
    }
  }


  /* set return values */
  *nextnodeptr=nextnode;
  return(0);
}


/* function: SetUpperEdge()
 * ------------------------
 */
static
int SetUpperEdge(long ncol, long tilerow, long tilecol, void **voidcosts,
                 void *voidcostsabove, float **unwphase,
                 float *unwphaseabove, void **voidupperedgecosts,
                 short **upperedgeflows, paramT *params, short **bulkoffsets){

  long col, reloffset;
  double dphi, dpsi;
  costT **upperedgecosts, **costs, *costsabove;
  smoothcostT **upperedgesmoothcosts, **smoothcosts, *smoothcostsabove;
  long nshortcycle;


  /* typecast generic pointers to costT pointers */
  upperedgecosts=(costT **)voidupperedgecosts;
  costs=(costT **)voidcosts;
  costsabove=(costT *)voidcostsabove;
  upperedgesmoothcosts=(smoothcostT **)voidupperedgecosts;
  smoothcosts=(smoothcostT **)voidcosts;
  smoothcostsabove=(smoothcostT *)voidcostsabove;

  /* see if tile is in top row */
  if(tilerow!=0){

    /* set up */
    nshortcycle=params->nshortcycle;
    reloffset=bulkoffsets[tilerow-1][tilecol]-bulkoffsets[tilerow][tilecol];

    /* loop over all arcs on the boundary */
    for(col=0;col<ncol;col++){
      dphi=(unwphaseabove[col]-unwphase[0][col])/TWOPI;
      upperedgeflows[0][col]=(short )LRound(dphi)-reloffset;
      dpsi=dphi-floor(dphi);
      if(dpsi>0.5){
        dpsi-=1.0;
      }
      if(CalcCost==CalcCostTopo || CalcCost==CalcCostDefo){
        upperedgecosts[0][col].offset=(short )LRound(nshortcycle*dpsi);
        upperedgecosts[0][col].sigsq=AvgSigSq(costs[0][col].sigsq,
                                              costsabove[col].sigsq);
        if(costs[0][col].dzmax>costsabove[col].dzmax){
          upperedgecosts[0][col].dzmax=costs[0][col].dzmax;
        }else{
          upperedgecosts[0][col].dzmax=costsabove[col].dzmax;
        }
        if(costs[0][col].laycost<costsabove[col].laycost){
          upperedgecosts[0][col].laycost=costs[0][col].laycost;
        }else{
          upperedgecosts[0][col].laycost=costsabove[col].laycost;
        }
      }else if(CalcCost==CalcCostSmooth){
        upperedgesmoothcosts[0][col].offset=(short )LRound(nshortcycle*dpsi);
        upperedgesmoothcosts[0][col].sigsq
          =AvgSigSq(smoothcosts[0][col].sigsq,smoothcostsabove[col].sigsq);
      }else if(CalcCost==CalcCostL0 || CalcCost==CalcCostL1
               || CalcCost==CalcCostL2 || CalcCost==CalcCostLP){
        ((short **)voidupperedgecosts)[0][col]=
          (((short **)voidcosts)[0][col]+((short *)voidcostsabove)[col])/2;
      }else if(CalcCost==CalcCostL0BiDir || CalcCost==CalcCostL1BiDir
               || CalcCost==CalcCostL2BiDir || CalcCost==CalcCostLPBiDir){
        ((bidircostT **)voidupperedgecosts)[0][col].posweight=
          (((bidircostT **)voidcosts)[0][col].posweight
           +((bidircostT *)voidcostsabove)[col].posweight)/2;
        ((bidircostT **)voidupperedgecosts)[0][col].negweight=
          (((bidircostT **)voidcosts)[0][col].negweight
           +((bidircostT *)voidcostsabove)[col].negweight)/2;
      }else{
        fflush(NULL);
        fprintf(sp0,"Illegal cost mode in SetUpperEdge().  This is a bug.\n");
        exit(ABNORMAL_EXIT);
      }
    }
  }else{
    if(CalcCost==CalcCostTopo || CalcCost==CalcCostDefo){
      for(col=0;col<ncol;col++){
        upperedgecosts[0][col].offset=LARGESHORT/2;
        upperedgecosts[0][col].sigsq=LARGESHORT;
        upperedgecosts[0][col].dzmax=LARGESHORT;
        upperedgecosts[0][col].laycost=0;
      }
    }else if(CalcCost==CalcCostSmooth){
      for(col=0;col<ncol;col++){
        upperedgesmoothcosts[0][col].offset=0;
        upperedgesmoothcosts[0][col].sigsq=LARGESHORT;
      }
    }else if(CalcCost==CalcCostL0 || CalcCost==CalcCostL1
             || CalcCost==CalcCostL2 || CalcCost==CalcCostLP){
      for(col=0;col<ncol;col++){
        ((short **)voidupperedgecosts)[0][col]=0;
      }
    }else if(CalcCost==CalcCostL0BiDir || CalcCost==CalcCostL1BiDir
             || CalcCost==CalcCostL2BiDir || CalcCost==CalcCostLPBiDir){
      for(col=0;col<ncol;col++){
        ((bidircostT **)voidupperedgecosts)[0][col].posweight=0;
        ((bidircostT **)voidupperedgecosts)[0][col].negweight=0;
      }
    }else{
      fflush(NULL);
      fprintf(sp0,"Illegal cost mode in SetUpperEdge().  This is a bug.\n");
      exit(ABNORMAL_EXIT);
    }
  }

  /* done */
  return(0);

}


/* function: SetLowerEdge()
 * ------------------------
 */
static
int SetLowerEdge(long nrow, long ncol, long tilerow, long tilecol,
                 void **voidcosts, void *voidcostsbelow,
                 float **unwphase, float *unwphasebelow,
                 void **voidloweredgecosts, short **loweredgeflows,
                 paramT *params, short **bulkoffsets){

  long *flowhistogram;
  long col, iflow, reloffset, nmax;
  long flowlimhi, flowlimlo, maxflow, minflow, tempflow;
  double dphi, dpsi;
  costT **loweredgecosts, **costs, *costsbelow;
  smoothcostT **loweredgesmoothcosts, **smoothcosts, *smoothcostsbelow;
  long nshortcycle;

  /* typecast generic pointers to costT pointers */
  loweredgecosts=(costT **)voidloweredgecosts;
  costs=(costT **)voidcosts;
  costsbelow=(costT *)voidcostsbelow;
  loweredgesmoothcosts=(smoothcostT **)voidloweredgecosts;
  smoothcosts=(smoothcostT **)voidcosts;
  smoothcostsbelow=(smoothcostT *)voidcostsbelow;

  /* see if tile is in bottom row */
  if(tilerow!=params->ntilerow-1){

    /* set up */
    nshortcycle=params->nshortcycle;
    flowlimhi=LARGESHORT;
    flowlimlo=-LARGESHORT;
    flowhistogram=(long *)CAlloc(flowlimhi-flowlimlo+1,sizeof(long));
    minflow=flowlimhi;
    maxflow=flowlimlo;

    /* loop over all arcs on the boundary */
    for(col=0;col<ncol;col++){
      dphi=(unwphase[nrow-1][col]-unwphasebelow[col])/TWOPI;
      tempflow=(short )LRound(dphi);
      loweredgeflows[0][col]=tempflow;
      if(tempflow<minflow){
        if(tempflow<flowlimlo){
          fflush(NULL);
          fprintf(sp0,"Overflow in tile offset\nAbort\n");
          exit(ABNORMAL_EXIT);
        }
        minflow=tempflow;
      }
      if(tempflow>maxflow){
        if(tempflow>flowlimhi){
          fflush(NULL);
          fprintf(sp0,"Overflow in tile offset\nAbort\n");
          exit(ABNORMAL_EXIT);
        }
        maxflow=tempflow;
      }
      flowhistogram[tempflow-flowlimlo]++;
      dpsi=dphi-floor(dphi);
      if(dpsi>0.5){
        dpsi-=1.0;
      }
      if(CalcCost==CalcCostTopo || CalcCost==CalcCostDefo){
        loweredgecosts[0][col].offset=(short )LRound(nshortcycle*dpsi);
        loweredgecosts[0][col].sigsq=AvgSigSq(costs[nrow-2][col].sigsq,
                                              costsbelow[col].sigsq);
        if(costs[nrow-2][col].dzmax>costsbelow[col].dzmax){
          loweredgecosts[0][col].dzmax=costs[nrow-2][col].dzmax;
        }else{
          loweredgecosts[0][col].dzmax=costsbelow[col].dzmax;
        }
        if(costs[nrow-2][col].laycost<costsbelow[col].laycost){
          loweredgecosts[0][col].laycost=costs[nrow-2][col].laycost;
        }else{
          loweredgecosts[0][col].laycost=costsbelow[col].laycost;
        }
      }else if(CalcCost==CalcCostSmooth){
        loweredgesmoothcosts[0][col].offset=(short )LRound(nshortcycle*dpsi);
        loweredgesmoothcosts[0][col].sigsq
          =AvgSigSq(smoothcosts[nrow-2][col].sigsq,smoothcostsbelow[col].sigsq);
      }else if(CalcCost==CalcCostL0 || CalcCost==CalcCostL1
               || CalcCost==CalcCostL2 || CalcCost==CalcCostLP){
        ((short **)voidloweredgecosts)[0][col]=
          (((short **)voidcosts)[nrow-2][col]
           +((short *)voidcostsbelow)[col])/2;
      }else if(CalcCost==CalcCostL0BiDir || CalcCost==CalcCostL1BiDir
               || CalcCost==CalcCostL2BiDir || CalcCost==CalcCostLPBiDir){
        ((bidircostT **)voidloweredgecosts)[0][col].posweight=
          (((bidircostT **)voidcosts)[nrow-2][col].posweight
           +((bidircostT *)voidcostsbelow)[col].posweight)/2;
        ((bidircostT **)voidloweredgecosts)[0][col].negweight=
          (((bidircostT **)voidcosts)[nrow-2][col].negweight
           +((bidircostT *)voidcostsbelow)[col].negweight)/2;
      }else{
        fflush(NULL);
        fprintf(sp0,"Illegal cost mode in SetLowerEdge().  This is a bug.\n");
        exit(ABNORMAL_EXIT);
      }
    }

    /* set bulk tile offset equal to mode of flow histogram */
    nmax=0;
    reloffset=0;
    for(iflow=minflow;iflow<=maxflow;iflow++){
      if(flowhistogram[iflow-flowlimlo]>nmax){
        nmax=flowhistogram[iflow-flowlimlo];
        reloffset=iflow;
      }
    }
    bulkoffsets[tilerow+1][tilecol]=bulkoffsets[tilerow][tilecol]-reloffset;

    /* subtract relative tile offset from edge flows */
    for(col=0;col<ncol;col++){
      loweredgeflows[0][col]-=reloffset;
    }

    /* finish up */
    free(flowhistogram);

  }else{
    if(CalcCost==CalcCostTopo || CalcCost==CalcCostDefo){
      for(col=0;col<ncol;col++){
        loweredgecosts[0][col].offset=LARGESHORT/2;
        loweredgecosts[0][col].sigsq=LARGESHORT;
        loweredgecosts[0][col].dzmax=LARGESHORT;
        loweredgecosts[0][col].laycost=0;
      }
    }else if(CalcCost==CalcCostSmooth){
      for(col=0;col<ncol;col++){
        loweredgesmoothcosts[0][col].offset=0;
        loweredgesmoothcosts[0][col].sigsq=LARGESHORT;
      }
    }else if(CalcCost==CalcCostL0 || CalcCost==CalcCostL1
             || CalcCost==CalcCostL2 || CalcCost==CalcCostLP){
      for(col=0;col<ncol;col++){
        ((short **)voidloweredgecosts)[0][col]=0;
      }
    }else if(CalcCost==CalcCostL0BiDir || CalcCost==CalcCostL1BiDir
             || CalcCost==CalcCostL2BiDir || CalcCost==CalcCostLPBiDir){
      for(col=0;col<ncol;col++){
        ((bidircostT **)voidloweredgecosts)[0][col].posweight=0;
        ((bidircostT **)voidloweredgecosts)[0][col].negweight=0;
      }
    }else{
      fflush(NULL);
      fprintf(sp0,"Illegal cost mode in SetLowerEdge().  This is a bug.\n");
      exit(ABNORMAL_EXIT);
    }
  }

  /* done */
  return(0);

}


/* function: SetLeftEdge()
 * -----------------------
 */
static
int SetLeftEdge(long nrow, long prevncol, long tilerow, long tilecol,
                void **voidcosts, void **voidlastcosts, float **unwphase,
                float **lastunwphase, void **voidleftedgecosts,
                short **leftedgeflows, paramT *params, short **bulkoffsets){

  long row, reloffset;
  double dphi, dpsi;
  costT  **leftedgecosts, **costs, **lastcosts;
  smoothcostT  **leftedgesmoothcosts, **smoothcosts, **lastsmoothcosts;
  long nshortcycle;

  /* typecast generic pointers to costT pointers */
  leftedgecosts=(costT **)voidleftedgecosts;
  costs=(costT **)voidcosts;
  lastcosts=(costT **)voidlastcosts;
  leftedgesmoothcosts=(smoothcostT **)voidleftedgecosts;
  smoothcosts=(smoothcostT **)voidcosts;
  lastsmoothcosts=(smoothcostT **)voidlastcosts;

  /* see if tile is in left column */
  if(tilecol!=0){

    /* set up */
    nshortcycle=params->nshortcycle;
    reloffset=bulkoffsets[tilerow][tilecol]-bulkoffsets[tilerow][tilecol-1];

    /* loop over all arcs on the boundary */
    for(row=0;row<nrow;row++){
      dphi=(unwphase[row][0]
            -lastunwphase[row][prevncol-1])/TWOPI;
      leftedgeflows[row][0]=(short )LRound(dphi)-reloffset;
      dpsi=dphi-floor(dphi);
      if(dpsi>0.5){
        dpsi-=1.0;
      }
      if(CalcCost==CalcCostTopo || CalcCost==CalcCostDefo){
        leftedgecosts[row][0].offset=(short )LRound(TILEDPSICOLFACTOR
                                                    *nshortcycle*dpsi);
        leftedgecosts[row][0].sigsq
          =AvgSigSq(costs[row+nrow-1][0].sigsq,
                    lastcosts[row+nrow-1][prevncol-2].sigsq);
        if(costs[row+nrow-1][0].dzmax>lastcosts[row+nrow-1][prevncol-2].dzmax){
          leftedgecosts[row][0].dzmax=costs[row+nrow-1][0].dzmax;
        }else{
          leftedgecosts[row][0].dzmax=lastcosts[row+nrow-1][prevncol-2].dzmax;
        }
        if(costs[row+nrow-1][0].laycost
           >lastcosts[row+nrow-1][prevncol-2].laycost){
          leftedgecosts[row][0].laycost=costs[row+nrow-1][0].laycost;
        }else{
          leftedgecosts[row][0].laycost
            =lastcosts[row+nrow-1][prevncol-2].laycost;
        }
      }else if(CalcCost==CalcCostSmooth){
        leftedgesmoothcosts[row][0].offset
          =(short )LRound(TILEDPSICOLFACTOR*nshortcycle*dpsi);
        leftedgesmoothcosts[row][0].sigsq
          =AvgSigSq(smoothcosts[row+nrow-1][0].sigsq,
                    lastsmoothcosts[row+nrow-1][prevncol-2].sigsq);
      }else if(CalcCost==CalcCostL0 || CalcCost==CalcCostL1
               || CalcCost==CalcCostL2 || CalcCost==CalcCostLP){
        ((short **)voidleftedgecosts)[row][0]=
          (((short **)voidcosts)[row+nrow-1][0]
           +((short **)voidlastcosts)[row+nrow-1][prevncol-2])/2;
      }else if(CalcCost==CalcCostL0BiDir || CalcCost==CalcCostL1BiDir
               || CalcCost==CalcCostL2BiDir || CalcCost==CalcCostLPBiDir){
        ((bidircostT **)voidleftedgecosts)[row][0].posweight=
          (((bidircostT **)voidcosts)[row+nrow-1][0].posweight
           +((bidircostT **)voidlastcosts)[row+nrow-1][prevncol-2].posweight)
          /2;
        ((bidircostT **)voidleftedgecosts)[row][0].negweight=
          (((bidircostT **)voidcosts)[row+nrow-1][0].negweight
           +((bidircostT **)voidlastcosts)[row+nrow-1][prevncol-2].negweight)
          /2;
      }else{
        fflush(NULL);
        fprintf(sp0,"Illegal cost mode in SetLeftEdge().  This is a bug.\n");
        exit(ABNORMAL_EXIT);
      }
    }
  }else{
    if(CalcCost==CalcCostTopo || CalcCost==CalcCostDefo){
      for(row=0;row<nrow;row++){
        leftedgecosts[row][0].offset=LARGESHORT/2;
        leftedgecosts[row][0].sigsq=LARGESHORT;
        leftedgecosts[row][0].dzmax=LARGESHORT;
        leftedgecosts[row][0].laycost=0;
      }
    }else if(CalcCost==CalcCostSmooth){
      for(row=0;row<nrow;row++){
        leftedgesmoothcosts[row][0].offset=0;
        leftedgesmoothcosts[row][0].sigsq=LARGESHORT;
      }
    }else if(CalcCost==CalcCostL0 || CalcCost==CalcCostL1
             || CalcCost==CalcCostL2 || CalcCost==CalcCostLP){
      for(row=0;row<nrow;row++){
        ((short **)voidleftedgecosts)[row][0]=0;
      }
    }else if(CalcCost==CalcCostL0BiDir || CalcCost==CalcCostL1BiDir
             || CalcCost==CalcCostL2BiDir || CalcCost==CalcCostLPBiDir){
      for(row=0;row<nrow;row++){
        ((bidircostT **)voidleftedgecosts)[row][0].posweight=0;
        ((bidircostT **)voidleftedgecosts)[row][0].negweight=0;
      }
    }else{
      fflush(NULL);
      fprintf(sp0,"Illegal cost mode in SetLeftEdge().  This is a bug.\n");
      exit(ABNORMAL_EXIT);
    }
  }

  /* done */
  return(0);

}


/* function: SetRightEdge()
 * ------------------------
 */
static
int SetRightEdge(long nrow, long ncol, long tilerow, long tilecol,
                 void **voidcosts, void **voidnextcosts,
                 float **unwphase, float **nextunwphase,
                 void **voidrightedgecosts, short **rightedgeflows,
                 paramT *params, short **bulkoffsets){

  long *flowhistogram;
  long row, iflow, reloffset, nmax;
  long flowlimhi, flowlimlo, maxflow, minflow, tempflow;
  double dphi, dpsi;
  costT  **rightedgecosts, **costs, **nextcosts;
  smoothcostT  **rightedgesmoothcosts, **smoothcosts, **nextsmoothcosts;
  long nshortcycle;

  /* typecast generic pointers to costT pointers */
  rightedgecosts=(costT **)voidrightedgecosts;
  costs=(costT **)voidcosts;
  nextcosts=(costT **)voidnextcosts;
  rightedgesmoothcosts=(smoothcostT **)voidrightedgecosts;
  smoothcosts=(smoothcostT **)voidcosts;
  nextsmoothcosts=(smoothcostT **)voidnextcosts;

  /* see if tile in right column */
  if(tilecol!=params->ntilecol-1){

    /* set up */
    nshortcycle=params->nshortcycle;
    flowlimhi=LARGESHORT;
    flowlimlo=-LARGESHORT;
    flowhistogram=(long *)CAlloc(flowlimhi-flowlimlo+1,sizeof(long));
    minflow=flowlimhi;
    maxflow=flowlimlo;

    /* loop over all arcs on the boundary */
    for(row=0;row<nrow;row++){
      dphi=(nextunwphase[row][0]
            -unwphase[row][ncol-1])/TWOPI;
      tempflow=(short )LRound(dphi);
      rightedgeflows[row][0]=tempflow;
      if(tempflow<minflow){
        if(tempflow<flowlimlo){
          fflush(NULL);
          fprintf(sp0,"Overflow in tile offset\nAbort\n");
          exit(ABNORMAL_EXIT);
        }
        minflow=tempflow;
      }
      if(tempflow>maxflow){
        if(tempflow>flowlimhi){
          fflush(NULL);
          fprintf(sp0,"Overflow in tile offset\nAbort\n");
          exit(ABNORMAL_EXIT);
        }
        maxflow=tempflow;
      }
      flowhistogram[tempflow-flowlimlo]++;
      dpsi=dphi-floor(dphi);
      if(dpsi>0.5){
        dpsi-=1.0;
      }
      if(CalcCost==CalcCostTopo || CalcCost==CalcCostDefo){
        rightedgecosts[row][0].offset=(short )LRound(TILEDPSICOLFACTOR
                                                     *nshortcycle*dpsi);
        rightedgecosts[row][0].sigsq
          =AvgSigSq(costs[row+nrow-1][ncol-2].sigsq,
                    nextcosts[row+nrow-1][0].sigsq);
        if(costs[row+nrow-1][ncol-2].dzmax>nextcosts[row+nrow-1][0].dzmax){
          rightedgecosts[row][0].dzmax=costs[row+nrow-1][ncol-2].dzmax;
        }else{
          rightedgecosts[row][0].dzmax=nextcosts[row+nrow-1][0].dzmax;
        }
        if(costs[row+nrow-1][ncol-2].laycost>nextcosts[row+nrow-1][0].laycost){
          rightedgecosts[row][0].laycost=costs[row+nrow-1][ncol-2].laycost;
        }else{
          rightedgecosts[row][0].laycost=nextcosts[row+nrow-1][0].laycost;
        }
      }else if(CalcCost==CalcCostSmooth){
        rightedgesmoothcosts[row][0].offset
          =(short )LRound(TILEDPSICOLFACTOR*nshortcycle*dpsi);
        rightedgesmoothcosts[row][0].sigsq
          =AvgSigSq(smoothcosts[row+nrow-1][ncol-2].sigsq,
                    nextsmoothcosts[row+nrow-1][0].sigsq);
      }else if(CalcCost==CalcCostL0 || CalcCost==CalcCostL1
               || CalcCost==CalcCostL2 || CalcCost==CalcCostLP){
        ((short **)voidrightedgecosts)[row][0]=
          (((short **)voidcosts)[row+nrow-1][ncol-2]
           +((short **)voidnextcosts)[row+nrow-1][0])/2;
      }else if(CalcCost==CalcCostL0BiDir || CalcCost==CalcCostL1BiDir
               || CalcCost==CalcCostL2BiDir || CalcCost==CalcCostLPBiDir){
        ((bidircostT **)voidrightedgecosts)[row][0].posweight=
          (((bidircostT **)voidcosts)[row+nrow-1][ncol-2].posweight
           +((bidircostT **)voidnextcosts)[row+nrow-1][ncol-2].posweight)/2;
        ((bidircostT **)voidrightedgecosts)[row][0].negweight=
          (((bidircostT **)voidcosts)[row+nrow-1][ncol-2].negweight
           +((bidircostT **)voidnextcosts)[row+nrow-1][ncol-2].negweight)/2;
      }else{
        fflush(NULL);
        fprintf(sp0,"Illegal cost mode in SetRightEdge().  This is a bug.\n");
        exit(ABNORMAL_EXIT);
      }
    }

    /* set bulk tile offset equal to mode of flow histogram */
    if(tilerow==0){
      nmax=0;
      reloffset=0;
      for(iflow=minflow;iflow<=maxflow;iflow++){
        if(flowhistogram[iflow-flowlimlo]>nmax){
          nmax=flowhistogram[iflow-flowlimlo];
          reloffset=iflow;
        }
      }
      bulkoffsets[tilerow][tilecol+1]=bulkoffsets[tilerow][tilecol]+reloffset;
    }else{
      reloffset=bulkoffsets[tilerow][tilecol+1]-bulkoffsets[tilerow][tilecol];
    }

    /* subtract relative tile offset from edge flows */
    for(row=0;row<nrow;row++){
      rightedgeflows[row][0]-=reloffset;
    }


    /* finish up */
    free(flowhistogram);

  }else{
    if(CalcCost==CalcCostTopo || CalcCost==CalcCostDefo){
      for(row=0;row<nrow;row++){
        rightedgecosts[row][0].offset=LARGESHORT/2;
        rightedgecosts[row][0].sigsq=LARGESHORT;
        rightedgecosts[row][0].dzmax=LARGESHORT;
        rightedgecosts[row][0].laycost=0;
      }
    }else if(CalcCost==CalcCostSmooth){
      for(row=0;row<nrow;row++){
        rightedgesmoothcosts[row][0].offset=0;
        rightedgesmoothcosts[row][0].sigsq=LARGESHORT;
      }
    }else if(CalcCost==CalcCostL0 || CalcCost==CalcCostL1
             || CalcCost==CalcCostL2 || CalcCost==CalcCostLP){
      for(row=0;row<nrow;row++){
        ((short **)voidrightedgecosts)[row][0]=0;
      }
    }else if(CalcCost==CalcCostL0BiDir || CalcCost==CalcCostL1BiDir
             || CalcCost==CalcCostL2BiDir || CalcCost==CalcCostLPBiDir){
      for(row=0;row<nrow;row++){
        ((bidircostT **)voidrightedgecosts)[row][0].posweight=0;
        ((bidircostT **)voidrightedgecosts)[row][0].negweight=0;
      }
    }else{
      fflush(NULL);
      fprintf(sp0,"Illegal cost mode in SetRightEdge().  This is a bug.\n");
      exit(ABNORMAL_EXIT);
    }
  }

  /* done */
  return(0);

}


/* function: AvgSigSq()
 * --------------------
 * Return average of sigsq values after chcking for special value and
 * clipping to short.
 */
static
short AvgSigSq(short sigsq1, short sigsq2){

  int sigsqavg;


  /* if either value is special LARGESHORT value, use that */
  if(sigsq1==LARGESHORT || sigsq2==LARGESHORT){
    return(LARGESHORT);
  }

  /* compute average */
  sigsqavg=(int )ceil(0.5*(((int )sigsq1)+((int )sigsq2)));

  /* clip */
  sigsqavg=LClip(sigsqavg,-LARGESHORT,LARGESHORT);

  /* return */
  return((short )sigsqavg);

}


/* function: TraceSecondaryArc()
 * -----------------------------
 */
static
int TraceSecondaryArc(nodeT *primaryhead, nodeT **scndrynodes,
                      nodesuppT **nodesupp, scndryarcT **scndryarcs,
                      long ***scndrycosts, long *nnewnodesptr,
                      long *nnewarcsptr, long tilerow, long tilecol,
                      long flowmax, long nrow, long ncol,
                      long prevnrow, long prevncol, paramT *params,
                      void **tilecosts, void **rightedgecosts,
                      void **loweredgecosts, void **leftedgecosts,
                      void **upperedgecosts, short **tileflows,
                      short **rightedgeflows, short **loweredgeflows,
                      short **leftedgeflows, short **upperedgeflows,
                      nodeT ***updatednontilenodesptr,
                      long *nupdatednontilenodesptr,
                      long *updatednontilenodesizeptr,
                      short **inontilenodeoutarcptr, long *totarclenptr){

  long i, row, col, nnewnodes, arclen, ntilerow, ntilecol, arcnum;
  long tilenum, nflow, primaryarcrow, primaryarccol, poscost, negcost, nomcost;
  long nnrow, nncol, calccostnrow, nnewarcs, arroffset, nshortcycle;
  long mincost, mincostflow, minweight, maxcost;
  long *scndrycostarr;
  double sigsq, sumsigsqinv, tempdouble, tileedgearcweight;
  short **flows;
  void **costs;
  nodeT *tempnode, *primarytail, *scndrytail, *scndryhead;
  nodeT *primarydummy, *scndrydummy;
  nodesuppT *supptail, *supphead, *suppdummy;
  scndryarcT *newarc;
  signed char primaryarcdir, zerocost;


  /* do nothing if source is passed or if arc already done in previous tile */
  if(primaryhead->pred==NULL
     || (tilerow!=0 && primaryhead->row==0 && primaryhead->pred->row==0)
     || (tilecol!=0 && primaryhead->col==0 && primaryhead->pred->col==0)){
    return(0);
  }

  /* set up */
  ntilerow=params->ntilerow;
  ntilecol=params->ntilecol;
  nnrow=nrow+1;
  nncol=ncol+1;
  tilenum=tilerow*ntilecol+tilecol;
  scndrycostarr=(long *)MAlloc((2*flowmax+2)*sizeof(long));
  tileedgearcweight=params->tileedgeweight;
  nshortcycle=params->nshortcycle;
  zerocost=FALSE;
  arroffset=0;
  sigsq=0;

  /* loop to determine appropriate value for arroffset */
  while(TRUE){

    /* initialize variables */
    arclen=0;
    sumsigsqinv=0;
    for(nflow=1;nflow<=2*flowmax;nflow++){
      scndrycostarr[nflow]=0;
    }

    /* loop over primary arcs on secondary arc again to get costs */
    primarytail=primaryhead->pred;
    tempnode=primaryhead;
    while(TRUE){

      /* get primary arc just traversed */
      arclen++;
      if(tempnode->col==primarytail->col+1){              /* rightward arc */
        primaryarcdir=1;
        primaryarccol=primarytail->col;
        if(primarytail->row==0){                               /* top edge */
          if(tilerow==0){
            zerocost=TRUE;
          }else{
            primaryarcrow=0;
            costs=upperedgecosts;
            flows=upperedgeflows;
            calccostnrow=2;
          }
        }else if(primarytail->row==nnrow-1){                /* bottom edge */
          if(tilerow==ntilerow-1){
            zerocost=TRUE;
          }else{
            primaryarcrow=0;
            costs=loweredgecosts;
            flows=loweredgeflows;
            calccostnrow=2;
          }
        }else{                                               /* normal arc */
          primaryarcrow=primarytail->row-1;
          costs=tilecosts;
          flows=tileflows;
          calccostnrow=nrow;
        }
      }else if(tempnode->row==primarytail->row+1){         /* downward arc */
        primaryarcdir=1;
        if(primarytail->col==0){                              /* left edge */
          if(tilecol==0){
            zerocost=TRUE;
          }else{
            primaryarcrow=primarytail->row;
            primaryarccol=0;
            costs=leftedgecosts;
            flows=leftedgeflows;
            calccostnrow=0;
          }
        }else if(primarytail->col==nncol-1){                 /* right edge */
          if(tilecol==ntilecol-1){
            zerocost=TRUE;
          }else{
            primaryarcrow=primarytail->row;
            primaryarccol=0;
            costs=rightedgecosts;
            flows=rightedgeflows;
            calccostnrow=0;
          }
        }else{                                               /* normal arc */
          primaryarcrow=primarytail->row+nrow-1;
          primaryarccol=primarytail->col-1;
          costs=tilecosts;
          flows=tileflows;
          calccostnrow=nrow;
        }
      }else if(tempnode->col==primarytail->col-1){         /* leftward arc */
        primaryarcdir=-1;
        primaryarccol=primarytail->col-1;
        if(primarytail->row==0){                               /* top edge */
          if(tilerow==0){
            zerocost=TRUE;
          }else{
            primaryarcrow=0;
            costs=upperedgecosts;
            flows=upperedgeflows;
            calccostnrow=2;
          }
        }else if(primarytail->row==nnrow-1){                /* bottom edge */
          if(tilerow==ntilerow-1){
            zerocost=TRUE;
          }else{
            primaryarcrow=0;
            costs=loweredgecosts;
            flows=loweredgeflows;
            calccostnrow=2;
          }
        }else{                                               /* normal arc */
          primaryarcrow=primarytail->row-1;
          costs=tilecosts;
          flows=tileflows;
          calccostnrow=nrow;
        }
      }else{                                                 /* upward arc */
        primaryarcdir=-1;
        if(primarytail->col==0){                              /* left edge */
          if(tilecol==0){
            zerocost=TRUE;
          }else{
            primaryarcrow=primarytail->row-1;
            primaryarccol=0;
            costs=leftedgecosts;
            flows=leftedgeflows;
            calccostnrow=0;
          }
        }else if(primarytail->col==nncol-1){                 /* right edge */
          if(tilecol==ntilecol-1){
            zerocost=TRUE;
          }else{
            primaryarcrow=primarytail->row-1;
            primaryarccol=0;
            costs=rightedgecosts;
            flows=rightedgeflows;
            calccostnrow=0;
          }
        }else{                                               /* normal arc */
          primaryarcrow=primarytail->row+nrow-2;
          primaryarccol=primarytail->col-1;
          costs=tilecosts;
          flows=tileflows;
          calccostnrow=nrow;
        }
      }

      /* keep absolute cost of arc to the previous node */
      if(!zerocost){

        /* accumulate incremental cost in table for each nflow increment */
        /* offset flow in flow array temporarily by arroffset then undo below */
        flows[primaryarcrow][primaryarccol]-=primaryarcdir*arroffset;
        nomcost=EvalCost(costs,flows,primaryarcrow,primaryarccol,calccostnrow,
                         params);
        for(nflow=1;nflow<=flowmax;nflow++){
          flows[primaryarcrow][primaryarccol]+=(primaryarcdir*nflow);
          poscost=EvalCost(costs,flows,primaryarcrow,primaryarccol,
                           calccostnrow,params);
          flows[primaryarcrow][primaryarccol]-=(2*primaryarcdir*nflow);
          negcost=EvalCost(costs,flows,primaryarcrow,primaryarccol,
                           calccostnrow,params);
          flows[primaryarcrow][primaryarccol]+=(primaryarcdir*nflow);
          tempdouble=(scndrycostarr[nflow]+(poscost-nomcost));
          if(tempdouble>LARGEINT){
            scndrycostarr[nflow]=LARGEINT;
          }else if(tempdouble<-LARGEINT){
            scndrycostarr[nflow]=-LARGEINT;
          }else{
            scndrycostarr[nflow]+=(poscost-nomcost);
          }
          tempdouble=(scndrycostarr[nflow+flowmax]+(negcost-nomcost));
          if(tempdouble>LARGEINT){
            scndrycostarr[nflow+flowmax]=LARGEINT;
          }else if(tempdouble<-LARGEINT){
            scndrycostarr[nflow+flowmax]=-LARGEINT;
          }else{
            scndrycostarr[nflow+flowmax]+=(negcost-nomcost);
          }
        }
        flows[primaryarcrow][primaryarccol]+=primaryarcdir*arroffset;

        /* accumulate term to be used for cost growth beyond table bounds */
        if(CalcCost==CalcCostTopo || CalcCost==CalcCostDefo){
          sigsq=((costT **)costs)[primaryarcrow][primaryarccol].sigsq;
        }else if(CalcCost==CalcCostSmooth){
          sigsq=((smoothcostT **)costs)[primaryarcrow][primaryarccol].sigsq;
        }else if(CalcCost==CalcCostL0 || CalcCost==CalcCostL1
                 || CalcCost==CalcCostL2 || CalcCost==CalcCostLP){
          minweight=((short **)costs)[primaryarcrow][primaryarccol];
          if(minweight<1){
            sigsq=LARGESHORT;
          }else{
            sigsq=1.0/(double )minweight;
          }
        }else if(CalcCost==CalcCostL0BiDir || CalcCost==CalcCostL1BiDir
                 || CalcCost==CalcCostL2BiDir || CalcCost==CalcCostLPBiDir){
          minweight=LMin(((bidircostT **)costs)[primaryarcrow][primaryarccol]
                         .posweight,
                         ((bidircostT **)costs)[primaryarcrow][primaryarccol]
                         .negweight);
          if(minweight<1){
            sigsq=LARGESHORT;
          }else{
            sigsq=1.0/(double )minweight;
          }
        }
        if(sigsq<LARGESHORT){    /* zero cost arc if sigsq == LARGESHORT */
          sumsigsqinv+=(1.0/sigsq);
        }
      }

      /* break if found the secondary arc tail */
      if(primarytail->group==ONTREE){
        break;
      }

      /* move up the tree */
      tempnode=primarytail;
      primarytail=primarytail->pred;

    } /* end while loop for tracing secondary arc for costs */

    /* break if we have a zero-cost arc on the edge of the full array */
    if(zerocost){
      break;
    }

    /* find flow index with minimum cost */
    mincost=0;
    maxcost=0;
    mincostflow=0;
    for(nflow=1;nflow<=flowmax;nflow++){
      if(scndrycostarr[nflow]<mincost){
        mincost=scndrycostarr[nflow];
        mincostflow=nflow;
      }
      if(scndrycostarr[flowmax+nflow]<mincost){
        mincost=scndrycostarr[flowmax+nflow];
        mincostflow=-nflow;
      }
      if(scndrycostarr[nflow]>maxcost){
        maxcost=scndrycostarr[nflow];
      }
      if(scndrycostarr[flowmax+nflow]>maxcost){
        maxcost=scndrycostarr[flowmax+nflow];
      }
    }

    /* if cost was all zero, treat as zero cost arc */
    if(maxcost==mincost){
      zerocost=TRUE;
      sumsigsqinv=0;
    }

    /* break if cost array adequately centered on minimum cost flow */
    if(mincostflow==0){
      break;
    }

    /* correct value of arroffset for next loop */
    if(mincostflow==flowmax){
      arroffset-=((long )floor(1.5*flowmax));
    }else if(mincostflow==-flowmax){
      arroffset+=((long )floor(1.5*flowmax));
    }else{
      arroffset-=mincostflow;
    }

  } /* end while loop for determining arroffset */


  /* ignore this arc if primary head is same as tail (ie, if arc loops) */
  /* only way this can happen is if region is connected at one corner only */
  /* so any possible improvements should have been made by primary solver */
  if(primaryhead==primarytail){
    free(scndrycostarr);
    return(0);
  }

  /* see if we have a secondary arc on the edge of the full-sized array */
  /* these arcs have zero cost since the edge is treated as a single node */
  /* secondary arcs whose primary arcs all have zero cost are also zeroed */
  if(zerocost){

    /* set sum of standard deviations to indicate zero-cost secondary arc */
    scndrycostarr[0]=0;
    for(nflow=1;nflow<=2*flowmax;nflow++){
      scndrycostarr[nflow]=0;
    }
    scndrycostarr[2*flowmax+1]=ZEROCOSTARC;

  }else{

    /* give extra weight to arcs on tile edges */
    if((primaryhead->row==primarytail->row
        && (primaryhead->row==0 || primaryhead->row==nnrow-1))
       || (primaryhead->col==primarytail->col
           && (primaryhead->col==0 || primaryhead->col==nncol-1))){
      for(nflow=1;nflow<=2*flowmax;nflow++){
        tempdouble=scndrycostarr[nflow]*tileedgearcweight;
        if(tempdouble>LARGEINT){
          scndrycostarr[nflow]=LARGEINT;
        }else if(tempdouble<-LARGEINT){
          scndrycostarr[nflow]=-LARGEINT;
        }else{
          scndrycostarr[nflow]=LRound(tempdouble);
        }
      }
      sumsigsqinv*=tileedgearcweight;

    }

    /* store sum of primary cost variances at end of secondary cost array */
    tempdouble=sumsigsqinv*nshortcycle*nshortcycle;
    if(tempdouble<LARGEINT){
      scndrycostarr[2*flowmax+1]=LRound(tempdouble);
    }else{
      scndrycostarr[2*flowmax+1]=LARGEINT;
    }
    scndrycostarr[0]=arroffset;

  }


  /* find secondary nodes corresponding to primary head, tail */
  if(primarytail->row==0 && tilerow!=0){
    scndrytail=FindScndryNode(scndrynodes,nodesupp,
                              (tilerow-1)*ntilecol+tilecol,
                              prevnrow,primarytail->col);
  }else if(primarytail->col==0 && tilecol!=0){
    scndrytail=FindScndryNode(scndrynodes,nodesupp,
                              tilerow*ntilecol+(tilecol-1),
                              primarytail->row,prevncol);
  }else{
    scndrytail=FindScndryNode(scndrynodes,nodesupp,tilenum,
                              primarytail->row,primarytail->col);
  }
  if(primaryhead->row==0 && tilerow!=0){
    scndryhead=FindScndryNode(scndrynodes,nodesupp,
                              (tilerow-1)*ntilecol+tilecol,
                              prevnrow,primaryhead->col);
  }else if(primaryhead->col==0 && tilecol!=0){
    scndryhead=FindScndryNode(scndrynodes,nodesupp,
                              tilerow*ntilecol+(tilecol-1),
                              primaryhead->row,prevncol);
  }else{
    scndryhead=FindScndryNode(scndrynodes,nodesupp,tilenum,
                              primaryhead->row,primaryhead->col);
  }

  /* see if there is already arc between secondary head, tail */
  row=scndrytail->row;
  col=scndrytail->col;
  for(i=0;i<nodesupp[row][col].noutarcs;i++){
    tempnode=nodesupp[row][col].neighbornodes[i];
    if((nodesupp[row][col].outarcs[i]==NULL
        && tempnode->row==primaryhead->row
        && tempnode->col==primaryhead->col)
       || (nodesupp[row][col].outarcs[i]!=NULL
           && tempnode->row==scndryhead->row
           && tempnode->col==scndryhead->col)){

      /* see if secondary arc traverses only one primary arc */
      primarydummy=primaryhead->pred;
      if(primarydummy->group!=ONTREE){

        /* arc already exists, free memory for cost array (will trace again) */
        free(scndrycostarr);

        /* set up dummy node */
        primarydummy->group=ONTREE;
        nnewnodes=++(*nnewnodesptr);
        scndrynodes[tilenum]=(nodeT *)ReAlloc(scndrynodes[tilenum],
                                              nnewnodes*sizeof(nodeT));
        scndrydummy=&scndrynodes[tilenum][nnewnodes-1];
        nodesupp[tilenum]=(nodesuppT *)ReAlloc(nodesupp[tilenum],
                                               nnewnodes*sizeof(nodesuppT));
        suppdummy=&nodesupp[tilenum][nnewnodes-1];
        scndrydummy->row=tilenum;
        scndrydummy->col=nnewnodes-1;
        suppdummy->row=primarydummy->row;
        suppdummy->col=primarydummy->col;
        suppdummy->noutarcs=0;
        suppdummy->neighbornodes=NULL;
        suppdummy->outarcs=NULL;

        /* recursively call TraceSecondaryArc() to set up arcs */
        TraceSecondaryArc(primarydummy,scndrynodes,nodesupp,scndryarcs,
                          scndrycosts,nnewnodesptr,nnewarcsptr,tilerow,tilecol,
                          flowmax,nrow,ncol,prevnrow,prevncol,params,tilecosts,
                          rightedgecosts,loweredgecosts,leftedgecosts,
                          upperedgecosts,tileflows,rightedgeflows,
                          loweredgeflows,leftedgeflows,upperedgeflows,
                          updatednontilenodesptr,nupdatednontilenodesptr,
                          updatednontilenodesizeptr,inontilenodeoutarcptr,
                          totarclenptr);
        TraceSecondaryArc(primaryhead,scndrynodes,nodesupp,scndryarcs,
                          scndrycosts,nnewnodesptr,nnewarcsptr,tilerow,tilecol,
                          flowmax,nrow,ncol,prevnrow,prevncol,params,tilecosts,
                          rightedgecosts,loweredgecosts,leftedgecosts,
                          upperedgecosts,tileflows,rightedgeflows,
                          loweredgeflows,leftedgeflows,upperedgeflows,
                          updatednontilenodesptr,nupdatednontilenodesptr,
                          updatednontilenodesizeptr,inontilenodeoutarcptr,
                          totarclenptr);
      }else{

        /* only one primary arc; just delete other secondary arc */
        /* find existing secondary arc (must be in this tile) */
        /* swap direction of existing secondary arc if necessary */
        arcnum=0;
        while(TRUE){
          if(scndryarcs[tilenum][arcnum].from==primarytail
             && scndryarcs[tilenum][arcnum].to==primaryhead){
            break;
          }else if(scndryarcs[tilenum][arcnum].from==primaryhead
                   && scndryarcs[tilenum][arcnum].to==primarytail){
            scndryarcs[tilenum][arcnum].from=primarytail;
            scndryarcs[tilenum][arcnum].to=primaryhead;
            break;
          }
          arcnum++;
        }

        /* assign cost of this secondary arc to existing secondary arc */
        free(scndrycosts[tilenum][arcnum]);
        scndrycosts[tilenum][arcnum]=scndrycostarr;

        /* update direction data in secondary arc structure */
        if(primarytail->col==primaryhead->col+1){
          scndryarcs[tilenum][arcnum].fromdir=RIGHT;
        }else if(primarytail->row==primaryhead->row+1){
          scndryarcs[tilenum][arcnum].fromdir=DOWN;
        }else if(primarytail->col==primaryhead->col-1){
          scndryarcs[tilenum][arcnum].fromdir=LEFT;
        }else{
          scndryarcs[tilenum][arcnum].fromdir=UP;
        }
      }

      /* we're done */
      return(0);
    }
  }

  /* set up secondary arc datastructures */
  nnewarcs=++(*nnewarcsptr);
  if(nnewarcs > SHRT_MAX){
    fflush(NULL);
    fprintf(sp0,"Exceeded maximum number of secondary arcs\n"
            "Decrease TILECOSTTHRESH and/or increase MINREGIONSIZE\n");
    exit(ABNORMAL_EXIT);
  }
  scndryarcs[tilenum]=(scndryarcT *)ReAlloc(scndryarcs[tilenum],
                                            nnewarcs*sizeof(scndryarcT));
  newarc=&scndryarcs[tilenum][nnewarcs-1];
  newarc->arcrow=tilenum;
  newarc->arccol=nnewarcs-1;
  scndrycosts[tilenum]=(long **)ReAlloc(scndrycosts[tilenum],
                                        nnewarcs*sizeof(long *));
  scndrycosts[tilenum][nnewarcs-1]=scndrycostarr;

  /* update secondary node data */
  /* store primary nodes in nodesuppT neighbornodes[] arrays since */
  /* secondary node addresses change in ReAlloc() calls in TraceRegions() */
  supptail=&nodesupp[scndrytail->row][scndrytail->col];
  supphead=&nodesupp[scndryhead->row][scndryhead->col];
  supptail->noutarcs++;
  supptail->neighbornodes=(nodeT **)ReAlloc(supptail->neighbornodes,
                                            supptail->noutarcs
                                            *sizeof(nodeT *));
  supptail->neighbornodes[supptail->noutarcs-1]=primaryhead;
  primarytail->level=scndrytail->row;
  primarytail->incost=scndrytail->col;
  supptail->outarcs=(scndryarcT **)ReAlloc(supptail->outarcs,
                                           supptail->noutarcs
                                           *sizeof(scndryarcT *));
  supptail->outarcs[supptail->noutarcs-1]=NULL;
  supphead->noutarcs++;
  supphead->neighbornodes=(nodeT **)ReAlloc(supphead->neighbornodes,
                                            supphead->noutarcs
                                            *sizeof(nodeT *));
  supphead->neighbornodes[supphead->noutarcs-1]=primarytail;
  primaryhead->level=scndryhead->row;
  primaryhead->incost=scndryhead->col;
  supphead->outarcs=(scndryarcT **)ReAlloc(supphead->outarcs,
                                           supphead->noutarcs
                                           *sizeof(scndryarcT *));
  supphead->outarcs[supphead->noutarcs-1]=NULL;

  /* keep track of updated secondary nodes that were not in this tile */
  if(scndrytail->row!=tilenum){
    if(++(*nupdatednontilenodesptr)==(*updatednontilenodesizeptr)){
      (*updatednontilenodesizeptr)+=INITARRSIZE;
      (*updatednontilenodesptr)=(nodeT **)ReAlloc((*updatednontilenodesptr),
                                                  (*updatednontilenodesizeptr)
                                                  *sizeof(nodeT *));
      (*inontilenodeoutarcptr)=(short *)ReAlloc((*inontilenodeoutarcptr),
                                                (*updatednontilenodesizeptr)
                                                *sizeof(short));
    }
    (*updatednontilenodesptr)[*nupdatednontilenodesptr-1]=scndrytail;
    (*inontilenodeoutarcptr)[*nupdatednontilenodesptr-1]=supptail->noutarcs-1;
  }
  if(scndryhead->row!=tilenum){
    if(++(*nupdatednontilenodesptr)==(*updatednontilenodesizeptr)){
      (*updatednontilenodesizeptr)+=INITARRSIZE;
      (*updatednontilenodesptr)=(nodeT **)ReAlloc((*updatednontilenodesptr),
                                                  (*updatednontilenodesizeptr)
                                                  *sizeof(nodeT *));
      (*inontilenodeoutarcptr)=(short *)ReAlloc((*inontilenodeoutarcptr),
                                                (*updatednontilenodesizeptr)
                                                *sizeof(short));
    }
    (*updatednontilenodesptr)[*nupdatednontilenodesptr-1]=scndryhead;
    (*inontilenodeoutarcptr)[*nupdatednontilenodesptr-1]=supphead->noutarcs-1;
  }

  /* set up node data in secondary arc structure */
  newarc->from=primarytail;
  newarc->to=primaryhead;

  /* set up direction data in secondary arc structure */
  tempnode=primaryhead->pred;
  if(tempnode->col==primaryhead->col+1){
    newarc->fromdir=RIGHT;
  }else if(tempnode->row==primaryhead->row+1){
    newarc->fromdir=DOWN;
  }else if(tempnode->col==primaryhead->col-1){
    newarc->fromdir=LEFT;
  }else{
    newarc->fromdir=UP;
  }

  /* add number of primary arcs in secondary arc to counter */
  (*totarclenptr)+=arclen;

  /* done */
  return(0);

}


/* function: FindScndryNode()
 * --------------------------
 */
static
nodeT *FindScndryNode(nodeT **scndrynodes, nodesuppT **nodesupp,
                      long tilenum, long primaryrow, long primarycol){

  long nodenum;
  nodesuppT *nodesuppptr;

  /* set temporary variables */
  nodesuppptr=nodesupp[tilenum];

  /* loop over all nodes in the tile until we find a match */
  nodenum=0;
  while(nodesuppptr[nodenum].row!=primaryrow
        || nodesuppptr[nodenum].col!=primarycol){
    nodenum++;
  }
  return(&scndrynodes[tilenum][nodenum]);
}


/* function: IntegrateSecondaryFlows()
 * -----------------------------------
 */
static
int IntegrateSecondaryFlows(long linelen, long nlines, nodeT **scndrynodes,
                            nodesuppT **nodesupp, scndryarcT **scndryarcs,
                            int *nscndryarcs, short **scndryflows,
                            short **bulkoffsets, outfileT *outfiles,
                            paramT *params){

  FILE *outfp;
  float **unwphase, **tileunwphase, **mag, **tilemag;
  float *outline;
  long row, col, colstart, nrow, ncol, nnrow, nncol, maxcol;
  long readtilelinelen, readtilenlines, nextcoloffset, nextrowoffset;
  long tilerow, tilecol, ntilerow, ntilecol, rowovrlp, colovrlp;
  long ni, nj, tilenum;
  double tileoffset;
  short **regions, **tileflows;
  char realoutfile[MAXSTRLEN], readfile[MAXSTRLEN], tempstring[MAXTMPSTRLEN];
  char path[MAXSTRLEN], basename[MAXSTRLEN];
  signed char writeerror;
  tileparamT readtileparams[1];
  outfileT readtileoutfiles[1];


  /* initialize stack structures to zero for good measure */
  memset(readtileparams,0,sizeof(tileparamT));
  memset(readtileoutfiles,0,sizeof(outfileT));
  memset(realoutfile,0,MAXSTRLEN);
  memset(readfile,0,MAXSTRLEN);
  memset(tempstring,0,MAXSTRLEN);
  memset(path,0,MAXSTRLEN);
  memset(basename,0,MAXSTRLEN);

  /* set up */
  fprintf(sp1,"Integrating secondary flows\n");
  ntilerow=params->ntilerow;
  ntilecol=params->ntilecol;
  rowovrlp=params->rowovrlp;
  colovrlp=params->colovrlp;
  ni=ceil((nlines+(ntilerow-1)*rowovrlp)/(double )ntilerow);
  nj=ceil((linelen+(ntilecol-1)*colovrlp)/(double )ntilecol);
  nextcoloffset=0;
  writeerror=FALSE;
  nrow=0;

  /* get memory */
  regions=(short **)Get2DMem(ni,nj,sizeof(short *),sizeof(short));
  tileflows=(short **)Get2DRowColMem(ni+2,nj+2,sizeof(short *),sizeof(short));
  tileunwphase=(float **)Get2DMem(ni,nj,sizeof(float *),sizeof(float));
  tilemag=(float **)Get2DMem(ni,nj,sizeof(float *),sizeof(float));
  unwphase=(float **)Get2DMem(ni,linelen,sizeof(float *),sizeof(float));
  mag=(float **)Get2DMem(ni,linelen,sizeof(float *),sizeof(float));
  outline=(float *)MAlloc(2*linelen*sizeof(float));

  /* flip sign of bulk offsets if flip flag is set */
  /* do this and flip flow signs instead of flipping phase signs */
  if(params->flipphasesign){
    for(row=0;row<ntilerow;row++){
      for(col=0;col<ntilecol;col++){
        bulkoffsets[row][col]*=-1;
      }
    }
  }

  /* open output file */
  outfp=OpenOutputFile(outfiles->outfile,realoutfile);

  /* process each tile row */
  for(tilerow=0;tilerow<ntilerow;tilerow++){

    /* process each tile column, place into unwrapped tile row array */
    nextrowoffset=0;
    for(tilecol=0;tilecol<ntilecol;tilecol++){

      /* use SetupTile() to set filenames; tile params overwritten below */
      SetupTile(nlines,linelen,params,readtileparams,outfiles,
                readtileoutfiles,tilerow,tilecol);
      colstart=readtileparams->firstcol;
      readtilenlines=readtileparams->nrow;
      readtilelinelen=readtileparams->ncol;

      /* set tile read parameters */
      SetTileReadParams(readtileparams,readtilenlines,readtilelinelen,
                        tilerow,tilecol,nlines,linelen,params);
      colstart+=readtileparams->firstcol;
      nrow=readtileparams->nrow;
      ncol=readtileparams->ncol;
      nnrow=nrow+1;
      nncol=ncol+1;

      /* read unwrapped phase */
      /* phase sign not flipped for positive baseline */
      /* since flow will be flipped if necessary */
      if(TMPTILEOUTFORMAT==ALT_LINE_DATA){
        ReadAltLineFile(&tilemag,&tileunwphase,readtileoutfiles->outfile,
                        readtilelinelen,readtilenlines,readtileparams);
      }else if(TMPTILEOUTFORMAT==FLOAT_DATA){
        Read2DArray((void ***)&tileunwphase,readtileoutfiles->outfile,
                    readtilelinelen,readtilenlines,readtileparams,
                    sizeof(float *),sizeof(float));
      }

      /* read regions */
      ParseFilename(outfiles->outfile,path,basename);
      sprintf(tempstring,"%s/%s%s_%ld_%ld.%ld%s",
              params->tiledir,TMPTILEROOT,basename,tilerow,tilecol,
              readtilelinelen,REGIONSUFFIX);
      StrNCopy(readfile,tempstring,MAXSTRLEN);
      Read2DArray((void ***)&regions,readfile,readtilelinelen,readtilenlines,
                  readtileparams,sizeof(short *),sizeof(short));

      /* remove temporary files unless told so save them */
      if(params->rmtmptile){
        unlink(readtileoutfiles->outfile);
        unlink(readfile);
      }

      /* zero out primary flow array */
      for(row=0;row<2*nrow+1;row++){
        if(row<nrow){
          maxcol=ncol;
        }else{
          maxcol=ncol+1;
        }
        for(col=0;col<maxcol;col++){
          tileflows[row][col]=0;
        }
      }

      /* loop over each secondary arc in this tile and parse flows */
      /* if flip flag set, flow derived from flipped phase array */
      /* flip flow for integration in ParseSecondaryFlows() */
      tilenum=tilerow*ntilecol+tilecol;
      ParseSecondaryFlows(tilenum,nscndryarcs,tileflows,regions,scndryflows,
                          nodesupp,scndryarcs,nrow,ncol,ntilerow,ntilecol,
                          params);

      /* place tile mag, adjusted unwrapped phase into output arrays */
      mag[0][colstart]=tilemag[0][0];
      if(tilecol==0){
        tileoffset=TWOPI*nextcoloffset;
      }else{
        tileoffset=TWOPI*nextrowoffset;
      }
      unwphase[0][colstart]=tileunwphase[0][0]+tileoffset;
      for(col=1;col<ncol;col++){
        mag[0][colstart+col]=tilemag[0][col];
        unwphase[0][colstart+col]
          =(float )((double )unwphase[0][colstart+col-1]
                    +(double )tileunwphase[0][col]
                    -(double )tileunwphase[0][col-1]
                    +tileflows[nnrow][col]*TWOPI);
      }
      if(tilecol!=ntilecol-1){
        nextrowoffset=(LRound((unwphase[0][colstart+ncol-1]
                               -tileunwphase[0][ncol-1])/TWOPI)
                       +tileflows[nnrow][nncol-1]
                       +bulkoffsets[tilerow][tilecol]
                       -bulkoffsets[tilerow][tilecol+1]);
      }
      for(row=1;row<nrow;row++){
        for(col=0;col<ncol;col++){
          mag[row][colstart+col]=tilemag[row][col];
          unwphase[row][colstart+col]
            =(float )((double )unwphase[row-1][colstart+col]
                      +(double )tileunwphase[row][col]
                      -(double )tileunwphase[row-1][col]
                      -tileflows[row][col]*TWOPI);
        }
      }
      if(tilecol==0 && tilerow!=ntilerow-1){
        nextcoloffset=(LRound((unwphase[nrow-1][colstart]
                              -tileunwphase[nrow-1][0])/TWOPI)
                       -tileflows[nnrow-1][0]
                       +bulkoffsets[tilerow][tilecol]
                       -bulkoffsets[tilerow+1][tilecol]);
      }

    } /* end loop over tile columns */

    /* write out tile row */
    for(row=0;row<nrow;row++){
      if(outfiles->outfileformat==ALT_LINE_DATA){
        if(fwrite(mag[row],sizeof(float),linelen,outfp)!=linelen
           || fwrite(unwphase[row],sizeof(float),linelen,outfp)!=linelen){
          writeerror=TRUE;
          break;
        }
      }else if(outfiles->outfileformat==ALT_SAMPLE_DATA){
        for(col=0;col<linelen;col++){
          outline[2*col]=mag[row][col];
          outline[2*col+1]=unwphase[row][col];
        }
        if(fwrite(outline,sizeof(float),2*linelen,outfp)!=2*linelen){
          writeerror=TRUE;
          break;
        }
      }else{
        if(fwrite(unwphase[row],sizeof(float),linelen,outfp)!=linelen){
          writeerror=TRUE;
          break;
        }
      }
    }
    if(writeerror){
      fflush(NULL);
      fprintf(sp0,"Error while writing to file %s (device full?)\nAbort\n",
              realoutfile);
      exit(ABNORMAL_EXIT);
    }

  } /* end loop over tile rows */


  /* close output file, free memory */
  fprintf(sp1,"Output written to file %s\n",realoutfile);
  if(fclose(outfp)){
    fflush(NULL);
    fprintf(sp0,"WARNING: problem closing file %s (disk full?)\n",realoutfile);
  }
  Free2DArray((void **)regions,ni);
  Free2DArray((void **)tileflows,ni);
  Free2DArray((void **)tileunwphase,ni);
  Free2DArray((void **)tilemag,ni);
  Free2DArray((void **)unwphase,ni);
  Free2DArray((void **)mag,ni);
  free(outline);
  return(0);

}


/* function: ParseSecondaryFlows()
 * -------------------------------
 */
static
int ParseSecondaryFlows(long tilenum, int *nscndryarcs, short **tileflows,
                         short **regions, short **scndryflows,
                         nodesuppT **nodesupp, scndryarcT **scndryarcs,
                         long nrow, long ncol, long ntilerow, long ntilecol,
                         paramT *params){

  nodeT *scndryfrom, *scndryto;
  long arcnum, nnrow, nncol, nflow, primaryfromrow, primaryfromcol;
  long prevrow, prevcol, thisrow, thiscol, nextrow, nextcol;
  signed char phaseflipsign;


  /* see if we need to flip sign of flow because of positive topo baseline */
  if(params->flipphasesign){
    phaseflipsign=-1;
  }else{
    phaseflipsign=1;
  }

  /* loop over all arcs in tile */
  for(arcnum=0;arcnum<nscndryarcs[tilenum];arcnum++){

    /* do nothing if prev arc has no secondary flow */
    nflow=phaseflipsign*scndryflows[tilenum][arcnum];
    if(nflow){

      /* get arc info */
      nnrow=nrow+1;
      nncol=ncol+1;
      scndryfrom=scndryarcs[tilenum][arcnum].from;
      scndryto=scndryarcs[tilenum][arcnum].to;
      if(scndryfrom->row==tilenum){
        primaryfromrow=nodesupp[scndryfrom->row][scndryfrom->col].row;
        primaryfromcol=nodesupp[scndryfrom->row][scndryfrom->col].col;
      }else if(scndryfrom->row==tilenum-ntilecol){
        primaryfromrow=0;
        primaryfromcol=nodesupp[scndryfrom->row][scndryfrom->col].col;
      }else if(scndryfrom->row==tilenum-1){
        primaryfromrow=nodesupp[scndryfrom->row][scndryfrom->col].row;
        primaryfromcol=0;
      }else{
        primaryfromrow=0;
        primaryfromcol=0;
      }
      if(scndryto->row==tilenum){
        thisrow=nodesupp[scndryto->row][scndryto->col].row;
        thiscol=nodesupp[scndryto->row][scndryto->col].col;
      }else if(scndryto->row==tilenum-ntilecol){
        thisrow=0;
        thiscol=nodesupp[scndryto->row][scndryto->col].col;
      }else if(scndryto->row==tilenum-1){
        thisrow=nodesupp[scndryto->row][scndryto->col].row;
        thiscol=0;
      }else{
        thisrow=0;
        thiscol=0;
      }

      /* set initial direction out of secondary arc head */
      switch(scndryarcs[tilenum][arcnum].fromdir){
      case RIGHT:
        nextrow=thisrow;
        nextcol=thiscol+1;
        tileflows[thisrow][thiscol]-=nflow;
        break;
      case DOWN:
        nextrow=thisrow+1;
        nextcol=thiscol;
        tileflows[nnrow+thisrow][thiscol]-=nflow;
        break;
      case LEFT:
        nextrow=thisrow;
        nextcol=thiscol-1;
        tileflows[thisrow][thiscol-1]+=nflow;
        break;
      default:
        nextrow=thisrow-1;
        nextcol=thiscol;
        tileflows[nnrow+thisrow-1][thiscol]+=nflow;
        break;
      }

      /* use region data to trace path between secondary from, to */
      while(!(nextrow==primaryfromrow && nextcol==primaryfromcol)){

        /* move to next node */
        prevrow=thisrow;
        prevcol=thiscol;
        thisrow=nextrow;
        thiscol=nextcol;

        /* check rightward arc */
        if(thiscol!=nncol-1){
          if(thisrow==0 || thisrow==nnrow-1
             || regions[thisrow-1][thiscol]!=regions[thisrow][thiscol]){
            if(!(thisrow==prevrow && thiscol+1==prevcol)){
              tileflows[thisrow][thiscol]-=nflow;
              nextcol++;
            }
          }
        }

        /* check downward arc */
        if(thisrow!=nnrow-1){
          if(thiscol==0 || thiscol==nncol-1
             || regions[thisrow][thiscol]!=regions[thisrow][thiscol-1]){
            if(!(thisrow+1==prevrow && thiscol==prevcol)){
              tileflows[nnrow+thisrow][thiscol]-=nflow;
              nextrow++;
            }
          }
        }

        /* check leftward arc */
        if(thiscol!=0){
          if(thisrow==0 || thisrow==nnrow-1
             || regions[thisrow][thiscol-1]!=regions[thisrow-1][thiscol-1]){
            if(!(thisrow==prevrow && thiscol-1==prevcol)){
              tileflows[thisrow][thiscol-1]+=nflow;
              nextcol--;
            }
          }
        }

        /* check upward arc */
        if(thisrow!=0){
          if(thiscol==0 || thiscol==nncol-1
             || regions[thisrow-1][thiscol-1]!=regions[thisrow-1][thiscol]){
            if(!(thisrow-1==prevrow && thiscol==prevcol)){
              tileflows[nnrow+thisrow-1][thiscol]+=nflow;
              nextrow--;
            }
          }
        }
      }
    }
  }
  return(0);
}


/* function: AssembleTileConnComps()
 * ---------------------------------
 * Assemble conntected components per tile.
 */
static
int AssembleTileConnComps(long linelen, long nlines,
                          outfileT *outfiles, paramT *params){

  int ipass;
  long k;
  long row, col, colstart, nrow, ncol;
  long readtilelinelen, readtilenlines;
  long tilerow, tilecol, ntilerow, ntilecol, rowovrlp, colovrlp;
  long ni, nj, tilenum;
  unsigned int iconncomp, iconncompmax;
  long ntileconncomp, nconncomp;
  long ntileconncompmem, nconncompmem, nmemold;
  char realoutfile[MAXSTRLEN], readfile[MAXSTRLEN], tempstring[MAXTMPSTRLEN];
  char path[MAXSTRLEN], basename[MAXSTRLEN];
  signed char writeerror;
  tileparamT readtileparams[1];
  outfileT readtileoutfiles[1];
  unsigned int *tilemapping;
  unsigned int **tileconncomps, **tilerowconncomps;
  unsigned char **ucharbuf;
  unsigned char *ucharoutbuf;
  conncompsizeT *tileconncompsizes, *conncompsizes;
  FILE *outfp;


  /* initialize stack structures to zero for good measure */
  memset(readtileparams,0,sizeof(tileparamT));
  memset(readtileoutfiles,0,sizeof(outfileT));
  memset(realoutfile,0,MAXSTRLEN);
  memset(readfile,0,MAXSTRLEN);
  memset(tempstring,0,MAXSTRLEN);
  memset(path,0,MAXSTRLEN);
  memset(basename,0,MAXSTRLEN);

  /* set up */
  fprintf(sp1,"Assembling tile connected components\n");
  ntilerow=params->ntilerow;
  ntilecol=params->ntilecol;
  rowovrlp=params->rowovrlp;
  colovrlp=params->colovrlp;
  ni=ceil((nlines+(ntilerow-1)*rowovrlp)/(double )ntilerow);
  nj=ceil((linelen+(ntilecol-1)*colovrlp)/(double )ntilecol);
  writeerror=FALSE;
  nrow=0;
  conncompsizes=NULL;
  nconncomp=0;
  nconncompmem=0;
  tileconncompsizes=NULL;
  ntileconncompmem=0;
  iconncompmax=0;

  /* get memory */
  tileconncomps=(unsigned int **)Get2DMem(ni,nj,sizeof(unsigned int *),
                                          sizeof(unsigned int));
  tilerowconncomps=(unsigned int **)Get2DMem(ni,linelen,sizeof(unsigned int *),
                                             sizeof(unsigned int));
  ucharbuf=(unsigned char **)Get2DMem(ni,nj,sizeof(unsigned char *),
                                      sizeof(unsigned char));
  ucharoutbuf=(unsigned char *)MAlloc(linelen*sizeof(unsigned char));
  tilemapping=NULL;

  /* open output file */
  outfp=OpenOutputFile(outfiles->conncompfile,realoutfile);

  /* do two passes looping over all tiles */
  for(ipass=0;ipass<2;ipass++){

    /* process each tile row */
    for(tilerow=0;tilerow<ntilerow;tilerow++){

      /* process each tile column */
      for(tilecol=0;tilecol<ntilecol;tilecol++){

        /* use SetupTile() to set filenames; tile params overwritten below */
        SetupTile(nlines,linelen,params,readtileparams,outfiles,
                  readtileoutfiles,tilerow,tilecol);
        colstart=readtileparams->firstcol;
        readtilenlines=readtileparams->nrow;
        readtilelinelen=readtileparams->ncol;

        /* set tile read parameters */
        SetTileReadParams(readtileparams,readtilenlines,readtilelinelen,
                          tilerow,tilecol,nlines,linelen,params);
        colstart+=readtileparams->firstcol;
        nrow=readtileparams->nrow;
        ncol=readtileparams->ncol;

        /* set tile number */
        tilenum=tilerow*ntilecol+tilecol;

        /* read connected components for tile */
        if(params->conncompouttype==CONNCOMPOUTTYPEUCHAR){
          Read2DArray((void ***)&ucharbuf,readtileoutfiles->conncompfile,
                      readtilelinelen,readtilenlines,readtileparams,
                      sizeof(unsigned char *),sizeof(unsigned char));
          for(row=0;row<nrow;row++){
            for(col=0;col<ncol;col++){
              tileconncomps[row][col]=(unsigned int )ucharbuf[row][col];
            }
          }
        }else{
          Read2DArray((void ***)&tileconncomps,readtileoutfiles->conncompfile,
                      readtilelinelen,readtilenlines,readtileparams,
                      sizeof(unsigned int *),sizeof(unsigned int));
        }

        /* see which pass we are in */
        if(ipass==0){

          /* first pass */

          /* initialize tileconncomps array for this tile */
          ntileconncomp=0;
          for(k=0;k<ntileconncompmem;k++){
            tileconncompsizes[k].tilenum=tilenum;
            tileconncompsizes[k].icomptile=k+1;
            tileconncompsizes[k].icompfull=0;
            tileconncompsizes[k].npix=0;
          }

          /* loop over kept pixels and count pixels in each conncomp */
          for(row=0;row<nrow;row++){
            for(col=0;col<ncol;col++){

              /* see if have connected component (conncomp number not zero) */
              iconncomp=tileconncomps[row][col];
              if(iconncomp>0){

                /* get more memory for tile conncompsizeT array if needed */
                while(iconncomp>ntileconncompmem){
                  nmemold=ntileconncompmem;
                  ntileconncompmem+=CONNCOMPMEMINCR;
                  tileconncompsizes
                    =(conncompsizeT *)ReAlloc(tileconncompsizes,
                                              (ntileconncompmem
                                               *sizeof(conncompsizeT)));
                  for(k=nmemold;k<ntileconncompmem;k++){
                    tileconncompsizes[k].tilenum=tilenum;
                    tileconncompsizes[k].icomptile=k+1;
                    tileconncompsizes[k].icompfull=0;
                    tileconncompsizes[k].npix=0;
                  }
                }

                /* count number of connected components in tile */
                if(tileconncompsizes[iconncomp-1].npix==0){
                  tileconncompsizes[iconncomp-1].icomptile=iconncomp;
                  ntileconncomp++;
                }

                /* count number of pixels in this connected component */
                tileconncompsizes[iconncomp-1].npix++;

                /* keep max number of connected components in any tile */
                if(iconncomp>iconncompmax){
                  iconncompmax=iconncomp;
                }

              }
            }
          }

          /* get more memory for full set of connected components sizes */
          nmemold=nconncompmem;
          if(ntileconncomp>0){
            nconncompmem+=ntileconncomp;
            conncompsizes=(conncompsizeT *)ReAlloc(conncompsizes,
                                                   (nconncompmem
                                                    *sizeof(conncompsizeT)));
          }

          /* store conncomp sizes from tile in full list */
          for(k=0;k<ntileconncompmem;k++){
            if(tileconncompsizes[k].npix>0){
              conncompsizes[nconncomp].tilenum=tileconncompsizes[k].tilenum;
              conncompsizes[nconncomp].icomptile=tileconncompsizes[k].icomptile;
              conncompsizes[nconncomp].icompfull=0;
              conncompsizes[nconncomp].npix=tileconncompsizes[k].npix;
              nconncomp++;
            }
          }

        }else{

          /* second pass */

          /* build lookup table for tile mapping for this tile */
          /* lookup table index is conncomp number minus one */
          for(k=0;k<iconncompmax;k++){
            tilemapping[k]=0;
          }
          for(k=0;k<nconncomp;k++){
            if(conncompsizes[k].tilenum==tilenum){
              iconncomp=conncompsizes[k].icomptile;
              tilemapping[iconncomp-1]=conncompsizes[k].icompfull;
            }
          }

          /* assign final conncomp number to output */
          for(row=0;row<nrow;row++){
            for(col=0;col<ncol;col++){
              iconncomp=tileconncomps[row][col];
              if(iconncomp>0){
                tilerowconncomps[row][colstart+col]=tilemapping[iconncomp-1];
              }else{
                tilerowconncomps[row][colstart+col]=0;
              }
            }
          }

          /* remove temporary files unless told so save them */
          if(params->rmtmptile){
            unlink(readtileoutfiles->conncompfile);
          }

        }

      } /* end loop over tile columns */

      /* write out tile row at end of second pass */
      if(ipass>0){
        for(row=0;row<nrow;row++){
          if(params->conncompouttype==CONNCOMPOUTTYPEUCHAR){
            for(k=0;k<linelen;k++){
              ucharoutbuf[k]=(unsigned char)tilerowconncomps[row][k];
            }
            if(fwrite(ucharoutbuf,sizeof(unsigned char),linelen,outfp)
               !=linelen){
              writeerror=TRUE;
              break;
            }
          }else{
            if(fwrite(tilerowconncomps[row],
                      sizeof(unsigned int),linelen,outfp)!=linelen){
              writeerror=TRUE;
              break;
            }
          }
        }
        if(writeerror){
          fflush(NULL);
          fprintf(sp0,"Error while writing to file %s (device full?)\nAbort\n",
                  realoutfile);
          exit(ABNORMAL_EXIT);
        }
      }

    } /* end loop over tile rows */

    /* at end of first pass, set up tile size array for next pass */
    if(ipass==0){

      /* sort tile size array into descending order */
      qsort(conncompsizes,nconncomp,sizeof(conncompsizeT),
            ConnCompSizeNPixCompare);

      /* keep no more than max number of connected components */
      if(nconncomp>params->maxncomps){
        nconncomp=params->maxncomps;
      }

      /* assign tile mappings */
      for(k=0;k<nconncomp;k++){
        conncompsizes[k].icompfull=k+1;
      }

      /* get memory for tile mapping lookup table */
      tilemapping=(unsigned int *)MAlloc(iconncompmax*sizeof(unsigned int));

    }

  } /* end loop over passes of tile reading */

  /* close output file */
  fprintf(sp1,"Assembled connected components (%ld) output written to file %s\n",
          nconncomp,realoutfile);
  if(fclose(outfp)){
    fflush(NULL);
    fprintf(sp0,"WARNING: problem closing file %s (disk full?)\n",
            realoutfile);
  }

  /* free memory */
  Free2DArray((void **)tileconncomps,ni);
  Free2DArray((void **)tilerowconncomps,ni);
  Free2DArray((void **)ucharbuf,ni);
  free(ucharoutbuf);
  if(ntileconncompmem>0){
    free(tileconncompsizes);
  }
  if(nconncompmem>0){
    free(conncompsizes);
  }
  free(tilemapping);

  /* done */
  return(0);

}


/* function: ConnCompSizeNPixCompare()
 * -----------------------------------
 * Compare npix member of conncompsizeT structures pointed to by
 * inputs for use with qsort() into descending order.
 */
static
int ConnCompSizeNPixCompare(const void *ptr1, const void *ptr2){
  return(((conncompsizeT *)ptr2)->npix-((conncompsizeT *)ptr1)->npix);
}



/* end snaphu_tile.c */
/* snaphu_util.c */
/*************************************************************************

  snaphu utility function source file
  Written by Curtis W. Chen
  Copyright 2002 Board of Trustees, Leland Stanford Jr. University
  Please see the supporting documentation for terms of use.
  No warranty.

*************************************************************************/

/* static (local) function prototypes */
static
int IsTrue(char *str);
static
int IsFalse(char *str);
static
double ModDiff(double f1, double f2);



/* function: IsTrue()
 * ------------------
 * Returns TRUE if the string input is any of TRUE, True, true, 1,
 * y, Y, yes, YES
 */
static
int IsTrue(char *str){

  if(!strcmp(str,"TRUE") || !strcmp(str,"true") || !strcmp(str,"True")
     || !strcmp(str,"1") || !strcmp(str,"y") || !strcmp(str,"Y")
     || !strcmp(str,"yes") || !strcmp(str,"YES") || !strcmp(str,"Yes")){
    return(TRUE);
  }else{
    return(FALSE);
  }
}


/* function: IsFalse()
 * ------------------
 * Returns FALSE if the string input is any of FALSE, False, false,
 * 0, n, N, no, NO
 */
static
int IsFalse(char *str){

  if(!strcmp(str,"FALSE") || !strcmp(str,"false") || !strcmp(str,"False")
     || !strcmp(str,"0") || !strcmp(str,"n") || !strcmp(str,"N")
     || !strcmp(str,"no") || !strcmp(str,"NO") || !strcmp(str,"No")){
    return(TRUE);
  }else{
    return(FALSE);
  }
}


/* function: SetBoolenaSignedChar()
 * --------------------------------
 * Sets the value of a signed character based on the string argument passed.
 * Returns TRUE if the string was not a valid value, FALSE otherwise.
 */
signed char SetBooleanSignedChar(signed char *boolptr, char *str){

  if(IsTrue(str)){
    (*boolptr)=TRUE;
    return(FALSE);
  }else if(IsFalse(str)){
    (*boolptr)=FALSE;
    return(FALSE);
  }
  return(TRUE);
}


/* function: ModDiff()
 * -------------------
 * Computes floating point difference between two numbers.
 * f1 and f2 should be between [0,2pi).  The result is the
 * modulo difference between (-pi,pi].  Assumes that
 * PI and TWOPI have been defined.
 */
static
double ModDiff(double f1, double f2){

  double f3;

  f3=f1-f2;
  if(f3>PI){
    f3-=TWOPI;
  }else if(f3<=-PI){
    f3+=TWOPI;
  }
  return(f3);
}


/* function: WrapPhase()
 * ---------------------
 * Makes sure the passed float array is properly wrapped into the [0,2pi)
 * interval.
 */
int WrapPhase(float **wrappedphase, long nrow, long ncol){

  long row, col;

  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      wrappedphase[row][col]-=TWOPI*floor(wrappedphase[row][col]/TWOPI);
    }
  }
  return(0);
}


/* function: CalcWrappedRangeDiffs()
 * ---------------------------------
 * Computes an array of wrapped phase differences in range (across rows).
 * Input wrapped phase array should be in radians.  Output is in cycles.
 */
int CalcWrappedRangeDiffs(float **dpsi, float **avgdpsi, float **wrappedphase,
                          long kperpdpsi, long kpardpsi,
                          long nrow, long ncol){
  long row, col;
  float **paddpsi;

  for(row=0;row<nrow;row++){
    for(col=0;col<ncol-1;col++){
      dpsi[row][col]=(wrappedphase[row][col+1]-wrappedphase[row][col])/TWOPI;
      if(dpsi[row][col]>=0.5){
        dpsi[row][col]-=1.0;
      }else if(dpsi[row][col]<-0.5){
        dpsi[row][col]+=1.0;
      }
    }
  }
  paddpsi=MirrorPad(dpsi,nrow,ncol-1,(kperpdpsi-1)/2,(kpardpsi-1)/2);
  if(paddpsi==dpsi){
    fflush(NULL);
    fprintf(sp0,"Wrapped-gradient averaging box too large "
            "for input array size\nAbort\n");
    exit(ABNORMAL_EXIT);
  }
  BoxCarAvg(avgdpsi,paddpsi,nrow,ncol-1,kperpdpsi,kpardpsi);
  Free2DArray((void **)paddpsi,nrow+kperpdpsi-1);
  return(0);

}


/* function: CalcWrappedAzDiffs()
 * ---------------------------------
 * Computes an array of wrapped phase differences in range (across rows).
 * Input wrapped phase array should be in radians.  Output is in cycles.
 */
int CalcWrappedAzDiffs(float **dpsi, float **avgdpsi, float **wrappedphase,
                       long kperpdpsi, long kpardpsi, long nrow, long ncol){
  long row, col;
  float **paddpsi;

  for(row=0;row<nrow-1;row++){
    for(col=0;col<ncol;col++){
      dpsi[row][col]=(wrappedphase[row][col]-wrappedphase[row+1][col])/TWOPI;
      if(dpsi[row][col]>=0.5){
        dpsi[row][col]-=1.0;
      }else if(dpsi[row][col]<-0.5){
        dpsi[row][col]+=1.0;
      }
    }
  }
  paddpsi=MirrorPad(dpsi,nrow-1,ncol,(kpardpsi-1)/2,(kperpdpsi-1)/2);
  if(paddpsi==dpsi){
    fflush(NULL);
    fprintf(sp0,"Wrapped-gradient averaging box too large "
            "for input array size\nAbort\n");
    exit(ABNORMAL_EXIT);
  }
  BoxCarAvg(avgdpsi,paddpsi,nrow-1,ncol,kpardpsi,kperpdpsi);
  Free2DArray((void **)paddpsi,nrow-1+kpardpsi-1);
  return(0);

}


/* function: CycleResidue()
 * ------------------------
 * Computes the cycle array of a phase 2D phase array. Input arrays
 * should be type float ** and signed char ** with memory pre-allocated.
 * Numbers of rows and columns in phase array should be passed.
 * Residue array will then have size nrow-1 x ncol-1.  Residues will
 * always be -1, 0, or 1 if wrapped phase is passed in.
 */
int CycleResidue(float **phase, signed char **residue,
                 int nrow, int ncol){

  int row, col;
  float **rowdiff, **coldiff;

  rowdiff=(float **)Get2DMem(nrow-1,ncol,sizeof(float *),sizeof(float));
  coldiff=(float **)Get2DMem(nrow,ncol-1,sizeof(float *),sizeof(float));

  for(row=0;row<nrow-1;row++){
    for(col=0;col<ncol;col++){
      rowdiff[row][col]=ModDiff(phase[row+1][col],phase[row][col]);
    }
  }
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol-1;col++){
      coldiff[row][col]=ModDiff(phase[row][col+1],phase[row][col]);
    }
  }

  for(row=0;row<nrow-1;row++){
    for(col=0;col<ncol-1;col++){
      residue[row][col]=(signed char)LRound((coldiff[row][col]
                                             +rowdiff[row][col+1]
                                             -coldiff[row+1][col]
                                             -rowdiff[row][col])/TWOPI);
    }
  }

  Free2DArray((void **)rowdiff,nrow-1);
  Free2DArray((void **)coldiff,nrow);
  return(0);

}


/* function: NodeResidue()
 * -----------------------
 * Compute residue of node from wrapped phase.  Assumes that memory
 * exists and that row and col are in bounds of the 2-D array of
 * wrapped phase values passed.
 */
int NodeResidue(float **wphase, long row, long col){

  int residue;


  /* compute residue */
  residue=(int )LRound((ModDiff(wphase[row][col+1],wphase[row][col])
                        +ModDiff(wphase[row+1][col+1],wphase[row][col+1])
                        +ModDiff(wphase[row+1][col],wphase[row+1][col+1])
                        +ModDiff(wphase[row][col],wphase[row+1][col]))/TWOPI);

  /* return */
  return(residue);

}


/* function: CalcFlow()
 * --------------------
 * Calculates flow based on unwrapped phase data in a 2D array.
 * Allocates memory for row and column flow arrays.
 */
int CalcFlow(float **phase, short ***flowsptr, long nrow, long ncol){

  long row, col;

  /* get memory for flow arrays */
  if((*flowsptr)==NULL){
    (*flowsptr)=(short **)Get2DRowColMem(nrow,ncol,
                                         sizeof(short *),sizeof(short));
  }

  /* get row flows (vertical phase differences) */
  for(row=0;row<nrow-1;row++){
    for(col=0;col<ncol;col++){
      (*flowsptr)[row][col]=(short)LRound((phase[row][col]-phase[row+1][col])
                                         /TWOPI);
    }
  }

  /* get col flows (horizontal phase differences) */
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol-1;col++){
      (*flowsptr)[nrow-1+row][col]=(short)LRound((phase[row][col+1]
                                                  -phase[row][col])
                                                 /TWOPI);
    }
  }

  /* done */
  return(0);

}


/* function: IntegratePhase()
 * --------------------------
 * This function takes row and column flow information and integrates
 * wrapped phase to create an unwrapped phase field.  The unwrapped
 * phase field will be the same size as the wrapped field.  The array
 * rowflow should have size N-1xM and colflow size NxM-1 where the
 * phase fields are NxM.  Output is saved to a file.
 */
int IntegratePhase(float **psi, float **phi, short **flows,
                   long nrow, long ncol){

  long row, col;
  short **rowflow, **colflow;
  rowflow=flows;
  colflow=&(flows[nrow-1]);

  /* set first element as seed */
  phi[0][0]=psi[0][0];

  /* integrate over first row */
  for(col=1;col<ncol;col++){
    phi[0][col]=phi[0][col-1]+(ModDiff(psi[0][col],psi[0][col-1])
      +colflow[0][col-1]*TWOPI);
  }

  /* integrate over columns */
  for(row=1;row<nrow;row++){
    for(col=0;col<ncol;col++){
      phi[row][col]=phi[row-1][col]+(ModDiff(psi[row][col],psi[row-1][col])
        -rowflow[row-1][col]*TWOPI);
    }
  }

  /* done */
  return(0);

}/* end IntegratePhase() */


/* function: ExtractFlow()
 * -----------------------
 * Given an unwrapped phase array, parse the data and find the flows.
 * Assumes only integer numbers of cycles have been added to get the
 * unwrapped phase from the wrapped pase.  Gets memory and writes
 * wrapped phase to passed pointer.  Assumes flows fit into short ints.
 */
float **ExtractFlow(float **unwrappedphase, short ***flowsptr,
                    long nrow, long ncol){

  long row, col;
  float **wrappedphase;

  /* get memory for wrapped phase array */
  wrappedphase=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));

  /* calculate wrapped phase */
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      /* fmod() gives wrong results here (maybe because of float argument?) */
      /* wrappedphase[row][col]=fmod(unwrappedphase[row][col],TWOPI); */
      wrappedphase[row][col]=unwrappedphase[row][col]
        -TWOPI*floor(unwrappedphase[row][col]/TWOPI);
    }
  }

  /* calculate flows */
  CalcFlow(unwrappedphase,flowsptr,nrow,ncol);

  /* return pointer to wrapped phase array */
  return(wrappedphase);

}


/* function: FlipPhaseArraySign()
 * ------------------------------
 * Flips the sign of all values in a passed array if the flip flag is set.
 * Otherwise, does nothing.
 */
int FlipPhaseArraySign(float **arr, paramT *params, long nrow, long ncol){

  long row, col;

  if(params->flipphasesign){
    for(row=0;row<nrow;row++){
      for(col=0;col<ncol;col++){
        arr[row][col]*=-1;
      }
    }
  }
  return(0);
}


/* function: FlipFlowArraySign()
 * ------------------------------
 * Flips the sign of all values in a row-by-column array if the flip
 * flip flag is set.  Otherwise, does nothing.
 */
int FlipFlowArraySign(short **arr, paramT *params, long nrow, long ncol){

  long row, col, maxcol;

  if(params->flipphasesign){
    for(row=0;row<2*nrow-1;row++){
      if(row<nrow-1){
        maxcol=ncol;
      }else{
        maxcol=ncol-1;
      }
      for(col=0;col<maxcol;col++){
        arr[row][col]=-arr[row][col];
      }
    }
  }
  return(0);
}


/* function: Get2DMem()
 * --------------------
 * Allocates memory for 2D array.
 * Dynamically allocates memory and returns pointer of
 * type void ** for an array of size nrow x ncol.
 * First index is row number: array[row][col]
 * size is size of array element, psize is size of pointer
 * to array element (eg sizeof(float *)).
 */
void **Get2DMem(int nrow, int ncol, int psize, size_t size){

  long row;
  void *baseptr;
  void **arr;

  /* return NULL if sizes are 0 */
  if(nrow < 1 || ncol < 1){
    return(NULL);
  }

  /* allocate memory */
  baseptr=CAlloc(nrow*ncol,size);

  /* set array of pointers to rows */
  arr=(void **)MAlloc(nrow*sizeof(void *));
  for(row=0;row<nrow;row++){
    arr[row]=&(((char *)baseptr)[row*ncol*size]);
  }

  /* return base pointer */
  return(arr);

}


/* function: Get2DRowColMem()
 * --------------------------
 * Allocates memory for 2D array.  The array will have 2*nrow-1 rows.
 * The first nrow-1 rows will have ncol columns, and the rest will
 * have ncol-1 columns.
 */
void **Get2DRowColMem(long nrow, long ncol, int psize, size_t size){

  void **array;

  /* always just function that initializes to zero */
  /* this function exists only for historical interface reasons */
  array = Get2DRowColZeroMem(nrow,ncol,psize,size);
  return(array);

}


/* function: Get2DRowColZeroMem()
 * ------------------------------
 * Allocates memory for 2D array.  The array will have 2*nrow-1 rows.
 * The first nrow-1 rows will have ncol columns, and the rest will
 * have ncol-1 columns.  Memory is initialized to zero.
 */
void **Get2DRowColZeroMem(long nrow, long ncol, int psize, size_t size){

  long row;
  void *baseptr;
  void **arr;

  /* return NULL if sizes are 0 */
  if(nrow < 1 || ncol < 1){
    return(NULL);
  }

  /* allocate memory */
  baseptr=CAlloc((nrow-1)*ncol+nrow*(ncol-1),size);

  /* set array of pointers to rows */
  arr=(void **)MAlloc((2*nrow-1)*sizeof(void *));
  for(row=0;row<nrow-1;row++){
    arr[row]=&(((char *)baseptr)[row*ncol*size]);
  }
  for(row=nrow-1;row<2*nrow-1;row++){
    arr[row]=&(((char *)baseptr)[((nrow-1)*ncol+(row-(nrow-1))*(ncol-1))*size]);
  }

  /* return base pointer */
  return(arr);

}


/* function: MAlloc()
 * --------------------
 * Has same functionality as malloc(), but exits if out of memory.
 */
void *MAlloc(size_t size){

  void *ptr;

  if(size==0){
    return(NULL);
  }
  if((ptr=malloc(size))==NULL){
    fflush(NULL);
    fprintf(sp0,"Out of memory\n");
    exit(ABNORMAL_EXIT);
  }
  return(ptr);
}


/* function: CAlloc()
 * ------------------
 * Has same functionality as calloc(), but exits if out of memory.
 */
void *CAlloc(size_t nitems, size_t size){

  void *ptr;

  if(size==0){
    return(NULL);
  }
  if((ptr=calloc(nitems,size))==NULL){
    fflush(NULL);
    fprintf(sp0,"Out of memory\n");
    exit(ABNORMAL_EXIT);
  }
  return(ptr);
}


/* function: ReAlloc()
 * -------------------
 * Has same functionality as realloc(), but exits if out of memory.
 */
void *ReAlloc(void *ptr, size_t size){

  void *ptr2;

  if(size==0){
    if(ptr!=NULL){
      free(ptr);
    }
    return(NULL);
  }
  if((ptr2=realloc(ptr,size))==NULL){
    fflush(NULL);
    fprintf(sp0,"Out of memory\n");
    exit(ABNORMAL_EXIT);
  }
  return(ptr2);
}


/* function: Free2DArray()
 * -----------------------
 * This function frees the dynamically allocated memory for a 2D
 * array.  Pass in a pointer to a pointer cast to a void **.
 * The function assumes the array is of the form arr[rows][cols]
 * so that nrow is the number of elements in the pointer array.
 */
int Free2DArray(void **array, unsigned int nrow){

  if(array != NULL){
    if(array[0] != NULL){
      free(array[0]);
      array[0] = NULL;
    }
    free(array);
  }
  return(0);

}


/* function: Set2DShortArray()
 * -------------------------
 * Sets all entries of a 2D array of shorts to the given value.  Assumes
 * that memory is already allocated.
 */
int Set2DShortArray(short **arr, long nrow, long ncol, long value){

  long row, col;

  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      arr[row][col]=value;
    }
  }
  return(0);
}


/* function: ValidDataArray()
 * --------------------------
 * Given a 2D floating point array, returns FALSE if any elements are NaN
 * or infinite, and TRUE otherwise.
 */
signed char ValidDataArray(float **arr, long nrow, long ncol){

  long row, col;

  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      if(!IsFinite(arr[row][col])){
        return(FALSE);
      }
    }
  }
  return(TRUE);
}


/* function: NonNegDataArray()
 * ---------------------------
 * Given a 2D floating point array, returns FALSE if any elements are
 * NaN or infinite or negative, and TRUE otherwise.
 */
signed char NonNegDataArray(float **arr, long nrow, long ncol){

  long row, col;

  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      if(arr[row][col] < 0){
        return(FALSE);
      }
    }
  }
  return(TRUE);
}


/* function: IsFinite()
 * --------------------
 * This function takes a double and returns a nonzero value if
 * the arguemnt is finite (not NaN and not infinite), and zero otherwise.
 * Different implementations are given here since not all machines have
 * these functions available.
 */
signed char IsFinite(double d){

#ifdef SNAPHU_USE_FINITE
  return(finite(d));
#else
  return(isfinite(d));
#endif
  /* return(!(isnan(d) || isinf(d))); */
  /* return(TRUE) */
}


/* function: LRound()
 * ------------------
 * Rounds a floating point number to the nearest integer.
 * The function takes a float and returns a long.
 */
long LRound(double a){

  return((long )rint(a));
}


/* function: LMin()
 * ----------------
 * Return the lesser of two long integers as a long.
 */
long LMin(long a, long b){

  if(a<b){
    return(a);
  }else{
    return(b);
  }
}


/* function: LClip()
 * -----------------
 * Clips the input long integer so that it is no less than minval and
 * no greater than maxval, and returns a long integer.
 */
long LClip(long a, long minval, long maxval){

  if(a<minval){
    return(minval);
  }else if(a>maxval){
    return(maxval);
  }else{
    return(a);
  }
}


/* function: Short2DRowColAbsMax()
 * -------------------------------
 * Returns the maximum of the absolute values of element in a
 * two-dimensional short array.  The number of rows and columns
 * should be passed in.
 */
long Short2DRowColAbsMax(short **arr, long nrow, long ncol){

  long row, col, maxval;

  maxval=0;
  for(row=0;row<nrow-1;row++){
    for(col=0;col<ncol;col++){
      if(labs(arr[row][col])>maxval){
        maxval=labs(arr[row][col]);
      }
    }
  }
  for(row=nrow-1;row<2*nrow-1;row++){
    for(col=0;col<ncol-1;col++){
      if(labs(arr[row][col])>maxval){
        maxval=labs(arr[row][col]);
      }
    }
  }
  return(maxval);
}


/* function: LinInterp1D()
 * -----------------------
 * Given an array of floats, interpolates at the specified noninteger
 * index.  Returns first or last array value if index is out of bounds.
 */
float LinInterp1D(float *arr, double index, long nelem){

  long intpart;
  double fracpart;

  intpart=(long )floor(index);
  fracpart=index-intpart;
  if(intpart<0){
    return(arr[0]);
  }else if(intpart>=nelem-1){
    return(arr[nelem-1]);
  }else{
    return(((1-fracpart)*arr[intpart]+fracpart*arr[intpart+1])/2.0);
  }
}


/* function: LinInterp2D()
 * -----------------------
 * Given a 2-D array of floats, interpolates at the specified noninteger
 * indices.  Returns first or last array values if index is out of bounds.
 */
float LinInterp2D(float **arr, double rowind, double colind ,
                  long nrow, long ncol){

  long rowintpart;
  double rowfracpart;

  rowintpart=(long )floor(rowind);
  rowfracpart=rowind-rowintpart;
  if(rowintpart<0){
    return(LinInterp1D(arr[0],colind,ncol));
  }else if(rowintpart>=nrow-1){
    return(LinInterp1D(arr[nrow-1],colind,ncol));
  }else{
    return(((1-rowfracpart)*LinInterp1D(arr[rowintpart],colind,ncol)
            +rowfracpart*LinInterp1D(arr[rowintpart+1],colind,ncol))/2.0);
  }
}


/* function: Despeckle()
 * ---------------------
 * Filters magnitude/power data with adaptive geometric filter to get rid of
 * speckle.  Allocates 2D memory for ei.  Does not square before averaging.
 */
int Despeckle(float **mag, float ***ei, long nrow, long ncol){

  float **intensity;
  double ratio, ratiomax, wfull, wstick, w[NARMS+1];
  long row, col, i, j, k, Irow, Icol;
  short jmin[5]={2,2,0,1,2};
  short jmax[5]={2,3,4,3,2};
  enum{ C=0, T, B, R, L, TR, BL, TL, BR};

  /* get memory for output array */
  if(*ei==NULL){
    (*ei)=(float **)Get2DMem(nrow,ncol,sizeof(float *),sizeof(float));
  }

  /* pad magnitude and place into new array (don't touch original data) */
  intensity=MirrorPad(mag,nrow,ncol,ARMLEN,ARMLEN);
  if(intensity==mag){
    fflush(NULL);
    fprintf(sp0,"Despeckling box size too large for input array size\n"
            "Abort\n");
    exit(ABNORMAL_EXIT);
  }


  /* do the filtering */
  for(row=0;row<nrow;row++){
    Irow=row+ARMLEN;
    for(col=0;col<ncol;col++){
      Icol=col+ARMLEN;

      /* filter only if input is nonzero so we preserve mask info in input */
      if(intensity[Irow][Icol]==0){

        (*ei)[row][col]=0;

      }else{

        for(k=0;k<NARMS+1;k++){
          w[k]=0;
        }
        for(i=-1;i<=1;i++){
          for(j=-1;j<=1;j++){
            w[C]+=intensity[Irow+i][Icol+j];
          }
        }
        for(i=-1;i<=1;i++){
          for(j=2;j<ARMLEN+1;j++){
            w[T]+=intensity[Irow-j][Icol+i];
            w[B]+=intensity[Irow+j][Icol+i];
            w[L]+=intensity[Irow+i][Icol-j];
            w[R]+=intensity[Irow+i][Icol+j];
          }
        }
        for(i=0;i<=4;i++){
          for(j=jmin[i];j<=jmax[i];j++){
            w[TR]+=intensity[Irow-i][Icol+j];
            w[BR]+=intensity[Irow+i][Icol+j];
            w[BL]+=intensity[Irow+i][Icol-j];
            w[TL]+=intensity[Irow-i][Icol-j];
          }
        }
        wfull=w[C]+w[T]+w[R]+w[B]+w[L];
        for(i=2;i<5;i++){
          for(j=2;j<7-i;j++){
            wfull+=intensity[Irow+i][Icol+j];
            wfull+=intensity[Irow-i][Icol+j];
            wfull+=intensity[Irow+i][Icol-j];
            wfull+=intensity[Irow-i][Icol-j];
          }
        }
        ratiomax=1;
        for(k=1;k<=NARMS;k+=2){
          wstick=w[0]+w[k]+w[k+1];
          if((ratio=wstick/(wfull-wstick))<1){
            ratio=1/ratio;
          }
          if(ratio>ratiomax){
            ratiomax=ratio;
            (*ei)[row][col]=wstick;
          }
        }
      }
    }
  }

  /* free memory */
  Free2DArray((void **)intensity,nrow+2*ARMLEN);
  return(0);

}



/* function: MirrorPad()
 * ---------------------
 * Returns pointer to 2D array where passed array is in center and
 * edges are padded by mirror reflections.  If the pad dimensions are
 * too large for the array size, a pointer to the original array is
 * returned.
 */
float **MirrorPad(float **array1, long nrow, long ncol, long krow, long kcol){

  long row, col;
  float **array2;

  /* get memory */
  array2=(float **)Get2DMem(nrow+2*krow,ncol+2*kcol,
                            sizeof(float *),sizeof(float));

  /* center array1 in new array */
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      array2[row+krow][col+kcol]=array1[row][col];
    }
  }

  /* make sure pad is less than image dimensions */
  /* if array is too small for pad dimensions, return original array */
  /* requires checking by calling function */
  if(krow>nrow || kcol>ncol){
    return(array1);
  }

  /* mirror reflect edges */
  for(row=0;row<krow;row++){
    for(col=0;col<kcol;col++){
      array2[row][col]=array2[2*krow-row][2*kcol-col];
      array2[row][ncol+kcol+col]
        =array2[2*krow-row][ncol+kcol-2-col];
      array2[nrow+krow+row][col]
        =array2[nrow+krow-2-row][2*kcol-col];
      array2[nrow+krow+row][ncol+kcol+col]
        =array2[nrow+krow-2-row][ncol+kcol-2-col];
    }
  }
  for(row=krow;row<nrow+krow;row++){
    for(col=0;col<kcol;col++){
      array2[row][col]=array2[row][2*kcol-col];
      array2[row][ncol+kcol+col]
        =array2[row][ncol+kcol-2-col];
    }
  }
  for(col=kcol;col<ncol+kcol;col++){
    for(row=0;row<krow;row++){
      array2[row][col]=array2[2*krow-row][col];
      array2[nrow+krow+row][col]
        =array2[nrow+krow-2-row][col];
    }
  }

  /* return a pointer to the padded array */
  return(array2);

}


/* function: BoxCarAvg()
 * ---------------------
 * Takes in 2-D array, convolves with boxcar filter of size specified.
 * Uses a recursion technique (but the function does not actually call
 * itself recursively) to compute the result, so there may be roundoff
 * errors.
 */
int BoxCarAvg(float **avgarr, float **padarr, long nrow, long ncol,
              long krow, long kcol){

  long i, row, col, n;
  double window;

  /* loop over all rows */
  for(row=0;row<nrow;row++){

    /* calculate first cell */
    window=0;
    for(i=row;i<row+krow;i++){
      for(col=0;col<kcol;col++){
        window+=padarr[i][col];
      }
    }
    avgarr[row][0]=(float )window;

    /* convolve window with row, using result of last cell */
    for(col=1;col<ncol;col++){
      for(i=row;i<row+krow;i++){
        window-=padarr[i][col-1];
        window+=padarr[i][col+kcol-1];
      }
      avgarr[row][col]=(float )window;
    }
  }

  /* normalize */
  n=krow*kcol;
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      avgarr[row][col]/=n;
    }
  }

  /* done */
  return(0);

}


/* function: StrNCopy()
 * --------------------
 * Just like strncpy(), but terminates string by putting a null
 * character at the end (may overwrite last character).
 */
char *StrNCopy(char *dest, const char *src, size_t n){

  char *s;

  s=strncpy(dest,src,n-1);
  dest[n-1]='\0';
  return(s);
}


/* function: FlattenWrappedPhase()
 * -------------------------------
 * Given a wrapped phase array and an unwrapped phase array, subtracts
 * the unwrapped data elementwise from the wrapped data and stores
 * the result, rewrapped to [0,2pi), in the wrapped array.
 */
int FlattenWrappedPhase(float **wrappedphase, float **unwrappedest,
                        long nrow, long ncol){

  long row, col;

  /* loop to subtract, rewrap, store in wrapped array. */
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      wrappedphase[row][col]-=unwrappedest[row][col];
      wrappedphase[row][col]=fmod(wrappedphase[row][col],TWOPI);
      if(wrappedphase[row][col]<0){
        wrappedphase[row][col]+=TWOPI;
      }
    }
  }

  /* done */
  return(0);

}


/* function: Add2DFloatArrays()
 * ----------------------------
 * Addes the values of two 2-D arrays elementwise.
 */
int Add2DFloatArrays(float **arr1, float **arr2, long nrow, long ncol){

  long row, col;

  /* loop over all rows and columns, add and store result in first array */
  for(row=0;row<nrow;row++){
    for(col=0;col<ncol;col++){
      arr1[row][col]+=arr2[row][col];
    }
  }

  /* done */
  return(0);

}


/* function: StringToDouble()
 * --------------------------
 * Uses strtod to convert a string to a double, but also does error checking.
 * If any part of the string is not converted, the function does not make
 * the assignment and returns TRUE.  Otherwise, returns FALSE.
 */
int StringToDouble(char *str, double *d){

  double tempdouble;
  char *endp;

  endp=str;
  tempdouble=strtod(str,&endp);
  if(strlen(endp) || tempdouble>=HUGE_VAL || tempdouble<=-HUGE_VAL){
    return(TRUE);
  }else{
    *d=tempdouble;
    return(FALSE);
  }
}


/* function: StringToLong()
 * ------------------------
 * Uses strtol to convert a string to a base-10 long, but also does error
 * checking.  If any part of the string is not converted, the function does
 * not make the assignment and returns TRUE.  Otherwise, returns FALSE.
 */
int StringToLong(char *str, long *l){

  long templong;
  char *endp;

  endp=str;
  templong=strtol(str,&endp,10);
  if(strlen(endp) || templong==LONG_MAX || templong==LONG_MIN){
    return(TRUE);
  }else{
    *l=templong;
    return(FALSE);
  }
}


/* function: CatchSignals()
 * ------------------------
 * Traps common signals that by default cause the program to abort.
 * Sets (pointer to function) Handler as the signal handler for all.
 * Note that SIGKILL usually cannot be caught.  No return value.
 */
int CatchSignals(void (*SigHandler)(int)){

  signal(SIGHUP,SigHandler);
  signal(SIGINT,SigHandler);
  signal(SIGQUIT,SigHandler);
  signal(SIGILL,SigHandler);
  signal(SIGABRT,SigHandler);
  signal(SIGFPE,SigHandler);
  signal(SIGSEGV,SigHandler);
  signal(SIGPIPE,SigHandler);
  signal(SIGALRM,SigHandler);
  signal(SIGTERM,SigHandler);
  signal(SIGBUS,SigHandler);
  return(0);

}


/* function: SetDump()
 * -------------------
 * Set the global variable dumpresults_global to TRUE if SIGINT or SIGHUP
 * signals recieved.  Also sets requestedstop_global if SIGINT signal
 * received.  This function should only be called via signal() when
 * a signal is caught.
 */
void SetDump(int signum){

  if(signum==SIGINT){

    /* exit if we receive another interrupt */
    signal(SIGINT,exit);

    /* print nice message and set global variables so program knows to exit */
    fflush(NULL);
    fprintf(sp0,"\n\nSIGINT signal caught.  Please wait for graceful exit\n");
    fprintf(sp0,"(One more interrupt signal halts job)\n");
    dumpresults_global=TRUE;
    requestedstop_global=TRUE;

  }else if(signum==SIGHUP){

    /* make sure the hangup signal doesn't revert to default behavior */
    signal(SIGHUP,SetDump);

    /* print a nice message, and set the dump variable */
    fflush(NULL);
    fprintf(sp0,"\n\nSIGHUP signal caught.  Dumping results\n");
    dumpresults_global=TRUE;

  }else{
    fflush(NULL);
    fprintf(sp0,"WARNING: Invalid signal (%d) passed to signal handler\n",
            signum);
  }

  /* done */
  return;

}


/* function: KillChildrenExit()
 * ----------------------------
 * Signal handler that sends a KILL signal to all processes in the group
 * so that children exit when parent exits.
 */
void KillChildrenExit(int signum){

  fflush(NULL);
  fprintf(sp0,"Parent received signal %d\nKilling children and exiting\n",
          signum);
  fflush(NULL);
  signal(SIGTERM,SIG_IGN);
  kill(0,SIGTERM);
  exit(ABNORMAL_EXIT);

  /* done */
  return;

}


/* function: SignalExit()
 * ----------------------
 * Signal hanlder that prints message about the signal received, then exits.
 */
void SignalExit(int signum){

  signal(SIGTERM,SIG_IGN);
  fflush(NULL);
  fprintf(sp0,"Exiting with status %d on signal %d\n",ABNORMAL_EXIT,signum);
  fflush(NULL);
  exit(ABNORMAL_EXIT);

  /* done */
  return;

}


/* function: StartTimers()
 * -----------------------
 * Starts the wall clock and CPU timers for use in conjunction with
 * DisplayElapsedTime().
 */
int StartTimers(time_t *tstart, double *cputimestart){

  struct rusage usagebuf;

  *tstart=time(NULL);
  *cputimestart=-1.0;
  if(!getrusage(RUSAGE_SELF,&usagebuf)){
    *cputimestart=(double )(usagebuf.ru_utime.tv_sec
                            +(usagebuf.ru_utime.tv_usec/(double )1000000)
                            +usagebuf.ru_stime.tv_sec
                            +(usagebuf.ru_stime.tv_usec/(double )1000000));
    if(!getrusage(RUSAGE_CHILDREN,&usagebuf)){
      *cputimestart+=(double )(usagebuf.ru_utime.tv_sec
                               +(usagebuf.ru_utime.tv_usec/(double )1000000)
                               +usagebuf.ru_stime.tv_sec
                               +(usagebuf.ru_stime.tv_usec/(double )1000000));
    }
  }

  /* done */
  return(0);

}


/* function: DisplayElapsedTime()
 * ------------------------------
 * Displays the elapsed wall clock and CPU times for the process and its
 * children.  Times should be initialized at the start of the program with
 * StartTimers().  The code is written to show the total processor time
 * for the parent process and all of its children, but whether or not
 * this is actually done depends on the implementation of the system time
 * functions.
 */
int DisplayElapsedTime(time_t tstart, double cputimestart){

  double cputime, walltime, seconds;
  long hours, minutes;
  time_t tstop;
  struct rusage usagebuf;

  cputime=-1.0;
  if(!getrusage(RUSAGE_CHILDREN,&usagebuf)){
    cputime=(double )(usagebuf.ru_utime.tv_sec
                       +(usagebuf.ru_utime.tv_usec/(double )1000000)
                       +usagebuf.ru_stime.tv_sec
                       +(usagebuf.ru_stime.tv_usec/(double )1000000));
    if(!getrusage(RUSAGE_SELF,&usagebuf)){
      cputime+=(double )(usagebuf.ru_utime.tv_sec
                         +(usagebuf.ru_utime.tv_usec/(double )1000000)
                         +usagebuf.ru_stime.tv_sec
                         +(usagebuf.ru_stime.tv_usec/(double )1000000));
    }
  }
  tstop=time(NULL);
  if(cputime>0 && cputimestart>=0){
    cputime-=cputimestart;
    hours=(long )floor(cputime/3600);
    minutes=(long )floor((cputime-3600*hours)/60);
    seconds=cputime-3600*hours-60*minutes;
    fprintf(sp1,"Elapsed processor time:   %ld:%02ld:%05.2f\n",
            hours,minutes,seconds);
  }
  if(tstart>0 && tstop>0){
    walltime=tstop-tstart;
    hours=(long )floor(walltime/3600);
    minutes=(long )floor((walltime-3600*hours)/60);
    seconds=walltime-3600*hours-60*minutes;
    fprintf(sp1,"Elapsed wall clock time:  %ld:%02ld:%02ld\n",
            hours,minutes,(long )seconds);
  }

  /* done */
  return(0);

}


/* function: LongCompare()
 * -----------------------
 * Compares two long integers.  For use with qsort().
 */
int LongCompare(const void *c1, const void *c2){
  return((*((long *)c1))-(*((long *)c2)));
}


/* end snaphu_util.c */
