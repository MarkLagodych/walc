(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main (local $x i32) (local $y i32)
        (i32.const 0x61) ;; 'a'
        (call $f)
        (call $output)

        i32.const 123

        local.tee $x

        local.get $x
        i32.const 123
        i32.eq
        i32.const 1
        call $compare32

        ;; Check what's on the stack
        i32.const 123
        i32.eq
        i32.const 1
        call $compare32

        i32.const 0
        local.get $y
        i32.eq
        i32.const 1
        call $compare32

        i32.const 321
        local.tee $y

        local.get $y
        i32.const 321
        i32.eq
        i32.const 1
        call $compare32

        i32.const 321
        i32.eq
        i32.const 1
        call $compare32
    )
    (func $f (param $p i32) (result i32) (local $x i32)
        local.get $p
        local.set $x
        local.get $x
        return
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
)
