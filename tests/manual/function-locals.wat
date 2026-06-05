(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        (i32.const 0x61) ;; 'a'
        (call $f)
        (call $output)
    )
    (func $f (param $p i32) (result i32) (local $x i32)
        local.get $p
        local.set $x
        local.get $x
        return
    )
)
