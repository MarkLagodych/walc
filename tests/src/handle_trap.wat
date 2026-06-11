(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        i32.const 0x51 ;; Q
        call $output

        i32.const 0
        i32.const 0
        i32.div_u ;; trap: division by zero
        drop

        ;; this should not execute
        i32.const 0x52 ;; R
        call $output
    )
    (export "handle_trap" (func $handle_trap))
    (func $handle_trap (param $trap_code i32)
        local.get $trap_code
        i32.const 0x61 ;; 'a'
        i32.add
        call $output ;; print('a' + trap_code)
        ;; must be 'b' (division error must be 1)
    )
)
