(module
    ;; mem_ptr, size
    (import "walc" "print" (func $print (param i32) (param i32)))
    (export "main" (func $main))
    (func $main
        (i32.const 123)
        (i32.const 456)
        (call $print)
    )
)
