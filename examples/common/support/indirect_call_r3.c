typedef void (*tc32_fn_t)(void);

void __tc32_indirect_call_r3(tc32_fn_t f) {
    f();
}
