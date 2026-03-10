(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        i32.const 3
        i32.const 4
        i32.gt_u

        i32.const 0
        call $compare32

        i32.const 4
        i32.const 3
        i32.gt_u

        i32.const 1
        call $compare32

        i32.const -3
        i32.const -4
        i32.lt_s

        i32.const 0
        call $compare32

        i32.const -4
        i32.const -3
        i32.lt_s

        i32.const 1
        call $compare32

        i64.const 3
        i64.const 4
        i64.gt_u

        i32.const 0
        call $compare32

        i64.const 4
        i64.const 3
        i64.gt_u

        i32.const 1
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
