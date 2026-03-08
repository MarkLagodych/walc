(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        i32.const 0x80000000
        i32.const 1
        i32.rotl

        i32.const 0x00000001
        call $compare32

        i32.const 0x80000000
        i32.const 1
        i32.rotr

        i32.const 0x40000000
        call $compare32

        i64.const 0x80000000_00000000
        i64.const 1
        i64.rotl

        i64.const 0x00000000_00000001
        call $compare64

        i64.const 0x80000000_00000001
        i64.const 1
        i64.rotr

        i64.const 0xE0000000_00000000
        call $compare64

        i64.const 0x80000000_00000000
        i64.const 1
        i64.rotr

        i64.const 0x40000000_00000000
        ;; FIXME this fails here!!!
        call $compare64

        i32.const 0xDEADBEEF
        i32.const 4
        i32.rotr

        i32.const 0xFDEADBEE
        call $compare32

        i64.const 0x000000BADB00B1E5
        i64.const 36
        i64.rotl

        i64.const 0xB00B1E5000000BAD
        call $compare64

        i64.const 0x000000BADB00B1E5
        i64.const -36
        i64.rotr

        i64.const 0xB00B1E5000000BAD
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
