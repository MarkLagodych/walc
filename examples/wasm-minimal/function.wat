(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        (i32.const 97) ;; 'a'
        (call $f)
        (call $output)
    )
    (func $f (param i32) (result i32)
        local.get 0
    )
)
