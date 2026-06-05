(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main
        i32.const 0
        if
            i32.const 0x61 ;; 'a'
            call $output
        else
            i32.const 0x62 ;; 'b'
            call $output
        end

        i32.const 1
        if
            i32.const 0x61 ;; 'a'
            call $output
        else
            i32.const 0x62 ;; 'b'
            call $output
        end
    )
)
