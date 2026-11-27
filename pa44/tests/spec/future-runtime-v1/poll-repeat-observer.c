#include <signal.h>
#include <sys/wait.h>
#include <sys/resource.h>
#include <unistd.h>
extern unsigned char observe(void);
int main(void) {
    struct rlimit limit = {0, 0};
    if (setrlimit(RLIMIT_CORE, &limit) != 0) return 4;
    pid_t child = fork();
    if (child < 0) return 1;
    if (child == 0) { (void)observe(); _exit(2); }
    int status = 0;
    if (waitpid(child, &status, 0) != child) return 3;
    return !(WIFSIGNALED(status) && WTERMSIG(status) == SIGILL);
}
