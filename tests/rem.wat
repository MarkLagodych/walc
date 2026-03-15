(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        i32.const 15
        i32.const 4
        i32.rem_u

        i32.const 3
        call $compare32

        i32.const 0x80000000
        i32.const 2
        i32.rem_u

        i32.const 0
        call $compare32

        i32.const 1000
        i32.const 999
        i32.rem_u

        i32.const 1
        call $compare32

        i32.const -15
        i32.const -4
        i32.rem_s

        i32.const -3
        call $compare32

        i32.const -80 ;; -77 + -3
        i32.const 11
        i32.rem_s

        i32.const -3
        call $compare32

        i32.const 0x80000000 ;; -2^31
        i32.const -1
        i32.rem_s ;; Should not trap (unlike division)
        
        i32.const 0
        call $compare32

        i32.const 1
        i32.const 0
        i32.rem_u ;; Should trap
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
