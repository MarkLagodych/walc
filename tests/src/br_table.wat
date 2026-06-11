(module
    (import "walc" "output" (func $output (param i32)))
    (export "main" (func $main))
    (func $main (local $i i32)
        loop $loop

            i32.const 5
            local.get $i
            i32.eq
            if
                return
            else
            end

            block $block0
                block $block1
                    block $block2

                        local.get $i
                        br_table $block2 $block1 $block0

                    end

                    i32.const 0x61 ;; 'a'
                    call $output

                    local.get $i
                    i32.const 1
                    i32.add
                    local.set $i
                    br $loop

                end

                i32.const 0x62 ;; 'b'
                call $output

                local.get $i
                i32.const 1
                i32.add
                local.set $i
                br $loop
            end

            i32.const 0x63 ;; 'c'
            call $output

            local.get $i
            i32.const 1
            i32.add
            local.set $i
            br $loop

        end
    )
)
