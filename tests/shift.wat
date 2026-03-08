(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        i32.const 3
        i32.const 1
        i32.shl

        i32.const 6
        call $compare32

        i32.const -1
        i32.const 1
        i32.shl

        i32.const -2
        call $compare32

        i32.const 3
        i32.const 1
        i32.shr_u

        i32.const 1
        call $compare32

        i32.const 3
        i32.const 1
        i32.shr_s

        i32.const 1
        call $compare32

        i32.const 0xFFFFFFFF
        i32.const 1
        i32.shr_u

        i32.const 0x7FFFFFFF
        call $compare32

        i32.const 0xFFFFFFFF
        i32.const 1
        i32.shr_s

        i32.const 0xFFFFFFFF
        call $compare32

        i64.const 3
        i64.const 1
        i64.shl

        i64.const 6
        call $compare64

        i64.const -1
        i64.const 1
        i64.shl

        i64.const -2
        call $compare64

        i64.const 3
        i64.const 1
        i64.shr_u

        i64.const 1
        call $compare64

        i64.const 3
        i64.const 1
        i64.shr_s

        i64.const 1
        call $compare64

        i64.const 0xFFFFFFFFFFFFFFFF
        i64.const 1
        i64.shr_u

        i64.const 0x7FFFFFFFFFFFFFFF
        call $compare64

        i64.const 0xFFFFFFFFFFFFFFFF
        i64.const 1
        i64.shr_s

        i64.const 0xFFFFFFFFFFFFFFFF
        call $compare64

        i32.const 0x80000000
        i32.const 31
        i32.shr_u

        i32.const 1
        call $compare32

        i64.const 0x8000000000000000
        i64.const 63
        i64.shr_u

        i64.const 1
        call $compare64

        i64.const 0x8000000000000000
        i64.const -1
        i64.shr_u

        i64.const 1
        call $compare64

        i64.const 0x8000000000000000
        i64.const 1
        i64.shr_u

        i64.const 0x4000000000000000
        call $compare64

        i64.const 0x8000000000000000
        i64.const 1
        i64.shr_s

        i64.const 0xC000000000000000
        call $compare64

        i64.const 0x8000000000000000
        i64.const 1
        i64.shl

        i64.const 0
        call $compare64

        i64.const 0x8000000000000000
        i64.const 1
        i64.shl
        i64.const 1
        i64.shr_u

        i64.const 0
        call $compare64

        i32.const 1
        i32.const 31
        i32.shl

        i32.const 0x80000000
        call $compare32

        i64.const 1
        i64.const 63
        i64.shl

        i64.const 0x8000000000000000
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
