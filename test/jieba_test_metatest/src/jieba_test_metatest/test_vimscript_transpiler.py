# Copyright 2026 Kaiwen Wu. All Rights Reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
# License for the specific language governing permissions and limitations under
# the License.

import pytest

from . import vimscript_transpiler as m


class TestVimExpr:
    def test_int(self):
        assert str(m.VimExpr.int_(4)) == "4"
        assert str(m.VimExpr.int_(-1)) == "-1"

    def test_literal(self):
        assert str(m.VimExpr.literal("foo")) == '"foo"'
        assert str(m.VimExpr.literal('"a=foo')) == '"\\"a=foo"'
        assert str(m.VimExpr.literal("\\<Space>")) == '"\\<Space>"'
        assert str(m.VimExpr.literal('"\\<Newline>')) == '"\\"\\<Newline>"'

    def test_var(self):
        assert str(m.VimExpr.var("g:foo")) == "g:foo"

    def test_list(self):
        assert str(m.VimExpr.list_([2, "foo", 3])) == '[2, "foo", 3]'
        assert str(m.VimExpr.list_([2, [1, "foo"]])) == '[2, [1, "foo"]]'

    def test_dict(self):
        assert str(m.VimExpr.dict_({"a": 2, "b": "x"})) == '{"a": 2, "b": "x"}'
        assert (
            str(m.VimExpr.dict_({"a": {"aa": 3}, "b": "x"}))
            == '{"a": {"aa": 3}, "b": "x"}'
        )

    def test_into_vimexpr(self):
        assert str(m.VimExpr.into_vimexpr(4)) == "4"
        assert str(m.VimExpr.into_vimexpr("foo")) == '"foo"'
        assert str(m.VimExpr.into_vimexpr([2, "foo"])) == '[2, "foo"]'
        assert (
            str(m.VimExpr.into_vimexpr({"a": 2, "b": "x"}))
            == '{"a": 2, "b": "x"}'
        )


class TestVimExprBuilder:
    def test_concat(self):
        assert str(m.VimExprBuilder("foo") + 3) == '"foo" . 3'

    def test_eq(self):
        assert str(m.VimExprBuilder("foo") == "bar") == '"foo" ==# "bar"'
        assert str(m.VimExprBuilder(3) == "foo") == '3 ==# "foo"'

    def test_ne(self):
        assert str(m.VimExprBuilder("foo") != "bar") == '"foo" !=# "bar"'

    def test_gt(self):
        assert str(m.VimExprBuilder(3) > 2) == "3 > 2"

    def test_ge(self):
        assert str(m.VimExprBuilder(3) >= 2) == "3 >= 2"

    def test_lt(self):
        assert str(m.VimExprBuilder(3) < 2) == "3 < 2"

    def test_le(self):
        assert str(m.VimExprBuilder(3) <= 2) == "3 <= 2"

    def test_getitem(self):
        assert str(m.VimExprBuilder([1, "foo"])[1]) == '[1, "foo"][1]'
        assert str(m.VimExprBuilder(m.VimExpr.var("g:foo"))[2]) == "g:foo[2]"
        assert (
            str(m.VimExprBuilder({"a": 1, "b": "x"})["a"])
            == '{"a": 1, "b": "x"}["a"]'
        )
        assert (
            str(m.VimExprBuilder(m.VimExpr.var("g:foo"))["a"]) == 'g:foo["a"]'
        )

    def test_call(self):
        assert (
            str(m.VimExprBuilder(m.VimExpr.var("Func"))(1, "foo"))
            == 'Func(1, "foo")'
        )
        assert (
            str(
                m.VimExprBuilder(m.VimExpr.var("call"))(
                    m.VimExprBuilder(m.VimExpr.var("function"))("Func"),
                    m.VimExpr.var("a:000"),
                )
            )
            == 'call(function("Func"), a:000)'
        )

    def test_and(self):
        assert (
            str((m.VimExprBuilder("foo") == "bar") & (m.VimExprBuilder(2) == 3))
            == '"foo" ==# "bar" && 2 ==# 3'
        )

    def test_or(self):
        assert (
            str((m.VimExprBuilder("foo") == "bar") | (m.VimExprBuilder(2) == 3))
            == '"foo" ==# "bar" || 2 ==# 3'
        )

    def test_invert(self):
        assert (
            str(~m.VimExprBuilder(m.VimExpr.var("has"))("nvim"))
            == '!(has("nvim"))'
        )
        assert str(~(m.VimExprBuilder("foo") != "bar")) == '!("foo" !=# "bar")'

    def test_group(self):
        assert (
            str(
                (
                    (m.VimExprBuilder("foo") == "bar")
                    | (m.VimExprBuilder(2) != 3)
                ).group()
                & (m.VimExprBuilder(4) > 2)
            )
            == '("foo" ==# "bar" || 2 !=# 3) && 4 > 2'
        )


class TestLuaExpr:
    def test_int(self):
        assert str(m.LuaExpr.int_(4)) == "4"
        assert str(m.LuaExpr.int_(-1)) == "-1"

    def test_literal(self):
        assert str(m.LuaExpr.literal("foo")) == '"foo"'
        assert str(m.LuaExpr.literal('"a=foo')) == '"\\"a=foo"'
        with pytest.raises(ValueError):
            _ = str(m.LuaExpr.literal("\\<Space>"))
        with pytest.raises(ValueError):
            _ = str(m.LuaExpr.literal('"\\<Newline>'))

    def test_var(self):
        assert str(m.LuaExpr.vim_var("g:foo")) == "vim.g.foo"
        with pytest.raises(ValueError):
            _ = str(m.LuaExpr.vim_var("s:foo"))
        with pytest.raises(ValueError):
            _ = str(m.LuaExpr.vim_var("a:foo"))
        assert str(m.LuaExpr.lua_var("foo")) == "foo"

    def test_list(self):
        assert str(m.LuaExpr.list_([2, "foo", 3])) == '{2, "foo", 3}'
        assert str(m.LuaExpr.list_([2, [1, "foo"]])) == '{2, {1, "foo"}}'

    def test_dict(self):
        assert str(m.LuaExpr.dict_({"a": 2, "b": "x"})) == '{a = 2, b = "x"}'
        assert (
            str(m.LuaExpr.dict_({"a": {"aa": 3}, "b": "x"}))
            == '{a = {aa = 3}, b = "x"}'
        )

    def test_into_luaexpr(self):
        assert str(m.LuaExpr.into_luaexpr(4)) == "4"
        assert str(m.LuaExpr.into_luaexpr("foo")) == '"foo"'
        assert str(m.LuaExpr.into_luaexpr([2, "foo"])) == '{2, "foo"}'
        assert (
            str(m.LuaExpr.into_luaexpr({"a": 2, "b": "x"}))
            == '{a = 2, b = "x"}'
        )

    def test_wrap_vim_int(self):
        assert str(m.LuaExpr.wrap_vim(m.VimExpr.int_(3))) == "3"

    def test_wrap_vim_literal(self):
        assert str(m.LuaExpr.wrap_vim(m.VimExpr.literal("foo"))) == '"foo"'

    def test_wrap_vim_var(self):
        assert str(m.LuaExpr.wrap_vim(m.VimExpr.var("g:foo"))) == "vim.g.foo"
        with pytest.raises(ValueError):
            _ = str(m.LuaExpr.wrap_vim(m.VimExpr.var("s:foo")))
        with pytest.raises(ValueError):
            _ = str(m.LuaExpr.wrap_vim(m.VimExpr.var("a:foo")))

    def test_wrap_vim_list(self):
        assert (
            str(m.LuaExpr.wrap_vim(m.VimExpr.list_([2, "foo", 3])))
            == '{2, "foo", 3}'
        )
        assert (
            str(m.LuaExpr.wrap_vim(m.VimExpr.list_([2, [1, "foo"]])))
            == '{2, {1, "foo"}}'
        )

    def test_wrap_vim_dict(self):
        assert (
            str(m.LuaExpr.wrap_vim(m.VimExpr.dict_({"a": 2, "b": "x"})))
            == '{a = 2, b = "x"}'
        )
        assert (
            str(m.LuaExpr.wrap_vim(m.VimExpr.dict_({"a": {"aa": 3}, "b": "x"})))
            == '{a = {aa = 3}, b = "x"}'
        )

    def test_wrap_vim_concat(self):
        assert (
            str(m.LuaExpr.wrap_vim(m.VimExprBuilder("foo") + 3)) == '"foo" .. 3'
        )

    def test_wrap_vim_eq(self):
        assert (
            str(m.LuaExpr.wrap_vim(m.VimExprBuilder("foo") == "bar"))
            == '"foo" == "bar"'
        )
        assert (
            str(m.LuaExpr.wrap_vim(m.VimExprBuilder(3) == "foo"))
            == '3 == "foo"'
        )

    def test_wrap_vim_ne(self):
        assert (
            str(m.LuaExpr.wrap_vim(m.VimExprBuilder("foo") != "bar"))
            == '"foo" ~= "bar"'
        )

    def test_wrap_vim_gt(self):
        assert str(m.LuaExpr.wrap_vim(m.VimExprBuilder(3) > 2)) == "3 > 2"

    def test_wrap_vim_ge(self):
        assert str(m.LuaExpr.wrap_vim(m.VimExprBuilder(3) >= 2)) == "3 >= 2"

    def test_wrap_vim_lt(self):
        assert str(m.LuaExpr.wrap_vim(m.VimExprBuilder(3) < 2)) == "3 < 2"

    def test_wrap_vim_le(self):
        assert str(m.LuaExpr.wrap_vim(m.VimExprBuilder(3) <= 2)) == "3 <= 2"

    def test_wrap_vim_getitem(self):
        with pytest.raises(ValueError):
            _ = str(m.LuaExpr.wrap_vim(m.VimExprBuilder([1, "foo"])[1]))
        with pytest.raises(ValueError):
            _ = str(
                m.LuaExpr.wrap_vim(m.VimExprBuilder(m.VimExpr.var("g:foo"))[2])
            )
        assert (
            str(m.LuaExpr.wrap_vim(m.VimExprBuilder({"a": 1, "b": "x"})["a"]))
            == '{a = 1, b = "x"}["a"]'
        )
        assert (
            str(
                m.LuaExpr.wrap_vim(
                    m.VimExprBuilder(m.VimExpr.var("g:foo"))["a"]
                )
            )
            == 'vim.g.foo["a"]'
        )

    def test_wrap_vim_call(self):
        assert (
            str(
                m.LuaExpr.wrap_vim(
                    m.VimExprBuilder(m.VimExpr.var("Func"))(1, "foo")
                )
            )
            == 'vim.fn.Func(1, "foo")'
        )
        assert (
            str(
                m.LuaExpr.wrap_vim(
                    m.VimExprBuilder(m.VimExpr.var("call"))(
                        m.VimExprBuilder(m.VimExpr.var("function"))("Func"),
                        m.VimExpr.var("b:foo"),
                    )
                )
            )
            == 'vim.fn.call(vim.fn.function("Func"), vim.b.foo)'
        )

    def test_and(self):
        assert (
            str(
                m.LuaExpr.wrap_vim(
                    (m.VimExprBuilder("foo") == "bar")
                    & (m.VimExprBuilder(2) == 3)
                )
            )
            == '"foo" == "bar" and 2 == 3'
        )

    def test_or(self):
        assert (
            str(
                m.LuaExpr.wrap_vim(
                    (m.VimExprBuilder("foo") == "bar")
                    | (m.VimExprBuilder(2) == 3)
                )
            )
            == '"foo" == "bar" or 2 == 3'
        )

    def test_invert(self):
        assert (
            str(
                m.LuaExpr.wrap_vim(
                    ~m.VimExprBuilder(m.VimExpr.var("has"))("nvim")
                )
            )
            == 'not (vim.fn.has("nvim"))'
        )

    def test_group(self):
        assert (
            str(
                m.LuaExpr.wrap_vim(
                    (
                        (m.VimExprBuilder("foo") == "bar")
                        | (m.VimExprBuilder(2) != 3)
                    ).group()
                    & (m.VimExprBuilder(4) > 2)
                )
            )
            == '("foo" == "bar" or 2 ~= 3) and 4 > 2'
        )


class TestLuaExprBuilder:
    def test_concat(self):
        assert str(m.LuaExprBuilder("foo") + 3) == '"foo" .. 3'

    def test_eq(self):
        assert str(m.LuaExprBuilder("foo") == "bar") == '"foo" == "bar"'
        assert str(m.LuaExprBuilder(3) == "foo") == '3 == "foo"'

    def test_ne(self):
        assert str(m.LuaExprBuilder("foo") != "bar") == '"foo" ~= "bar"'

    def test_gt(self):
        assert str(m.LuaExprBuilder(3) > 2) == "3 > 2"

    def test_ge(self):
        assert str(m.LuaExprBuilder(3) >= 2) == "3 >= 2"

    def test_lt(self):
        assert str(m.LuaExprBuilder(3) < 2) == "3 < 2"

    def test_le(self):
        assert str(m.LuaExprBuilder(3) <= 2) == "3 <= 2"

    def test_getitem(self):
        with pytest.raises(TypeError):
            _ = str(m.LuaExprBuilder([1, "foo"])[1])
        with pytest.raises(TypeError):
            _ = str(m.LuaExprBuilder(m.LuaExpr.vim_var("g:foo"))[2])
        assert (
            str(m.LuaExprBuilder({"a": 1, "b": "x"})["a"])
            == '{a = 1, b = "x"}["a"]'
        )
        assert (
            str(m.LuaExprBuilder(m.LuaExpr.vim_var("g:foo"))["a"])
            == 'vim.g.foo["a"]'
        )

    def test_call(self):
        assert (
            str(m.LuaExprBuilder(m.LuaExpr.vim_var("Func"))(1, "foo"))
            == 'vim.fn.Func(1, "foo")'
        )
        assert (
            str(
                m.LuaExprBuilder(m.LuaExpr.lua_var("io.stderr:write"))(
                    m.LuaExprBuilder("foo") + 3 + "\n"
                )
            )
            == 'io.stderr:write("foo" .. 3 .. "\\n")'
        )

    def test_and(self):
        assert (
            str((m.LuaExprBuilder("foo") == "bar") & (m.LuaExprBuilder(2) == 3))
            == '"foo" == "bar" and 2 == 3'
        )

    def test_or(self):
        assert (
            str((m.LuaExprBuilder("foo") == "bar") | (m.LuaExprBuilder(2) == 3))
            == '"foo" == "bar" or 2 == 3'
        )

    def test_invert(self):
        assert (
            str(~m.LuaExprBuilder(m.LuaExpr.lua_var("has"))("nvim"))
            == 'not (has("nvim"))'
        )
        assert (
            str(~(m.LuaExprBuilder("foo") != "bar")) == 'not ("foo" ~= "bar")'
        )

    def test_group(self):
        assert (
            str(
                (
                    (m.LuaExprBuilder("foo") == "bar")
                    | (m.LuaExprBuilder(2) != 3)
                ).group()
                & (m.LuaExprBuilder(4) > 2)
            )
            == '("foo" == "bar" or 2 ~= 3) and 4 > 2'
        )


def test_json_encoded():
    dct = m.VimExpr.dict_({"a": 3, "b": "x"})
    assert (
        str(m.json_encoded(dct))
        == 'escape(json_encode({"a": 3, "b": "x"}), "\\\\")'
    )
    assert (
        str(m.json_encoded(m.LuaExpr.wrap_vim(dct)))
        == 'vim.fn.json_encode({a = 3, b = "x"})'
    )


def test_not_eq_test_as_str():
    actual = m.var("getreg")("a")
    expected = "foo"
    assert (
        m.not_eq_test_as_str('unexpected register "a', actual, expected)
        == """\
if getreg("a") !=# "foo"
    if has("nvim")
        lua <<EOF
io.stderr:write("unexpected register \\"a" .. " actual:: " .. vim.fn.json_encode(vim.fn.getreg("a")) .. " expected:: " .. "foo" .. "\\n")
EOF
    else
        execute "!echo " . shellescape("unexpected register \\"a" . " actual:: " . escape(json_encode(getreg("a")), "\\\\") . " expected:: " . "foo", 1) . " >&2"
    endif
endif
"""
    )
