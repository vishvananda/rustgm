extern int entry(void);

int main(void) {
    return entry() == 42 ? 0 : 1;
}
