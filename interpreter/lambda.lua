---A simple lambda interpreter implemented using a simple and efficient
---parser and absolutely no recursion.


---@alias Term
---| { parent:Term|nil, type: "variable", [1]: string }
---| { parent:Term|nil, type: "lambda", [1]: string, [2]: Term }
---| { parent:Term|nil, type: "application", [1]: Term, [2]: Term }

local BRACKETS = "()[]"
local WHITESPACES = " \t\r\n"
local COMMENT = "#"
local NOT_ID = BRACKETS .. WHITESPACES .. COMMENT

---@param char string
---@param str string
---@return boolean
local function char_in(char, str)
    return str:find(char, 1, true) ~= nil
end

---@param file file*
local function skip_whitespaces(file)
    local c
    while true do
        c = file:read(1)
        if not c then
            return
        elseif char_in(c, WHITESPACES) then
            -- nothing
        elseif c == COMMENT then
            local _ignored_comment = file:read("l")
        else
            file:seek("cur", -1)
            return
        end
    end
end


---@param file file*
---@return "("|")"|"["|"]"|nil
local function read_bracket(file)
    local c = file:read(1)

    if not c then
        return nil
    elseif char_in(c, BRACKETS) then
        return c
    else
        file:seek("cur", -1)
        return nil
    end
end


---@param file file*
---@return string|nil
local function read_id(file)
    local id = nil
    local c
    while true do
        c = file:read(1)
        if not c then
            return id
        elseif not char_in(c, NOT_ID) then
            id = (id or "") .. c
        else
            file:seek("cur", -1)
            return id
        end
    end
end

---@param file file*
---@return "("|")"|"["|"]"|string|nil
local function read_token(file)
    skip_whitespaces(file)
    return read_bracket(file) or read_id(file)
end


---@param file file*
---@return Term|nil
local function parse(file)
    local term = nil ---@type Term|nil

    while true do
        local token = read_token(file)

        local new_term = nil

        if not token then
            break
        elseif token == "(" then
            new_term = { type = "lambda", read_token(file) }
        elseif token == "[" then
            new_term = { type = "application" }
        elseif token == ")" or token == "]" then
            if not term then
                error("Unmatched closing bracket")
            end

            if term.parent == nil then
                break
            end

            term = term.parent
        else
            new_term = { type = "variable", token }
        end

        if new_term then
            new_term.parent = term

            if term then
                table.insert(term, new_term)

                if #term > 4 then
                    error("Too many subterms in " .. term.type)
                end
            end

            if new_term.type ~= "variable" then
                term = new_term
            end
        end
    end

    return term
end


---This is recursive, only use for debugging small lambda expressions!
---@param term Term
local function dump(term)
    if term.type == "variable" then
        io.write(term[1])
    elseif term.type == "lambda" then
        io.write("(", term[1], " ")
        dump(term[2])
        io.write(")")
    elseif term.type == "application" then
        io.write("[")
        dump(term[1])
        io.write(" ")
        dump(term[2])
        io.write("]")
    end
end


---@param term Term
local function eval(term)
end


local function main()
    if #arg == 0 or arg[1] == "--help" then
        print("Usage: lua lambda.lua <filename> FLAGS...")
        return
    end

    local filename = arg[1]
    local flags = {
        dump = false,
    }

    for i = 2, #arg do
        if arg[i] == "--dump" then
            flags.dump = true
        end
    end

    local file = io.open(filename, "r")

    if not file then
        error("Could not open file " .. filename)
        return
    end

    local term = parse(file)

    file:close()

    if flags.dump then
        dump(term)
        io.write("\n")
        return
    end

    eval(term)
end

main()
