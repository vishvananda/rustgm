extern unsigned char read_at_runtime(unsigned char);
extern unsigned char read_nested(void);

int main(void) {
    return read_at_runtime(5) == 1 && read_at_runtime(7) == 3
        && read_nested() == 9 ? 0 : 1;
}
