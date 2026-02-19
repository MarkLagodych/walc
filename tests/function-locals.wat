(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        (i32.const 97) ;; 'a'
        (call $f)
        (call $output)
    )
    (func $f (param i32) (result i32) (local i32)
        local.get 0
        local.set 1
        local.get 1
    )
)
