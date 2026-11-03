//#Object:merge-data-segments2.c

//#Config:no-merge:default
//#LinkArgs:--no-merge-data-segments

static const char ro[] = "ab";
int da = 0x11;
int bss_a;
extern const char ro2[];
extern int db;
extern int bss_b;
static const int *const p = &da;
static const int *const q = &db;

void _start(void) {
  if (ro[0] != 'a' || ro[1] != 'b' || ro[2] != '\0') {
    __builtin_trap();
  }
  if (ro2[0] != 'c' || ro2[1] != 'd' || ro2[2] != '\0') {
    __builtin_trap();
  }
  if (da != 0x11 || db != 0x22 || *p != 0x11 || *q != 0x22) {
    __builtin_trap();
  }
  if (bss_a != 0 || bss_b != 0) {
    __builtin_trap();
  }
  bss_a = 3;
  bss_b = 4;
  if (bss_a != 3 || bss_b != 4) {
    __builtin_trap();
  }
}
