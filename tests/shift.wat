(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        i32.const 3
        i32.const 1
        i32.shl

        i32.const 6
        call $compare

        i32.const -1
        i32.const 1
        i32.shl

        i32.const -2
        call $compare

        i32.const 3
        i32.const 1
        i32.shr_u

        i32.const 1
        call $compare

        i32.const 3
        i32.const 1
        i32.shr_s

        i32.const 1
        call $compare

        i32.const 0xFFFFFFFF
        i32.const 1
        i32.shr_u

        i32.const 0x7FFFFFFF
        call $compare

        i32.const 0xFFFFFFFF
        i32.const 1
        i32.shr_s

        i32.const 0xFFFFFFFF
        call $compare

    )
    (func $compare (param i32 i32)
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
)
