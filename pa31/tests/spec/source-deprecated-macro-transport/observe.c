extern unsigned char read_direct(void);
extern unsigned char read_alias(void);
extern unsigned char read_scheduled(void);
extern unsigned char read_modern(void);
extern unsigned char read_legacy(void);

int main(void) {
    return read_direct() == 7 && read_alias() == 7 && read_scheduled() == 8
        && read_modern() == 9 && read_legacy() == 10 ? 0 : 1;
}
