extern unsigned inherited_slice(void);
extern unsigned inherited_array(void);
extern unsigned scoped_slice(void);

int main(void) {
    return inherited_slice() == 23 && inherited_array() == 37 && scoped_slice() == 61 ? 0 : 1;
}
