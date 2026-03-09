(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        i32.const 3
        i32.const 4
        i32.mul

        i32.const 12
        call $compare32

        i64.const -3
        i64.const -4
        i64.mul

        i64.const 12
        call $compare64

        i32.const 0x7FFFFFFF
        i32.const 2
        i32.mul

        i32.const 0xFFFFFFFE
        call $compare32

        i64.const 0x7FFFFFFF_FFFFFFFF
        i64.const 2
        i64.mul

        i64.const 0xFFFFFFFF_FFFFFFFE
        call $compare64
    )
    (func $print32 (param i32)
        local.get 0

        i32.const 0xf
        i32.and
        i32.const 0x61
        i32.add
        call $output

        local.get 0
        i32.const 8
        i32.shr_u
        local.tee 0

        i32.const 0xf
        i32.and
        i32.const 0x61
        i32.add
        call $output

        local.get 0
        i32.const 8
        i32.shr_u
        local.tee 0

        i32.const 0xf
        i32.and
        i32.const 0x61
        i32.add
        call $output
        local.get 0
        i32.const 8
        i32.shr_u
        local.tee 0

        i32.const 0xf
        i32.and
        i32.const 0x61
        i32.add
        call $output
        local.get 0
        i32.const 8
        i32.shr_u
        local.tee 0

        i32.const 0xf
        i32.and
        i32.const 0x61
        i32.add
        call $output
        local.get 0
        i32.const 8
        i32.shr_u
        local.tee 0

        i32.const 0xf
        i32.and
        i32.const 0x61
        i32.add
        call $output
        local.get 0
        i32.const 8
        i32.shr_u
        local.tee 0

        i32.const 0xf
        i32.and
        i32.const 0x61
        i32.add
        call $output
        local.get 0
        i32.const 8
        i32.shr_u
        local.tee 0

        i32.const 0xf
        i32.and
        i32.const 0x61
        i32.add
        call $output

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
