(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        (i32.const 97) ;; 'a'
        (call $output)
    )
)
