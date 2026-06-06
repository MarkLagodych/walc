(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        i64.const 0x00F123456
        i64.const 0x0F0114448
        i64.or

        i64.const 0x0FF13745E
        call $compare64
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
