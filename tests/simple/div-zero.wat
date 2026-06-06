(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        i32.const 0x61 ;; 'a'
        call $output

        i32.const 1
        i32.const 0
        i32.div_u ;; Should trap
        drop

        i32.const 0x4E ;; 'N'
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
)
