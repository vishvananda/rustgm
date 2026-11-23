#include <stdio.h>

#include "lowir_interface.h"

static int compile_main(CppgmLowir *module)
{
    uint32_t symbol = cppgm_lowir_symbol(module, "main", 4);
    uint32_t signature = cppgm_lowir_signature(
        module, CPPGM_TYPE_I32, NULL, 0, CPPGM_ARITY_FIXED);
    const LowirBodyCounts counts = {.instructions = 1, .literals = 1, .blocks = 1};
    LowirBodyBuffers body;
    if (symbol == CPPGM_NO_ID || signature == CPPGM_NO_ID ||
        !cppgm_lowir_function(module, symbol, signature, CPPGM_BINDING_STRONG,
                              CPPGM_FUNCTION_DEFINITION) ||
        !cppgm_lowir_body_reserve(module, symbol, &counts, &body))
        return 0;

    /* A single block returns the integer at index zero in the literal table. */
    body.layout[0] = 0;
    body.literals[0] = (LowirLiteral){.low = 42};
    body.instructions[0] = (LowirInstruction){
        .op = CPPGM_OP_RETURN,
        .result = CPPGM_NO_ID,
        .type = CPPGM_TYPE_I32,
        .ops = {{.kind = CPPGM_VALUE_INTEGER, .index = 0}},
    };
    /* Returning a literal defines no temporaries. */
    return cppgm_lowir_body_commit(module, symbol, &counts, 0) &&
           cppgm_lowir_validate(module) &&
           cppgm_lowir_compile(module, 1, CPPGM_CODE_PIC);
}

static int write_object(CppgmLowir *module, const char *path)
{
    FILE *file = fopen(path, "wb");
    if (file == NULL) {
        perror(path);
        return 0;
    }
    /* The object bytes are owned by the module until it is destroyed. */
    uint64_t size = cppgm_lowir_object_size(module);
    int written = size != 0 && fwrite(cppgm_lowir_object_data(module), 1, size, file) == size;
    int closed = fclose(file) == 0;
    if (!written || !closed)
        fprintf(stderr, "cannot write object to %s\n", path);
    return written && closed;
}

int main(int argc, char **argv)
{
    if (argc != 2) {
        fprintf(stderr, "usage: %s OUTPUT.o\n", argv[0]);
        return 2;
    }
    CppgmLowir *module = cppgm_lowir_create(CPPGM_LOWIR_ABI_VERSION, CPPGM_TARGET_LINUX);
    if (module == NULL) {
        fputs("cannot initialize the CPPGM LowIR library\n", stderr);
        return 1;
    }
    int success = compile_main(module);
    if (success)
        success = write_object(module, argv[1]);
    else
        fprintf(stderr, "%s\n", cppgm_lowir_error(module));
    cppgm_lowir_destroy(module);
    return success ? 0 : 1;
}
