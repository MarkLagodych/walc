(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        i32.const 1
        i64.extend_i32_u

        i64.const 1
        call $compare64

        i32.const 0xFFFFFFFF
        i64.extend_i32_u

        i64.const 0xFFFFFFFF
        call $compare64

        i32.const 123
        i64.extend_i32_s

        i64.const 123
        call $compare64

        i32.const 0x7FFFFFFF
        i64.extend_i32_s

        i64.const 0x7FFFFFFF
        call $compare64

        i32.const 0x81234567
        i64.extend_i32_s

        i64.const 0xFFFFFFFF81234567
        call $compare64

        i32.const 123
        i32.extend8_s

        i32.const 123
        call $compare32

        i32.const 0x7F
        i32.extend8_s

        i32.const 0x7F
        call $compare32

        i32.const 0x80
        i32.extend8_s

        i32.const 0xFFFFFF80
        call $compare32


        i64.const 0x0123456789ABCDEF
        i32.wrap_i64

        i32.const 0x89ABCDEF
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
