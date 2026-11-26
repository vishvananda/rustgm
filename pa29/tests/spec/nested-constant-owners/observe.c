extern unsigned char read_free(void);
extern unsigned char read_associated(void);

int main(void) {
    return read_free() == 11 && read_associated() == 17 ? 0 : 1;
}
