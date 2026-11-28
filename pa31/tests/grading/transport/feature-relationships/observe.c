extern unsigned char read_implied(void);
extern unsigned char read_renamed(void);

int main(void) { return read_implied() == 2 && read_renamed() == 3 ? 0 : 1; }
