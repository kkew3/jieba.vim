-- Copyright 2026 Kaiwen Wu. All Rights Reserved.
--
-- Licensed under the Apache License, Version 2.0 (the "License"); you may not
-- use this file except in compliance with the License. You may obtain a copy
-- of the License at
--
--     http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
-- WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
-- License for the specific language governing permissions and limitations
-- under the License.

local M = { word_motion = nil }

M.jieba_vim_rs = require("jieba_vim.jieba_vim_rs")

M.buffer = {
    getline = function(lnum)
        if lnum == 1 and vim.api.nvim_buf_line_count(0) == 0 then
            return ""
        else
            return vim.api.nvim_buf_get_lines(0, lnum - 1, lnum, false)[1]
        end
    end,

    lines = function()
        return vim.api.nvim_buf_line_count(0)
    end
}

function M.init_word_motion(self, user_dict, isk, lazy)
    if self.word_motion ~= nil then
        return ""
    end

    if user_dict == "" then
        user_dict = nil
    end

    local ok, wm = pcall(function()
        if lazy == 1 or lazy == "1" then
            return self.jieba_vim_rs.LazyWordMotion(isk, user_dict)
        else
            return self.jieba_vim_rs.WordMotion(isk, user_dict)
        end
    end)
    if not ok then
        return string.format("jieba_vim: failed to load user dict: %s", tostring(user_dict))
    end

    self.word_motion = wm
    return ""
end

function M.nmap(self, buffer, motion, cursor, count)
    return self.word_motion:nmap(buffer, motion, cursor, count)
end

function M.xmap(self, buffer, visualmode, motion, visual_begin, visual_end, count)
    return self.word_motion:xmap(buffer, visualmode, motion, visual_begin, visual_end, count)
end

function M.omap(self, buffer, motion, cursor, count, operator)
    return self.word_motion:omap(buffer, motion, cursor, count, operator)
end

function M.imap_ctrl_w(self, buffer, cursor)
    return self.word_motion:imap_ctrl_w(buffer, cursor)
end

function M.preview_nmap(self, buffer, motion, cursor, preview_limit)
    return self.word_motion:preview_nmap(buffer, motion, cursor, preview_limit)
end

function M.update_isk(self, isk)
    self.word_motion:set_isk(isk)
end

return M
