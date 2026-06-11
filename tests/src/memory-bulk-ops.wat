;; This tests LIME1 bulk-memory-opt extensions.
(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (memory $mem0 1)
    (func $main
        i32.const 0
        i32.const 0x123456AB
        i32.const 4
        memory.fill

        i32.const 0
        i32.load

        i32.const 0xABABABAB
        call $compare32

        i32.const 4
        i32.load
        i32.const 0
        call $compare32

        i32.const 4
        i32.const 0
        i32.const 4
        memory.copy

        i32.const 4
        i32.load
        i32.const 0xABABABAB
        call $compare32
    )
    (func $compare32 (param i32 i32)
        local.get 0
        local.get 1
        i32.eq
        if (result i32)
            i32.const 0x59 ;; 'Y'
        else
            i32.const 0x4E ;; 'N'
        end

        call $output
    )
    (func $compare64 (param i64 i64)
        local.get 0
        local.get 1
        i64.eq
        if (result i32)
            i32.const 0x59 ;; 'Y'
        else
            i32.const 0x4E ;; 'N'
        end

        call $output
    )
)
