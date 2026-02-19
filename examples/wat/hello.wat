(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        (i32.const 0x48) ;; 'H'
        (call $output)
        (i32.const 0x65) ;; 'e'
        (call $output)
        (i32.const 0x6C) ;; 'l'
        (call $output)
        (i32.const 0x6C) ;; 'l'
        (call $output)
        (i32.const 0x6F) ;; 'o'
        (call $output)
        (i32.const 0x20) ;; ' '
        (call $output)
        (i32.const 0x77) ;; 'w'
        (call $output)
        (i32.const 0x6F) ;; 'o'
        (call $output)
        (i32.const 0x72) ;; 'r'
        (call $output)
        (i32.const 0x6C) ;; 'l'
        (call $output)
        (i32.const 0x64) ;; 'd'
        (call $output)
        (i32.const 0x21) ;; '!'
        (call $output)
        (i32.const 0x0A) ;; newline
        (call $output)
    )
)
