(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main (local $x i32)
        i32.const 1
        local.set $x

        loop $l
            i32.const 0x61 ;; 'a'
            call $output

            local.get $x
            if
                i32.const 0
                local.set $x
                br $l ;; continue
            end

            i32.const 0x62 ;; 'b'
            call $output
        end
    )
)
