(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (memory $mem0 1)
    (func $main
        i32.const 12 ;; address
        i32.const 0x61 ;; 'a'
        i32.store8 offset=34

        i32.const 34 ;; address
        i32.load8_u offset=12

        call $output
    )
)
