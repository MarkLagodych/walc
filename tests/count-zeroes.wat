(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        i32.const 0
        i32.ctz
        i32.const 32
        call $compare32

        i32.const 1
        i32.ctz
        i32.const 0
        call $compare32

        i32.const 0x8
        i32.ctz
        i32.const 3
        call $compare32

        i32.const 0x80000000
        i32.ctz
        i32.const 31
        call $compare32


        i32.const 0
        i32.clz
        i32.const 32
        call $compare32

        i32.const 0x80000000
        i32.clz
        i32.const 0
        call $compare32

        i32.const 1
        i32.clz
        i32.const 31
        call $compare32


        i64.const 0
        i64.ctz
        i64.const 64
        call $compare64

        i64.const 1
        i64.ctz
        i64.const 0
        call $compare64


        i64.const 0xf
        i64.clz
        i64.const 60
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
