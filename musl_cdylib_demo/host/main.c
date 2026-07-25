// Compile with:
//   musl-gcc -o host main.c -ldl
#include <stdio.h>
#include <dlfcn.h>

typedef void (*run_fn)(void);

int main(void) {
    // RTLD_NOW resolves all symbols at load time, so any
    // unresolved libc symbol in the .so fails loudly here
    // rather than crashing later mid-call.
    void *handle = dlopen("./libmusl_cdylib_demo.so", RTLD_NOW);
    if (!handle) {
        fprintf(stderr, "dlopen failed: %s\n", dlerror());
        return 1;
    }

    dlerror(); // clear any existing error
    run_fn run;
    *(void **)(&run) = dlsym(handle, "run");
    char *err = dlerror();
    if (err) {
        fprintf(stderr, "dlsym failed: %s\n", err);
        dlclose(handle);
        return 1;
    }

    run();

    dlclose(handle);
    return 0;
}
