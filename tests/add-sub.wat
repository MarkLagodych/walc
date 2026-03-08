(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        i32.const 3
        i32.const 4
        i32.add

        i32.const 7
        call $compare32

        i32.const 3
        i32.const 4
        i32.sub

        i32.const -1
        call $compare32

        i32.const 0
        i32.const 1
        i32.sub

        i32.const -1
        call $compare32

        i64.const 5
        i64.const -10
        i64.add

        i64.const -5
        call $compare64

        i64.const 0
        i64.const 10
        i64.sub

        i64.const -10
        call $compare64

        i64.const 5
        i64.const -10
        i64.sub

        i64.const 15
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
