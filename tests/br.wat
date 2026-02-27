(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        block
            i32.const 0x61 ;; 'a'
            call $output

            br 0

            i32.const 0x62 ;; 'b'
            call $output
        end
    )
)
