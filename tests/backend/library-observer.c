/* Original RustGM maintainer input: an independent C caller of the object
 * that library.c compiled in process. */
extern const long rg_library_seed;
long rg_library_answer(long);

int main(void)
{
	if (rg_library_seed != 40)
		return 1;
	if (rg_library_answer(2) != 42)
		return 2;
	if (rg_library_answer(-40) != 0)
		return 3;
	return 0;
}
