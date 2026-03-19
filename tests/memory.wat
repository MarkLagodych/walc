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

        i32.const 0x61
        call $compare32


        i32.const 0
        i32.const 0x8A
        i32.store8

        i32.const 0
        i32.load8_s

        i32.const 0xFFFFFF8A
        call $compare32


        i32.const 0
        i32.const 0x12345678
        i32.store

        i32.const 0
        i32.load

        i32.const 0x12345678
        call $compare32


        i32.const 0
        i64.const 0x0123456789ABCDEF
        i64.store

        i32.const 0
        i64.load

        i64.const 0x0123456789ABCDEF
        call $compare64


        i32.const 0
        i32.const 0xABCD8765

        i32.store16 offset=3

        i32.const 0
        i32.load16_s offset=3

        i32.const 0xFFFF8765
        call $compare32

        i32.const 0
        i32.load16_u offset=3

        i32.const 0x8765
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
