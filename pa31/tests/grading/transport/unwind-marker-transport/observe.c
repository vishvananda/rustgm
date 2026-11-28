extern unsigned char read_owned(void);
extern unsigned char read_shared(void);
extern unsigned char read_shared_safe(void);

int main(void) {
    return read_owned() == 7 && read_shared() == 7 && read_shared_safe() == 11
        ? 0 : 1;
}
