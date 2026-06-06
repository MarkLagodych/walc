(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        i32.const 0
        i32.popcnt
        i32.const 0
        call $compare32

        i32.const 0x80000000
        i32.popcnt
        i32.const 1
        call $compare32

        i32.const 0x0f0f0f0f
        i32.popcnt
        i32.const 16
        call $compare32

        i64.const 0
        i64.popcnt
        i64.const 0
        call $compare64

        i64.const 0x8000000000000000
        i64.popcnt
        i64.const 1
        call $compare64

        i64.const 1
        i64.popcnt
        i64.const 1
        call $compare64

        i64.const 0xffffffffffffffff
        i64.popcnt
        i64.const 64
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
